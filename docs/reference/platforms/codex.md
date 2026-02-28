# Codex Platform Guide

## Overview

CCR 支持管理 Codex CLI 配置与多 Profile 切换，采用 **两路分发模式** 处理不同类型的 Provider：

1. **官方模式（Official）**：完全重置 `~/.codex/config.toml` 与 `~/.codex/auth.json` 到默认状态
2. **第三方模式（ThirdParty）**：读取-修改-写入，保留所有非 Provider 相关字段

所有配置写入均使用 **原子操作**（临时文件 + 重命名），并通过 **文件锁** 保证并发安全。

## Platform Information

- **Platform Name**: `codex`
- **Display Name**: Codex CLI
- **Icon**: 💻
- **Status**: ✅ Fully Implemented
- **Codex CLI Config**: `~/.codex/config.toml`
- **Codex CLI Auth**: `~/.codex/auth.json`
- **Profiles Path**: `~/.ccr/platforms/codex/profiles.toml`

## Prerequisites

- Codex CLI 已安装（并使用 `~/.codex/` 配置目录）
- 你所使用 Provider 的 API Token（如 OpenAI 兼容 key）

## Profile 分类机制

CCR 通过 `provider_type` 字段判断 Profile 类型：

| `provider_type` | 分类 | 切换行为 |
|-----------------|------|----------|
| `official_relay` | 官方模式 | 完全重置配置到默认状态 |
| 其他 / 未设置 | 第三方模式 | 保留现有配置，仅更新 Provider 相关字段 |

**回退逻辑**：若未设置 `provider_type`，则根据 `base_url` 判断 —— 无 `base_url` 或为空视为官方模式。

## Token Format

### OpenAI 兼容 Provider

通常使用 OpenAI 兼容的 API key（示例：`sk-...`），最终会被写入 `~/.codex/auth.json`。

## Quick Start

### Initialize Codex Platform

```bash
# Initialize Codex platform (creates directory structure)
ccr platform init codex

# Switch to Codex platform
ccr platform switch codex

# Verify current platform
ccr platform current
```

### Add Your First Profile

```bash
# Interactive mode
ccr add
```

### Configuration Example

在 `~/.ccr/platforms/codex/profiles.toml` 中创建 Profile：

```toml
default_config = "duckcoding"
current_config = "duckcoding"

[settings]
skip_confirmation = false

# 官方模式 - 切换时完全重置配置
[official]
description = "Codex 官方默认配置"
provider = "openai"
provider_type = "official_relay"

# 第三方模式 - 切换时保留非 Provider 字段
[duckcoding]
description = "DuckCoding (OpenAI 兼容)"
base_url = "https://jp.duckcoding.com/v1"
auth_token = "sk-...your-token"
model = "gpt-5.1-codex"
provider = "duckcoding"
provider_type = "third_party_model"
wire_api = "responses"
env_key = "DUCKCODING_API_KEY"
requires_openai_auth = true
approval_policy = "on-request"
sandbox_mode = "workspace-write"
model_reasoning_effort = "high"
network_access = "enabled"
disable_response_storage = true
```

## 两路分发详解

### 官方模式（Official）

当切换到 `provider_type = "official_relay"` 的 Profile 时：

1. **自动备份** 当前 `config.toml` 和 `auth.json`
2. **完全重置** `config.toml` 为空 TOML
3. **完全重置** `auth.json` 为空 JSON
4. 更新 `profiles.toml` 的 `current_config`

适用场景：恢复 Codex CLI 默认行为，使用 OpenAI 官方服务。

### 第三方模式（ThirdParty）

当切换到非官方 Profile 时：

1. **读取** 现有 `config.toml`（保留所有字段）
2. **更新** Provider 相关字段：`model`、`model_provider`、`[model_providers.{id}]`
3. **可选设置** 运行参数：`approval_policy`、`sandbox_mode` 等
4. **原子写入** `config.toml`
5. **更新** `auth.json` 中的 API key
6. 更新 `profiles.toml` 的 `current_config`

适用场景：使用第三方 OpenAI 兼容 Provider，同时保留已有的非 Provider 配置。

## Profile Management

### List Profiles

```bash
# Switch to Codex platform first
ccr platform switch codex

# List all Codex profiles
ccr list
```

### Switch Between Profiles

```bash
# Switch to specific Codex profile
ccr switch duckcoding

# Or use shorthand
ccr duckcoding
```

### Update Profile

```bash
# Edit profiles.toml manually
vim ~/.ccr/platforms/codex/profiles.toml

# Validate changes
ccr validate
```

### Delete Profile

```bash
# Interactive deletion with confirmation
ccr delete old-profile

# Force deletion (skip confirmation)
ccr delete old-profile --force
```

## Codex CLI Config / Auth

当激活 **第三方模式** Profile 时，CCR 会写入：

- `~/.codex/config.toml`（Provider 与运行参数）
- `~/.codex/auth.json`（API key 存放）

`~/.codex/config.toml` 示例：

```toml
model_provider = "duckcoding"
model = "gpt-5.1-codex"
model_reasoning_effort = "high"
approval_policy = "on-request"
sandbox_mode = "workspace-write"
network_access = "enabled"
disable_response_storage = true

[model_providers.duckcoding]
name = "duckcoding"
base_url = "https://jp.duckcoding.com/v1"
wire_api = "responses"
requires_openai_auth = true
env_key = "DUCKCODING_API_KEY"
```

`~/.codex/auth.json` 示例：

```json
{
  "OPENAI_API_KEY": "paste-your-token-here",
  "DUCKCODING_API_KEY": "paste-your-token-here"
}
```

## 原子写入与并发安全

CCR 使用 `CodexConfigManager` 管理 Codex 配置，提供：

| 特性 | 说明 |
|------|------|
| **原子写入** | 通过临时文件 + 重命名，避免写入中断导致配置损坏 |
| **文件锁** | 跨进程锁防止并发写入冲突（资源名：`codex_config`） |
| **自动备份** | 切换官方模式前自动备份，保留最近 10 个备份 |
| **配置缓存** | 30 秒 TTL 缓存，减少重复读取（`CachedCodexConfigManager`） |

备份文件存储在 `~/.codex/backups/`，格式为：
```
config.pre_official.20260225_120000.toml.bak
auth.pre_official.20260225_120000.json.bak
```

## Common Use Cases

### Development Workflow

```bash
# Morning: Start with official config
ccr platform switch codex
ccr switch official

# Afternoon: Test with custom provider
ccr switch duckcoding

# View operation history
ccr history
```

### Multi-Account Management

CCR 为 Codex CLI 提供强大的多账号管理功能，让您可以轻松在不同的账号之间切换。

#### 保存和管理账号

```bash
# 保存当前登录为命名账号
ccr codex auth save work

# 保存时添加描述
ccr codex auth save personal -d "个人账号"

# 保存时设置过期时间
ccr codex auth save temp --expires-at 2026-02-01T00:00:00Z

# 强制覆盖已存在的账号
ccr codex auth save work --force

# 列出所有已保存的账号
ccr codex auth list

# 切换到指定账号
ccr codex auth switch work

# 显示当前账号信息
ccr codex auth current

# 删除账号
ccr codex auth delete old-account

# 删除时跳过确认
ccr codex auth delete old-account --force
```

#### 导出与导入账号

```bash
# 导出所有账号到 Downloads 文件夹
ccr codex auth export

# 导出时不包含敏感数据（Token）
ccr codex auth export --no-secrets

# 从文件导入账号（交互式）
ccr codex auth import

# 使用替换模式导入（覆盖同名账号）
ccr codex auth import --replace

# 使用强制模式导入（合并模式下覆盖已存在账号）
ccr codex auth import --force
```

**导入模式说明：**
- **合并模式（默认）**：跳过已存在的账号，只添加新账号
- **合并 + --force**：强制覆盖已存在的账号，显示被覆盖账号列表
- **替换模式（--replace）**：始终覆盖同名账号

**功能特性：**
- 🟢 Token 新鲜度指示：新鲜 (<1天) | 🟡 陈旧 (1-7天) | 🔴 过期 (>7天)
- 📧 邮箱脱敏保护隐私（如 `use***@example.com`）
- 🔒 自动备份轮转，保留最近 10 个备份
- ⚠️ 切换前进程检测警告
- 🔐 Unix 系统下 auth 文件权限自动设置为 0600

#### 交互式 TUI

启动 Codex 账号管理界面：
```bash
ccr codex
```

**键盘快捷键：**
| 按键 | 功能 |
|------|------|
| `↑` / `↓` / `j` / `k` | 选择账号 |
| `Enter` | 切换到选中的账号并退出 |
| `Space` | 切换到选中的账号（保持 TUI） |
| `q` / `Esc` | 退出 |

### Testing Free Tokens

```bash
# Use temporary token override (doesn't modify profiles.toml)
ccr temp-token set sk-test-free-token-xxxxxxxxxxxx \
  --base-url https://api.example.com/v1

# Verify temporary config
ccr temp-token show

# Apply and auto-clear
ccr switch duckcoding

# Next switch uses permanent config
ccr switch duckcoding
```

## Platform-Specific Features

### Token Validation

CCR 自动验证 Codex Profile：

```bash
# Validate all Codex profiles
ccr validate

# Output includes:
# ✅ 官方模式 Profile 无需验证
# ✅ 第三方模式 Profile：检查 base_url、auth_token、wire_api
# ❌ 旧版 api_mode=github Profile 返回弃用错误
```

### Backup and Restore

```bash
# Automatic backup before official profile switch
ccr switch official
# → Creates ~/.codex/backups/config.pre_official.{timestamp}.toml.bak
# → Creates ~/.codex/backups/auth.pre_official.{timestamp}.json.bak

# Manual backup
ccr backup codex

# List backups
ccr backups list

# Restore from backup
ccr restore ~/.ccr/backups/codex/settings_20250125_120000.json.bak
```

### History Tracking

```bash
# View Codex-specific history
ccr history --platform codex

# View recent 10 operations
ccr history -l 10

# Filter by operation type
ccr history -t switch
```

## Migration Guide

### From Legacy GitHub Mode

> **Note**: GitHub Copilot 兼容模式（`api_mode: "github"`）已在 v4.2.6 中弃用并移除。
> 如果你有旧的 GitHub 模式 Profile，切换时会收到明确的弃用错误提示。

迁移步骤：

1. 删除旧的 GitHub 模式 Profile
2. 根据实际需求创建新的官方模式或第三方模式 Profile

```bash
# 删除旧的 GitHub 模式 Profile
ccr delete github-old

# 创建新的第三方模式 Profile
ccr add
# 按提示输入新的配置信息
```

### From Other Platforms

```bash
# Migrate from Claude to Codex
ccr platform migrate claude codex

# This creates equivalent Codex profiles from Claude profiles
# (Note: Requires manual token replacement)
```

## Troubleshooting

### Issue: Legacy GitHub Profile Error

**Symptoms:**
```
❌ GitHub Copilot 兼容模式已弃用，请使用第三方模式替代
```

**Solution:**
1. 删除旧的 `api_mode = "github"` Profile
2. 按照第三方模式重新创建 Profile
3. 参见上方"Migration Guide"章节

### Issue: Codex CLI Not Found

**Symptoms:**
```
⚠️ Codex CLI not detected in PATH
```

**Solution:**
CCR 仅管理配置文件。如需 Codex CLI 本体：
1. 单独安装 Codex CLI
2. CCR 会继续管理你的 Profile 和配置

### Issue: Settings Not Updating

**Symptoms:**
Profile 切换成功但 `~/.codex/config.toml` 未变更

**Solution:**
```bash
# Check file permissions
ls -la ~/.codex/config.toml

# Fix permissions if needed
chmod 600 ~/.codex/config.toml

# Verify lock files
ls -la ~/.ccr/.locks/

# Clean stale locks if present
rm -rf ~/.ccr/.locks/*
```

### Issue: Profile Conflicts

**Symptoms:**
```
❌ Profile name already exists
```

**Solution:**
```bash
# List existing profiles
ccr list

# Delete conflicting profile
ccr delete conflicting-name

# Or use different name
ccr add  # Enter unique name
```

## Advanced Configuration

### Custom API Endpoints

```toml
[custom-provider]
description = "Custom OpenAI-compatible Provider"
base_url = "https://api.custom-provider.example.com/v1"
auth_token = "sk-xxx"
model = "gpt-4"
provider = "custom"
provider_type = "third_party_model"
wire_api = "responses"
env_key = "CUSTOM_API_KEY"
requires_openai_auth = true
```

### Model Selection

```toml
# Use high-end model for main tasks
[premium]
description = "Premium with GPT-4"
base_url = "https://api.example.com/v1"
auth_token = "sk-xxx"
model = "gpt-4"
small_fast_model = "gpt-3.5-turbo"
provider = "premium"
provider_type = "third_party_model"

# Use fast model for speed
[fast]
description = "Fast with GPT-3.5"
base_url = "https://api.example.com/v1"
auth_token = "sk-xxx"
model = "gpt-3.5-turbo"
small_fast_model = "gpt-3.5-turbo"
provider = "fast"
provider_type = "third_party_model"
```

### WebDAV Sync

```bash
# Configure sync for Codex profiles
ccr sync config

# Push Codex profiles to cloud
ccr platform switch codex
ccr sync push

# Pull on another machine
ccr platform switch codex
ccr sync pull
```

## Security Best Practices

1. **Token Storage**: Tokens 以明文存储在 `~/.ccr/platforms/codex/profiles.toml`
   ```bash
   # Ensure proper file permissions
   chmod 600 ~/.ccr/platforms/codex/profiles.toml
   ```

2. **Token Masking**: CCR 会在以下场景自动掩码 Token：
   - 控制台输出
   - 历史日志
   - 错误消息

3. **Backup Security**: 备份文件同样包含 Token
   ```bash
   # Secure backup directory
   chmod 700 ~/.ccr/backups/codex
   chmod 700 ~/.codex/backups
   ```

4. **Export Without Secrets**:
   ```bash
   # Export profiles without tokens (for sharing)
   ccr export -o codex-profiles.toml --no-secrets
   ```

5. **Token Rotation**: 定期更换 API Token
   ```bash
   # Update profile with new token
   vim ~/.ccr/platforms/codex/profiles.toml
   ccr validate  # Verify format
   ```

## Related Commands

```bash
# Platform management
ccr platform list           # List all platforms
ccr platform switch codex   # Switch to Codex
ccr platform current        # Show current platform

# Profile management
ccr list                    # List Codex profiles
ccr switch <name>           # Switch Codex profile
ccr add                     # Add new profile
ccr delete <name>           # Delete profile

# Multi-account management
ccr codex auth save <name>  # Save current login as named account
ccr codex auth list         # List all saved accounts
ccr codex auth switch <name> # Switch to specific account
ccr codex auth current      # Show current account info
ccr codex auth delete <name> # Delete account
ccr codex auth export       # Export accounts to file
ccr codex auth import       # Import accounts from file
ccr codex                   # Launch interactive TUI

# Validation and diagnostics
ccr validate                # Validate all profiles
ccr history                 # View operation history

# Backup and restore
ccr backups list            # List backups
ccr restore <file>          # Restore from backup
```

## See Also

- [Migration Guide](./migration.md) - Migrating between platforms
- [Gemini Platform Guide](./gemini.md) - Gemini CLI configuration
- [Main README](../../README.md) - CCR overview
