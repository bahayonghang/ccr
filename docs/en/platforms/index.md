# Platform Support

CCR supports managing configurations for multiple AI CLI platforms from a single unified tool.

## Supported Platforms

| Platform | Status | Description | Settings Path |
|----------|--------|-------------|---------------|
| [**Claude Code**](./claude) | ✅ Fully Implemented | Anthropic's official CLI | `~/.claude/settings.json` |
| [**Codex**](./codex) | ✅ Fully Implemented | GitHub Copilot CLI | `~/.codex/settings.json` |
| [**Gemini CLI**](./gemini) | ✅ Fully Implemented | Google Gemini CLI | `~/.gemini/settings.json` |
| **Qwen CLI** | 🚧 Planned | Alibaba Qwen CLI | `~/.qwen/settings.json` |
| **iFlow CLI** | 🚧 Planned | iFlow AI CLI | `~/.iflow/settings.json` |

## Platform Features

### ✅ Platform Isolation
Each platform has its own:
- Separate profile storage (`profiles.toml`)
- Independent operation history
- Dedicated backup directory
- Platform-specific settings

### ✅ Unified Management
- Switch between platforms with single command
- Consistent interface across all platforms
- Centralized configuration registry
- Cross-platform history tracking

### ✅ Concurrent Safety
- File locks prevent conflicts
- Atomic operations ensure consistency
- Multi-process safe operations
- Auto-cleanup of stale locks

## Quick Start

### Initialize Platforms

```bash
# Claude Code (default platform)
ccr platform init claude

# GitHub Copilot
ccr platform init codex

# Google Gemini CLI
ccr platform init gemini
```

### Switch Between Platforms

```bash
# List all platforms
ccr platform list

# Switch to different platform
ccr platform switch codex

# Now all commands operate on Codex
ccr add                    # Add Codex profile
ccr list                   # List Codex profiles
ccr switch my-github-token # Switch to Codex profile

# Switch back to Claude
ccr platform switch claude
```

### Platform Information

```bash
# Current active platform
ccr platform current

# Detailed platform info
ccr platform info claude
ccr platform info codex
ccr platform info gemini
```

## Platform Structure

### Unified Mode Directory

```
~/.ccr/
├── config.toml              # Platform registry
├── platforms/
│   ├── claude/              # Claude Code platform
│   │   ├── profiles.toml    # Claude profiles
│   │   ├── history.json     # Claude operations
│   │   └── backups/         # Claude backups
│   ├── codex/               # GitHub Copilot platform
│   │   ├── profiles.toml
│   │   ├── history.json
│   │   └── backups/
│   └── gemini/              # Gemini CLI platform
│       ├── profiles.toml
│       ├── history.json
│       └── backups/
├── history/                 # Global history (all platforms)
└── backups/                 # Global config backups
```

### Platform Registry

The `config.toml` file manages platform configuration:

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
```

## Platform-Specific Guides

- [**Claude Code**](./claude) - Anthropic's official CLI platform
- [**Codex**](./codex) - GitHub Copilot CLI integration
- [**Gemini CLI**](./gemini) - Google Gemini CLI support
- [**Migration**](./migration) - Migrating between platforms

## Use Cases

### Development Teams
Manage multiple API keys for different team members or projects:

```bash
# Claude for production
ccr platform switch claude
ccr switch production-api

# Codex for development
ccr platform switch codex
ccr switch dev-github-copilot
```

### Multi-Provider Setup
Use different AI providers for different tasks:

```bash
# Claude for complex reasoning
ccr platform switch claude
ccr switch anthropic-official

# Gemini for quick tasks
ccr platform switch gemini
ccr switch google-free-tier
```

### Testing & Development
Separate configurations for testing:

```bash
# Production environment
ccr platform switch claude
ccr switch prod-config

# Development/testing environment
ccr platform init claude-dev
ccr platform switch claude-dev
ccr add  # Add test API keys
```

## Platform Detection

CCR automatically detects configuration mode:

1. **Check `CCR_ROOT` environment variable** → Unified mode if set
2. **Check `~/.ccr/config.toml`** → Unified mode if exists
3. **Fallback to Legacy mode** → `~/.ccs_config.toml`

## Migration

### From Legacy to Unified Mode

```bash
# Check if migration is needed
ccr migrate --check

# Migrate all platforms
ccr migrate

# Migrate specific platform only
ccr migrate --platform claude
```

### Between Platforms

```bash
# Export from Claude
ccr platform switch claude
ccr export -o claude-profiles.toml

# Import to Codex
ccr platform switch codex
ccr import claude-profiles.toml --merge
```

## Next Steps

- 📖 [Claude Code Platform Guide](./claude)
- 💻 [Codex Platform Guide](./codex)
- ✨ [Gemini CLI Platform Guide](./gemini)
- 🔄 [Platform Migration Guide](./migration)
- 💡 [Multi-Platform Setup Examples](../examples/multi-platform-setup)
