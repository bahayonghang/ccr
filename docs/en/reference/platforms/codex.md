# Codex Platform

## Overview

CCR provides comprehensive management for Codex CLI configurations with a **two-way dispatch** model:

1. **Official Mode**: Fully resets `~/.codex/config.toml` and `~/.codex/auth.json` to defaults
2. **ThirdParty Mode**: Read-modify-write, preserving all non-provider fields

All configuration writes use **atomic operations** (temp file + rename) with **file locking** for concurrent safety.

## Quick Start

```bash
ccr platform init codex
ccr platform switch codex
ccr add
```

## Profile Classification

CCR uses the `provider_type` field to determine profile behavior:

| `provider_type` | Classification | Switch Behavior |
|-----------------|---------------|-----------------|
| `official_relay` | Official | Full config reset to defaults |
| Other / unset | ThirdParty | Read-modify-write, preserving existing fields |

**Fallback**: If `provider_type` is not set, profiles without a `base_url` (or with an empty one) are treated as Official.

## Two-Way Dispatch

### Official Mode

When switching to a profile with `provider_type = "official_relay"`:

1. **Auto-backup** current `config.toml` and `auth.json`
2. **Full reset** `config.toml` to empty TOML
3. **Full reset** `auth.json` to empty JSON
4. Update `profiles.toml` `current_config`

Use case: Restore Codex CLI to default behavior with the official OpenAI service.

### ThirdParty Mode

When switching to a non-official profile:

1. **Read** existing `config.toml` (preserving all fields)
2. **Update** provider fields: `model`, `model_provider`, `[model_providers.{id}]`
3. **Optionally set** runtime params: `approval_policy`, `sandbox_mode`, etc.
4. **Atomic write** `config.toml`
5. **Update** API key in `auth.json`
6. Update `profiles.toml` `current_config`

Use case: Use third-party OpenAI-compatible providers while preserving existing non-provider configuration.

## Atomic Writes & Concurrent Safety

CCR uses `CodexConfigManager` for all Codex config operations:

| Feature | Description |
|---------|-------------|
| **Atomic writes** | Temp file + rename prevents corruption from interrupted writes |
| **File locking** | Cross-process locks prevent concurrent write conflicts (resource: `codex_config`) |
| **Auto-backup** | Automatic backup before official mode reset, keeps last 10 backups |
| **Config caching** | 30s TTL cache reduces redundant reads (`CachedCodexConfigManager`) |

Backup files are stored in `~/.codex/backups/`:
```
config.pre_official.20260225_120000.toml.bak
auth.pre_official.20260225_120000.json.bak
```

## Configuration

### Example profiles.toml

```toml
default_config = "duckcoding"
current_config = "duckcoding"

[settings]
skip_confirmation = false

# Official mode - full config reset on switch
[official]
description = "Codex official default config"
provider = "openai"
provider_type = "official_relay"

# ThirdParty mode - preserves non-provider fields on switch
[duckcoding]
description = "DuckCoding (OpenAI compatible)"
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

### Generated config.toml (ThirdParty mode)

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

### Generated auth.json

```json
{
  "OPENAI_API_KEY": "paste-your-token-here",
  "DUCKCODING_API_KEY": "paste-your-token-here"
}
```

## Multi-Account Management

CCR provides comprehensive multi-account management for Codex CLI.

### Basic Commands

```bash
# Save current login as a named account
ccr codex auth save work

# Save with description
ccr codex auth save personal -d "Personal account"

# Save with expiry time
ccr codex auth save temp --expires-at 2026-02-01T00:00:00Z

# Force overwrite existing account
ccr codex auth save work --force

# List all saved accounts
ccr codex auth list

# Switch to a specific account
ccr codex auth switch work

# Show current account info
ccr codex auth current

# Delete an account
ccr codex auth delete old-account

# Delete without confirmation
ccr codex auth delete old-account --force
```

### Export & Import

```bash
# Export all accounts to Downloads folder
ccr codex auth export

# Export without sensitive data (tokens)
ccr codex auth export --no-secrets

# Import accounts from file (interactive)
ccr codex auth import

# Import in replace mode (overwrite existing accounts)
ccr codex auth import --replace

# Import with force (overwrite in merge mode)
ccr codex auth import --force
```

**Import Modes:**
- **Merge (default)**: Skip existing accounts, only add new ones
- **Merge + --force**: Overwrite existing accounts with imported data
- **Replace (--replace)**: Always overwrite accounts with the same name

**Features:**
- 🟢 Token freshness indicators: Fresh (<1 day) | 🟡 Stale (1-7 days) | 🔴 Old (>7 days)
- 📧 Email masking for privacy (e.g., `use***@example.com`)
- 🔒 Automatic backup rotation, keeps last 10 backups
- ⚠️ Process detection warnings before switching
- 🔐 Auto-set auth file permissions to 0600 on Unix systems

### Interactive TUI

Launch the Codex account management interface:
```bash
ccr codex
```

**Keyboard Shortcuts:**
| Key | Action |
|-----|--------|
| `↑` / `↓` / `j` / `k` | Select account |
| `Enter` | Switch to selected account and exit |
| `Space` | Switch to selected account (stay in TUI) |
| `q` / `Esc` | Quit |

## Validation

```bash
ccr validate

# Output includes:
# ✅ Official mode profiles: no validation needed
# ✅ ThirdParty mode profiles: checks base_url, auth_token, wire_api
# ❌ Legacy api_mode=github profiles: returns deprecation error
```

## Migration from Legacy GitHub Mode

> **Note**: GitHub Copilot compatible mode (`api_mode: "github"`) was deprecated and removed in v4.2.6.
> Switching to a legacy GitHub mode profile will return a clear deprecation error.

Migration steps:

1. Delete old GitHub mode profiles
2. Create new Official or ThirdParty profiles as needed

```bash
# Delete old GitHub mode profile
ccr delete github-old

# Create a new ThirdParty profile
ccr add
# Follow the prompts
```

## Troubleshooting

### Issue: Legacy GitHub Profile Error

**Symptoms:**
```
❌ GitHub Copilot compatible mode is deprecated, use ThirdParty mode instead
```

**Solution:**
Delete old `api_mode = "github"` profiles and recreate using Official or ThirdParty mode.

### Issue: Settings Not Updating

**Symptoms:**
Profile switch succeeds but `~/.codex/config.toml` unchanged.

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
ccr codex auth save <name>   # Save current login as named account
ccr codex auth list          # List all saved accounts
ccr codex auth switch <name> # Switch to specific account
ccr codex auth current       # Show current account info
ccr codex auth delete <name> # Delete account
ccr codex auth export        # Export accounts to file
ccr codex auth import        # Import accounts from file
ccr codex                    # Launch interactive TUI

# Validation and diagnostics
ccr validate                # Validate all profiles
ccr history                 # View operation history
```

## See Also

- [Platform Overview](./index) - All supported platforms
- [Claude Code Platform](./claude) - Claude Code configuration
- [Gemini Platform](./gemini) - Gemini CLI configuration
- [Migration Guide](./migration) - Migrating between platforms
- [Multi-Platform Setup](../../examples/multi-platform-setup) - Setup examples
