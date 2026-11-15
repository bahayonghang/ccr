# `disable` - 已弃用的顶层禁用命令

> [!WARNING]
> **已弃用并已隐藏（7.x 兼容）**：顶层 `ccr disable <config_name> [--force]` 在 CCR `7.x` 中**已弃用**且在 `--help` 中**已隐藏**。调用时会向 `stderr` 输出弃用警告（不污染 `stdout`），计划于 `8.0` 移除。

## 规范替代命令

| 场景 | 规范替代命令 |
|---|---|
| 禁用 Claude Profile | `ccr claude profile disable <name> [--force] [--json]` |
| 禁用 Codex Profile | `ccr codex profile disable <name> [--force] [--json]` |
| 禁用 Grok Profile | `ccr grok profile disable <name> [--force] [--json]` |
| 先退出当前激活运行态再禁用 | `ccr <claude\|codex\|grok> profile off` 后执行 `ccr <claude\|codex\|grok> profile disable <name>` |

## 关键行为差异

- **显式平台作用域与结构化输出**：旧 `ccr disable` 仅作用于历史 Claude Profile 作用域；规范命令显式指定 `claude`、`codex` 或 `grok`，并支持 `--json` 输出 `{ ok, platform, name, message, enabled, current_profile }`。
- **激活状态保护**：`profile disable` 默认拒绝禁用当前激活的 Profile（除非附加 `--force`）；而 `profile edit <name> --disable` 始终拒绝将当前激活项设为禁用。推荐先执行 `profile switch <other>` 或 `profile off` 再禁用旧条目。

## 相关页面

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
