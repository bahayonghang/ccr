# 📚 CCR 配置示例文档

本目录包含 CCR 多平台配置的完整示例和使用指南。

## 📋 文件列表

### 核心配置示例

| 文件 | 描述 | 用途 |
|------|------|------|
| [`config.toml`](./config.toml) | 统一配置文件示例 | 管理所有平台的注册和切换 |
| [`claude-profiles.toml`](./claude-profiles.toml) | Claude Code Profiles 示例 | Claude 平台的多配置管理 |
| [`codex-profiles.toml`](./codex-profiles.toml) | Codex (GitHub Copilot CLI) Profiles 示例 | Codex 平台的多配置管理 |
| [`gemini-profiles.toml`](./gemini-profiles.toml) | Gemini CLI Profiles 示例 | Gemini 平台的多配置管理 |

### 使用指南

| 文件 | 描述 |
|------|------|
| [`troubleshooting.md`](./troubleshooting.md) | 故障排除指南 - 常见问题和解决方案 |

## 🚀 快速开始

### 1. 初始化 CCR

```bash
# 安装 CCR
cargo install --git https://github.com/bahayonghang/ccr ccr

# 初始化默认平台 (Claude)
ccr init
```

### 2. 初始化其他平台

```bash
# 初始化 Codex 平台
ccr platform init codex

# 初始化 Gemini 平台
ccr platform init gemini

# 查看所有平台
ccr platform list
```

### 3. 使用示例配置

```bash
# 复制示例配置到系统位置
cp docs/examples/config.toml ~/.ccr/config.toml

# 复制平台 profiles
cp docs/examples/claude-profiles.toml ~/.ccr/platforms/claude/profiles.toml
cp docs/examples/codex-profiles.toml ~/.ccr/platforms/codex/profiles.toml
cp docs/examples/gemini-profiles.toml ~/.ccr/platforms/gemini/profiles.toml

# 查看当前配置
ccr platform current
```

## 📁 目录结构

CCR 使用以下目录结构管理配置：

```
~/.ccr/                         # CCR 根目录
├── config.toml                 # 统一配置文件
├── platforms/                  # 平台目录
│   ├── claude/
│   │   ├── profiles.toml      # Claude profiles
│   │   └── settings.json      # 平台特定设置 (可选)
│   ├── codex/
│   │   └── profiles.toml      # Codex profiles
│   └── gemini/
│       └── profiles.toml      # Gemini profiles
├── history/                    # 历史记录
│   ├── claude.json
│   ├── codex.json
│   └── gemini.json
└── backups/                    # 自动备份
    ├── claude/
    ├── codex/
    └── gemini/
```

## 🎯 示例场景

### 场景 1: 管理多个 Claude API 账号

使用 [`claude-profiles.toml`](./claude-profiles.toml) 示例，配置：
- `anthropic`: Anthropic 官方 API
- `bedrock`: AWS Bedrock 企业版
- `vertex-ai`: Google Cloud Vertex AI
- `dev`: 本地开发环境

```bash
# 切换到 Claude 平台
ccr platform switch claude

# 切换到官方 API
ccr switch anthropic

# 切换到 AWS Bedrock
ccr switch bedrock
```

### 场景 2: 在不同 AI 平台之间切换

```bash
# 使用 Claude
ccr platform switch claude
ccr switch anthropic

# 切换到 GitHub Copilot (Codex)
ccr platform switch codex
ccr switch github

# 切换到 Google Gemini
ccr platform switch gemini
ccr switch google
```

### 场景 3: 开发和生产环境隔离

每个平台都可以配置 `dev` 和生产 profiles：

```bash
# 开发环境使用本地 mock API
ccr platform switch claude
ccr switch dev

# 生产环境使用官方 API
ccr switch anthropic
```

### 场景 4: 使用第三方代理

配置使用 OpenRouter 或其他代理服务：

```bash
# Claude via OpenRouter
ccr platform switch claude
ccr switch openrouter

# Gemini via OpenRouter
ccr platform switch gemini
ccr switch openrouter
```

## 🔍 配置字段说明

### config.toml

| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `default_platform` | string | ✅ | 默认平台 |
| `current_platform` | string | ✅ | 当前激活的平台 |
| `[platform_name]` | table | ✅ | 平台注册表条目 |
| `enabled` | bool | ✅ | 是否启用该平台 |
| `current_profile` | string | ❌ | 当前使用的 profile |
| `description` | string | ❌ | 平台描述 |
| `last_used` | datetime | ❌ | 最后使用时间 |

### profiles.toml

| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `description` | string | ❌ | Profile 描述 |
| `base_url` | string | ✅ | API 基础 URL |
| `auth_token` | string | ✅ | 认证令牌/API Key |
| `model` | string | ✅ | 默认模型 |
| `small_fast_model` | string | ❌ | 快速小模型 |
| `provider` | string | ❌ | 提供商名称 |
| `provider_type` | string | ❌ | 提供商类型 |
| `account` | string | ❌ | 账号标识 |
| `tags` | array | ❌ | 标签列表 |
| 其他字段 | any | ❌ | 平台特定扩展字段 |

### provider_type 枚举值

- `official`: 官方 API
- `enterprise`: 企业服务 (AWS Bedrock, GCP Vertex AI, etc.)
- `proxy`: 第三方代理 (OpenRouter, Poe, etc.)
- `development`: 本地开发/测试环境

## 🔐 安全注意事项

1. **敏感信息保护**
   - `auth_token` 和其他认证信息是敏感数据
   - 不要将包含真实 token 的配置文件提交到 Git
   - CCR 会自动在日志和历史中掩码敏感信息

2. **文件权限**
   ```bash
   # 确保配置文件权限正确
   chmod 600 ~/.ccr/platforms/*/profiles.toml
   chmod 644 ~/.ccr/config.toml
   ```

3. **备份**
   - CCR 在修改配置前会自动创建备份
   - 备份位置: `~/.ccr/backups/<platform>/`
   - 定期清理旧备份: `ccr clean`

## 🛠️ 自定义配置

### 添加自定义字段

profiles.toml 支持任意自定义字段（存储在 `platform_data`）：

```toml
[my-custom-profile]
# 标准字段
description = "My Custom Configuration"
base_url = "https://api.example.com"
auth_token = "token-here"
model = "model-name"

# 自定义字段
custom_field_1 = "value1"
custom_field_2 = 42
custom_field_3 = ["array", "values"]

[my-custom-profile.nested]
key = "value"
```

### 使用环境变量覆盖

```bash
# 设置自定义 CCR 根目录
export CCR_ROOT=/custom/path/.ccr
ccr platform list

# 设置日志级别（日志同时输出到终端和文件）
export CCR_LOG_LEVEL=debug
ccr platform switch claude

# 日志文件位置：~/.ccr/logs/ccr.YYYY-MM-DD.log
# 按天轮转，自动清理超过14天的旧日志
```

## 📖 相关文档

- [故障排除指南](./troubleshooting.md) - 常见问题和解决方案
- [配置迁移](../migration.md) - 从 Legacy 模式迁移到 Unified 模式
- [架构设计](../architecture.md) - CCR 架构和设计原则
- [命令参考](../commands/README.md) - 完整命令列表

## 🤝 贡献示例

欢迎贡献新的配置示例！请遵循以下规范：

1. **文件命名**: `<platform>-profiles.toml` 或 `<use-case>.md`
2. **注释规范**: 使用 `#` 开头的注释，说明每个字段的作用
3. **隐私保护**: 不要包含真实的 API keys 或 tokens
4. **完整性**: 包含所有必需字段的示例

## 📝 版本历史

- **v2.0.0**: 添加多平台支持，创建统一配置结构
- **v1.x**: Legacy 模式，仅支持 Claude

## 📄 许可证

MIT License - 查看项目根目录的 LICENSE 文件
