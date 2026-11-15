# `add` - 已弃用的顶层交互添加命令

> [!WARNING]
> **已弃用并已隐藏（7.x 兼容）**：顶层 `ccr add` 在 CCR `7.x` 中**已弃用**且在 `--help` 中**已隐藏**。调用时会向 `stderr` 输出弃用警告（不污染 `stdout`），计划于 `8.0` 移除。

## 规范替代命令

| 场景 | 规范替代命令 |
|---|---|
| 非交互创建 Claude Profile | `printf '%s' "$ANTHROPIC_AUTH_TOKEN" \| ccr claude profile create <name> --base-url <url> --model <model> --auth-mode api_key --auth-token-stdin` |
| 非交互创建 Codex Profile | `printf '%s' "$OPENAI_API_KEY" \| ccr codex profile create <name> --base-url <url> --model <model> --auth-mode openai_api_key --auth-token-stdin` |
| 非交互创建 Grok Profile | `printf '%s' "$GROK_API_KEY" \| ccr grok profile create <name> --base-url <url> --model <model> --api-key-stdin`（或 `--env-key <ENV_VAR>`） |
| 终端交互式 TUI 新建与管理 | `ccr`（或 [`tui`](./tui)） |
| 用编辑器批量编辑目录投影 | `ccr <claude\|codex\|grok> profile open` |

## 关键行为差异

- **显式三平台覆盖与非交互自动化**：旧 `ccr add` 仅支持交互式向导且固定写入历史 Claude 作用域；`ccr <claude|codex|grok> profile create <name>` 统一覆盖三平台并支持 `--expected-revision` 与 `--json` 结构化回执。
- **安全的标准输入密钥录入**：规范命令通过 `--auth-token-stdin` / `--api-key-stdin`（或 Grok `--env-key`）接收凭据，避免密钥暴露在命令行历史与进程列表中，且新建条目保持非激活状态直至显式执行 `profile switch`。

## 相关页面

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
- [`tui`](./tui)
