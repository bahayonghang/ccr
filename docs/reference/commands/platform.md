# platform - 平台注册表兼容视图

`ccr platform` 现在只保留注册表视图语义，不再承担 auth/profile 路由主路径。

## 当前推荐用法

```bash
ccr platform list
ccr current
```

- `ccr platform list`：查看平台注册表、启用状态、每个平台的 `current_profile`
- `ccr current`：查看 Claude Runtime / Codex Runtime 实际状态

## 已退休子命令

以下子命令会返回迁移指引，而不是继续作为主路径执行：

- `ccr platform switch <platform>`
- `ccr platform current`
- `ccr platform info <platform>`
- `ccr platform init <platform>`
- `ccr platform profile ...`

## 迁移映射

| 旧路径 | 新路径 |
|---|---|
| `ccr platform switch claude` | `ccr claude profile switch <name>` 或 `ccr claude auth ...` |
| `ccr platform switch codex` | `ccr codex profile switch <name>` 或 `ccr codex auth ...` |
| `ccr platform switch grok` | `ccr grok profile switch <name>` |
| `ccr platform current` | `ccr current` |
| `ccr platform init claude` | `ccr claude profile init` |
| `ccr platform init codex` | `ccr codex profile init` |
| `ccr platform init grok` | `ccr grok profile init` |
| `ccr platform profile create claude ...` | `ccr claude profile create ...` |
| `ccr platform profile create codex ...` | `ccr codex profile create ...` |
| `ccr platform profile create grok ...` | `ccr grok profile create ...` |

## 说明

- 旧 registry 中的 `default_platform` / `current_platform` 仍可兼容读取。
- 但当前 auth/profile 路由真相是各平台自己的 `current_profile`。

## 相关页面

- [current](./current)
- [CLI 工作流](/guide/cli-workflows)
- [迁移指南](/reference/migration)
