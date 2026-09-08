# `grok` - Grok Build Profile Runtime

`ccr grok` 管理 Grok Build 的模型与第三方 provider profile，并提供官方会话查询、登出及 TUI 多账号管理。Profile 操作管理 `~/.grok/config.toml` 中的 `[model.custom]`、`[models].default` 和 `[models].default_reasoning_effort`；账号服务受控读取 `$GROK_HOME/auth.json`（未设置时为 `~/.grok/auth.json`）并保存官方 OAuth 凭据。界面和命令输出不展示 token。CCR 不读取、写入、备份或校验 `mcp_credentials.json`。

## Official Auth

| 命令 | 说明 |
|---|---|
| `ccr grok auth` | 有 TUI launcher 时进入 Grok Auth 标签；否则打印帮助 |
| `ccr grok auth current` | 报告官方会话是否存在；支持 `--json`；不输出 token |
| `ccr grok auth off` | 登出当前官方运行时登录；支持 `--json` |

`auth off` 与 `profile off` 独立，不修改 profile 指针或 `[model.custom]`。登出会删除整个 `auth.json`，包括其他 scope/API-key 缓存；可唯一识别的已保存账号会先回存最新凭据，CCR 保存账号仍保留。它不修改 `mcp_credentials.json`。

Grok Auth TUI 支持官方 OAuth 个人/团队账号：

- `s`：按别名保存当前 scope 的完整副本到 `<CCR_ROOT>/platforms/grok/auth/accounts.json`（默认 `~/.ccr/platforms/grok/auth/accounts.json`）。可在 Grok 运行时保存，当前账号继续使用；多个来源时先选择 scope，同名覆盖需确认。
- `Enter`：确认切换所选账号。请先自行结束当前 Grok，写入后供新会话使用。切换只替换目标 scope，并先保留可识别原账号的最新凭据；未保存或身份不明确时要求先保存。
- `d`：确认删除 CCR 保存项，运行时凭据不变。
- `o`：确认登出整个运行时，保留 CCR 保存项；`r` 仅重读本地状态。

“本地匹配”不表示服务端认证有效。切换保留原始 token 时间，不主动刷新或登录，也不退出第三方 profile；当前认证路线仍可能由 profile 或环境变量决定。企业 OIDC、external、API-key 和旧 web_login 不纳入账号保存管理。暂无对应的 CLI save/switch 子命令。

身份元数据缺失的副本可显式保存；若缺少 Grok 原生格式要求的 `user_id`，切换会拒绝写入，不补造身份。应在官方登录产生完整凭据后重新保存。

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
| `ccr grok profile open` | 用 $VISUAL/$EDITOR 或系统关联程序打开 profiles.toml；文件不存在时先从模板创建 |
| `ccr grok profile off` | 退出 profile mode，并删除 `[model.custom]` 与 `[models].default` |

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
- 账号服务可保存和恢复官方 OAuth scope；`auth off` 删除整个 `auth.json` 前保护可识别的已保存账号。两者不修改 `mcp_credentials.json`，profile 路径仍不读写 `auth.json`。
- URL 输出会移除 userinfo、query 和 fragment。

## 示例

- [CCR Grok profiles](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-profiles.toml)
- [Grok config.toml](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-cli-config.toml)
- [平台迁移映射](./platform)
