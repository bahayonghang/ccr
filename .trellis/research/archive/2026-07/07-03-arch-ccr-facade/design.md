# Design: 收拢 ccr facade

> 依据：`research/inventory.md`（事实盘点）+ `research/feasibility.md`（否决式调研判定），2026-07-05。
> 调研结论：PRD 前提 1 被推翻（循环依赖），前提 2 部分推翻（删 3 移 1，且实际死依赖面 ~29），前提 3 成立（~58 删 / ~70 留）。本设计是缩水后的可行形状。

## PRD 勘误（以调研为准）

| PRD 说法 | 实际 |
|---|---|
| dispatch.rs 整体迁入 ccr-cli | 不可行：4 个 `#[cfg(feature = "tui")]` 分支调 `ccr_tui::tui::*`，而 ccr-tui 依赖 ccr-cli，整体迁移即 ccr-cli→ccr-tui→ccr-cli 循环。改为 **TUI 启动器注入式**迁移（见下） |
| 4 个死依赖 | ccr-skills/ccr-codex/ccr-sync 为真死依赖；**ccr-config 被 `tests/commands/{claude,codex}_profile.rs` 6 处消费**（`profile_to_section` 无 ccr:: 转发路径），需移入新增的 `[dev-dependencies]`。另有 ~25 个 src 0 引用的 [dependencies]（anyhow/chrono/reqwest/rayon/…），tests 额外用 indexmap/serde_json/tempfile |
| `crates/ccr/tests/commands/` 24 个集成测试 | 实为 10 个文件 / 54 个 `#[test]`（全目录 155 个 test fn），全部黑盒子进程测试（`CARGO_BIN_EXE_ccr`） |

## 决策 1：dispatch 迁移 —— TUI 启动器注入式（feasibility 形状 1）

- `crates/ccr/src/cli/dispatch.rs`（748 行）+ `crates/ccr/src/help.rs`（52 行，dispatch 17 处调用）整体迁入 `crates/ccr-cli/src/cli/`。
- 新增 `TuiLaunchers` 结构（ccr-cli 内，与 dispatch 同模块）：4 个 `fn() -> Result<(), CcrError>` 字段（main / codex_auth / opencode_auth / claude_auth，对应 dispatch.rs 原 L176/375/575/609）。
- `CommandDispatcher::dispatch` 增加 `Option<&TuiLaunchers>` 参数：`Some` → 调启动器；`None` → 走原 `#[cfg(not(feature = "tui"))]` 降级分支。**原 4 组 cfg 双分支改为运行时 Option 判断，两条路径始终参与编译**（消除 cfg 组合盲区）。
- `crates/ccr/src/main.rs`：`#[cfg(feature = "tui")]` 时构造 `TuiLaunchers { main: ccr_tui::tui::run_tui, … }` 注入，否则传 `None`。ccr 的 `tui` feature 与 ccr-tui optional 依赖保持不变。
- `crates/ccr/src/cli/mod.rs`：加转发 `pub use ccr_cli::cli::dispatch;`（含 `CommandDispatcher`、`handle_error`），main.rs 的 `ccr::cli::dispatch::handle_error` 路径不变。
- **lib.rs 一字不动**。快照守卫 `public_api_compat.rs` 只扫 lib.rs（`include_str!`，L117），cli/mod.rs 不在快照内 → 三个测试预期零变化。
- `handle_error` 随迁（唯一消费方 main.rs；与 `ccr-error-freeze.md:59` "渲染逻辑靠近 dispatch" 方向一致；CcrError 本体不动）。

被拒形状：拆两段（TUI 分支留 ccr）——"ccr 收缩为薄 main.rs"目标打折且路由测试缺 TUI 分支；反向搬 ccr-tui 脱离 ccr-cli——ccr-tui 消费 ccr-cli 约 50 个符号，不现实。

## 决策 2：路由直接测试的范围（诚实缩水）

dispatch 的 match 把路由选择与命令执行耦死（直接 await 真实 `*_command`，写真实文件系统），`handle_error` 调 `process::exit`。**不做**全量可注入执行器改造（110+ 命令分支，成本远超收益，dispatch 3 个月零新增分支）。直接测试范围：

1. `crates/ccr-cli/tests/dispatch_routing.rs`（新增）：
   - 4 个 TUI 入口的注入路由：注入记录型启动器（static AtomicBool），断言 `ccr`（无子命令）/`ccr codex`/`ccr opencode`/`ccr claude` 各自命中正确启动器；`None` 时走降级分支不 panic。
   - 纯输出分支（Version 等）直接调 `CommandDispatcher::dispatch` 断言 Ok。
   - 若干只读命令经 tempdir 隔离 `CCR_ROOT`/`HOME` 的进程内集成测试（不再依赖 ccr 层子进程）。
2. ccr 层既有黑盒测试（legacy_routing.rs 等 155 个）原样保留作为端到端兜底。

## 决策 3：死依赖清理 —— 全量口径

验收标准 2 措辞是"无死依赖"，按全量执行而非只删 PRD 点名的 4 个：

- `[dependencies]` 收敛到 src 真实引用集：`ccr-cli`、`ccr-core`、`ccr-store`（lib.rs:306-310）、`ccr-tui`(optional)、`clap`、`tokio`；其余 ~25 个全删。
- 新增 `[dev-dependencies]`：`ccr-config`、`indexmap`、`serde_json`、`tempfile`（tests 实际引用，见 inventory B）。
- feature 段（`tui = ["ccr-tui"]` 等）与平台条件依赖按删除结果同步核对。
- 验证口径：人工 rg（inventory B 的复现命令）+ `cargo check -p ccr`（含 `--no-default-features`，src-tauri 以该形态依赖 ccr）。

## 决策 4：re-export 墙瘦身 —— 按 inventory C8 白名单执行

- 删除 ~58 个无墙路径消费方的符号（models 8、managers 14、services ~36）；多数藏在分组 `pub use` 内，操作是**改组不是删行**。
- 保留 ~70 个，消费方以 `research/inventory.md` C8 表为盘点文档（验收标准 3 的"注释或盘点文档"取盘点文档；mod.rs 顶部加一行注释指向 spec 的墙规则，不逐条加注释）。
- 陷阱（inventory 已标注）：
  - `OpenCodeReadSnapshot` 双路径导出，只删 services 副本，models 副本被 ccr-tui 消费。
  - 快照/legacy 锁定项（managers 16、services 6+`codex_session_service` 模块行、models Platform 四件套 + OpenAiAuthMethod + skills 全组、platforms 5、sync 组 13）不动。
  - src-tauri install 流全走 `install_service::`/`install_types::` 模块路径，services 平铺 install 组 11 个可安全删除。
- `crates/ccr-tui/src/lib.rs:3` 别名墙（`pub use ccr_cli::{models, platforms, services}`）0 外部消费，删除；TUI 内部 ~15 个文件 `crate::models/…` 改 `ccr_cli::…` 直连。`pub mod tui` 保留（`ccr::lib.rs:150` 桥 + 快照锁定）。

## 兼容与守卫

- `public-api-boundary.md` 冻结不破：lib.rs 不动、`ccr::cli` 同名路径继续可达、prelude 形状不变（`CcrError`/`Result` 按 ccr-error-freeze ADR 不动）。
- 每步硬门槛：`cargo check --manifest-path ccr-ui/src-tauri/Cargo.toml` 通过；`cargo test -p ccr --test public_api_compat` 3/3 零快照变化。
- 回滚形状：四个决策各自独立成 commit（依赖清理 / dispatch 迁移 / 墙瘦身 / tui 别名），任一步出问题单独 revert。

## Spec 回写（3.3 阶段）

- `public-api-boundary.md`：dispatch/help 新位置、ccr 依赖收敛结果、墙条目"新增 re-export 必须指出消费方"规则、7.0 breaking 候选登记（删除 ccr 兼容桥）。
- 若瘦身中发现新锁定项，回写 inventory 勘误。
