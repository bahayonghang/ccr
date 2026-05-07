# codex - Codex 运行时与多账号管理

`ccr codex` 是 Codex 平台的专项命令组，当前重点能力包括：

- `ccr codex auth ...`：official auth 多账号管理
- `ccr codex profile ...`：runtime/profile 路由管理
- `ccr codex sync-history ...`：修复 provider namespace 切换后的历史可见性

## 常用命令

```bash
ccr codex auth current
ccr codex auth list
ccr codex profile list
ccr codex profile switch <name>
ccr codex profile current
ccr codex profile off
```

## `auth` 与 `profile` 的区别

| 命令组 | 作用 |
|---|---|
| `ccr codex auth ...` | 保存、切换、导出、导入 official auth 账号 |
| `ccr codex profile ...` | 把某个 CCR profile 应用到 Codex runtime，或退出回到 official auth runtime |

## `profile` 当前支持面

- `list`
- `current`
- `switch <name>`
- `off`
- `create`
- `set-field`
- `enable`
- `disable`
- `delete`

## `sync-history`

保留原有用途：修复 `openai` / `custom` provider namespace 切换后，旧历史在 Codex CLI / App 中不可见的问题。
