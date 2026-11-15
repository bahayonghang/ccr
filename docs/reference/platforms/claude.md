# Claude 平台指南

Claude（`claude`）是当前显式运行时与配置库模型中的一等支持平台。完整 CLI 子命令与 JSON 输出契约见 [`claude` 命令参考](/reference/commands/claude)。

## 当前主路径

```bash
# 初始化并编辑 Claude 配置库（~/.ccr/platforms/claude/profiles.db）
ccr claude profile init
printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile create work \
  --base-url https://api.example.com \
  --auth-token-stdin
ccr claude profile open

# 查看、切换与退出 Profile 模式
ccr claude auth current
ccr claude profile list
ccr claude profile switch work
ccr claude profile current
ccr claude profile off

# 导出与导入 ExchangeV1 TOML
ccr claude profile export --no-secrets -o claude-profiles.toml
ccr claude profile import claude-profiles.toml --dry-run
```

## 模型说明

- `ccr claude auth ...`：管理官方登录账号快照与认证状态
- `ccr claude profile ...`：管理 `profiles.db` 配置库并将指定 Profile 应用到 `~/.claude/settings.json`
- `ccr claude profile off`：退出 Profile 模式，清除托管环境变量并回到官方登录态

## 第三方模型（GLM / DeepSeek / Kimi 等）

要让 Claude Code 跑第三方模型，Profile 必须是 **api_key** 模式（不是 `subscription`）：

- `auth_mode = "api_key"`：写入 `ANTHROPIC_BASE_URL` / `ANTHROPIC_AUTH_TOKEN` 及模型映射。
- `auth_mode = "subscription"`：应用时会清空所有 `ANTHROPIC_*`，回落官方登录——第三方配置会被丢弃。

为避免误配，凡是「`base_url` 与 `auth_token` 同时存在」或「`provider_type = third_party_model`」的 Profile，保存与应用时都会自动把 `auth_mode` 纠正为 `api_key`（`ccr-ui` 也会给出内联提示）。

模型映射字段 → 环境变量：

| Profile 字段 | 环境变量 |
| --- | --- |
| `default_opus_model` | `ANTHROPIC_DEFAULT_OPUS_MODEL` |
| `default_sonnet_model` | `ANTHROPIC_DEFAULT_SONNET_MODEL` |
| `default_haiku_model` | `ANTHROPIC_DEFAULT_HAIKU_MODEL` |
| `default_fable_model` | `ANTHROPIC_DEFAULT_FABLE_MODEL` |
| `default_opus_model_name` | `ANTHROPIC_DEFAULT_OPUS_MODEL_NAME` |
| `default_sonnet_model_name` | `ANTHROPIC_DEFAULT_SONNET_MODEL_NAME` |
| `default_haiku_model_name` | `ANTHROPIC_DEFAULT_HAIKU_MODEL_NAME` |
| `default_fable_model_name` | `ANTHROPIC_DEFAULT_FABLE_MODEL_NAME` |
| `subagent_model` | `CLAUDE_CODE_SUBAGENT_MODEL` |
| `custom_model_option` | `ANTHROPIC_CUSTOM_MODEL_OPTION` |
| `custom_model_option_name` | `ANTHROPIC_CUSTOM_MODEL_OPTION_NAME` |
| `effort_level` | `CLAUDE_CODE_EFFORT_LEVEL` |
| `claude_code_auto_compact_window` | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` |
| `api_timeout_ms` | `API_TIMEOUT_MS` |
| `claude_code_disable_nonessential_traffic` | `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` |

### Z.AI / 智谱 GLM 模板

CCR 内置 GLM Claude Code 模板当前采用 Z.AI 兼容 Anthropic 端点：

```toml
base_url = "https://api.z.ai/api/anthropic"
provider = "glm"
provider_type = "third_party_model"
auth_mode = "api_key"
default_opus_model = "glm-5.2[1m]"
default_sonnet_model = "glm-5.2[1m]"
default_haiku_model = "glm-4.7"
default_fable_model = "glm-5.2[1m]"
claude_code_auto_compact_window = "1000000"
api_timeout_ms = "3000000"
claude_code_disable_nonessential_traffic = "1"
```

模板不会写入真实 API Key。创建 Profile 后，请通过标准输入写入自己的 Z.AI / 智谱密钥，再切换并执行体检：

```bash
printf '%s' "$ZAI_API_KEY" | ccr claude profile edit <name> --auth-token-stdin
ccr claude profile switch <name>
ccr doctor --platform claude
```

应用 API-Key Profile 时，CCR 会把 `~/.claude/settings.json` 中的 `env` 更新为 Profile 对应的托管环境变量，并尝试在 `~/.claude.json` 中补写 `hasCompletedOnboarding = true`。如果 `~/.claude.json` 损坏或不可写，Profile 切换仍会继续，`ccr doctor` 会报告 onboarding 状态 warning。

注意：

- Claude Code 的 `/model` 列表仍显示 Opus/Sonnet/Haiku 文案——它不会把内置别名改成第三方模型名，但底层实际命中你映射的模型。
- `glm-5.2[1m]` 这类 `[1m]` 后缀需较新版本的 Claude Code 才能识别。
- `ccr doctor --platform claude` 会提示占位 Token、`settings.json` 环境变量与当前 Profile 不一致、GLM 1M 缺 compact window，以及 onboarding 状态缺失等常见问题。

## 关键路径

- 宿主运行时设置：`~/.claude/settings.json`
- Claude Code 状态文件：`~/.claude.json`
- Profile 权威配置库：`~/.ccr/platforms/claude/profiles.db`（`profiles.toml` 仅作 `ExchangeV1` 导入导出与 `profile open` 投影格式）
- 注册表指针：`~/.ccr/config.toml` 中 `[claude].current_profile`

## 相关文档

- [`claude` 命令参考](/reference/commands/claude)
- [平台支持概览](./index)
- [平台命令迁移](./migration)
