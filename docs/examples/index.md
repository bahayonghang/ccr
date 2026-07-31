# CCR 配置示例

本目录提供可直接复制的 CCR 配置示例，重点包含 Codex 平台配置。

## 文件列表

| 文件 | 用途 |
|------|------|
| [`config.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/config.toml) | Unified 平台注册配置 |
| [`claude-profiles.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/claude-profiles.toml) | Claude profiles 示例 |
| [`codex-profiles.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/codex-profiles.toml) | Codex profiles（官方 + 第三方） |
| [`codex-cli-config.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/codex-cli-config.toml) | DeepSeek `~/.codex/config.toml` 示例 |
| [`codex-auth.example.json`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/codex-auth.example.json) | `~/.codex/auth.json` 示例 |
| [`gemini-profiles.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/gemini-profiles.toml) | Gemini profiles 示例 |
| [`grok-profiles.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-profiles.toml) | Grok profiles（官方 + 第三方 api_key） |
| [`grok-cli-config.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-cli-config.toml) | `~/.grok/config.toml` 示例 |
| [`troubleshooting.md`](./troubleshooting.md) | 常见故障排查 |

## Codex 配置速查

- 平台配置说明：[`/reference/platforms/codex`](../reference/platforms/codex)
- 核心字段：`model`、`model_reasoning_effort`、`base_url`、`auth_token`、`env_key`
- 推荐顺序：先配置 `profiles.toml`，再 `ccr validate`，最后 `ccr switch <profile>`

## 快速使用（Codex）

```bash
# 1) 初始化 Codex profiles 模板
ccr codex profile init

# 2) 编辑、校验并切换
vim ~/.ccr/platforms/codex/profiles.toml
ccr validate
ccr codex profile switch duckcoding
```

## 说明

- `codex-profiles.toml` 是 CCR 输入配置。
- 切换后 CCR 会写入 `~/.codex/config.toml` 与 `~/.codex/auth.json`。
- 分享配置请使用 `ccr export --no-secrets`，不要提交真实 token。

## Grok 配置速查

- 命令说明：[`/reference/commands/grok`](../reference/commands/grok)
- Grok 直接密钥使用 `api_key`；`env_key` 只填写环境变量名。
- 示例不包含真实 provider、账号或凭据。
