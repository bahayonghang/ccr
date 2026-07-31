# Claude 凭据文件写入安全加固

## Goal

修复 C3/C4/C5/C10:让官方 OAuth 凭据(`.credentials.json` 与 ccr 账号快照)的落盘达到与仓库其他凭据路径相称的保护级别;统一 `settings.json` 双写路径的**锁域与 RMW 协议**(不只是 secret/backup);消除账号切换后的身份元数据错配。父任务序列第 3 号。

## 问题(源码已核实,含二轮复核)

1. **C3 无备份覆盖凭据**:`switch_account`(`crates/ccr-cli/src/services/claude_auth_service.rs:622-638`)直接用快照覆盖 `.credentials.json`。当前活跃登录若从未存快照即永久丢失。TUI 守卫(`crates/ccr-tui/src/tui/claude_auth/app.rs:420-442`)挡不住"当前登录未保存"。
2. **C10 身份元数据错配(二轮新增,P0)**:`save_current` 把 `oauthAccount` 存入快照(`claude_auth_service.rs:600-605`),但 `switch_account` 只恢复 `credentials`,快照的 `oauth_account` 字段**从未被读取**;诊断 `build_current_info`(`:408-424`)的 UUID/email 取自现存 `.claude.json`。A 切 B 后形成"B 凭据 + A 元数据",CLI/TUI/UI 可能继续显示 A 的身份。
3. **C4 裸写路径**:`ClaudeAuthService::write_atomic`(`:313-332`)手搓 tempfile→persist:无 secret(0600)、无锁、无 fsync、无备份,违反 `atomic-writer.md:113/:135`。负责写 `.credentials.json`、auth registry、快照(含明文 accessToken/refreshToken)。
4. **C5 双写路径不等价且互不互斥(二轮修订)**:
   - 安全属性:UI(`ccr-ui/src-tauri/src/platform/local.rs:89-100`)`secret:false` + `BackupPolicy::SameDir` 同目录明文备份;CLI(`managers/settings.rs:165-186`)`secret:true` 无备份。
   - **锁域**:UI 走 guarded_write 的路径派生锁;CLI 走固定 `claude_settings` 命名锁(`settings.rs:167`)后直接 `AtomicWriter`(绕过 guarded_write)。两把锁互不相识 → CLI/UI 并发 read-modify-write 可互相丢更新。且两侧的 RMW 均未在同一锁/版本令牌下完成(load 与 save 之间无保护)。
   - 结论:只统一 secret/备份目录**不解决丢更新**,必须统一 RMW API、锁域与冲突协议(`guarded_write` 已有 CAS 原语 `write_guarded_versioned` / `VersionedWriteOutcome`,`crates/ccr-core/src/core/guarded_write.rs:31-42`)。

## Requirements

- R1(**已定夺**):`switch_account` 覆盖 `.credentials.json` 前,若当前凭据与任何已存快照都不匹配 → **拒绝切换**并提示先 `ccr claude auth save`(不采用自动匿名备份)。
- R2:消除 C10 ——采用方案 (b):诊断/展示层以当前凭据和已存快照的内存身份令牌精确匹配,匹配时优先读快照 `oauth_account`;未匹配登录才回落到当前 `.claude.json` 元数据。不得为身份显示回写 `.claude.json`。
- R3:`ClaudeAuthService` 全部写入迁移到 `guarded_write`,`secret: true`,遵守锁序与 fsync-before-rename;凭据类文件不得同目录备份。
- R4:统一 settings.json 的 RMW:由 `SettingsManager` 提供 CAS 版本令牌 + 最多 3 次重读重放/冲突失败协议,CLI 与本地 UI 共用同一路径派生锁域;UI `write_config` 对凭据类目标启用 `secret: true`,备份改集中目录。
- R5:`profile use` / `auth switch` 触碰 settings.json 前产生备份(轮换上限沿用 BACKUP_KEEP=10)。
- R6:掩码不回退:新增日志/错误不得输出明文 token。
- R7:**OS 范围**:本任务只覆盖 Windows/Linux 的 `.credentials.json`;macOS 上 Claude Code 凭据在 Keychain,auth save/switch 应显式报"不支持",不得静默操作错误文件——现状核实与补齐纳入本任务。
- R8:触碰的 atomic-writer / 凭据边界契约条款在本任务内同步更新规范。

## Acceptance Criteria

- [ ] 当前登录未存快照时 `auth switch` 被拒绝并提示保存;已存快照则成功。
- [ ] **双账号回归**:A 存快照 → 登录态为 A → 切 B → CLI/TUI/UI 显示的账号身份(email/UUID)与 `.credentials.json` 实际凭据一致(均为 B);再切回 A 同样成立。
- [ ] `.credentials.json`、快照、registry 在 Unix 上 0600(Windows 继承用户目录 ACL);写入走 guarded_write(fsync 先于 rename)。
- [ ] 生效 `<config_dir>/` 下不再产生 `*.bak` / 明文临时副本;UI 与 CLI 写 settings.json 的安全属性一致,备份只进入集中目录。
- [ ] **并发保字段测试**:CLI 与 UI 并发对 settings.json 做 RMW(注入延迟),无丢更新或按冲突协议显式失败,用户自有字段不丢。
- [ ] macOS 行为:auth save/switch 显式报不支持,有测试覆盖。
- [ ] `rust-security-reviewer` 复查通过;`just lint-strict` + `just test` 通过;UI 侧过 `just frontend-check-quick`。
- [ ] 相关 spec 条款同步更新。

## Notes

- 依赖:`07-29-claude-authmode-consistency`(清理语义正确)与 `07-29-claude-config-dir-consistency`(统一路径解析)先合入;C10 已选择不回写 state_file,不依赖后续 #4 的 MCP CAS 实现。
- 快照文件加密(对齐 ccr-checkin AES-GCM)明确 out-of-scope;本任务先完成权限 + guarded_write + 无明文备份,加密另立任务评估,不新增生产依赖。
- Planning status:`design.md` 已选择 C10 零写面方案并定夺 CAS-RMW/锁域迁移,`implement.md`/JSONL 已就绪;#1/#2 完成后 start。
