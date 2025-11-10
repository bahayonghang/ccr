# Claude Code Platform

CCR's primary and most mature platform support for Anthropic's Claude Code CLI.

## Overview

Claude Code is Anthropic's official command-line interface for interacting with Claude AI. CCR provides comprehensive management for Claude Code configurations, including multiple API providers and profile switching.

## Features

- ✅ Direct `settings.json` manipulation
- ✅ Multiple profile support
- ✅ Atomic operations with file locking
- ✅ Complete audit trail
- ✅ Automatic backups
- ✅ Environment variable management

## Configuration

### Settings Path

```
~/.claude/settings.json
```

### Environment Variables

CCR manages these Claude Code environment variables:

- `ANTHROPIC_BASE_URL` - API endpoint
- `ANTHROPIC_AUTH_TOKEN` - Authentication token
- `ANTHROPIC_MODEL` - Default model
- `ANTHROPIC_SMALL_FAST_MODEL` - Fast model for quick tasks

### Example Configuration

```toml
[profiles.anthropic-official]
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-api03-..."
model = "claude-3-5-sonnet-20241022"
small_fast_model = "claude-3-5-haiku-20241022"
provider = "anthropic"
```

## Supported Providers

### Official Anthropic API

```bash
ccr add
# Profile name: anthropic-official
# Base URL: https://api.anthropic.com
# Token: sk-ant-api03-...
# Model: claude-3-5-sonnet-20241022
```

### Third-Party Providers

**OpenRouter:**
```bash
ccr add
# Profile name: openrouter
# Base URL: https://openrouter.ai/api/v1
# Token: sk-or-...
# Model: anthropic/claude-3.5-sonnet
```

**Duck.ai (Free):**
```bash
ccr add
# Profile name: duck-free
# Base URL: https://duckduckgo-duck-api-gateway...
# Token: (check duck.ai for current endpoint)
```

## Quick Start

### Initialize Claude Platform

```bash
# Unified Mode (default)
ccr platform init claude

# Legacy Mode
export CCR_LEGACY_MODE=1
ccr init
```

### Add Claude Profile

```bash
ccr add
```

Interactive prompts:
1. **Profile Name**: Identifier for this configuration
2. **API Base URL**: Endpoint (default: https://api.anthropic.com)
3. **API Token**: Your authentication token
4. **Model**: Default model (optional)
5. **Small Fast Model**: Fast model (optional)

### Switch Profiles

```bash
# List available profiles
ccr list

# Switch to specific profile
ccr switch anthropic-official

# Or use shorthand
ccr anthropic-official
```

## Common Operations

### View Current Configuration

```bash
ccr current
```

Output includes:
- Current profile name
- API endpoint
- Active models
- Environment variables (masked)

### Validate Configuration

```bash
ccr validate
```

Checks:
- Required fields presence
- URL format validity
- Token format
- Settings file integrity

### Operation History

```bash
# View all operations
ccr history

# Last 10 operations
ccr history -l 10

# Filter by operation type
ccr history -t switch
```

## Advanced Usage

### Temporary Token Override

Test different tokens without modifying profiles:

```bash
# Set temporary token
ccr temp-token set sk-test-token \
  --base-url https://test.api.com \
  --model claude-3-5-sonnet-20241022

# View temporary config
ccr temp-token show

# Use it (auto-applied on next switch)
ccr switch anthropic

# Cleared after use
```

### Multiple Development Environments

```bash
# Development
ccr add  # Name: dev-claude

# Staging
ccr add  # Name: staging-claude

# Production
ccr add  # Name: prod-claude

# Quick switching
ccr dev-claude
ccr staging-claude
ccr prod-claude
```

### API Provider Testing

```bash
# Test official API
ccr switch anthropic-official

# Test free provider
ccr switch duck-free

# Compare response times/quality
ccr current  # Check which is active
```

## File Structure

### Unified Mode

```
~/.ccr/platforms/claude/
├── profiles.toml        # Profile configurations
├── history.json         # Operation history
└── backups/             # Settings backups
    ├── settings_20250101_120000.json.bak
    └── settings_20250101_130000.json.bak

~/.claude/
└── settings.json        # Claude Code settings (managed by CCR)
```

### Legacy Mode

```
~/.ccs_config.toml       # Profile configurations
~/.claude/
├── settings.json        # Claude Code settings
├── backups/             # Automatic backups
├── ccr_history.json     # Operation audit trail
└── .locks/              # File locks
```

## Best Practices

### Security

```bash
# Use descriptive names
anthropic-official     # ✅ Clear
my-api                 # ❌ Vague

# Export without secrets for sharing
ccr export --no-secrets -o template.toml

# Regular validation
ccr validate
```

### Organization

```bash
# Prefix by environment
dev-anthropic
staging-anthropic
prod-anthropic

# Or by provider
anthropic-official
anthropic-bedrock
openrouter-claude
```

### Maintenance

```bash
# Regular backups
ccr export -o backup-$(date +%Y%m%d).toml

# Clean old backups
ccr clean -d 30

# Update CCR regularly
ccr update --check
```

## Troubleshooting

### Common Issues

**Issue: Settings not taking effect**
```bash
# Verify current configuration
ccr current

# Validate settings
ccr validate

# Check Claude Code is using correct settings
cat ~/.claude/settings.json
```

**Issue: Lock timeout**
```bash
# Check for stale locks
ls -la ~/.claude/.locks/

# Remove if no CCR process running
rm -rf ~/.claude/.locks/*
```

**Issue: Multiple profiles conflict**
```bash
# Ensure only one profile is current
ccr current

# Switch explicitly
ccr switch anthropic-official
```

## Integration

### With Claude Code

```bash
# Switch profile, then use Claude Code normally
ccr switch prod-claude
claude "Your prompt here"
```

### With Scripts

```bash
#!/bin/bash
# deploy.sh

ccr switch prod-claude
# Run deployment commands
ccr history >> deployment.log
```

### With CI/CD

```bash
# Use environment variable for token
export ANTHROPIC_API_KEY=$CI_SECRET_TOKEN

ccr temp-token set $ANTHROPIC_API_KEY
ccr switch production
# Run tests/deployment
```

## See Also

- [Platform Overview](./index) - All supported platforms
- [Quick Start](../quick-start) - Getting started guide
- [Command Reference](../commands/) - All commands
- [Configuration Guide](../configuration) - Advanced configuration
