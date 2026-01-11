# Factory Droid Platform Guide

## Overview

Factory Droid is an AI-powered CLI assistant developed by Factory AI. CCR provides comprehensive support for managing Droid configurations, custom models, and profiles.

## Platform Information

- **Platform Name**: `droid`
- **Display Name**: Factory Droid
- **Icon**: 🏭
- **Status**: ✅ Fully Implemented
- **Settings Path**: `~/.factory/settings.json`
- **Profiles Path**: `~/.ccr/platforms/droid/profiles.toml`

## Prerequisites

- Factory Droid CLI installed
- API key from your chosen provider (Anthropic, OpenAI, or compatible services)

## Installation

### Windows

```powershell
irm https://app.factory.ai/cli/windows | iex
```

### macOS/Linux

```bash
curl -fsSL https://app.factory.ai/cli/install | bash
```

## Token Formats

Droid supports multiple providers with different token formats:

### Anthropic

```
sk-ant-api03-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

**How to Obtain:**
1. Go to [Anthropic Console](https://console.anthropic.com/)
2. Navigate to API Keys section
3. Click "Create API Key"
4. Copy and store securely

### OpenAI

```
sk-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

**How to Obtain:**
1. Go to [OpenAI Platform](https://platform.openai.com/api-keys)
2. Create new secret key
3. Copy and store securely

### Generic Chat Completion API

Supports OpenRouter, Ollama, vLLM, and other OpenAI-compatible services.

## Quick Start

### Initialize Droid Platform

```bash
# Initialize Droid platform (creates directory structure)
ccr platform init droid

# Verify initialization
ccr platform info droid
```

### Switch to Droid Platform

```bash
# Switch from another platform to Droid
ccr platform switch droid

# Verify current platform
ccr platform current
```

### Add Droid Profile

```bash
# Interactive profile creation
ccr add -p droid

# Follow the prompts to enter:
# - Profile name
# - Description
# - Base URL
# - API token
# - Model name
# - Provider type
```

## Configuration Examples

### Example 1: Anthropic API

```toml
[anthropic-api]
description = "Anthropic Official API"
base_url = "https://api.anthropic.com/v1"
auth_token = "sk-ant-api03-your-token-here"
model = "claude-sonnet-4-5-20250929"
provider = "anthropic"
max_output_tokens = 8192
tags = ["official", "anthropic"]
enabled = true
```

### Example 2: OpenAI Compatible

```toml
[openai-compatible]
description = "OpenAI Compatible API"
base_url = "https://api.openai.com/v1"
auth_token = "sk-your-openai-key"
model = "gpt-4o"
provider = "openai"
max_output_tokens = 4096
tags = ["openai", "gpt"]
enabled = true
```

### Example 3: OpenRouter

```toml
[openrouter]
description = "OpenRouter Aggregated API"
base_url = "https://openrouter.ai/api/v1"
auth_token = "sk-or-v1-your-openrouter-key"
model = "anthropic/claude-3.5-sonnet"
provider = "generic-chat-completion-api"
max_output_tokens = 4096
tags = ["openrouter", "aggregator"]
enabled = true
```

### Example 4: Local Ollama

```toml
[ollama-local]
description = "Ollama Local Deployment"
base_url = "http://localhost:11434/v1"
auth_token = "ollama"
model = "qwen3:14b"
provider = "generic-chat-completion-api"
tags = ["local", "ollama"]
enabled = true
```

## Profile Management

### List All Droid Profiles

```bash
# Make sure you're on Droid platform
ccr platform switch droid

# List all profiles
ccr list
```

### Switch Between Profiles

```bash
# Switch to specific profile
ccr switch anthropic-api

# Or use -p flag from any platform
ccr switch -p droid openrouter
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

# Force delete without confirmation
ccr delete old-profile --force
```

## Provider Types

Droid supports three provider types:

| Provider | Description |
|----------|-------------|
| `anthropic` | Anthropic API (Claude models) |
| `openai` | OpenAI API (GPT models) |
| `generic-chat-completion-api` | OpenAI-compatible services (OpenRouter, Ollama, vLLM, etc.) |

## Configuration Fields

| Field | Required | Description |
|-------|----------|-------------|
| `description` | ❌ | Human-readable profile description |
| `base_url` | ✅ | API endpoint URL |
| `auth_token` | ✅ | API key / authentication token |
| `model` | ❌ | Model identifier |
| `provider` | ✅ | Provider type |
| `max_output_tokens` | ❌ | Maximum output tokens (Droid-specific) |
| `tags` | ❌ | Organization tags |
| `enabled` | ❌ | Profile enabled status |

## Common Workflows

### Workflow 1: Testing Different Providers

```bash
# Test Anthropic API
ccr switch -p droid anthropic-api

# Test OpenRouter
ccr switch -p droid openrouter

# Test local Ollama
ccr switch -p droid ollama-local
```

### Workflow 2: Development vs Production

```bash
# Development with free tier
ccr switch -p droid ollama-local

# Production with official API
ccr switch -p droid anthropic-api
```

## Troubleshooting

### Issue 1: Profile Not Found

**Error:**
```
✗ Profile not found: anthropic-api
```

**Solution:**
```bash
# Check current platform
ccr platform current

# Switch to Droid platform
ccr platform switch droid

# Verify profile exists
ccr list
```

### Issue 2: Configuration Not Applied

**Error:**
Settings file not updated after switching.

**Solution:**
```bash
# Check settings file
cat ~/.factory/settings.json

# Verify current config
ccr current

# Re-apply profile
ccr switch anthropic-api
```

### Issue 3: API Key Invalid

**Error:**
API calls fail with authentication error.

**Solution:**
- Verify token format matches provider
- Check token is complete (no truncation)
- Ensure base_url matches provider

## Best Practices

### 1. Organize by Environment

```toml
# Production
[prod-anthropic]
tags = ["production", "anthropic"]

# Development
[dev-ollama]
tags = ["development", "local"]

# Testing
[test-openrouter]
tags = ["testing", "aggregator"]
```

### 2. Use Descriptive Names

```toml
# Good naming
[anthropic-claude-sonnet]
[openrouter-claude-haiku]
[ollama-qwen-local]

# Avoid generic names
[config1]
[test]
```

### 3. Set Appropriate Token Limits

```toml
[openai-gpt4]
max_output_tokens = 4096  # GPT-4 limit

[anthropic-claude]
max_output_tokens = 8192  # Claude limit
```

## Related Resources

- [Claude Platform](claude.md) - Anthropic Claude Code CLI
- [Codex Platform](codex.md) - GitHub Copilot CLI
- [Gemini Platform](gemini.md) - Google Gemini CLI
- [Migration Guide](migration.md) - Cross-platform migration

---

*For platform-specific API documentation, visit [Factory AI Documentation](https://factory.ai/docs).*
