# Codex (GitHub Copilot CLI) Platform Guide

## Overview

Codex is the GitHub Copilot command-line interface platform. CCR provides full support for managing Codex configurations and profiles.

## Platform Information

- **Platform Name**: `codex`
- **Display Name**: Codex (GitHub Copilot CLI)
- **Icon**: 💻
- **Status**: ✅ Fully Implemented
- **Settings Path**: `~/.codex/settings.json`
- **Profiles Path**: `~/.ccr/codex/profiles.toml`

## Prerequisites

- GitHub Copilot subscription
- GitHub Personal Access Token with appropriate scopes
- Codex CLI installed (if using settings synchronization)

## Token Format

Codex requires GitHub Personal Access Tokens in the following format:

```
ghp_XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

**Characteristics:**
- Starts with `ghp_`
- 40 characters total
- Alphanumeric characters

**How to Generate:**

1. Go to GitHub Settings → Developer settings → Personal access tokens
2. Click "Generate new token (classic)"
3. Select scopes:
   - `copilot` (required for Copilot access)
   - `read:org` (optional, for organization access)
4. Generate token and copy immediately (shown only once)

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

# Or use direct command (if implemented)
ccr codex add-profile \
  --name github-official \
  --token ghp_YOUR_GITHUB_TOKEN \
  --model gpt-4
```

### Configuration Example

Create a profile in `~/.ccr/codex/profiles.toml`:

```toml
[github-official]
description = "GitHub Official Copilot"
base_url = "https://api.github.com/copilot"
auth_token = "ghp_YOUR_GITHUB_TOKEN_HERE"
model = "gpt-4"
small_fast_model = "gpt-3.5-turbo"

[github-enterprise]
description = "GitHub Enterprise Copilot"
base_url = "https://github.enterprise.com/api/copilot"
auth_token = "ghp_YOUR_ENTERPRISE_TOKEN"
model = "gpt-4"
small_fast_model = "gpt-3.5-turbo"
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
vim ~/.ccr/codex/profiles.toml

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

## Environment Variables

When a Codex profile is active, CCR manages these environment variables in `~/.codex/settings.json`:

```json
{
  "env": {
    "GITHUB_COPILOT_BASE_URL": "https://api.github.com/copilot",
    "GITHUB_COPILOT_TOKEN": "ghp_YOUR_TOKEN",
    "GITHUB_COPILOT_MODEL": "gpt-4",
    "GITHUB_COPILOT_SMALL_FAST_MODEL": "gpt-3.5-turbo"
  }
}
```

**Variable Mapping:**

| TOML Field | Environment Variable | Description |
|------------|---------------------|-------------|
| `base_url` | `GITHUB_COPILOT_BASE_URL` | API endpoint |
| `auth_token` | `GITHUB_COPILOT_TOKEN` | GitHub token (masked in logs) |
| `model` | `GITHUB_COPILOT_MODEL` | Default model |
| `small_fast_model` | `GITHUB_COPILOT_SMALL_FAST_MODEL` | Fast model (optional) |

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

```bash
# Personal account
[personal]
description = "Personal GitHub Account"
base_url = "https://api.github.com/copilot"
auth_token = "ghp_personal_token"
model = "gpt-4"

# Work account
[work]
description = "Work GitHub Account"
base_url = "https://api.github.com/copilot"
auth_token = "ghp_work_token"
model = "gpt-4"

# Switch between accounts
ccr switch personal  # Use personal
ccr switch work      # Use work
```

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
# → Creates ~/.ccr/codex/backups/settings_20250125_120000.json.bak

# Manual backup
ccr backup codex

# List backups
ccr backups list

# Restore from backup
ccr restore ~/.ccr/codex/backups/settings_20250125_120000.json.bak
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
Profile switch command succeeds but `~/.codex/settings.json` unchanged

**Solution:**
```bash
# Check file permissions
ls -la ~/.codex/settings.json

# Fix permissions if needed
chmod 600 ~/.codex/settings.json

# Verify lock files
ls -la ~/.ccr/codex/.locks/

# Clean stale locks if present
rm -rf ~/.ccr/codex/.locks/*
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

1. **Token Storage**: Tokens are stored in plaintext in `~/.ccr/codex/profiles.toml`
   ```bash
   # Ensure proper file permissions
   chmod 600 ~/.ccr/codex/profiles.toml
   ```

2. **Token Masking**: CCR automatically masks tokens in:
   - Console output
   - History logs
   - Error messages

3. **Backup Security**: Backups also contain tokens
   ```bash
   # Secure backup directory
   chmod 700 ~/.ccr/codex/backups
   ```

4. **Export Without Secrets**:
   ```bash
   # Export profiles without tokens (for sharing)
   ccr export -o codex-profiles.toml --no-secrets
   ```

5. **Token Rotation**: Regularly rotate GitHub tokens
   ```bash
   # Update profile with new token
   vim ~/.ccr/codex/profiles.toml
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
