# `claude` - Claude Auth 与 Profile Runtime

`ccr claude` 管理两条相互独立的 Claude Code runtime：官方订阅账号快照和 API-key profile。

## 入口

```bash
ccr claude
ccr claude help
ccr help claude auth
ccr help claude profile
```

不带子命令时，支持 TUI 的构建进入 Claude Auth 标签；无 TUI launcher 的调用退化为账号列表。

## Official Auth

| 命令 | 说明 |
|---|---|
| `ccr claude auth save <name>` | 保存当前官方登录快照 |
| `ccr claude auth list` | 列出已保存账号 |
| `ccr claude auth switch <name>` | 切换官方账号 |
| `ccr claude auth delete <name>` | 删除账号；`--force` 跳过确认 |
| `ccr claude auth current` | 显示当前官方登录；支持 `--json` |

`save` 支持 `--description <text>` 和 `--force`。Auth snapshot 不等同于 API token profile。

## Profile Runtime

| 命令 | 说明 |
|---|---|
| `ccr claude profile current` | 当前 profile/runtime；支持 `--json` |
| `ccr claude profile list` | 列出 profiles；支持 `--json` |
| `ccr claude profile switch <name>` | 应用指定 profile |
| `ccr claude profile create <name>` | 创建 profile |
| `ccr claude profile set-field <name> <field>` | 更新或清空单个字段 |
| `ccr claude profile enable <name>` | 启用 profile |
| `ccr claude profile disable <name>` | 禁用 profile；当前项需要 `--force` |
| `ccr claude profile delete <name>` | 删除 profile；支持 `--force` |
| `ccr claude profile open` | 用 $VISUAL/$EDITOR 或系统关联程序打开 profiles.toml；文件不存在时先从模板创建 |
| `ccr claude profile off` | 退出 profile mode，回到 official auth runtime |

创建第三方 API profile：

```bash
ccr claude profile create work \
  --base-url https://api.example.com \
  --auth-token "$ANTHROPIC_AUTH_TOKEN" \
  --model claude-sonnet-4-5 \
  --auth-mode api_key

ccr claude profile switch work
ccr claude profile current --json
```

`create` 还支持 description、small-fast-model、provider、provider-type、account、重复 `--tag`、`--disabled` 和 `--json`。用 `ccr help claude profile create` 查看当前完整选项。

`set-field` 接受 `--value`、适用于数组的 `--value-json`，或 `--clear`；三者互斥。

## Auth Mode 边界

- `subscription` profile 清理 CCR 管理的 `ANTHROPIC_*` 和相关 Claude Code 环境覆盖。
- `api_key` profile 将类型化字段写入 `~/.claude/settings.json.env`。
- 第三方 profile 应使用 `api_key`；base URL 与 auth token 同时存在时，CCR 会按运行时规则纠正明显失配的旧配置。
- 输出和诊断不得打印完整 token。

## 相关页面

- [配置模型](/guide/configuration)
- [CLI 工作流](/guide/cli-workflows)
- [`current`](./current)
- [`doctor`](./doctor)
