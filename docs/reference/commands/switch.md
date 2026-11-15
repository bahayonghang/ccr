# `switch` - 已退休的旧 Profile 切换入口

> [!WARNING]
> **已退休并已隐藏（7.x 兼容拦截）**：顶层 `ccr switch <config_name>` 与裸快捷入口 `ccr <config_name>` 在 CCR `7.x` 中**已退休**且从 `--help` 中**已隐藏**。调用时会直接返回迁移引导错误并以非零状态退出，不会修改任何平台运行态，计划于 `8.0` 彻底移除。

## 规范替代命令

请显式指定目标平台（`claude`、`codex` 或 `grok`）：

```bash
ccr claude profile switch <name> [--json]
ccr codex profile switch <name> [--json]
ccr grok profile switch <name> [--json]
```

| 已退休入口 | 规范替代命令 |
|---|---|
| `ccr switch <name>`（Claude 配置） | `ccr claude profile switch <name> [--json]` |
| `ccr switch <name>`（Codex 配置） | `ccr codex profile switch <name> [--json]` |
| `ccr switch <name>`（Grok 配置） | `ccr grok profile switch <name> [--json]` |
| 裸位置参数 `ccr <name>` | `ccr <claude\|codex\|grok> profile switch <name>` |

## 关键行为差异

- **消除隐式全局平台推断**：旧入口依赖历史全局 `current_platform` 猜测目标平台，在 Claude Code、Codex 与 Grok Build 并行运行时极易误改错误的宿主配置。
- **结构化回执与原子事务**：`ccr <claude|codex|grok> profile switch <name>` 独立维护各自平台的运行态文件与 `profiles.db` 激活指针，并支持 `--json` 单行机器可读回执。

## 相关页面

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
- [`current`](./current)
- [`platform`](./platform)
- [迁移指南](/reference/migration)
