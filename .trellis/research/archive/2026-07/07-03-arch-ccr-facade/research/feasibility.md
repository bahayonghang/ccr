# Research: ccr facade 收拢 — 可行性判定与否决点分析（feasibility）

- **Query**: 否决式调研——尝试推翻 prd.md 三个前提
- **Scope**: internal
- **Date**: 2026-07-05
- **证据基础**: 同目录 `inventory.md`（含全部复现命令）

---

## (a) dispatch 迁移是否存在硬否决点 —— **是，循环依赖成立，"整体迁入"被否决**

- 依赖方向事实：`ccr-tui → ccr-cli`（ccr-tui/Cargo.toml:16）。
- dispatch.rs 在 4 个分支（L176 / L375 / L575 / L609，均 `#[cfg(feature = "tui")]`）调用 `crate::tui::*`，经 `ccr::lib.rs:150` 实为 **`ccr_tui::tui::*`**。
- 因此 "dispatch.rs 748 行整体迁入 ccr-cli" 会构成 `ccr-cli → ccr-tui → ccr-cli`，cargo 拒绝编译。**PRD 第 1 条要求按字面执行不可行。**

反向搬（让 ccr-tui 不依赖 ccr-cli）不现实：ccr-tui 消费 ccr-cli 约 50 个符号（inventory C7），且 ccr-tui 的四个 TUI 入口内部大量调用 ccr-cli services。

### 可行的缩水形状（判定，非实施计划）

TUI 耦合点极小且形状统一——4 个入口全是 `fn() -> Result<(), CcrError>`，其余 744 行只依赖 ccr_cli + ccr_core。两种皆可行：

1. **注入式（dispatch 全部迁入 ccr-cli）**：dispatch 接受一个 TUI 启动器参数（4 个 `fn() -> Result<(), CcrError>` 的 struct 或 trait，non-tui 构建给 None/降级闭包），ccr 的 main.rs 负责把 `ccr_tui::tui::{run_tui, codex_auth::…, opencode_auth::…, claude_auth::…}` 注入。ccr-cli 已有现成的空 `tui` feature（definitions.rs:69 `is_tui_mode` 在用），cfg 语义可对齐。
2. **拆两段（路由入 ccr-cli，TUI 分支留 ccr）**：非 TUI 路由（约 95% 行数）迁 ccr-cli；ccr 侧保留一个 ~40 行薄 wrapper 先截获 4 个 TUI 分支再委托。改动更小但 "ccr 收缩为薄 main.rs" 目标打折，且路由测试仍缺 TUI 分支。

两种形状下均需同迁 `crates/ccr/src/help.rs`（52 行，dispatch 17 处调用；只依赖 clap + build_cli_command，无阻碍）。

### 其余迁移风险点（非否决）

- `handle_error`：唯一消费方 main.rs；`ccr::cli::dispatch::handle_error` 不在任何快照/测试锁定范围（快照只扫 lib.rs）；ccr-error-freeze spec L59 本就建议渲染逻辑靠近 dispatch。迁走**无公开面影响**。
- 快照守卫：只要 lib.rs 不动、`ccr::cli` 模块（cli/mod.rs）继续 re-export `CommandDispatcher`/`handle_error` 同名路径，`public_api_compat` 三个测试均不受影响。
- tests/commands 全部黑盒子进程（`CARGO_BIN_EXE_ccr`），dispatch 搬家零影响。
- "为路由补直接测试" 的真实成本：dispatch 当前把路由选择与命令执行耦在同一 match（直接 await 真实 *_command，写真实文件系统），且 handle_error 调 `process::exit`。迁移本身不解耦；要单测路由需额外引入可注入执行器或只测纯输出分支——验收标准 1 的工作量主要在这里，而不在搬文件。

## (b) 若否决，缩水形状

见上：**推荐形状 1（TUI 启动器注入）**，它同时满足 "dispatch 位于 ccr-cli" 与 "ccr 收缩为薄 main.rs"；形状 2 是保守回退。无论哪种，`ccr` crate 保留：main.rs、lib.rs（一字不动）、cli/mod.rs（转发层，可能加一行转发 dispatch）。

## (c) 死依赖前提真伪 —— **3 真 1 伪（部分推翻）**

| PRD 声称 | 判定 |
|---|---|
| ccr-skills / ccr-codex / ccr-sync 死依赖 | **真**：src + tests 均 0 引用，可直接从 [dependencies] 删除（类型可达性经 ccr-cli 传递，lib.rs 无直连） |
| ccr-config 死依赖 | **伪**：`tests/commands/{claude,codex}_profile.rs` 6 处直接使用（`ProfileConfig` + `profile_to_section`，后者无任何 ccr:: 转发路径可达）。ccr 目前没有 [dev-dependencies] 段——需新增该段并把 ccr-config 移入，或改写测试。"删除 4 个死依赖" 的验收措辞需要修正为 "删 3 + 移 1" |

**附带发现（PRD 低估了规模）**：ccr src 实际只引用 6 个 extern（ccr-cli、ccr-core、ccr-store、ccr-tui[optional]、clap、tokio）；[dependencies] 里另有 **~25 个 src 0 引用**的依赖（anyhow、chrono、serde、reqwest、rayon、reqwest_dav、sysinfo…），tests 用到其中 indexmap/serde_json/tempfile（同样需转 dev-deps）。若目标是 "无死依赖"（验收标准 2 用 cargo-udeps 验证），清理面是 ~29 个而不是 4 个。这不改变可行性，但显著扩大验收标准 2 的实际范围，design.md 需要明确是否收窄到 PRD 点名的 4 个。

## (d) 墙瘦身安全清单规模

以 "仅删除无任何墙路径消费方的条目" 为口径（inventory C8 全表）：

- **可删约 58 个符号**：models 8（CodexAuthExport、CodexAuthExportAccount、CodexProfileSecret、CodexProfileSecretStore、normalize_auth_map_for_intent、OpenCodeAuthAccount、OpenCodeCurrentAuthInfo、OpenCodeOpenAiAuth）；managers 14（含 `ccr_sync::{SyncConfig, SyncConfigManager}` 整行、McpPreset 三件套、Cached* 两个、ConfigValidator/ValidationReport/ConfigFileHandler、TuiConfig、EnvChange/HistoryStats、SyncFolderManager）；services ~36（ccr_codex 大组 17 个、doctor 平铺 5 个、install 平铺 11 个、UiService、SyncService、ClaudeAuthReadSnapshot、CodexSessionInventory 平铺行）。
- **必须保留约 70 个**，其中 **快照/legacy 测试锁定不可动**（除非走有意快照更新）：managers 16 项、services 6 项 + `codex_session_service` 模块行、models 的 `ccr_config` 四件套 + OpenAiAuthMethod + skills 全组、platforms 5 项、sync 组 13 项——这些正是 lib.rs 桥的原料，PRD 约束（桥冻结）已正确预判。
- 陷阱提示：多数可删符号藏在**分组 `pub use ccr_codex::…::{…}` 行内**，删除是"改组"不是"删行"；且 `OpenCodeReadSnapshot` 在 models 与 services 各出现一次（tui 走 models 路径，services 副本才可删）。src-tauri 的 install 流全部走模块路径（`install_service::` / `install_types::`），services 平铺 install 组可安全删除，`cargo check --manifest-path ccr-ui/src-tauri/Cargo.toml` 不会断。
- ccr-tui/src/lib.rs:3 别名（`pub use ccr_cli::{models, platforms, services}`）：全仓 0 外部消费，可删；代价是 TUI 内部 ~15 个文件把 `crate::models/…` 改为 `ccr_cli::…` 直连。

## 三前提总判定

| PRD 前提 | 判定 |
|---|---|
| 1. dispatch 整体迁 ccr-cli | **被推翻（循环依赖）**，但注入式缩水形状可达成同等目标 |
| 2. 4 个死依赖 0 引用可删 | **部分推翻**：3 个成立；ccr-config 被 tests 消费，需 dev-deps 迁移；且实际死依赖面 ~29 个，远大于 4 |
| 3. 墙上存在大量无消费 re-export 可瘦身 | **成立**：~58 删 / ~70 留，且 PRD 对桥冻结的约束与快照机制吻合，无冲突 |
