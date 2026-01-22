# Codex Platform Guide

## Overview

CCR 支持管理 Codex CLI 配置与多 profile 切换，并同时兼容两种常见工作模式：

1. **OpenAI 兼容 Provider（推荐）**：写入 `~/.codex/config.toml` 与 `~/.codex/auth.json`
2. **GitHub Copilot 兼容模式（可选）**：写入 `~/.codex/settings.json`

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
- 你所使用 Provider 的 API Token（如 OpenAI 兼容 key、GitHub Token 等）

## Token Format

### OpenAI 兼容 Provider（推荐）

通常使用 OpenAI 兼容的 API key（示例：`sk-...`），最终会被写入 `~/.codex/auth.json`。

### GitHub Copilot 兼容模式（可选）

CCR 会校验 GitHub Token 前缀：

- `ghp_`（PAT）
- `gho_`（OAuth）
- `github_pat_`（fine-grained PAT）

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

Create a profile in `~/.ccr/platforms/codex/profiles.toml`:

```toml
default_config = "duckcoding"
current_config = "duckcoding"

[settings]
skip_confirmation = false

[duckcoding]
description = "DuckCoding (OpenAI 兼容)"
base_url = "https://jp.duckcoding.com/v1"
auth_token = "sk-...your-token"
model = "gpt-5.1-codex"
provider = "duckcoding"
api_mode = "custom"
wire_api = "responses"
env_key = "DUCKCODING_API_KEY"
requires_openai_auth = true
approval_policy = "on-request"
sandbox_mode = "workspace-write"
model_reasoning_effort = "high"
network_access = "enabled"
disable_response_storage = true

[github]
description = "GitHub Copilot (legacy)"
base_url = "https://api.github.com/copilot"
auth_token = "ghp_...your-github-token"
model = "gpt-4"
provider = "github"
api_mode = "github"
```

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
ccr switch github-official

# Or use shorthand
ccr github-official
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
ccr delete github-enterprise

# Force deletion (skip confirmation)
ccr delete github-enterprise --force
```

## Codex CLI Config / Auth

当激活 **OpenAI 兼容 Provider** profile 时，CCR 会写入：

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

## Common Use Cases

### Development Workflow

```bash
# Morning: Start with GitHub official
ccr platform switch codex
ccr switch github-official

# Afternoon: Test with enterprise token
ccr switch github-enterprise

# View operation history
ccr history
```

### Multi-Account Management

CCR 为 Codex CLI 提供强大的多账号管理功能，让您可以轻松在不同的 GitHub 账号之间切换。

#### 保存和管理账号

```bash
# 保存当前登录为命名账号
ccr codex auth save work

# 保存时添加描述
ccr codex auth save personal -d "个人 GitHub 账号"

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
ccr temp-token set ghp_test_free_token_xxxxxxxxxxxx \
  --base-url https://api.github.com/copilot

# Verify temporary config
ccr temp-token show

# Apply and auto-clear
ccr switch github-official

# Next switch uses permanent config
ccr switch github-official
```

## Platform-Specific Features

### Token Validation

CCR validates Codex tokens automatically:

```bash
# Validate all Codex profiles
ccr validate

# Output includes:
# ✅ Valid GitHub token format (ghp_...)
# ❌ Invalid token format
# ⚠️ Token format correct but not verified active
```

### Backup and Restore

```bash
# Automatic backup before profile switch
ccr switch new-profile
# → Creates ~/.ccr/backups/codex/settings_20250125_120000.json.bak

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

### From Legacy CCS Configuration

If you were using CCS (shell version) for GitHub Copilot:

```bash
# Old CCS config (before CCR multi-platform)
# ~/.ccs_config.toml
[github-copilot]
description = "GitHub Copilot"
base_url = "https://api.github.com/copilot"
auth_token = "ghp_xxx"
model = "gpt-4"

# Migrate to CCR Codex platform
ccr platform init codex
ccr platform migrate claude codex  # Migrate compatible profiles

# Or manually recreate profiles
ccr platform switch codex
ccr add  # Follow interactive prompts
```

### From Other Platforms

```bash
# Migrate from Claude to Codex
ccr platform migrate claude codex

# This creates equivalent Codex profiles from Claude profiles
# (Note: Requires manual token replacement)
```

## Troubleshooting

### Issue: Token Invalid

**Symptoms:**
```
❌ Invalid GitHub token format
```

**Solution:**
1. Verify token starts with `ghp_` and is 40 characters
2. Regenerate token on GitHub if necessary
3. Update profile with new token

### Issue: Codex CLI Not Found

**Symptoms:**
```
⚠️ Codex CLI not detected in PATH
```

**Solution:**
CCR manages configuration files only. If you need the actual Codex CLI:
1. Install Codex CLI separately
2. CCR will still manage your profiles and settings

### Issue: Settings Not Updating

**Symptoms:**
Profile switch command succeeds but `~/.codex/config.toml` unchanged

**Solution:**
```bash
# Check file permissions
ls -la ~/.codex/config.toml

# Fix permissions if needed
chmod 600 ~/.codex/config.toml

# Verify lock files
ls -la ~/.claude/.locks/

# Clean stale locks if present
rm -rf ~/.claude/.locks/*
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
[github-proxy]
description = "GitHub via Proxy"
base_url = "https://github-proxy.example.com/api/copilot"
auth_token = "ghp_xxx"
model = "gpt-4"
```

### Model Selection

```toml
# Use GPT-4 for main tasks
[premium]
description = "Premium with GPT-4"
auth_token = "ghp_xxx"
model = "gpt-4"
small_fast_model = "gpt-3.5-turbo"

# Use GPT-3.5 for speed
[fast]
description = "Fast with GPT-3.5"
auth_token = "ghp_xxx"
model = "gpt-3.5-turbo"
small_fast_model = "gpt-3.5-turbo"
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

1. **Token Storage**: Tokens are stored in plaintext in `~/.ccr/platforms/codex/profiles.toml`
   ```bash
   # Ensure proper file permissions
   chmod 600 ~/.ccr/platforms/codex/profiles.toml
   ```

2. **Token Masking**: CCR automatically masks tokens in:
   - Console output
   - History logs
   - Error messages

3. **Backup Security**: Backups also contain tokens
   ```bash
   # Secure backup directory
   chmod 700 ~/.ccr/backups/codex
   ```

4. **Export Without Secrets**:
   ```bash
   # Export profiles without tokens (for sharing)
   ccr export -o codex-profiles.toml --no-secrets
   ```

5. **Token Rotation**: Regularly rotate GitHub tokens
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
- [GitHub Copilot Docs](https://docs.github.com/en/copilot) - Official GitHub Copilot documentation
