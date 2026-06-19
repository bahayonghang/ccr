# Claude 平台指南

Claude 是当前显式 runtime/profile 模型中的一等平台。

## 当前主路径

```bash
ccr claude auth current
ccr claude profile list
ccr claude profile switch <name>
ccr claude profile current
ccr claude profile off
```

## 模型说明

- `ccr claude auth ...`：管理 official auth 账号与登录态
- `ccr claude profile ...`：把某个 profile 应用到 `~/.claude/settings.json`
- `ccr claude profile off`：退出 profile mode，回到 official auth runtime

## 第三方模型（GLM / DeepSeek / Kimi 等）

要让 Claude Code 跑第三方模型，profile 必须是 **api_key** 模式（不是 subscription）：

- `auth_mode = "api_key"`：写入 `ANTHROPIC_BASE_URL` / `ANTHROPIC_AUTH_TOKEN` 及模型映射。
- `auth_mode = "subscription"`：应用时会清空所有 `ANTHROPIC_*`，回落官方登录——第三方配置会被丢弃。

为避免误配，凡是「`base_url` 与 `auth_token` 同时存在」或「`provider_type = third_party_model`」的 profile，保存与应用时都会自动把 `auth_mode` 纠正为 `api_key`（ccr-ui 也会给出内联提示）。

模型映射字段 → 环境变量：

| Profile 字段 | 环境变量 |
| --- | --- |
| `default_opus_model` | `ANTHROPIC_DEFAULT_OPUS_MODEL` |
| `default_sonnet_model` | `ANTHROPIC_DEFAULT_SONNET_MODEL` |
| `default_haiku_model` | `ANTHROPIC_DEFAULT_HAIKU_MODEL` |
| `subagent_model` | `CLAUDE_CODE_SUBAGENT_MODEL` |
| `custom_model_option` | `ANTHROPIC_CUSTOM_MODEL_OPTION` |
| `custom_model_option_name` | `ANTHROPIC_CUSTOM_MODEL_OPTION_NAME` |
| `effort_level` | `CLAUDE_CODE_EFFORT_LEVEL` |

注意：

- Claude Code 的 `/model` 列表仍显示 Opus/Sonnet/Haiku 文案——它不会把内置别名改成第三方模型名，但底层实际命中你映射的模型。
- `glm-5.2[1m]` 这类 `[1m]` 后缀需较新版本的 Claude Code 才能识别。

## 关键路径

- Runtime settings：`~/.claude/settings.json`
- Profiles：`~/.ccr/platforms/claude/profiles.toml`
- Registry pointer：`~/.ccr/config.toml` 中 `[claude].current_profile`

## 迁移提醒

以下旧路径不再推荐：

- `ccr platform switch claude`
- `ccr platform current`
- `ccr switch <name>`

改用：

- `ccr current`
- `ccr claude profile switch <name>`
- `ccr claude profile off`
