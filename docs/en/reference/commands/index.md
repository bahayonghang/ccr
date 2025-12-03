# Command Reference

CCR provides a comprehensive set of commands for configuration management. All commands support `--help` for detailed usage information.

## Command Overview

### Configuration Management

| Command | Description |
|---------|-------------|
| [`init`](./init) | Initialize CCR configuration structure |
| [`add`](./add) | Add new profile interactively |
| [`delete`](./delete) | Delete an existing profile |
| [`list`](./list) | List all profiles in table format |
| [`current`](./current) | Show current profile and environment status |
| [`switch`](./switch) | Switch to a different profile |
| [`validate`](./validate) | Validate all profiles and settings |

### User Interface

| Command | Description |
|---------|-------------|
| [`tui`](./tui) | Launch terminal user interface |
| [`web`](./web) | Launch lightweight **legacy** web API server (for programmatic access; prefer `ccr ui` for browser UI) |

### Data Management

| Command | Description |
|---------|-------------|
| [`export`](./export) | Export profiles to file |
| [`import`](./import) | Import profiles from file |
| [`clean`](./clean) | Clean old backup files |

### History & Audit

| Command | Description |
|---------|-------------|
| [`history`](./history) | View operation history |
| [`stats`](./stats) | View usage statistics and cost analysis |

### Cloud Sync

| Command | Description |
|---------|-------------|
| [`sync config`](./sync) | Configure WebDAV cloud sync |
| [`sync status`](./sync) | Check sync status |
| [`sync push`](./sync) | Upload config to cloud |
| [`sync pull`](./sync) | Download config from cloud |

### Temporary Configuration

| Command | Description |
|---------|-------------|
| [`temp-token set`](./temp-token) | Set temporary token override |
| [`temp-token show`](./temp-token) | Show temporary config status |
| [`temp-token clear`](./temp-token) | Clear temporary config |

### System

| Command | Description |
|---------|-------------|
| [`update`](./update) | Update CCR from GitHub |
| [`version`](./version) | Show version and features |

## Platform Management Commands

CCR supports managing multiple AI CLI platforms. These commands help you work with different platforms:

| Command | Description | Example |
|---------|-------------|---------|
| `ccr platform list` | List all platforms with status | Shows enabled platforms and profiles count |
| `ccr platform current` | Show current active platform details | Platform name, current profile, last used time |
| `ccr platform switch <name>` | Switch to different platform | `ccr platform switch codex` |
| `ccr platform init <name>` | Initialize new platform | `ccr platform init gemini` |
| `ccr platform info <name>` | Show detailed platform info | All profiles and settings for the platform |

## Global Options

All commands support these global options:

```bash
--help, -h       # Show command help
--version, -V    # Show version information
```

## Common Usage Patterns

### Quick Start Workflow

```bash
# Initialize
ccr init

# Add your first profile
ccr add

# List and switch
ccr list
ccr switch anthropic
```

### Daily Operations

```bash
# Switch between profiles
ccr switch anthropic-official
ccr switch anthropic-free

# Check current status
ccr current

# View history
ccr history -l 10
```

### TUI for Visual Management

```bash
# Launch interactive terminal interface
ccr tui

# In TUI:
# - Press '1' for Configs tab
# - Use ↑↓ or j/k to navigate
# - Press Enter to switch
# - Press 'q' to quit
```

### Web Interface

```bash
# Launch full CCR UI application (recommended web interface)
ccr ui

# Launch lightweight legacy web API server (for compatibility / programmatic access)
ccr web
```

### Multi-Platform Management

```bash
# Initialize platforms
ccr platform init claude
ccr platform init codex
ccr platform init gemini

# Switch between platforms
ccr platform switch codex
ccr add                        # Add Codex profile
ccr switch my-github-token     # Use specific profile

ccr platform switch claude
ccr switch anthropic           # Back to Claude profile
```

### Backup & Sync

```bash
# Export profiles
ccr export -o backup.toml

# Configure cloud sync
ccr sync config

# Push to cloud
ccr sync push

# Pull from cloud (on another machine)
ccr sync pull

# Import profiles
ccr import backup.toml --merge
```

### Maintenance

```bash
# Clean old backups (older than 7 days)
ccr clean

# Clean backups older than 30 days
ccr clean -d 30

# Dry run (see what would be deleted)
ccr clean --dry-run

# Update CCR
ccr update

# Check for updates only
ccr update --check
```

## Command Aliases

Some commands have convenient aliases:

| Command | Aliases |
|---------|---------|
| `list` | `ls` |
| `current` | `show`, `status` |
| `validate` | `check` |
| `version` | `ver` |
| `switch <name>` | `<name>` (direct shorthand) |

**Example:**
```bash
ccr anthropic    # Equivalent to: ccr switch anthropic
```

## Exit Codes

CCR uses standard exit codes:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | Validation error |
| 4 | Lock timeout |
| 5 | File I/O error |

These can be used in scripts:

```bash
if ccr validate; then
    echo "All profiles valid"
else
    echo "Validation failed"
    exit 1
fi
```

## Environment Variables

CCR respects these environment variables for development and testing:

| Variable | Description | Default |
|----------|-------------|---------|
| `CCR_CONFIG_PATH` | Custom config file path | `~/.ccs_config.toml` or `~/.ccr/config.toml` |
| `CCR_SETTINGS_PATH` | Custom settings path | `~/.claude/settings.json` |
| `CCR_BACKUP_DIR` | Custom backup directory | `~/.claude/backups/` |
| `CCR_HISTORY_PATH` | Custom history file path | `~/.claude/ccr_history.json` |
| `CCR_LOCK_DIR` | Custom lock directory | `~/.claude/.locks/` |
| `CCR_LOG_LEVEL` | Logging level (trace/debug/info/warn/error) | `info` |
| `CCR_LEGACY_MODE` | Enable legacy mode | `0` (disabled) |

**Log Output:**
- **Terminal**: ANSI colored output
- **File**: `~/.ccr/logs/ccr.YYYY-MM-DD.log` (daily rotation, 14-day retention)

**Example:**
```bash
export CCR_LOG_LEVEL=debug
ccr switch anthropic

# View log file
tail -f ~/.ccr/logs/ccr.$(date +%Y-%m-%d).log
```

## Getting Help

For detailed help on any command:

```bash
ccr <command> --help
```

**Examples:**
```bash
ccr init --help
ccr switch --help
ccr sync --help
```

## Next Steps

- 📖 Browse individual [command documentation](./init)
- 🎨 Learn about the [Web Interface](../web-guide)
- 💡 See [practical examples](../examples/)
- 🔧 Advanced [configuration options](../configuration)
