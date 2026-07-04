# Implement: 合并 ClaudeSettings

前置：design.md 已定稿。执行顺序即提交顺序，每步结束处于可编译状态。

## Step 1 — feat(types): ccr-types 吸收变更逻辑

- [ ] `crates/ccr-types/src/claude_settings.rs`：
  - 迁入 18 个 env key 常量（pub）+ `NON_ANTHROPIC_MANAGED_KEYS`（pub）
  - 新增固有方法：`new` / `clear_anthropic_vars` / `clear_managed_vars` / `apply_managed_env` / `anthropic_env_status` / `has_anthropic_overrides` / `validate_api_key_mode` / `validate`（错误 `Result<(), String>`，中文文案逐字保留）
  - `tracing` 不引入（ccr-types 无此依赖）——原方法内 `tracing::debug!/info!` 行随迁移丢弃，属可接受损失（纯数据方法不打点）
- [ ] `crates/ccr-types/src/lib.rs`：按需导出新常量（`ClaudeSettings` 已导出）
- [ ] 单元测试：迁入改造 clear/apply/env_status/validate 系列 + 新增往返保留测试（富字段+未知字段+legacy hooks 归一化）；统一 `expect`
- [ ] 验证：`cargo test -p ccr-types -- --test-threads=1` && `cargo clippy -p ccr-types --all-targets -- -D warnings`
- [ ] 提交 1（git-commit skill）

## Step 2 — feat(config): ConfigSection 映射

- [ ] `crates/ccr-config/Cargo.toml`：新增 `ccr-types = { path = "../ccr-types" }`
- [ ] `crates/ccr-config/src/managers/config/types.rs`：
  - `ConfigSection::to_managed_env_pairs()` 迁入 18 键映射（含 expose 注释）
  - `to_anthropic_env_status` 注释更新指向新方法（逻辑不动）
- [ ] 测试：映射断言（改造自原 update_from_config 系列）+ 防串档组合测试（消费 ClaudeSettings）
- [ ] 验证：`cargo test -p ccr-config -- --test-threads=1` && `cargo clippy -p ccr-config --all-targets -- -D warnings`
- [ ] 提交 2

## Step 3 — refactor(cli): 切换与收缩

- [ ] `crates/ccr-cli/src/managers/settings.rs`：删本地 struct/impl/Validatable impl/常量，`pub use ccr_types::ClaudeSettings;`；本文件 update_from_config 系测试迁出（已在 Step 1/2 落位），SettingsManager IO 测试保留；补磁盘级往返保留测试
- [ ] 调用点：
  - `platforms/claude.rs:348`、`services/settings_service.rs:70,80` → `apply_managed_env(section.to_managed_env_pairs())`
  - `doctor_service.rs:931-933`、`commands/lifecycle/validate.rs:190` → 错误包装适配（`map_err(CcrError::ValidationError)` 或直接消费 String）
  - `doctor_service.rs:1819` struct literal → `..Default::default()`
- [ ] ccr 根 tests：`workflows/temp_override.rs`、`managers/general.rs`、`commands/{doctor,current,validate}.rs` 改写（组合调用 + struct literal + unused import 清理）
- [ ] `ccr-ui/src-tauri/src/commands/claude.rs:27-28` 注释更新
- [ ] 验证（全量门禁）：
  - `just version-check` && `just fmt-check`
  - `just lint-strict`
  - `just test`
  - `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml settings`
  - `just frontend-check-quick`
  - `rg -n 'struct ClaudeSettings' crates/ ccr-ui/` 唯一命中 ccr-types
- [ ] 提交 3

## Step 4 — 收尾

- [ ] trellis-update-spec：契约沉淀（ClaudeSettings 唯一归属、to_managed_env_pairs 映射唯一性、public-api 快照零改动的理由）
- [ ] 归档 task + journal

## 回滚点

- Step 1/2 后中断：纯新增，revert 单提交即净。
- Step 3 编译红：优先修调用点；若结构性受阻（如发现未侦查到的泛型 Validatable 消费），revert 提交 3 回到双 shape，问题回写 design.md 再进。
