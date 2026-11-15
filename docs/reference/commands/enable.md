# `enable` - 已弃用的顶层启用命令

> [!WARNING]
> **已弃用并已隐藏（7.x 兼容）**：顶层 `ccr enable <config_name>` 在 CCR `7.x` 中**已弃用**且在 `--help` 中**已隐藏**。调用时会向 `stderr` 输出弃用警告（不污染 `stdout`），计划于 `8.0` 移除。

## 规范替代命令

| 场景 | 规范替代命令 |
|---|---|
| 启用 Claude Profile | `ccr claude profile enable <name> [--json]` |
| 启用 Codex Profile | `ccr codex profile enable <name> [--json]` |
| 启用 Grok Profile | `ccr grok profile enable <name> [--json]` |
| 为缺密钥导入项补录凭据并同时启用 | `printf '%s' "$TOKEN" \| ccr <claude\|codex> profile edit <name> --auth-token-stdin --enable`（Grok 用 `--api-key-stdin`） |

## 关键行为差异

- **显式平台作用域**：旧 `ccr enable` 仅固定操作历史 Claude Profile 作用域；规范命令显式指定 `claude`、`codex` 或 `grok` 并支持 `--json` 输出。
- **启用前领域完整性校验**：规范命令 `ccr <platform> profile enable <name>` 在平台操作锁内先执行领域校验再写入 `enabled = true`；若条目缺少必需密钥（如来自 `--no-secrets` 导出的模板），会拒绝启用并提示先通过 `profile edit` 补录密钥。启用后如需立即投入使用，请继续执行 `ccr <platform> profile switch <name>`。

## 相关页面

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
