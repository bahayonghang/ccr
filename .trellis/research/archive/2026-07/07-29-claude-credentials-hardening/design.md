# 设计:凭据持久化与 settings CAS-RMW

## 账号切换保护

### 当前登录是否已保存

对 `ClaudeCredentialsDocument` 进行稳定序列化,用 `ccr_core::content_version_token` 计算仅存在于内存的身份令牌。逐个加载 registry 指向的快照并比较令牌:

- 当前 credentials 不存在:没有待保护登录,允许切换到已存目标。
- 当前 credentials 精确匹配任一有效快照:允许切换,记录原快照用于失败恢复。
- 当前 credentials 存在但不匹配:拒绝,提示先 `ccr claude auth save <name>`。
- 当前 credentials 损坏/不可读:拒绝,不得覆盖。

身份令牌不落盘、不进日志、不进入错误/DTO。token 刷新后内容变化会被视为“未保存的新登录状态”,这是保守保护而非误报:用户先 save 才不会丢掉刷新后的凭据。

### C10 元数据来源

`load_current_runtime_auth` 先用当前 credentials 匹配快照。匹配时用该快照的 `oauth_account` 构造 email/UUID/billing,并用 credentials 自身提供 subscription/expiry;只有未匹配登录才读取当前 state_file 的 `oauthAccount`。这样 A->B 后立即显示 B,且不写 Claude Code 私有状态。

registry 的 `current_auth` 不是身份真相,只作为最近成功切换记录;实际 current/login 判断仍以凭据匹配结果为准。

## 凭据文件写策略

删除 `ClaudeAuthService::write_atomic` 和 `NamedTempFile` 旁路。所有 auth durable writes 使用 `write_guarded`:

| 文件 | secret | backup | 说明 |
|---|---:|---|---|
| `.credentials.json` | true | None | 当前凭据;旧值已有已保存快照保护 |
| `auth/<name>.json` | true | None | 含 access/refresh token,禁止明文副本 |
| `auth_registry.toml` | true | None | 账号元数据按认证状态同等级保护 |

guarded write 已提供路径锁、temp owner-only 权限、fsync-before-rename。目录创建失败或写失败原目标保持不变。save_current 先写快照再更新 registry;registry 失败时快照成为未引用文件,后续相同名称 save/修复可覆盖,不得删除可能含唯一凭据的文件。

## settings.json 单一 RMW

在 `SettingsManager` 增加同步/异步 `update_atomic`(以及原始 JSON patch 所需入口):

1. 读取原始 bytes 与 version token;缺失文件按空 object/空 token。
2. 解析并对最新值执行确定性 mutation closure。
3. 用 `write_guarded_versioned` 写入,options 固定 `secret:true` + `BackupPolicy::Dir { backups_dir, prefix:"settings" }`。
4. Conflict 时重新读取并重放 mutation,最多 3 次;耗尽返回可操作的 concurrency error。

CAS 内部路径锁统一 CLI/Tauri 本地 UI 的锁域,备份与替换在同一把锁下且轮换保留 10 份。`save_atomic` 只保留给“调用者确实拥有完整替换语义”的恢复路径;所有 load->mutate->save 生产调用迁移到 update API。

CLI 迁移范围包括 profile apply/off、auth switch 清理、clear、temp override/token、SettingsService 等对 Claude settings 的变更调用点。Tauri 的 Claude agents/hooks/plugins/slash/settings/statusline 等本地 mutation 通过共享 helper 调用同一个 `SettingsManager`;SSH/WSL 远程写不与本机 CLI 共享文件系统,保持现有环境接口并明确不在本任务的跨进程保证内。

`LocalEnvironment::write_config` 对仍存在的 Claude settings 完整写入口使用 `secret:true` + 集中 backup,不得产生同目录 `*.bak`。

## 切换失败语义

目标快照、当前保护状态与 settings mutation 均先验证。写目标 credentials 后若 settings 清理失败,尝试用已识别的原快照恢复原 credentials;恢复失败时返回组合错误但不得打印内容。registry 只在凭据写和 settings 清理都成功后更新。

跨 credentials/settings/registry 不是单文件事务;设计承诺“无静默成功、无未保存凭据覆盖、可从已存快照恢复”,不承诺崩溃瞬间的多文件原子性。

## OS 与兼容

- Windows/Linux 走文件凭据路径。
- macOS 的 save/switch 在入口显式返回 Keychain 不支持错误;list/delete ccr 自管快照可继续,但不得假装读到了官方 runtime 凭据。
- 快照 v1 JSON schema 保持兼容;不加密、不改 token 字段。
- UI/CLI/TUI 的现有 DTO 默认不变,C10 只修正值来源。

## 测试

- 未保存/损坏/缺失/已保存当前 credentials 的切换矩阵。
- A/B 快照元数据与凭据匹配,覆盖 A->B->A 以及 state_file 仍为 A。
- Unix 0600、无同目录 backup、guarded write 冲突/错误保留旧文件。
- SettingsManager 两线程/两入口 CAS 注入延迟,验证两个不同字段均保留或冲突显式失败。
- Tauri local mutation 与 CLI mutation 并发;未知字段和用户 env 保留。
- macOS cfg 单元测试验证错误分支不触碰文件。

## 回滚

若 CAS 迁移暴露某调用点需要完整替换,逐调用点记录并保留显式 replace API;不得恢复两套锁或无版本 load/save。加密作为独立迁移任务,不夹带进本任务。
