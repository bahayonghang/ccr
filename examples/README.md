# CCR 配置示例

本目录包含 CCR (Claude Code Configuration Switcher) 的各种配置示例，帮助用户快速了解如何配置和使用 CCR。

## 📁 文件结构

```
examples/
├── README.md                           # 本文件
├── .ccs_config.toml.example           # Legacy 模式完整配置示例
├── auto_confirm_config.toml           # 自动确认配置示例
├── sync_config_example.toml           # WebDAV 同步配置示例
├── claude/
│   └── profiles.toml                  # Claude Code 平台配置示例
├── codex/
│   └── profiles.toml                  # Codex (GitHub Copilot) 平台配置示例
└── gemini/
    └── profiles.toml                  # Gemini CLI 平台配置示例
```

## 🚀 快速开始

### Legacy 模式（单一配置文件）

适合只使用 Claude Code 的用户：

```bash
# 复制示例配置
cp examples/.ccs_config.toml.example ~/.ccs_config.toml

# 编辑配置文件，替换 API Token
vim ~/.ccs_config.toml

# 初始化 CCR
ccr init

# 查看配置列表
ccr list

# 切换配置
ccr switch <配置名称>
```

### Unified 模式（多平台配置）

适合使用多个 AI CLI 平台的用户：

```bash
# 初始化 Claude 平台
ccr platform init claude

# 复制 Claude 配置示例
cp examples/claude/profiles.toml ~/.ccr/platforms/claude/profiles.toml

# 编辑配置文件
vim ~/.ccr/platforms/claude/profiles.toml

# 初始化 Gemini 平台
ccr platform init gemini
cp examples/gemini/profiles.toml ~/.ccr/platforms/gemini/profiles.toml

# 初始化 Codex 平台
ccr platform init codex
cp examples/codex/profiles.toml ~/.ccr/platforms/codex/profiles.toml

# 切换平台
ccr platform switch claude

# 查看当前平台的配置
ccr list
```

## 📄 配置文件说明

### .ccs_config.toml.example

**完整的 Legacy 模式配置示例**，包含：

- 🔄 **官方中转服务配置** (10+ 示例)
  - AnyRouter (多账号)
  - 虎三 API
  - Duck API
  - iKun API
  - LycheeShare
  - ShareYourCC
  - 88code
  - AICodeMirror
  - 文文AI

- 🤖 **第三方模型服务配置** (7+ 示例)
  - 智谱 GLM (多账号)
  - 月之暗面 Kimi (多模型)
  - SiliconFlow (多模型)
  - 魔搭社区

- 🆕 **新功能示例**
  - `usage_count` - 自动追踪使用次数
  - `enabled` - 启用/禁用配置
  - `provider` - 提供商分类
  - `provider_type` - 类型标识
  - `account` - 账号区分
  - `tags` - 灵活标签

### claude/profiles.toml

**Claude Code 平台配置示例**，包含：

- Anthropic 官方 API
- Claude 转发服务
- 第三方兼容服务
- 开发/测试环境
- 禁用配置示例

### gemini/profiles.toml

**Gemini CLI 平台配置示例**，包含：

- Google Gemini 官方 API
- Gemini Pro 配置
- Gemini 转发服务
- 开发/测试环境
- 禁用配置示例

### codex/profiles.toml

**Codex (GitHub Copilot) 平台配置示例**，包含：

- GitHub Copilot 官方
- Copilot Enterprise
- Codex 转发服务
- 开发/测试环境
- 禁用配置示例

### auto_confirm_config.toml

展示如何配置全局设置：

```toml
[settings]
skip_confirmation = true   # 跳过确认提示
auto_backup = true        # 自动备份
backup_retention_days = 7 # 备份保留天数
```

### sync_config_example.toml

展示如何配置 WebDAV 云同步：

```toml
[settings.sync]
enabled = true
webdav_url = "https://dav.jianguoyun.com/dav/"
username = "your-email@example.com"
password = "your-app-password"
remote_path = "/ccr/.ccs_config.toml"
auto_sync = false
```

## 🆕 新增字段说明

### usage_count（使用次数）

自动追踪每个配置被切换的次数，帮助你了解使用习惯。

```toml
[anthropic]
description = "Anthropic Official API"
# ...其他字段...
usage_count = 0      # 默认为 0，自动递增
```

查看使用统计：

```bash
ccr list  # 在 "使用" 列中显示次数
```

### enabled（启用状态）

控制配置是否可用。禁用的配置不会在切换列表中显示。

```toml
[old_api]
description = "旧版 API（已禁用）"
# ...其他字段...
enabled = false      # 禁用配置
```

启用/禁用配置：

```bash
ccr disable old_api   # 禁用配置
ccr enable old_api    # 启用配置
```

## 💡 使用技巧

### 1. 配置分类管理

使用 `provider`, `provider_type`, `account`, `tags` 字段进行灵活分类：

```toml
[anyrouter_main]
provider = "anyrouter"           # 同一提供商
provider_type = "official_relay" # 官方中转
account = "github_5953"          # 账号标识
tags = ["free", "stable", "primary"]  # 灵活标签
```

### 2. 多账号管理

为同一服务的不同账号创建独立配置：

```toml
[glm_personal]
provider = "glm"
account = "personal"

[glm_work]
provider = "glm"
account = "work"
```

### 3. 备用配置

使用 `tags` 标记备用配置：

```toml
[anyrouter_backup1]
tags = ["free", "backup"]
```

### 4. 开发/生产环境分离

为不同环境创建配置：

```toml
[claude_prod]
tags = ["production", "stable"]

[claude_dev]
tags = ["development", "testing"]
```

## 🔒 安全提示

⚠️ **重要**：

1. 配置文件包含敏感信息（API Token），请勿提交到公共代码仓库
2. 使用 `.gitignore` 排除配置文件
3. 定期轮换 API Token
4. 为生产和开发环境使用不同的 Token

## 📚 更多信息

- 查看主项目文档: [README.md](../README.md)
- 多平台配置指南: [CLAUDE.md](../CLAUDE.md)
- 问题反馈: [GitHub Issues](https://github.com/bahayonghang/ccr/issues)

## ⚡ 常见问题

**Q: Legacy 模式和 Unified 模式有什么区别？**

A: 
- **Legacy 模式**: 单一配置文件 `~/.ccs_config.toml`，只管理 Claude Code
- **Unified 模式**: 多平台架构 `~/.ccr/platforms/`，支持 Claude、Codex、Gemini 等

**Q: 如何从 Legacy 模式迁移到 Unified 模式？**

A:
```bash
ccr migrate         # 自动迁移所有平台
ccr migrate --platform claude  # 迁移特定平台
```

**Q: 旧配置文件缺少新字段怎么办？**

A: CCR 会自动补全缺失字段，无需手动修改。

**Q: 如何禁用不再使用的配置？**

A:
```bash
ccr disable old_config     # 禁用配置
ccr list                   # 禁用的配置不显示
ccr enable old_config      # 重新启用
```
