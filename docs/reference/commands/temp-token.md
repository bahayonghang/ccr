# temp-token - 命令式临时覆盖

`ccr temp-token` 用于以命令行方式临时覆盖当前活动 Claude settings（`~/.claude/settings.json`）中的 Token、Base URL 和模型。

## 用法

```bash
ccr temp-token help
ccr temp-token set <token> [--base-url <url>] [--model <model>]
ccr temp-token show
ccr temp-token clear
```

## 子命令

| 子命令 | 说明 |
|--------|------|
| `help` | 显示命令帮助 |
| `set` | 立即应用临时 Token / Base URL / Model 覆盖 |
| `show` | 查看当前临时覆盖状态（Token 脱敏显示） |
| `clear` | 清理当前临时覆盖状态 |

## 适用场景

- 短期测试新的中转地址或临时凭据
- 在不修改 `profiles.db` 永久配置库的前提下覆盖当前宿主 settings
- 本地调试脚本中快速替换运行参数

## 示例

```bash
ccr claude profile switch work
ccr temp-token set "$TEMP_TOKEN" --base-url https://api.example.com --model claude-sonnet-4-5
ccr temp-token show
ccr temp-token clear
```

## 与 `ccr temp` 的区别

| 命令 | 方式 | 适合场景 |
|------|------|----------|
| `ccr temp` | 交互式提示 | 手动逐项输入并预览一套临时配置 |
| `ccr temp-token` | 命令行子命令 | 查看、应用或清理显式的临时覆盖项 |

## 注意事项

- `set` 直接作用于当前活动宿主 settings，不会修改 `~/.ccr/platforms/claude/profiles.db`。
- 在交互式 Shell 中请注意命令历史记录风险；长期使用的凭据应通过 `printf '%s' "$TOKEN" | ccr claude profile create <name> ... --auth-token-stdin` 存入 `profiles.db`，并通过 `ccr claude profile switch <name>` 切换。

## 相关命令

- [`temp`](./temp.md) - 交互式临时配置
- [`claude`](./claude.md) - 管理永久 Claude Profile 配置库
- [`current`](./current.md) - 查看当前运行时状态
