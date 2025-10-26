# Claude Code Platform Guide

## Overview

Claude Code is Anthropic's official command-line interface platform. CCR provides comprehensive support for managing Claude Code configurations, profiles, and settings.

## Platform Information

- **Platform Name**: `claude`
- **Display Name**: Claude Code AI Assistant
- **Icon**: 🤖
- **Status**: ✅ Fully Implemented
- **Settings Path**: `~/.claude/settings.json`
- **Profiles Path**: `~/.ccr/platforms/claude/profiles.toml` (Unified mode) or `~/.ccs_config.toml` (Legacy mode)

## Prerequisites

- Claude Code CLI installed
- Anthropic API key or relay service token
- Active Anthropic account (for official API)

## Token Formats

Claude Code supports multiple token formats depending on your service provider:

### Official Anthropic API

```
sk-ant-api03-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

**Characteristics:**
- Starts with `sk-ant-api03-`
- 108 characters total
- Base64-encoded characters

**How to Obtain:**

1. Go to [Anthropic Console](https://console.anthropic.com/)
2. Navigate to API Keys section
3. Click "Create API Key"
4. Copy the key immediately (shown only once)
5. Store securely (never commit to version control)

### Relay Services

CCR supports various relay services with different token formats:

#### AnyRouter
```
sk-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```
- Free tier available with GitHub/LinuxDo account
- Multiple models support
- Good for testing and development

#### HuSan API
```
sk-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```
- Paid service with high-speed access
- Stable performance
- Recommended for production use

#### Third-Party Model Services

##### Zhipu GLM (智谱GLM)
```
api_key.XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```
- Chinese LLM service
- Official GLM-4.6 model
- Good Chinese language support

##### Moonshot (月之暗面)
```
sk-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```
- Kimi K2 model
- Fast response
- Chinese language optimized

## Quick Start

### Initialize Claude Platform

```bash
# Initialize Claude platform (creates directory structure)
ccr platform init claude

# Verify initialization
ccr platform info claude
```

### Switch to Claude Platform

```bash
# Switch from another platform to Claude
ccr platform switch claude

# Verify current platform
ccr platform current
```

### Add Claude Profile

```bash
# Interactive profile creation
ccr add

# Follow the prompts to enter:
# - Profile name (e.g., "anthropic", "anyrouter")
# - Description
# - Base URL
# - API token
# - Model name
# - Small fast model (optional)
```

### Example Profile Configuration

**Official Anthropic API:**

```toml
[anthropic]
description = "Anthropic Official API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-api03-..."
model = "claude-sonnet-4-5-20250929"
small_fast_model = "claude-3-5-haiku-20241022"
provider = "Anthropic"
provider_type = "official_relay"
tags = ["official", "stable", "primary"]
```

**AnyRouter Relay:**

```toml
[anyrouter]
description = "AnyRouter Free Service (GitHub Account)"
base_url = "https://anyrouter.top"
auth_token = "sk-..."
provider = "anyrouter"
provider_type = "official_relay"
account = "github_5953"
tags = ["free", "stable", "primary"]
```

**Third-Party Model (GLM):**

```toml
[glm]
description = "Zhipu GLM-4.6 Model"
base_url = "https://open.bigmodel.cn/api/paas/v4"
auth_token = "api_key...."
model = "glm-4.6"
provider = "glm"
provider_type = "third_party_model"
tags = ["chinese", "official", "coding"]
```

## Profile Management

### List All Claude Profiles

```bash
# Make sure you're on Claude platform
ccr platform switch claude

# List all profiles
ccr list
```

### Switch Between Profiles

```bash
# Switch to specific profile
ccr switch anthropic

# Or use short form (profile name as direct command)
ccr anyrouter
```

### View Current Profile

```bash
# Show current active profile with details
ccr current
```

### Delete Profile

```bash
# Delete a profile (with confirmation)
ccr delete old-profile

# Force delete without confirmation (not recommended)
ccr delete old-profile --force
```

## Configuration Examples

### Example 1: Multiple Relay Services

```toml
default_config = "anyrouter"
current_config = "husan"

[settings]
skip_confirmation = false

# Primary free service
[anyrouter]
description = "AnyRouter Main Service (github_5953)"
base_url = "https://anyrouter.top"
auth_token = "sk-gCJhGGGIDEKDFVTM3NYa8M4XWM8MsgU0pWhreTFg3oI0Pzi2"
provider = "anyrouter"
provider_type = "official_relay"
account = "github_5953"
tags = ["free", "stable", "primary"]

# Backup free service
[anyrouter2]
description = "AnyRouter Backup 1 (github_5962)"
base_url = "https://anyrouter.top"
auth_token = "sk-..."
provider = "anyrouter"
provider_type = "official_relay"
account = "github_5962"
tags = ["free", "backup"]

# Paid high-speed service
[husan]
description = "HuSan API"
base_url = "https://husanai.com"
auth_token = "sk-uyv3753vanVsmbdeHRwpx8mD0EREkewvf3WuIkohYCcQvh21"
provider = "husan"
provider_type = "official_relay"
tags = ["paid", "stable", "high-speed"]
```

### Example 2: Mixed Configuration (Relay + Third-Party)

```toml
default_config = "anthropic"
current_config = "anthropic"

[settings]
skip_confirmation = false

# Official Anthropic API
[anthropic]
description = "Anthropic Official API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-api03-..."
model = "claude-sonnet-4-5-20250929"
small_fast_model = "claude-3-5-haiku-20241022"
provider = "Anthropic"
provider_type = "official_relay"
tags = ["official", "stable", "primary"]

# AnyRouter relay
[anyrouter]
description = "AnyRouter Relay"
base_url = "https://anyrouter.top"
auth_token = "sk-..."
provider = "anyrouter"
provider_type = "official_relay"
tags = ["free", "relay"]

# Zhipu GLM third-party model
[glm]
description = "Zhipu GLM-4.6"
base_url = "https://open.bigmodel.cn/api/paas/v4"
auth_token = "api_key...."
model = "glm-4.6"
provider = "glm"
provider_type = "third_party_model"
tags = ["chinese", "third-party", "coding"]

# Moonshot Kimi K2
[moonshot]
description = "Moonshot Kimi K2 Turbo"
base_url = "https://api.moonshot.cn/v1"
auth_token = "sk-..."
model = "kimi-k2-turbo-preview"
provider = "moonshot"
provider_type = "third_party_model"
tags = ["chinese", "kimi", "fast"]
```

## Advanced Features

### Provider Classification

CCR supports classifying Claude profiles by provider:

- **provider**: Provider name (e.g., "anyrouter", "glm", "moonshot")
- **provider_type**: Type of provider
  - `official_relay`: Services providing official Claude models
  - `third_party_model`: Services providing their own models

### Account Tracking

Track multiple accounts from the same provider:

```toml
[anyrouter]
account = "github_5953"
tags = ["free", "stable", "primary"]

[anyrouter2]
account = "linuxdo_79797"
tags = ["free", "backup"]
```

### Tag-based Organization

Use tags to organize and filter profiles:

```toml
tags = ["free", "stable", "primary"]    # Free, stable, primary service
tags = ["paid", "high-speed"]           # Paid high-speed service
tags = ["chinese", "official"]          # Chinese official service
tags = ["backup"]                        # Backup configuration
```

## Temporary Token Override

Need to test a token temporarily without modifying your config file?

```bash
# Set temporary token (one-time use)
ccr temp-token set sk-free-test-xxx

# Optional: Override additional fields
ccr temp-token set sk-xxx \
  --base-url https://api.temp.com \
  --model claude-opus-4

# View current temporary config
ccr temp-token show

# Apply temporary config (auto-applies and clears after use)
ccr switch anyrouter

# Next switch uses permanent config
ccr switch anyrouter  # Uses config from profiles.toml
```

## Migration from Legacy Mode

If you're using Legacy mode (`~/.ccs_config.toml`), you can migrate to Unified mode:

```bash
# Check if you should migrate
ccr migrate --check

# Migrate all Claude profiles to Unified mode
ccr migrate --platform claude

# Or migrate all platforms at once
ccr migrate
```

### Migration Effects

**Before (Legacy Mode):**
```
~/.ccs_config.toml        # All profiles in one file
~/.claude/settings.json   # Claude settings
```

**After (Unified Mode):**
```
~/.ccr/
  ├── config.toml                      # Platform registry
  └── platforms/
      └── claude/
          ├── profiles.toml            # Claude profiles (migrated)
          ├── history.json             # Operation history
          └── backups/                 # Automatic backups
```

## Common Workflows

### Workflow 1: Quick Switch Between Relay Services

```bash
# List all relay services
ccr list

# Switch to free service for testing
ccr switch anyrouter

# Switch to paid service for production
ccr switch husan
```

### Workflow 2: Testing Different Models

```bash
# Test Moonshot Kimi K2
ccr switch moonshot

# Test Zhipu GLM
ccr switch glm

# Test SiliconFlow
ccr switch siliconflow
```

### Workflow 3: Multi-Account Management

```bash
# Primary GitHub account
ccr switch anyrouter

# Backup LinuxDo account
ccr switch anyrouter3

# Student account
ccr switch anyrouter4
```

## Troubleshooting

### Issue 1: Profile Not Found

**Error:**
```
✗ 配置段未找到: anyrouter
```

**Solution:**
```bash
# Check current platform
ccr platform current

# Make sure you're on Claude platform
ccr platform switch claude

# Verify profile exists
ccr list
```

### Issue 2: Settings File Not Updated

**Error:**
Settings don't change after switching profiles.

**Solution:**
```bash
# Check settings file
cat ~/.claude/settings.json

# Verify current config
ccr current

# Try switching again with validation
ccr validate
ccr switch anyrouter
```

### Issue 3: Token Format Error

**Error:**
```
✗ 配置验证失败: auth_token 格式无效
```

**Solution:**
- Verify token format matches provider requirements
- Check for extra spaces or newlines
- Ensure token is complete and not truncated
- Re-copy token from source

## Best Practices

### 1. Organize by Purpose

```toml
# Production
[anthropic]
tags = ["production", "official"]

# Development
[anyrouter]
tags = ["development", "free"]

# Testing
[test]
tags = ["testing", "temporary"]
```

### 2. Use Descriptive Names

```toml
# Good naming
[anyrouter-github-primary]
[husan-prod]
[glm-chinese-dev]

# Avoid generic names
[config1]
[test]
[temp]
```

### 3. Document Your Profiles

```toml
[anyrouter]
description = "AnyRouter Free Service (GitHub Account #5953) - For development and testing"
account = "github_5953"
tags = ["free", "development", "primary"]
```

### 4. Regular Cleanup

```bash
# Remove unused profiles
ccr delete old-profile

# Clean old backups (older than 7 days)
ccr clean -d 7

# Optimize config file structure
ccr optimize
```

## Environment Variables

When you switch to a Claude profile, CCR sets these environment variables in `~/.claude/settings.json`:

- `ANTHROPIC_BASE_URL` - API endpoint
- `ANTHROPIC_AUTH_TOKEN` - Authentication token (masked in logs)
- `ANTHROPIC_MODEL` - Default model
- `ANTHROPIC_SMALL_FAST_MODEL` - Fast model (optional)

## Related Resources

- [Configuration Guide](../configuration.md) - Detailed configuration reference
- [Migration Guide](migration.md) - Legacy to Unified mode migration
- [Quick Start](../quick-start.md) - Getting started guide
- [Codex Platform](codex.md) - GitHub Copilot CLI platform
- [Gemini Platform](gemini.md) - Google Gemini CLI platform

## Statistics

**Current Claude Platform (Example Configuration):**
- **Total Profiles**: 16
- **Relay Services**: 12 (75%)
- **Third-Party Models**: 4 (25%)
- **Free Services**: 8 (50%)
- **Paid Services**: 4 (25%)

**Profile Distribution:**
- Official Relay: anyrouter (4 accounts), husan, duck, ikun, etc.
- Third-Party Models: glm, moonshot, siliconflow, modelscope

---

*For more platform options, see [Codex](codex.md) and [Gemini](gemini.md) platform guides.*
