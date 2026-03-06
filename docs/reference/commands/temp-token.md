# temp-token - 命令式临时覆盖

`ccr temp-token` 用于以命令式方式临时覆盖当前活动 settings 中的 token、base URL 和 model。

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
| `help` | 显示帮助 |
| `set` | 立即应用临时 token / base URL / model |
| `show` | 查看当前临时覆盖状态 |
| `clear` | 清理当前临时覆盖状态 |

## 适用场景

- 短期测试新的 token 或中转地址
- 在不改 TOML profile 的前提下覆盖当前 settings
- 自动化脚本里快速替换当前运行参数

## 示例

```bash
ccr switch work
ccr temp-token set sk-temp-xxx --base-url https://api.example.com --model claude-sonnet-4-5
ccr temp-token show
ccr temp-token clear
```

## 与 `ccr temp` 的区别

| 命令 | 方式 | 适合场景 |
|------|------|----------|
| `ccr temp` | 交互式 | 快速手动输入一套临时配置 |
| `ccr temp-token` | 命令式 | 脚本、复制粘贴、显式传参 |

## 注意

- `set` 会立即作用于当前活动 settings。
- 该命令面向“临时覆盖”，不替代永久 profile 管理。
- 永久配置仍应通过 `ccr add` / `ccr switch` / `profiles.toml` 管理。
