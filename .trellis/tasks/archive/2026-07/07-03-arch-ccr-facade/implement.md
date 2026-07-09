# Implement: 收拢 ccr facade

> 前置阅读顺序：`prd.md` → `design.md` → `research/inventory.md`（操作白名单/复现命令）。
> 每步结束必须过该步的验证门槛才能进入下一步；每步独立 commit（回滚点）。

## Step 1 — ccr 依赖收敛（design 决策 3）

- [ ] `crates/ccr/Cargo.toml`：`[dependencies]` 收敛到 `ccr-cli / ccr-core / ccr-store / ccr-tui(optional) / clap / tokio`；删除其余（含 ccr-config、ccr-skills、ccr-codex、ccr-sync 与 ~25 个 src 0 引用项）。
- [ ] 新增 `[dev-dependencies]`：`ccr-config`、`indexmap`、`serde_json`、`tempfile`（版本沿用 workspace 写法，对照其他 crate）。
- [ ] 核对 `[features]` 与平台条件依赖段是否引用了被删项。
- 验证：
  - [ ] `rg -n 'ccr_(config|skills|codex|sync)::' crates/ccr/src` → 0
  - [ ] `cargo check -p ccr` 与 `cargo check -p ccr --no-default-features`
  - [ ] `cargo test -p ccr -- --test-threads=1`（含 claude_profile/codex_profile 两个 dev-dep 消费方）
  - [ ] `cargo check --manifest-path ccr-ui/src-tauri/Cargo.toml`
- Commit: `refactor(ccr): 🧹 收敛 Cargo.toml 依赖到真实引用集`

## Step 2 — dispatch/help 迁入 ccr-cli + TUI 启动器注入（design 决策 1）

- [ ] `crates/ccr/src/help.rs` → `crates/ccr-cli/src/cli/help.rs`（依赖仅 clap + build_cli_command，改 `crate::` 路径）。
- [ ] `crates/ccr/src/cli/dispatch.rs` → `crates/ccr-cli/src/cli/dispatch.rs`：
  - `crate::commands/services/cli::` 路径原样成立（ccr-cli 本地路径）；`crate::help` 改新位置。
  - 新增 `pub struct TuiLaunchers`（4 个 `fn() -> ccr_core::core::error::Result<()>` 字段）。
  - `CommandDispatcher::dispatch` 加 `Option<&TuiLaunchers>` 参数；原 4 组 `#[cfg(feature = "tui")]`/`not` 双分支（L176/375/575/609）改为运行时 `match launchers`，两路径均编译。
- [ ] `crates/ccr-cli/src/cli/mod.rs` 挂载 `dispatch`/`help` 模块并导出。
- [ ] `crates/ccr/src/cli/mod.rs`：删本地模块声明，转发 `pub use ccr_cli::cli::dispatch;`（保住 `ccr::cli::dispatch::handle_error`、`CommandDispatcher` 路径）；`crates/ccr/src/help.rs`、`crates/ccr/src/cli/dispatch.rs` 删除。
- [ ] `crates/ccr/src/main.rs`：按 feature 构造 `TuiLaunchers`（`ccr_tui::tui::run_tui` 等 4 个）注入 dispatch；**lib.rs 一字不动**。
- 验证：
  - [ ] `cargo check --workspace` + `cargo check -p ccr --no-default-features`
  - [ ] `cargo test -p ccr --test public_api_compat` 3/3，零快照变化
  - [ ] `cargo test -p ccr -- --test-threads=1`（155 test，黑盒行为不变）
  - [ ] `cargo check --manifest-path ccr-ui/src-tauri/Cargo.toml`
- Commit: `refactor(cli): 🏗️ dispatch/help 迁入 ccr-cli，TUI 启动器注入解循环`

## Step 3 — 路由直接测试（design 决策 2）

- [ ] 新增 `crates/ccr-cli/tests/dispatch_routing.rs`：
  - 4 个 TUI 入口注入路由测试（记录型启动器 + AtomicBool，断言命中正确启动器）。
  - `launchers = None` 时降级分支不 panic（4 个入口）。
  - Version 等纯输出分支 `dispatch` 返回 Ok。
  - 2-3 个只读命令的进程内集成测试（tempdir 隔离 `CCR_ROOT`/`HOME`）。
- 验证：
  - [ ] `cargo test -p ccr-cli -- --test-threads=1` 全绿
- Commit: `test(cli): ✅ dispatch 路由直接测试（TUI 注入 + 降级 + 纯输出分支）`

## Step 4 — re-export 墙瘦身（design 决策 4，严格按 inventory C8 白名单）

- [ ] `crates/ccr-cli/src/models/mod.rs`：删 8 个（codex_auth 组 5 + opencode_auth 组 3；改组不删行）。
- [ ] `crates/ccr-cli/src/managers/mod.rs`：删 14 个（含 `ccr_sync::{SyncConfig, SyncConfigManager}` 整行、McpPreset 三件套等）。
- [ ] `crates/ccr-cli/src/services/mod.rs`：删 ~36 个（ccr_codex 大组 17、install 平铺组 11、doctor 平铺 5、UiService/SyncService/ClaudeAuthReadSnapshot/CodexSessionInventory 平铺）。
- [ ] 陷阱核对：`OpenCodeReadSnapshot` 只删 services 副本；锁定项（inventory C8 "锁定"标记）一律不动。
- [ ] 三个 mod.rs 顶部加一行中文注释：保留条目的消费方盘点见 spec（3.6 步回写的墙规则）。
- 验证：
  - [ ] `cargo check --workspace`
  - [ ] `cargo check --manifest-path ccr-ui/src-tauri/Cargo.toml`（install 流走模块路径，预期不断）
  - [ ] `cargo test -p ccr --test public_api_compat` 3/3
  - [ ] `cargo test -p ccr -p ccr-cli -p ccr-tui -- --test-threads=1`
- Commit: `refactor(cli): 🧹 re-export 墙瘦身：删除 58 个无消费方条目`

## Step 5 — ccr-tui 别名墙清理

- [ ] 删 `crates/ccr-tui/src/lib.rs:3`（`pub use ccr_cli::{models, platforms, services}`）；保留 `pub mod tui`。
- [ ] TUI 内部 ~15 个文件 `crate::{models,services,platforms}::` 改 `ccr_cli::` 直连。
- 验证：
  - [ ] `cargo check --workspace`；`cargo test -p ccr-tui -- --test-threads=1`
  - [ ] `rg -n 'crate::(models|services|platforms)::' crates/ccr-tui/src` → 0
- Commit: `refactor(tui): 🧹 移除 ccr-cli 别名墙，改直接 import`

## Step 6 — 全量验证（2.2 最后一轮全量检查）

- [ ] `just version-check` → `just fmt-check`（若 fmt 修复过文件，检查 diff）
- [ ] `just lint-strict`
- [ ] `just test`
- [ ] `cargo check --manifest-path ccr-ui/src-tauri/Cargo.toml`
- [ ] 验收标准逐条对照 prd.md 勾选（含勘误后的口径：删 3 移 1 + 全量收敛、10 文件/54 黑盒测试全绿）。

## Step 7 — Spec 回写 + 收尾（Phase 3）

- [ ] `public-api-boundary.md`：dispatch/help 新位置与注入形状、ccr 依赖收敛结果、墙规则（新增 re-export 必须指出消费方）、7.0 breaking 候选登记（删除 ccr 兼容桥）。
- [ ] 归档 task、journal 记录、按 git-commit 约定分笔提交。

## 回滚点

Step 1/2/4/5 各自独立 commit；任一步验证失败先修复，修不动则 revert 该步 commit 并把原因记入 journal 与 design.md。
