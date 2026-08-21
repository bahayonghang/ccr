# OpenCode Auth 表面（删除范围证据）

调研日期：2026-08-20。只记录仓库事实。

## TUI

- 独立入口：`ccr opencode` 无子命令 → `TuiLaunchers.opencode_auth` → `ccr::tui::opencode_auth::run_opencode_auth_tui`（`crates/ccr/src/main.rs`、`cli/dispatch.rs:650`）
- 主 TUI 页签：`TabVariant::OpenCodeAuth`，挂在 Codex 平台旁（`ccr-tui/src/tui/app.rs`）
- 模块：`crates/ccr-tui/src/tui/opencode_auth/{mod,app,ui}.rs`
- 页签内按 `i` 预览/确认 `import-codex`

## CLI

| 命令 | 作用 |
| --- | --- |
| `ccr opencode` | 启动 OpenCode Auth TUI；无 TUI launcher 时打印帮助 |
| `ccr opencode help` / `ccr help opencode` | 帮助 |
| `ccr opencode auth help` | Auth 帮助 |
| `ccr opencode auth import-codex [--dry-run] [--json]` | 把已保存的兼容 Codex OAuth 账号增量导入 OpenCode registry |

Clap：`OpenCodeAction` 只有 `Help` 和 `Auth`；`OpenCodeAuthAction` 只有 `Help` 和 `ImportCodex`（`cli/subcommands/opencode.rs`）。

处理器：`crates/ccr-cli/src/commands/opencode/auth/import_codex.rs`。服务：`crates/ccr-codex/src/services/opencode_auth_service.rs`。

## 非 TUI 表面

- ccr-ui 的 OpenCode 模块是 providers / MCP / agents / settings（`/opencode/...`），不是这套 Auth TUI。
- VS Code 无 opencode auth 包装。
- 文档：`docs/reference/commands/opencode.md`、`docs/guide/cli-workflows.md` 工作流 6。

## 删除时至少要动的命令入口

去掉 TUI 页后，`ccr opencode` 无子命令的 TUI 启动器和 `TuiLaunchers.opencode_auth` 会变成死入口，必须改掉（删除命令、改为帮助、或改为别的子命令）。

`import-codex` 是独立 CLI，TUI 只是调用同一服务。产品决定 D7：连同命令组删除。

Tauri/ccr-ui 的 `opencode_*` 命令只读写 `opencode.json` / agents / MCP 等，不调用 `OpenCodeAuthService`。删除 CLI 命令组不会拆掉 `/opencode` 配置页。

仅被 Auth TUI 使用、删除后面成为孤儿的模块：

- `crates/ccr-codex/src/services/opencode_auth_service.rs`
- `crates/ccr-codex/src/services/opencode_quota_service.rs`
- `crates/ccr-codex/src/services/opencode_usage_service.rs`
- `crates/ccr-codex/src/models/opencode_auth.rs`（若无其它引用）

