# `grok` - Grok Build Profile Runtime

`ccr grok` 管理 Grok Build 的模型与第三方 provider profile。CCR 管理 `~/.grok/config.toml` 中的 `[model.custom]`、`[models].default` 和 `[models].default_reasoning_effort`，不会读取或写入 `auth.json`、`mcp_credentials.json`。

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

第三方 provider 可直接使用 Grok Build 的 `api_key` 字段：

```bash
ccr grok profile create relay \
  --base-url https://api.example.com/v1 \
  --model grok-example \
  --api-key sk-your-grok-relay-api-key \
  --api-backend responses \
  --context-window 1000000 \
  --reasoning-effort high \
  --supports-backend-search

ccr grok profile switch relay
ccr grok profile current --json
```

`api_backend` 允许 `chat_completions`、`responses`、`messages`。`reasoning_effort` 接受 Grok Build 的规范等级 `none`、`minimal`、`low`、`medium`、`high`、`xhigh`、`max`；其他值会被拒绝。`set-field` 支持 `api_backend`、`api_key`、`env_key`、`context_window`、`supports_backend_search`、`reasoning_effort`，并可用 `--clear` 删除字段：

```bash
ccr grok profile set-field relay reasoning_effort --value high
ccr grok profile current --json
```

第三方 profile 会将强度写入 `[model.custom].reasoning_effort`、派生 `[model.custom].supports_reasoning_effort = true`，并同步 `[models].default_reasoning_effort`。官方 profile 只写全局默认值。切换到未设置该字段的 profile 或执行 `off` 时，CCR 恢复进入 profile mode 前的默认推理强度。

## 凭据边界

- `api_key` 是 Grok Build 的直接密钥字段；`--api-key` 会把密钥明文写入 CCR profiles、其轮换备份以及 Grok `config.toml`，但命令输出会省略该值。旧的 `--auth-token` 是兼容别名。
- `env_key` 仍用于环境变量名，不能填写实际 API key。
- 官方 profile 不接受 `api_key`、`auth_token` 或 `env_key`。Grok 自身的登录会话和 `XAI_API_KEY` 保持由 Grok 管理。
- URL 输出会移除 userinfo、query 和 fragment。

## 示例

- [CCR Grok profiles](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-profiles.toml)
- [Grok config.toml](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-cli-config.toml)
- [平台迁移映射](./platform)
