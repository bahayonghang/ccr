# Claude profile fable 层支持与官方 GLM profile

## Background

用户提供了一张第三方 GLM 接入的 `~/.claude/settings.json` 截图（bigmodel 官方接入方式），其 `env` 完整形态为：

```jsonc
{
  "ANTHROPIC_BASE_URL": "https://open.bigmodel.cn/api/anthropic",
  "ANTHROPIC_AUTH_TOKEN": "***",
  "ANTHROPIC_DEFAULT_OPUS_MODEL": "glm-5.2[1m]",
  "ANTHROPIC_DEFAULT_OPUS_MODEL_NAME": "GLM-5.2",
  "ANTHROPIC_DEFAULT_SONNET_MODEL": "glm-5.2[1m]",
  "ANTHROPIC_DEFAULT_SONNET_MODEL_NAME": "GLM-5.2",
  "ANTHROPIC_DEFAULT_HAIKU_MODEL": "glm-5.2[1m]",
  "ANTHROPIC_DEFAULT_HAIKU_MODEL_NAME": "GLM-5.2",
  "ANTHROPIC_DEFAULT_FABLE_MODEL": "glm-5.2[1m]",
  "ANTHROPIC_DEFAULT_FABLE_MODEL_NAME": "GLM-5.2",
  "ENABLE_TOOL_SEARCH": "0",
  "CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS": "1",
  "CLAUDE_CODE_AUTO_COMPACT_WINDOW": "1000000"
}
```

经源码核对（见「现状」），ccr 当前 Claude profile schema 无法完整表达这份配置：**缺少 fable 模型层**，也**缺少各层 `*_MODEL_NAME` 显示名变量**。

> 原始依据飞书教程 `https://my.feishu.cn/docx/KNeqdPly7o6iWexfRKOcTqBxnbH` 需登录无法抓取，已与用户确认改为「按截图 + Claude Code 官方 env 约定」推进。

## 现状（源码核对结论）

ccr 的 profile → `settings.json` env 映射只覆盖：`ANTHROPIC_BASE_URL/AUTH_TOKEN/MODEL/SMALL_FAST_MODEL`、`ANTHROPIC_DEFAULT_OPUS/SONNET/HAIKU_MODEL`、`CLAUDE_CODE_SUBAGENT_MODEL`、`ANTHROPIC_CUSTOM_MODEL_OPTION(_NAME)`、`CLAUDE_CODE_EFFORT_LEVEL`。

- `crates/ccr-config/src/managers/config/types.rs` `ConfigSection`
- `crates/ccr-config/src/models/platform.rs` `ProfileConfig`
- `crates/ccr-cli/src/managers/settings.rs` 常量 + `update_from_config`(:106) + `clear_managed_vars`(:82)
- `crates/ccr-cli/src/platforms/claude.rs` `get_env_var_names`(:335)

**缺口确认（全仓库 `FABLE` 零命中）：**
1. **无 fable 层**：无 `default_fable_model` 字段、无 `ANTHROPIC_DEFAULT_FABLE_MODEL` 常量；不在 `update_from_config` / `get_env_var_names` / `clear_managed_vars`。
   - 风险：用户手动加的 `ANTHROPIC_DEFAULT_FABLE_MODEL` **切换 profile 时不会被清掉**（不在托管 key 列表），造成跨 profile 串档。
2. **无 `*_MODEL_NAME` 显示名**：仅 `ANTHROPIC_CUSTOM_MODEL_OPTION_NAME`，缺 OPUS/SONNET/HAIKU/FABLE 的 `_MODEL_NAME`。

Fable 5（`claude-fable-5`）是当前真实模型层级，故缺口属实。

## Goal

1. 让 ccr 的 Claude profile 完整表达并安全托管 fable 层与各层显示名（与现有 opus/sonnet/haiku 字段同构、同样纳入 clear/apply 托管）。
2. 据此按 GLM 官方接入配置创建可直接使用的「官方 GLM profile」（写入真实 `~/.ccr`，api key 占位），并在仓库内置同款示例/预设。

## 子任务映射

- **`06-20-fable-model-support`**（复杂，prd+design+implement）：后端 fable 层 + 各层 `*_MODEL_NAME` 端到端打通；ccr-ui 表单录入。TUI 经核对无「按模型层编辑 profile」入口（仅切换/鉴权），不在本轮实现内（见该子任务 PRD 假设）。
- **`06-20-official-glm-profile`**（轻量，prd+implement）：基于上面能力创建官方 GLM profile（写入 `~/.ccr/platforms/claude/profiles.toml`，api key 占位）+ 仓库内置示例/预设。**依赖** `fable-model-support` 先落地，否则 fable 字段无法 apply。

## 跨子任务验收

- [ ] CA1：fable 层与显示名字段在「后端类型 / env 映射 / 托管清理 / env 名称登记」四处一致出现；切换 profile 时旧 `ANTHROPIC_DEFAULT_FABLE_MODEL` 等被清掉。
- [ ] CA2：用 ccr 创建/应用「官方 GLM profile」后，`~/.claude/settings.json` 复现截图四层模型 + fable + 显示名（除 token 外一致）。
- [ ] CA3：`just lint-strict`、`just test`、`just frontend-check-quick` 全绿。

## Constraints

- 遵守根 CLAUDE.md 安全约束：secrets masking、改动前备份、文件锁、原子写不回退；日志不得泄露 token；占位 api key 不得是任何真实可用值。
- 内部实现注释中文，公共 API doc 英文。
- 保持「任意 Anthropic 兼容第三方」通用性，不引入 GLM 专有分支逻辑（GLM 仅作为 profile 数据/预设示例存在）。

## Out of Scope

- 不改 Codex/Gemini/Droid 模型字段模型（仅限 Claude 平台）。
- 不改 Claude Code 对 `/model` 文案的渲染。
- `ENABLE_TOOL_SEARCH` / `CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS` / `CLAUDE_CODE_AUTO_COMPACT_WINDOW` 等非模型 env 不纳入 typed 字段（官方 profile 如需，见子任务讨论）。

## References

- 截图：`~/.claude/settings.json`（GLM via bigmodel 官方接入）
- 前置任务：`.trellis/tasks/archive/2026-06/06-19-claude-third-party-model-authmode`（已将 `custom_model_option` 提升为一等字段，本任务沿用同一模式）
- Claude Code 官方：Model configuration
