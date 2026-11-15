# CCR 配置示例

本目录提供 CCR 配置与宿主运行时文件的参考示例，覆盖 Claude、Codex 与 Grok 等平台。

> [!IMPORTANT]
> CCR 7.x 中各平台的权威活体存储是 SQLite 配置库 `~/.ccr/platforms/<platform>/profiles.db`（旧版 `profiles.toml` 会自动迁移并备份）。本目录下的 `.toml` 文件仅作字段参考或 `ExchangeV1` 导入模板，**请勿直接复制覆盖到 `~/.ccr/platforms/*/profiles.toml` 作为主存储**。

## 文件列表

| 文件 | 用途 |
|------|------|
| [`config.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/config.toml) | Unified 平台注册表参考示例 |
| [`claude-profiles.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/claude-profiles.toml) | Claude Profile 字段与交换格式示例 |
| [`codex-profiles.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/codex-profiles.toml) | Codex Profile 示例（官方 + 第三方中转/模型） |
| [`codex-cli-config.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/codex-cli-config.toml) | DeepSeek `~/.codex/config.toml` 宿主输出示例 |
| [`codex-auth.example.json`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/codex-auth.example.json) | `~/.codex/auth.json` 示例 |
| [`grok-profiles.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-profiles.toml) | Grok Profile 示例（官方会话、`env_key` 与直接 `api_key`） |
| [`grok-cli-config.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-cli-config.toml) | `~/.grok/config.toml` 宿主输出示例 |
| [`gemini-profiles.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/gemini-profiles.toml) | 历史结构参考（`gemini` 为保留兼容适配器 `legacy_adapter`，无 CLI `profile` 子命令） |
| [`multi-platform-setup.md`](./multi-platform-setup) | 多平台配置与工作流指南 |
| [`troubleshooting.md`](./troubleshooting) | 常见故障排查指南 |

## 快速使用（Codex 示例）

```bash
# 1) 初始化 Codex Profile 配置库（~/.ccr/platforms/codex/profiles.db）
ccr codex profile init

# 2) 通过标准输入安全创建 Profile，或在 $EDITOR 中编辑投影
printf '%s' "$OPENAI_API_KEY" | ccr codex profile create duckcoding \
  --base-url https://api.example.com/v1 \
  --auth-token-stdin
ccr codex profile open

# 3) 校验并切换路由
ccr validate
ccr codex profile switch duckcoding
```

## Codex 配置速查

- 平台配置说明：[`/reference/platforms/codex`](../reference/platforms/codex)
- 核心字段：`model`、`model_reasoning_effort`、`base_url`、`auth_token`、`env_key`
- 切换后 CCR 会原子更新 `~/.codex/config.toml` 与 `~/.codex/auth.json`
- 备份或分享配置请使用 `ccr codex profile export --no-secrets -o codex-profiles.toml`，切勿提交真实密钥

## Claude 与 Grok 配置速查

- Claude 平台指南：[`/reference/platforms/claude`](../reference/platforms/claude)（支持 `ccr claude profile init|create|open|switch|export|import`，密钥请用 `--auth-token-stdin` 传入）
- Grok 命令说明：[`/reference/commands/grok`](../reference/commands/grok)（直接密钥通过 `--api-key-stdin` 传入，`--env-key` 仅填写环境变量名，如 `XAI_API_KEY`）
- 本目录示例均使用占位值，不包含真实服务商凭据或账号信息
