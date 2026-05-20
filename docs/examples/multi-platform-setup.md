# 🌟 CCR 多平台配置完整指南

## 📊 配置概览

### 已配置的平台

| 平台 | 状态 | Profiles 数量 | 当前 Profile | 描述 |
|------|------|---------------|--------------|------|
| **Claude** | ✅ 已启用 | 16 | husan | Claude Code AI Assistant |
| **Codex** | ✅ 已启用 | 3 | default | Codex CLI |
| **Antigravity CLI** | ✅ 已启用 | 6 | google | Google Antigravity CLI（内部 key: gemini） |
| **Qwen** | ⏸️ 未启用 | 0 | - | Alibaba Qwen CLI (计划中) |

### 目录结构

```
~/.ccr/                                    # CCR 根目录
├── config.toml                            # 平台注册表
├── platforms/
│   ├── claude/
│   │   └── profiles.toml                  # 16 个 Claude profiles
│   ├── codex/
│   │   └── profiles.toml                  # 5 个 Codex profiles
│   └── gemini/
│       └── profiles.toml                  # 6 个 Gemini profiles
├── history/
│   ├── claude.json                        # Claude 操作历史
│   ├── codex.json                         # Codex 操作历史
│   └── gemini.json                        # Gemini 操作历史
└── backups/
    ├── claude/                            # Claude 备份
    ├── codex/                             # Codex 备份
    └── gemini/                            # Gemini 备份
```

## 🚀 快速开始

### 1. 列出所有平台

```bash
ccr platform list
```

输出：
```
┌────────┬──────────┬──────┬──────────────┬──────────────────────────┐
│ 状态   ┆ 平台名称 ┆ 启用 ┆ 当前 Profile ┆ 描述                     │
╞════════╪══════════╪══════╪══════════════╪══════════════════════════╡
│ ▶ 当前 ┆ claude   ┆ ✓    ┆ husan        ┆ Claude Code AI Assistant │
│        ┆ codex    ┆ ✓    ┆ default      ┆ Codex CLI                │
│        ┆ gemini   ┆ ✓    ┆ google       ┆ Antigravity CLI          │
└────────┴──────────┴──────┴──────────────┴──────────────────────────┘
```

### 2. 切换平台

```bash
# 切换到 Codex CLI
ccr platform switch codex

# 切换到 Antigravity CLI（内部 key 仍为 gemini）
ccr platform switch gemini

# 切换回 Claude
ccr platform switch claude
```

### 3. 查看当前平台

```bash
ccr platform current
```

### 4. 管理平台内的 Profiles

```bash
# 列出当前平台的所有 profiles
ccr list

# 切换 profile
ccr switch anyrouter

# 查看当前 profile 详情
ccr current
```

## 📋 Claude 平台配置 (16 Profiles)

### 官方中转服务

| Profile | 描述 | Provider | Tags |
|---------|------|----------|------|
| anyrouter | AnyRouter 主服务 (github_5953) | anyrouter | free, stable, primary |
| anyrouter2 | AnyRouter 备用1 (github_5962) | anyrouter | free, backup |
| anyrouter3 | AnyRouter 备用2 (linuxdo_79797) | anyrouter | free, backup |
| anyrouter4 | AnyRouter 学生账号 | anyrouter | free, student |
| husan | 虎三api | husan | paid, stable, high-speed |
| duck | Duck API | duck | free |
| ikun | iKun API | ikun | free |
| lycheeshare | LycheeShare API | lycheeshare | free |
| share | ShareYourCC API | share | free, us-server |
| 88code | 88Code API | 88code | free |
| aicodemirror | AICodeMirror API | aicodemirror | free |
| wenwen | 文文AI API | wenwen | free |

### 第三方模型服务

| Profile | 描述 | Model | Tags |
|---------|------|-------|------|
| glm | 智谱GLM API | glm-4.6 | chinese, official |
| moonshot | 月之暗面 Kimi K2 | kimi-k2-turbo-preview | chinese, kimi, fast |
| siliconflow | SiliconFlow Kimi | moonshotai/Kimi-K2-Instruct | chinese, multi-model, kimi |
| modelscope | 魔搭社区 | Qwen/Qwen3-Coder-480B-A35B-Instruct | chinese, qwen, coding |

**默认配置**: anyrouter
**当前配置**: husan

## 💻 Codex 平台配置 (3 Profiles)

| Profile | 描述 | Model | Provider |
|---------|------|-------|----------|
| default | OpenAI Codex Official | 由 Codex CLI 选择 | OpenAI |
| duckcoding | DuckCoding 中转站 | 自定义 | DuckCoding |
| 88code | 88Code 中转站 | 自定义 | 88Code |

**默认配置**: default
**当前配置**: default

## ✨ Antigravity CLI 平台配置 (6 Profiles，内部 key: gemini)

| Profile | 描述 | Model | Provider |
|---------|------|-------|----------|
| google | Google Official | gemini-pro | - |
| profile-1 | Thread 1 | gemini-2.0-flash-exp / gemini-1.5-flash | Google |
| profile-2 | Thread 2 | gemini-2.0-flash-exp / gemini-1.5-flash | Google |
| profile-3 | Thread 3 | gemini-2.0-flash-exp / gemini-1.5-flash | Google |
| profile-4 | Thread 4 | gemini-2.0-flash-exp / gemini-1.5-flash | Google |
| profile-5 | Thread 5 | gemini-2.0-flash-exp / gemini-1.5-flash | Google |

**默认配置**: google
**当前配置**: google

## 🔄 常见工作流

### 场景 1: 快速切换中转服务

```bash
# 查看所有 Claude 中转服务
ccr list

# 切换到免费的 AnyRouter
ccr switch anyrouter

# 切换到付费的虎三 (更快更稳定)
ccr switch husan
```

### 场景 2: 测试不同模型

```bash
# 切换到 Moonshot Kimi K2
ccr switch moonshot

# 切换到智谱 GLM
ccr switch glm

# 切换到 SiliconFlow
ccr switch siliconflow
```

### 场景 3: 跨平台切换

```bash
# 从 Claude 切换到 Codex CLI
ccr platform switch codex
ccr current

# 切换到 Antigravity CLI / Google Gemini API
ccr platform switch gemini
ccr current

# 切换回 Claude
ccr platform switch claude
ccr current
```

### 场景 4: 查看平台详细信息

```bash
# 查看 Codex 平台信息
ccr platform info codex

# 查看 Gemini 平台信息
ccr platform info gemini

# 查看当前平台信息
ccr platform current
```

## 🛠️ 配置文件示例

### ~/.ccr/config.toml (平台注册表)

```toml
default_platform = "claude"
current_platform = "claude"

[claude]
enabled = true
current_profile = "husan"
description = "Claude Code AI Assistant"
last_used = "2025-10-25T15:23:43.177265328+00:00"

[codex]
enabled = true
current_profile = "default"
description = "Codex CLI"
last_used = "2025-10-25T15:23:43.182209794+00:00"

[gemini]
enabled = true
current_profile = "google"
description = "Antigravity CLI (legacy key: gemini)"
last_used = "2025-10-25T15:18:58.727924189+00:00"
```

### ~/.ccr/platforms/claude/profiles.toml (Claude Profiles)

```toml
default_config = "anyrouter"
current_config = "husan"

[settings]
skip_confirmation = false

[anyrouter]
description = "AnyRouter 主服务 (github_5953)"
base_url = "https://anyrouter.top"
auth_token = "sk-gCJhGGGIDEKDFVTM3NYa8M4XWM8MsgU0pWhreTFg3oI0Pzi2"
provider = "anyrouter"
provider_type = "official_relay"
account = "github_5953"
tags = ["free", "stable", "primary"]

[husan]
description = "虎三api"
base_url = "https://husanai.com"
auth_token = "sk-uyv3753vanVsmbdeHRwpx8mD0EREkewvf3WuIkohYCcQvh21"
provider = "husan"
provider_type = "official_relay"
tags = ["paid", "stable", "high-speed"]

# ... 其他 14 个 profiles
```

### ~/.ccr/platforms/codex/profiles.toml (Codex Profiles)

```toml
default_config = "default"
current_config = "default"

[settings]
skip_confirmation = false

[default]
description = "OpenAI Codex Official"
auth_token = ""
enabled = true

[duckcoding]
description = "DuckCoding 中转站"
base_url = "https://jp.duckcoding.com/v1"
auth_token = "sk-...your-duckcoding-codex-token"
enabled = true

[88code]
description = "88Code 中转站"
base_url = "https://www.88code.ai/openai/v1"
auth_token = "sk-your-api-key"
enabled = true
```

### ~/.ccr/platforms/gemini/profiles.toml (Gemini Profiles)

```toml
default_config = "google"
current_config = "google"

[settings]
skip_confirmation = false

[google]
base_url = "https://api.google.com"
auth_token = "AIzaSy1234567890123456789012345678901234"
model = "gemini-pro"

[profile-1]
description = "Test Gemini profile: thread-1"
base_url = "https://generativelanguage.googleapis.com/v1"
auth_token = "AIzaSy1234567890123456789012345678901234"
model = "gemini-2.0-flash-exp"
small_fast_model = "gemini-1.5-flash"
provider = "Google"

# ... 其他 4 个 profiles
```

## 📚 命令参考

### 平台管理命令

| 命令 | 说明 | 示例 |
|------|------|------|
| `ccr platform list` | 列出所有平台 | `ccr platform list` |
| `ccr platform switch <name>` | 切换到指定平台 | `ccr platform switch codex` |
| `ccr platform current` | 显示当前平台信息 | `ccr platform current` |
| `ccr platform info <name>` | 显示平台详细信息 | `ccr platform info gemini` |
| `ccr platform init <name>` | 初始化新平台 | `ccr platform init qwen` |

### Profile 管理命令

| 命令 | 说明 | 示例 |
|------|------|------|
| `ccr list` | 列出当前平台的所有 profiles | `ccr list` |
| `ccr switch <name>` | 切换到指定 profile | `ccr switch anyrouter` |
| `ccr current` | 显示当前 profile 详情 | `ccr current` |
| `ccr add` | 添加新 profile | `ccr add` |
| `ccr delete <name>` | 删除 profile | `ccr delete test` |

## 🎯 最佳实践

### 1. 平台隔离

每个平台完全独立，互不干扰：
- ✅ 独立的 profiles.toml
- ✅ 独立的操作历史
- ✅ 独立的备份目录
- ✅ 独立的设置文件

### 2. 命名规范

**Claude Platform**:
- 官方中转：使用服务商名称 (anyrouter, husan, duck)
- 第三方模型：使用模型提供商名称 (glm, moonshot, siliconflow)
- 添加描述性 tags: ["free", "paid", "stable", "backup"]

**Codex Platform**:
- OpenAI 官方：default
- 中转站配置：duckcoding、88code 等

**Antigravity CLI (`gemini` key)**:
- Google 官方：google
- 测试配置：profile-1, profile-2, etc.

### 3. 自动检测

CCR 会自动检测并使用 Unified 模式，无需手动设置环境变量：

```bash
# 自动检测逻辑：
# 1. 检查 CCR_ROOT 环境变量
# 2. 检查 ~/.ccr/config.toml 是否存在
# 3. 如果是 Unified 模式，使用平台特定配置
# 4. 否则回退到 Legacy 模式 (~/.ccs_config.toml)
```

### 4. 备份策略

- ✅ 自动备份：每次切换 profile 前自动备份
- ✅ 保留策略：每个平台保留最近 10 个备份
- ✅ 时间戳命名：`settings_20251025_152343.json.bak`
- ✅ 平台隔离：不同平台的备份互不影响

## 🔧 故障排除

### 问题 1: 配置文件不存在

**错误信息**:
```
✗ 配置文件不存在: /home/lyh/.ccs_config.toml
```

**解决方案**:
```bash
# 确保 ~/.ccr/ 目录存在
ls -la ~/.ccr/

# 检查平台配置文件
ls -la ~/.ccr/platforms/*/profiles.toml

# 如果缺少必需字段，手动添加:
# default_config = "profile_name"
# current_config = "profile_name"
```

### 问题 2: 平台切换失败

**错误信息**:
```
✗ 配置格式无效: TOML 解析失败
```

**解决方案**:
```bash
# 检查配置文件格式
cat ~/.ccr/platforms/codex/profiles.toml

# 确保包含必需字段:
# 1. default_config
# 2. current_config
# 3. [settings]
```

### 问题 3: Profile 不显示

**解决方案**:
```bash
# 1. 检查当前平台
ccr platform current

# 2. 切换到正确的平台
ccr platform switch claude

# 3. 列出 profiles
ccr list
```

## 📈 统计信息

**总配置数**: 27 个 Profiles
- Claude: 16 个 (59%)
- Codex: 5 个 (19%)
- Gemini: 6 个 (22%)

**配置类型分布**:
- 官方中转: 12 个 (44%)
- 第三方模型: 4 个 (15%)
- 测试配置: 11 个 (41%)

**活跃平台**: 3 个 (Claude, Codex, Antigravity/Gemini key)
**计划平台**: 1 个 (Qwen)

## 🎉 总结

CCR 多平台配置系统为你提供了：

✅ **完整隔离**: 每个平台独立管理，互不干扰
✅ **灵活切换**: 一键切换平台和 profile
✅ **自动检测**: 无需手动配置，自动识别 Unified 模式
✅ **完整审计**: 每个平台独立的操作历史
✅ **安全备份**: 自动备份机制，保护数据安全

享受多平台配置管理的强大功能吧！ (￣▽￣)ﾉ
