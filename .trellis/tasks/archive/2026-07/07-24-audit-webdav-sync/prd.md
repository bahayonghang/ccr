# WebDAV 同步路径/事务/传输三层加固

> 父任务：`07-24-audit-remediation` ｜ 覆盖：P1-03、P1-04、P1-10、P2-05、P2-06、P2-07、P2-19 ｜ 报告 Epic C1-C7

## Goal

补齐 WebDAV 同步缺失的 path / transaction / transport 三层边界，并把敏感资产的机密性从"只靠 TLS"提升到客户端加密。

## 背景 / 证据（已核实）

### Path boundary（P1-03）
- `crates/ccr-sync/src/sync/service.rs:580-586` — `extract_filename` 取 href 末段，可返回 `..`
- `crates/ccr-sync/src/sync/service.rs:597-647` — `should_exclude_from_sync` 显式放行 `.`/`..`（`name != "." && name != ".."`），非安全 validator
- `crates/ccr-sync/src/sync/service.rs:312,333` — `local_dir.join(&file_name)` 直接拼接，无 containment；Windows `\` 未拒绝

### Transaction boundary（P1-04, P2-05, P2-07）
- `ccr-ui/src-tauri/src/commands/sync.rs:543-589` — `pull_asset_config` 先 `backup_existing_path`(rename) 再 pull，pull 失败无回滚
- `ccr-ui/src-tauri/src/commands/sync.rs:610-613` — sync (true,true)+!force 调 `push_asset_config(..., false)`，后者 `:527-535` 见 remote exists 必报错（正常 sync 分支稳定失败，P2-05）
- `ccr-ui/src-tauri/src/commands/sync.rs:635-658` — WebDAV 配置顺序写 legacy manager + folder manager，第二写失败第一份已提交（P2-07）

### Transport / storage boundary（P1-10, P2-06, P2-19）
- `crates/ccr-sync/src/sync/service.rs:37-48` — 任意 URL 交给 Basic Auth client，无 HTTPS 策略
- `crates/ccr-sync/src/sync/service.rs:253-256` — `.bytes()` 整体缓冲；`pull_directory` 无 depth/entries/bytes 上限（P2-06）
- `ccr-ui/src-tauri/src/commands/sync.rs:117-161` — ccr-platforms / claude-settings / codex-config 标记 sensitive 但明文 PUT（P2-19）

## Requirements

### Path（C1）
- [x] href 末段 percent-decode 后要求"恰好一个 `Component::Normal`"；拒绝空、`.`、`..`、`/`、`\`、drive/UNC
- [x] 落盘前对 `local_dir.join(name)` 做 canonical containment 校验

### Transaction（C2, C4）
- [x] pull 改 stage→validate→fsync→swap→fsync(parent) 状态机：下载到 sibling staging，验证后 atomic swap；commit 前失败不动 active，commit 中失败自动 restore backup
- [x] 修复 sync 真值表（P2-05）：prefer-local 直接 push overwrite，或返回 typed conflict 等用户选择；写 truth-table 测试
- [x] WebDAV 配置改单一 source of truth，另一份 read-through/migration 生成；过渡期用 journal/compensation（P2-07）

### Transport / storage（C3, C5, C6）
- [x] WebDAV URL 默认强制 HTTPS，仅 localhost + explicit dev flag 允许 HTTP；保存与连接双重校验并显示 blocking error（P1-10）
- [x] streaming write；max file/total bytes、max depth、max entries、deadline；超限 rollback（P2-06, C3）
- [x] 敏感资产 AEAD envelope（version/salt/nonce/ciphertext/metadata，Argon2id + keychain/passphrase key）；支持 v1 plaintext 只读迁移，默认新写 v2；UI 明示加密状态（P2-19, C6）

## Acceptance Criteria

- [x] 恶意 href 语料全部被拒（报告 §9.1 WebDAV）：`../`、`%2e%2e/`、`..\evil`、`C:\evil`、`//server/share`、empty segment、超长名、超深度、同 href cycle
- [x] fault injection 覆盖 list/GET/stream/mkdir/write/fsync/rename/parent-fsync/backup-restore，任一失败后 active bytes 不变（100% 保留）
- [x] 非 localhost `http://` 保存/连接均拒绝
- [x] sync 四组合 × force 真值表测试通过
- [x] `just lint-strict` + `just test` 通过

## Out of Scope

- 不信任 WebDAV 客户端库替代应用层路径/资源预算校验
- 不静默覆盖或删除 v1 plaintext remote；迁移必须显式且保留本地备份
- 不把 WebDAV Basic Auth 密码直接复用为端到端加密密钥

## Key Decision

- 2026-07-26 用户同意 v2 sensitive asset 使用同步时输入的独立口令。口令只在单次同步操作内存中存在，不写入 WebDAV 配置、本机 secret store、日志或事件；跨设备输入同一口令后通过现有 Argon2id + AES-GCM 能力派生密钥并解密

## Notes

- **先验证项**（报告 §11）：`reqwest_dav` 是否在反序列化前对 href 强制规范化——用 fake DAV server + 固定依赖版本测试确认，无论结论应用层都独立保证 containment
- 需保持 secret masking / backup-before-destructive / 文件锁 / 原子写（CLAUDE.md 规则）
- 触发 rust-security-reviewer 复查；AEAD 部分与 persistence 子任务的 crypto 能力对齐
- 报告 §5 2B 回滚：v1 remote 只读 + export backup，v2 写入按 asset feature flag
