# Configuration Guide

This guide covers CCR's configuration system, including file structure, settings, and customization options.

## Configuration Modes

CCR supports two configuration modes:

### Unified Mode (Default, Recommended)

Multi-platform configuration management supporting Claude, Codex, Gemini, and future platforms.

**Directory Structure:**
```
~/.ccr/
├── config.toml              # Platform registry
├── platforms/
│   ├── claude/              # Claude Code platform
│   │   ├── profiles.toml    # Claude profiles
│   │   ├── history.json     # Operation history
│   │   └── backups/         # Backup files
│   ├── codex/               # GitHub Copilot platform
│   │   ├── profiles.toml
│   │   ├── history.json
│   │   └── backups/
│   └── gemini/              # Google Gemini CLI
│       ├── profiles.toml
│       ├── history.json
│       └── backups/
├── history/                 # Global history
└── backups/                 # Global backups
```

**Enable Unified Mode:**
```bash
# Default, automatically enabled
ccr init
```

### Legacy Mode

Single-platform configuration for backward compatibility with CCS.

**File Structure:**
```
~/.ccs_config.toml           # Main config file (shared with CCS)
~/.claude/settings.json      # Claude settings (managed by CCR)
~/.claude/backups/           # Backup files
~/.claude/ccr_history.json   # Operation history
~/.claude/.locks/            # File locks
```

**Enable Legacy Mode:**
```bash
export CCR_LEGACY_MODE=1
ccr init
```

## Platform Registry (Unified Mode)

The `config.toml` file in `~/.ccr/` manages platform registration:

```toml
[platforms.claude]
enabled = true
name = "Claude Code"
description = "Anthropic's official CLI"
settings_path = "~/.claude/settings.json"
current_profile = "anthropic-official"

[platforms.codex]
enabled = true
name = "Codex"
description = "GitHub Copilot CLI"
settings_path = "~/.codex/settings.json"
current_profile = "github-copilot"

[platforms.gemini]
enabled = false
name = "Gemini CLI"
description = "Google Gemini CLI"
settings_path = "~/.gemini/settings.json"
```

## Profile Configuration

Profiles are stored in `profiles.toml` within each platform directory.

### Profile Format

```toml
[profiles.anthropic-official]
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-api03-..."
model = "claude-3-5-sonnet-20241022"
small_fast_model = "claude-3-5-haiku-20241022"
provider = "anthropic"

[profiles.anthropic-free]
base_url = "https://free.api.example.com"
auth_token = "sk-free-..."
model = "claude-3-5-sonnet-20241022"
provider = "third-party"

[profiles.openrouter]
base_url = "https://openrouter.ai/api/v1"
auth_token = "sk-or-..."
model = "anthropic/claude-3.5-sonnet"
provider = "openrouter"
```

### Profile Fields

| Field | Required | Description |
|-------|----------|-------------|
| `base_url` | ✅ Yes | API endpoint URL |
| `auth_token` | ✅ Yes | API authentication token |
| `model` | No | Default model (uses platform default if not set) |
| `small_fast_model` | No | Fast model for quick tasks |
| `provider` | No | Provider identifier for categorization |

## Settings Management

CCR directly manages the platform's `settings.json` file.

### Claude Code Settings Example

```json
{
  "environmentVariables": {
    "ANTHROPIC_BASE_URL": "https://api.anthropic.com",
    "ANTHROPIC_AUTH_TOKEN": "sk-ant-api03-...",
    "ANTHROPIC_MODEL": "claude-3-5-sonnet-20241022",
    "ANTHROPIC_SMALL_FAST_MODEL": "claude-3-5-haiku-20241022"
  }
}
```

### Settings Update Flow

1. **Read** target profile from `profiles.toml`
2. **Validate** profile completeness and format
3. **Backup** current settings.json
4. **Write** new settings atomically (with file lock)
5. **Record** operation in history
6. **Verify** settings were written correctly

## Cloud Sync Configuration

Configure WebDAV-based cloud synchronization in `profiles.toml` or `config.toml`:

```toml
[settings.sync]
enabled = true
webdav_url = "https://dav.jianguoyun.com/dav/"
username = "user@example.com"
password = "app-password"
remote_path = "/ccr/"
auto_sync = false
```

### Supported Cloud Services

- **Nutstore (坚果云)** - Recommended for China users
- **Nextcloud / ownCloud** - Self-hosted or managed
- **Any WebDAV-compatible server**

### Setup Cloud Sync

```bash
# Interactive configuration
ccr sync config

# Manual configuration
# Edit ~/.ccr/config.toml or ~/.ccs_config.toml
# Add [settings.sync] section
```

## History Tracking

Operation history is stored in `history.json`:

```json
{
  "operations": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "timestamp": "2025-01-01T12:00:00Z",
      "operation_type": "switch",
      "actor": "username",
      "from_config": "anthropic-official",
      "to_config": "anthropic-free",
      "env_changes": {
        "ANTHROPIC_BASE_URL": "https://api.anthropic.com → https://free.api.example.com",
        "ANTHROPIC_AUTH_TOKEN": "sk-ant-*** → sk-free-***"
      },
      "result": "success",
      "backup_path": "~/.claude/backups/settings_20250101_120000.json.bak"
    }
  ]
}
```

## Backup Management

### Auto Backup

CCR automatically creates backups before modifying settings:

```
~/.claude/backups/
├── settings_20250101_120000_anthropic.json.bak
├── settings_20250101_130000_openrouter.json.bak
└── settings_20250101_140000_anthropic.json.bak
```

### Backup Retention

- **Default:** Keep last 10 backups automatically
- **Cleanup:** Use `ccr clean` to remove old backups
- **Restore:** Manually copy backup to `settings.json`

## Environment Variables

Override default paths for development or testing:

```bash
# Configuration paths
export CCR_CONFIG_PATH="/custom/path/config.toml"
export CCR_SETTINGS_PATH="/custom/path/settings.json"

# Working directories
export CCR_BACKUP_DIR="/custom/path/backups"
export CCR_HISTORY_PATH="/custom/path/history.json"
export CCR_LOCK_DIR="/custom/path/locks"

# Behavior
export CCR_LOG_LEVEL="debug"          # trace|debug|info|warn|error
export CCR_LEGACY_MODE="1"            # Enable legacy mode
```

## Validation Rules

CCR validates profiles automatically:

### URL Validation
- Must be valid HTTP/HTTPS URL
- Must be accessible (optional check)

### Token Validation
- Must not be empty
- Must match expected format (if known)

### Model Validation
- Must not be empty if specified
- Should match known model patterns (warning only)

### Required Fields
- `base_url` - API endpoint
- `auth_token` - Authentication token

## File Locking

CCR uses file locks to prevent concurrent modifications:

### Lock Behavior
- **Timeout:** 10 seconds default
- **Auto-cleanup:** Locks removed after operation
- **Stale detection:** Removes locks from dead processes

### Lock Files
```
~/.claude/.locks/
├── settings.lock
├── config.lock
└── history.lock
```

## Security Best Practices

### Token Protection
- ✅ Tokens masked in logs and output
- ✅ Tokens not shown in plain text
- ✅ Use app passwords for cloud sync
- ✅ Set file permissions: `chmod 600 ~/.ccs_config.toml`

### Backup Security
- ⚠️ Backups contain full tokens
- ⚠️ Secure backup directory permissions
- ⚠️ Don't commit config files to version control
- ⚠️ Use `--no-secrets` when sharing exports

## Migration

### From CCS to CCR

CCR is fully compatible with CCS and shares the same config file:

```bash
# Both tools can coexist
ccs list      # Shell-based CCS
ccr list      # Rust-based CCR

# Same configuration file
~/.ccs_config.toml
```

### From Legacy to Unified Mode

```bash
# Check migration status
ccr migrate --check

# Perform migration
ccr migrate

# Migrate specific platform
ccr migrate --platform claude
```

## Advanced Topics

### Custom Platform Support

Add custom platform definitions to `config.toml`:

```toml
[platforms.custom]
enabled = true
name = "Custom AI CLI"
description = "Custom AI platform"
settings_path = "~/.custom/settings.json"
```

### Batch Operations

Use scripts for bulk operations:

```bash
# Switch between profiles in sequence
for profile in anthropic openrouter duck; do
    ccr switch $profile
    # Run your commands here
done
```

### API Integration

Use CCR's web API for programmatic access:

```bash
# Start API server
ccr web --port 8080 --no-browser

# Use API endpoints
curl http://localhost:8080/api/configs
curl -X POST http://localhost:8080/api/switch -d '{"name":"anthropic"}'
```

## Troubleshooting

See the [Troubleshooting Guide](./examples/troubleshooting) for common issues and solutions.

## Next Steps

- 📖 [Command Reference](./commands/) - Learn all commands
- 🚀 [Quick Start](./quick-start) - Get started quickly  
- 💡 [Examples](./examples/) - Real-world usage examples
- 🌐 [Web Guide](./web-guide) - Explore web interface
