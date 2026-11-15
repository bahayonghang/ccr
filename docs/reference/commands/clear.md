# `clear` - 已弃用的顶层设置清理命令

> [!WARNING]
> **已弃用并已隐藏（7.x 兼容）**：顶层 `ccr clear [--force]` 在 CCR `7.x` 中**已弃用**且在 `--help` 中**已隐藏**。调用时会向 `stderr` 输出弃用警告（不污染 `stdout`），计划于 `8.0` 移除。

## 规范替代命令

| 场景 | 规范替代命令 |
|---|---|
| 退出 Claude Profile 模式并清理 CCR 写入的运行态覆盖 | `ccr claude profile off [--json]` |
| 退出 Codex Profile 模式并清理路由与运行期 `auth.json` | `ccr codex profile off [--json]` |
| 退出 Grok Profile 模式并恢复进入前的模型/推理配置 | `ccr grok profile off [--json]` |
| 登出官方运行时登录会话（与 `profile off` 独立） | `ccr <claude\|codex\|grok> auth off [--json]` |
| 清理临时 Token 覆盖 | `ccr temp-token clear` |

```bash
ccr claude profile off --json
ccr codex profile off
ccr grok profile off
```

## 关键行为差异

- **覆盖三平台并同步清理激活指针**：旧 `ccr clear` 仅清空 Claude `settings.json` 中的环境变量而不重置平台 Profile 指针；规范命令 `ccr <claude|codex|grok> profile off` 同时清空平台 `current_profile` 指针并恢复对应宿主运行态（写入前建立事务快照，中途失败自动回滚）。
- **外部认证压制源诊断**：`ccr claude profile off --json` 除返回 `{ ok, changed, previous_profile, runtime_mode, warnings }` 外，还通过 `remaining_suppressors` 列出 CCR 不会自动删除但仍会压制官方登录的外部凭据来源（如进程环境变量）。

## 相关页面

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
- [`temp-token`](./temp-token)
