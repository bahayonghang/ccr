# Implement: Secret 掩码 newtype

> 前置：已读 `.trellis/spec/ccr-core/backend/backend-guidelines.md`、`atomic-writer.md`（secret 0o600 契约）、`public-api-boundary`（ccr 根 facade 冻结）。
> 批次约束：B0 先行；B1/B2/B3 依赖 B0 但互相独立；B2 波及面最大（~30 文件），放在 B1 用小面积验证 serde 模式之后。

## B0 — ccr-core `Secret` 类型（地基）

- [x] 新建 `core/secret.rs`：`Secret(String)`；`new`/`expose`/`is_empty`；`From<String>`/`From<&str>`；`Clone`/`Default`/`PartialEq`/`Eq`/`PartialEq<str>`/`PartialEq<&str>`；Debug/Display → `mask_sensitive`；默认 `Serialize` → 掩码串；`Deserialize` 透明收明文；`expose_plaintext`/`expose_plaintext_option` 序列化函数。公共 API 文档英文、实现注释中文。
- [x] `core/mod.rs` + `lib.rs` 根 re-export `Secret`、`expose_plaintext`、`expose_plaintext_option`；**不动** `ccr` 根 facade（public_api_compat 冻结）。
- [x] 按 design §6.1 写单元测试（Display/Debug/双格式默认掩码/expose 注解/明文反序列化/Option 组合/PartialEq/is_empty）。
- [x] 验证：`cargo test -p ccr-core -- --test-threads=1` && `cargo clippy -p ccr-core --all-targets --all-features -- -D warnings` && `cargo test -p ccr --test public_api_compat -- --test-threads=1`
- [x] ⏸ 回滚点：单独 commit `feat(core)`

## B1 — ccr-sync 密码迁移（+ ccr-cli sync + ccr-ui sync 消费方）

- [x] `sync/config.rs`：`SyncConfig.password: Secret` + `expose_plaintext` 注解。
- [x] `sync/folder.rs`：`WebDavConfig.password: Secret` + 注解；Default 适配。
- [x] `sync/folder_manager.rs`：`save_config` 改 `write_toml_opts(secret: true)`（文件含密码）；测试构造适配。
- [x] `sync/service.rs:45`：`expose().to_string()`。
- [x] `ccr-cli sync/commands.rs`：`read_password()` 返回值包 `Secret::new`；构造/克隆 ×6 适配。
- [x] `ccr-ui src-tauri sync.rs`：完整性检查 ×~8 改 `expose().trim().is_empty()`；克隆/搬运不变；`has_password` 改 `!p.is_empty()`；测试断言 `PartialEq<&str>`。
- [x] 测试：sync.toml + sync_folders.toml 旧明文 round-trip 无损；`{:?}` 不含明文；`sync_folders.toml` 0o600（unix）。
- [x] 验证：`cargo test -p ccr-sync -p ccr-cli -- --test-threads=1` && clippy 两 crate && `cd ccr-ui/src-tauri && cargo clippy --all-targets -- -D warnings && cargo test`
- [x] ⏸ 回滚点：commit `refactor(sync)`

## B2 — ccr-config auth_token 迁移（+ 全消费方）

- [x] `managers/config/types.rs`：`ConfigSection.auth_token: Option<Secret>` + 注解；`Validatable`/`to_anthropic_env_status` 适配（后者 `expose().to_string()`）。
- [x] `models/platform.rs`：`ProfileConfig.auth_token: Option<Secret>` + 注解。
- [x] `config_service.rs`：`export_config` 掩码分支改 `Secret::new(t.to_string())`；`ConfigInfo.auth_token: Option<Secret>`。
- [x] `platforms/base.rs` ×3、`ccr-cli platforms/{claude,gemini,droid}.rs`、`claude_auth_service`/`codex_auth_service`/`codex_runtime_service`/`doctor_service`/`health_check`/`runtime_overview_service`/`provider_cmd`/`settings.rs`：env/settings/auth 构造点 `expose()`；显示点 Display。
- [x] `ccr-cli managers/temp_override.rs`：`Option<Secret>` + 注解（写路径不动）；`temp_token.rs`/`temp_cmd.rs` 显示位 Display。
- [x] `ccr-cli commands/profile/*`、`platform/profile.rs`、`claude/profile.rs`、`codex/profile.rs`、CLI 参数入口：clap 参数保持 String，进模型时 `Secret::new`。
- [x] `ccr-tui ui.rs` ×2：`expose().trim().is_empty()`。
- [x] `ccr-codex platforms/codex.rs` ×7：消费点适配。
- [x] `ccr-ui config.rs`：删除 `mask_token`，列表掩码改 Display（格式不变）；`:229/:525` 写入路径 `Secret::new`。
- [x] `ccr-ui claude.rs:456` / `codex.rs:1629`：`as_ref().map(Secret::expose)` + 行为保持注释（design §0.4）。
- [x] `crates/ccr/tests/` ×16：断言/构造适配。
- [x] 测试：`.ccs_config.toml` 旧明文 round-trip 无损；`{:?}` 不含明文；masked export 行为不变测试。
- [x] 验证：`cargo test -p ccr-config -p ccr-cli -p ccr-tui -p ccr-codex -p ccr -- --test-threads=1` && clippy 全部 && ccr-ui/src-tauri clippy+test
- [x] ⏸ 回滚点：commit `refactor(config)`

## B3 — ccr-checkin/ccr-db 掩码收敛

- [x] 删除 `ccr-checkin core/crypto.rs mask_api_key`（含测试）。
- [x] 删除 `ccr-db models/checkin/account.rs mask_cookies_json`/`mask_value`（含 mod.rs re-export 与测试）。
- [x] `CryptoManager::decrypt` 返回 `Secret`；`CreateAccountRequest`/`UpdateAccountRequest.cookies_json: Secret`；`get_cookies_json` 返回 `(Secret, String)`；HTTP 头构造/加密点 `expose()`。
- [x] `account_manager` 私有 `masked_cookies_display`：JSON map 迭代 + 每值 Secret Display；`get_info`/占位符 `"****"` 路径接入；新格式断言测试；`{:?}` 请求不泄明文测试。
- [x] 验证：`cargo test -p ccr-checkin -p ccr-db -- --test-threads=1` && clippy 两 crate && ccr-ui/src-tauri clippy+test（checkin 命令波及）
- [x] ⏸ 回滚点：commit `refactor(checkin)`

## B4 — 收尾全量检查

- [x] AC#1 扫描：`mask_api_key`/`mask_cookies_json`/`mask_value`/`mask_token` 全仓 0 命中（`ref/` 除外）；掩码算法仅 `utils/mask.rs` 一处。
- [x] AC#3 扫描：`rg 'password.*String|auth_token.*Option<String>'` 凭据结构 0 残留（clap 参数/env map 白名单除外，白名单写进 spec）。
- [x] `rg 'expose_plaintext'` 输出即全部明文落盘点清单，逐一确认都是持久化字段。
- [x] `just version-check` → `just fmt-check` → `just lint-strict` → `just test`；ccr-ui/src-tauri `cargo clippy && cargo test`。
- [x] rust-security-reviewer 子代理审查（触发条件：凭据处理）；发现项修复或记录。
- [x] Spec 更新（trellis-update-spec）：design §9。
- [x] 对照 prd.md Acceptance Criteria 逐条勾验；journal 记录；归档。

## 全局回滚策略

任一批失败且无法快速修复 → `git revert` 该批 commit；B0 回滚需先回滚已合入的 B1-B3。磁盘格式零变化，回滚无数据迁移成本。
