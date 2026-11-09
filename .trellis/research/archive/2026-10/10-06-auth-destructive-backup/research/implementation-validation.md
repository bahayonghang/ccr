# P3 实施与验证

状态：SCOPED_IMPLEMENTATION_VALIDATED；等待独立 Trellis check。日期：2026-10-06。全部数据为临时目录中的合成账号；未读写真实凭据，未安装、提交、归档或推送。

## 改动

- `services/codex_auth_backup.rs`：注册表和逐账号精确备份池，共享 `CCR_LOCK_DIR` 的哈希池锁；最新内容去重并刷新匹配文件 mtime；同秒追加 `_N`；私有 guarded atomic write。保留所有旧文件，不运行清理。Windows 池锁和池匹配折叠大小写，文件布局和账号名称保留。
- `services/mod.rs`：注册 crate-private 备份模块。
- `services/codex_registry_store.rs`：普通保存传播写前备份失败。新增仅供删除/重命名使用的 `update_with_prepared_backup` 窄回调；现有 registry 写锁覆盖重读、只读检查、全部备份、文件变更和私有提交。回调成功后不再重复备份。
- `services/codex_auth_service.rs`：备份委托共享帮助函数。删除和重命名先取得 P1 稳定路径/完整身份锁，再在 registry 写锁内重读并验证逐来源身份。所有需要的备份成功后才删除或移动。保留 current_auth、IndexMap 顺序和 usage_ledger 更新语义；重命名的原 copy fallback 留待 P4。

锁序：凭据资源 → registry 写锁 → 备份池 → guarded writer 叶锁。独立备份只取池锁和叶锁；没有反向取得 registry 锁。只读命令预检仍在凭据锁副作用之前。

## 失败与修正记录

| 证据 | 退出 | 结果 |
| --- | --- | --- |
| `first-failure.log` | 101 | 1 passed / 3 failed：delete 缺少快照备份、force rename 忽略备份失败、registry save 忽略自动备份失败 |
| `first-retest.log` | 0 | 7 passed / 0 failed |
| `lifecycle-retest.log` | 101 | 12 passed / 1 failed：Windows PowerShell 无法自动加载 Get-Acl 模块；产品备份操作成功，ACL 探针失败 |
| `lifecycle-corrected-retest.log` | 0 | 13 passed / 0 failed：ACL 探针改用 Windows 内置 .NET `File.GetAccessControl` |
| `clippy-validation.log` | 101 | 备份委托后 `registry_path` 仅测试使用；生产 dead-code 拒绝 |
| `clippy-corrected-validation.log` | 0 | `registry_path` 限定 `#[cfg(test)]` 后严格检查通过 |

首轮的同名去重表面断言在旧实现中也会通过，因为旧实现覆盖同名文件。最终公开入口回归把既有备份改为较旧时间戳文件名，再调用 `backup()`，要求复用该旧路径；`public-dedup-retest.log` 为 1 passed / 0 failed。固定秒不同内容回归逐一检查每个备份的原字节。

## 检查

| 命令 | 退出与证据 |
| --- | --- |
| `cargo test -p ccr-codex --all-features p3_ -- --skip export_bindings` | 0；13 passed，包含一个供独立子进程调用的辅助入口；`lifecycle-corrected-retest.log` |
| `cargo test -p ccr-codex --all-features -- --skip export_bindings` | 0；353 passed / 0 failed / 2 原有 ignored；doc tests 0；默认并行；`codex-validation.log` |
| `cargo test -p ccr-codex --all-features p3_registry_backup_reuses_latest_identical_bytes -- --skip export_bindings` | 0；1 passed；公开去重断言增强后的单项重测；`public-dedup-retest.log` |
| `cargo clippy -p ccr-codex --all-targets --all-features -- -D warnings -D clippy::unwrap_used` | 0；`clippy-corrected-validation.log` |
| `just fmt-check` | 0；Rust/Tauri fmt、5 个格式检查器测试与 11 个 JSON 文件验证通过；`fmt-validation.log` |
| `python scripts/quality/check_secret_writes.py` | 0；`Sensitive persistence policy check passed` |
| `git diff --check --` 三个已跟踪实施文件 | 0；无输出；新模块也经过 rustfmt 与 fmt-check |

完整包检查之后仅增强公开去重测试前置条件、修正文档的多文件原子性描述，并限定测试帮助函数；最终 all-targets Clippy 和 fmt-check覆盖当前源码。独立检查须以当前源码为准。源码和全部原始回执 SHA256 见 `implementation-hashes.json`；首败 SHA256 为 `197756f0fb3793d0b7e115d8a981b4e6a3ac5977bff4bfbfd37de721f9ae890b`。

## 行为证据

- 16 个旧账号备份与相似别名、registry 备份均保留。`foo`/`foo_bar` 不合池；相同秒的新前像返回不同路径，原字节不变。
- 8 个线程和 3 个独立测试子进程的同秒不同内容均保留。独立进程输出不包含凭据。
- Windows 大小写别名共用池锁；预持池锁时另一个大小写别名等待，释放后保留两版字节。新账号备份 DACL 为 protected，仅当前进程用户 SID 的 Allow FullControl；探针只检查相等和权限，不输出 SID。
- 删除后备份可恢复完全相同的快照字节。备份目录为文件时，删除返回错误且 registry/snapshot 保留。
- force rename 在目标、源、registry 三个备份阶段分别注入固定失败；两个快照和 registry 字节保持。测试回调局部注入，无生产故障环境开关。
- 完成全部备份后阻断备份目录，重命名仍完成最终提交，证明破坏性步骤之后不会发起第二次 registry 备份。
- loopback OAuth POST 持有配额凭据操作锁时，重命名等待；POST/GET 成功后，新 refresh token 随快照移到新别名。该证据是合成 HTTP/文件生命周期验证。
- 既有 P2 只读删除/重命名回归随完整包检查通过。

## 限制与回滚

- 备份失败前保留源、目标和 registry。最终移动、权限或 registry 私有发布失败仍可能部分完成；没有通用多文件事务或补偿。
- 普通 `save` 的整表输入仍由调用者构造。新增破坏性回调在写锁内重读；其他原有 load/save 调用没有通用读改写事务保证。外部 Codex 或编辑器不参加 CCR 锁协议。
- 类形检索还发现 `codex_model_provider_store.rs:63` 忽略其独立 provider store 备份失败，位于 P3 白名单之外；未修改。Auth registry/service 的被忽略备份调用已清除。
- Unix owner-only 测试在 Windows 为 NOT_RUN。workspace `just lint-strict`、`just test` 和 `just ci` 由父任务在全部子任务完成后统一运行。真实账号恢复、已安装二进制和终端运行状态为 NOT_RUN。
- 源码回滚应仅撤销 P3 对四个实施文件的改动，保留 P1/P2 与其他角色修改；未执行任何 Git 回滚。数据恢复需先核对账号身份，再以私有原子写恢复匹配快照和匹配 registry 备份。旧 OAuth 授权的服务器有效性不由备份恢复保证。
