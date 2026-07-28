# `grok` - Grok Build Profile Runtime

`ccr grok` 管理 Grok Build 的模型与第三方 provider profile。CCR 只修改 `~/.grok/config.toml` 中的 `[model.custom]` 和 `[models].default`，不会读取或写入 `auth.json`、`mcp_credentials.json`。

## 命令

| 命令 | 说明 |
|---|---|
| `ccr grok profile current` | 显示当前 profile；支持 `--json` |
| `ccr grok profile list` | 列出 profiles；支持 `--json` |
| `ccr grok profile switch <name>` | 应用 profile |
| `ccr grok profile create <name>` | 创建 profile |
| `ccr grok profile set-field <name> <field>` | 更新或清空单个字段 |
| `ccr grok profile enable <name>` | 启用 profile |
| `ccr grok profile disable <name>` | 禁用 profile |
| `ccr grok profile delete <name>` | 删除 profile；活动项需先 off，或使用 `--force` |
| `ccr grok profile off` | 恢复进入 CCR profile mode 前的 Grok 配置 |

## 创建 Profile

官方模型选择器不接管认证：

```bash
ccr grok profile create official \
  --model grok-example
```

第三方 provider 推荐引用环境变量：

```bash
ccr grok profile create relay \
  --base-url https://api.example.com/v1 \
  --model grok-example \
  --env-key GROK_RELAY_API_KEY \
  --api-backend responses \
  --context-window 1000000 \
  --supports-backend-search

ccr grok profile switch relay
ccr grok profile current --json
```

`api_backend` 允许 `chat_completions`、`responses`、`messages`。`set-field` 支持 `api_backend`、`env_key`、`context_window`、`supports_backend_search`，并可用 `--clear` 删除字段。

## 凭据边界

- 推荐 `env_key`：CCR 只保存环境变量名，密钥由运行 Grok 的环境提供。
- `--auth-token` 会把密钥明文写入 CCR profiles、其轮换备份以及 Grok `config.toml`；命令输出仍会掩码或省略该值。
- 官方 profile 不接受 `auth_token` 或 `env_key`。Grok 自身的登录会话和 `XAI_API_KEY` 保持由 Grok 管理。
- URL 输出会移除 userinfo、query 和 fragment。

## 示例

- [CCR Grok profiles](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-profiles.toml)
- [Grok config.toml](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-cli-config.toml)
- [平台迁移映射](./platform)
