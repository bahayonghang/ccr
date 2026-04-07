# CCR 配置示例（Unified）

本目录提供 CCR（Claude Code Configuration Switcher）Unified 模式的示例平台配置，适用于 Claude、Codex CLI、Gemini CLI。将对应文件复制到 `~/.ccr/platforms/<platform>/profiles.toml` 后按需修改即可。

## 📁 内容一览

```
examples/
├── README.md                     # 本文件
├── claude/
│   ├── profiles.example.toml     # Claude Code 平台示例（推荐）
│   └── profiles.toml             # 旧文件（历史遗留，可能包含非 UTF-8 内容）
├── codex/
│   └── profiles.toml             # Codex CLI 平台示例
└── gemini/
    └── profiles.toml             # Gemini CLI 平台示例
```

> 说明：当前仓库已移除 Legacy 模式示例（如 `.ccs_config.toml.example`、`auto_confirm_config.toml` 等），仅保留 Unified 平台配置示例。

## 🚀 快速使用（Unified 模式）

```bash
# Claude 平台
ccr platform init claude
cp examples/claude/profiles.example.toml ~/.ccr/platforms/claude/profiles.toml
vim ~/.ccr/platforms/claude/profiles.toml

# Gemini 平台
ccr platform init gemini
cp examples/gemini/profiles.toml ~/.ccr/platforms/gemini/profiles.toml

# Codex CLI 平台
ccr platform init codex
cp examples/codex/profiles.toml ~/.ccr/platforms/codex/profiles.toml

# 切换/查看
ccr platform switch claude
ccr list
```

## 🗂️ 示例文件解读

- `claude/profiles.toml`：Anthropic 官方、转发服务、第三方兼容、开发/测试、禁用示例。
- `codex/profiles.toml`：OpenAI Codex 官方、中转站、自定义模板示例。
- `gemini/profiles.toml`：Google Gemini 官方、Gemini Pro、开发/测试、禁用示例。
- `codex/config.example.toml` / `codex/auth.example.json`：Codex CLI（`~/.codex/`）的示例配置（与 CCR profiles 独立）。

## 🔑 常用字段

### `usage_count`

记录配置被切换的次数，便于统计使用习惯。

```toml
[anthropic]
description = "Anthropic Official API"
# ...
usage_count = 0  # 默认 0，切换时自动递增
```

查看统计：`ccr list`（“使用”列）。

### `enabled`

控制配置是否可用，禁用后切换列表中不显示。

```toml
[old_api]
description = "旧版 API（已禁用）"
# ...
enabled = false
```

启用/禁用：`ccr disable old_api`、`ccr enable old_api`。

## 💡 配置技巧

1) **分类管理**：用 `provider`、`provider_type`、`account`、`tags` 标记来源、类型和账号。

```toml
[anyrouter_main]
provider = "anyrouter"
provider_type = "official_relay"
account = "github_5953"
tags = ["free", "stable", "primary"]
```

2) **多账号**：同一服务分个人/工作配置。

```toml
[glm_personal]
provider = "glm"
account = "personal"

[glm_work]
provider = "glm"
account = "work"
```

3) **备用线路**：用 `tags` 标记备份。

```toml
[anyrouter_backup1]
tags = ["free", "backup"]
```

4) **环境分离**：开发/生产分别建配置。

```toml
[claude_prod]
tags = ["production", "stable"]

[claude_dev]
tags = ["development", "testing"]
```

## 🔒 安全提示

1. 配置包含 Token，请勿提交到公共仓库。
2. 使用 `.gitignore` 排除本地配置文件。
3. 定期轮换 Token。
4. 生产与开发使用不同 Token。

## 📚 更多信息

- 主项目文档：[`README.md`](../README.md)
- 多平台配置指南：[`CLAUDE.md`](../CLAUDE.md)
- 反馈：GitHub Issues <https://github.com/bahayonghang/ccr/issues>

## ⚡ 常见问题

- **Legacy 与 Unified 区别？**
  - Legacy：单一文件 `~/.ccs_config.toml`，主要管 Claude Code。
  - Unified：多平台目录 `~/.ccr/platforms/`，支持 Claude/Codex/Gemini 等。

- **如何迁移到 Unified？**

```bash
ccr migrate                 # 迁移所有平台
ccr migrate --platform claude  # 仅迁移 Claude
```

- **旧配置缺字段？** CCR 会自动补全，无需手动修改。

- **如何禁用配置？**

```bash
ccr disable old_config
ccr list          # 禁用项不显示
ccr enable old_config
```
