# `list` - 已弃用的顶层配置列表命令

> [!WARNING]
> **已弃用并已隐藏（7.x 兼容）**：顶层 `ccr list`（及别名 `ccr ls`）在 CCR `7.x` 中**已弃用**且在 `--help` 中**已隐藏**。调用时会向 `stderr` 输出弃用警告（不污染 `stdout`），计划于 `8.0` 移除。

## 规范替代命令

| 场景 | 规范替代命令 |
|---|---|
| 查看 Claude Profile 列表 | `ccr claude profile list [--json]` |
| 查看 Codex Profile 列表 | `ccr codex profile list [--json]` |
| 查看 Grok Profile 列表 | `ccr grok profile list [--json]` |
| 查看跨平台运行态摘要 | `ccr current [--verbose \| --json]`（别名 `ccr status`） |
| 查看已注册平台与激活指针 | `ccr platform list [--json]` |

```bash
ccr claude profile list
ccr codex profile list --json
ccr grok profile list
```

## 关键行为差异

- **显式平台作用域**：旧 `ccr list` 仅固定读取历史 Claude Profile 作用域；规范命令 `ccr <claude|codex|grok> profile list` 按平台独立列出对应 `profiles.db` 目录。
- **结构化 JSON 与乐观并发版本**：各平台 `profile list --json` 稳定返回 `{ "current_profile": "..." | null, "revision": "..." | null, "profiles": [...] }`，其中顶层 `revision` 可配合 `--expected-revision` 用于并发安全写入。

## 相关页面

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
- [`current`](./current)
- [`platform`](./platform)
