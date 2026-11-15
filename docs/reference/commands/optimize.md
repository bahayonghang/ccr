# `optimize` - 已弃用的顶层配置排序命令

> [!WARNING]
> **已弃用并已隐藏（7.x 兼容）**：顶层 `ccr optimize` 在 CCR `7.x` 中**已弃用**且在 `--help` 中**已隐藏**。调用时会向 `stderr` 输出弃用警告（不污染 `stdout`），计划于 `8.0` 移除。

## 规范替代命令

| 场景 | 规范替代方式 |
|---|---|
| 日常持久化存储顺序维护 | 无需手动执行：SQLite 目录库 `~/.ccr/platforms/<platform>/profiles.db` 自动维护规范结构 |
| 在编辑器中检视或手动调整规范 TOML 投影 | `ccr <claude\|codex\|grok> profile open [--json]` |
| 导出确定性排序的可交换 TOML 文件 | `ccr <claude\|codex\|grok> profile export [--no-secrets] [-o <FILE>]` |
| 跨平台只读校验配置完整性 | `ccr validate [--json]` |

## 关键行为差异

- **SQLite 权威目录无需手工重排**：旧 `ccr optimize` 用于将历史单文件 Claude TOML 配置节按字母重排写回磁盘；`7.x` 起三平台均以 SQLite `profiles.db` 为唯一权威存储，天然保持规范化结构。
- **投影与导出始终确定性输出**：无论是通过 `ccr <claude|codex|grok> profile open` 打开临时投影，还是通过 `profile export` 导出 `ExchangeV1` 文件，CCR 都会直接生成规范化 TOML，不再需要单独的优化步骤。

## 相关页面

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
- [`validate`](./validate)
