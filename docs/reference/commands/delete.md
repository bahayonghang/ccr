# `delete` - 已弃用的顶层删除命令

> [!WARNING]
> **已弃用并已隐藏（7.x 兼容）**：顶层 `ccr delete <config_name> [--force]` 在 CCR `7.x` 中**已弃用**且在 `--help` 中**已隐藏**。调用时会向 `stderr` 输出弃用警告（不污染 `stdout`），计划于 `8.0` 移除。

## 规范替代命令

| 场景 | 规范替代命令 |
|---|---|
| 删除 Claude Profile | `ccr claude profile delete <name> [--force] [--json]` |
| 删除 Codex Profile | `ccr codex profile delete <name> [--force] [--json]` |
| 删除 Grok Profile | `ccr grok profile delete <name> [--force] [--json]` |

```bash
ccr claude profile delete temp-relay
ccr codex profile delete active-relay --force --json
```

## 关键行为差异

- **显式平台作用域**：旧 `ccr delete` 仅作用于历史 Claude Profile 目录；规范命令显式指定 `claude`、`codex` 或 `grok`。
- **激活项运行态保护与安全清理**：旧 `ccr delete` 在确认删除当前配置后仍会在宿主运行态遗留旧环境变量；规范命令默认拒绝删除当前激活的 Profile，显式传入 `--force` 时会先原子执行 `profile off` 清理宿主运行态与当前指针，再从 `profiles.db` 删除记录。

## 相关页面

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
