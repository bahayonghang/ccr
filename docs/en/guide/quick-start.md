# Quick Start

CCR provides simple yet powerful configuration management features. This guide will help you get started quickly.

## Installation

### Method 1: Quick Install (Recommended)

Install directly from GitHub using cargo:

```bash
cargo install --git https://github.com/bahayonghang/ccr ccr
```

After installation, the `ccr` command will be available in your PATH.

### Method 2: Build from Source

```bash
# Clone repository
git clone https://github.com/bahayonghang/ccr.git
cd ccr

# Build release version
cargo build --release -p ccr

# Install to system path (optional)
cargo install --path crates/ccr
```

**Requirements:** Rust 1.90+ (the installable CLI crate in `crates/ccr` requires it)

> Workspace note: the installable CLI crate now lives in `crates/ccr`. Repository-root folders such as `docs/`, `scripts/`, and `examples/` stay in place, and `outputs/` is reserved for collected artifacts when present. See the [Migration Guide](/en/reference/migration).

## First Time Use

### 1. Initialize Configuration Structure

When using CCR for the first time, you need to initialize the configuration file:

```bash
ccr init
```

This creates the **Unified Mode** multi-platform configuration structure:

```
~/.ccr/
├── config.toml              # Platform registry
├── platforms/
│   └── claude/              # Claude Code platform (default)
│       ├── profiles.toml    # Will be created on first use
│       ├── history/         # Operation history
│       └── backups/         # Backups directory
├── history/                 # Global history
└── backups/                 # Global backups
```

::: info Info
CCR now defaults to Unified Mode, supporting multi-platform configuration management (Claude, Codex, Gemini, etc.)

For traditional single-file configuration (`.ccs_config.toml`), set the environment variable:
```bash
export CCR_LEGACY_MODE=1
ccr init
```
:::

**Example initialization output:**

```
CCR Configuration Initialization
═════════════════════════════════

▶ Creating CCR directory structure
✓ CCR root directory: /home/user/.ccr
✓ Platforms directory: /home/user/.ccr/platforms

▶ Initializing default platform: Claude
✓ Claude platform directory: /home/user/.ccr/platforms/claude
✓ History directory: /home/user/.ccr/history
✓ Backup directory: /home/user/.ccr/backups/claude

▶ Creating platform registry
✓ Configuration file: /home/user/.ccr/config.toml

────────────────────────────────────────────────────────

✓ CCR configuration initialized successfully (Unified Mode)
```

### 2. View All Platforms

```bash
ccr platform list
```

**Example output:**

```
Platform List
═════════════

ℹ Configuration file: /home/user/.ccr/config.toml
ℹ Default platform: claude

┌──────────┬──────────┬─────────┬──────────────────────────┐
│ Platform │ Status   │ Profiles│ Current Profile          │
├──────────┼──────────┼─────────┼──────────────────────────┤
│ ▶ claude │ Enabled  │ 0       │ None                     │
│   codex  │ Disabled │ 0       │ -                        │
│   gemini │ Disabled │ 0       │ -                        │
└──────────┴──────────┴─────────┴──────────────────────────┘

▶ = Current active platform
```

### 3. Add Your First Profile

Use the interactive wizard to add your first API configuration:

```bash
ccr add
```

**Interactive prompts:**

```
Add New Profile
═══════════════

Profile Name: anthropic-official
API Base URL [https://api.anthropic.com]: 
API Token: sk-ant-api03-...
Model (optional) [claude-3-5-sonnet-20241022]: 
Small Fast Model (optional) [claude-3-5-haiku-20241022]: 

✓ Profile 'anthropic-official' added successfully
```

### 4. List and Switch Profiles

```bash
# List all profiles
ccr list

# Switch to a profile
ccr switch anthropic-official

# Or use shorthand
ccr anthropic-official
```

## Basic Commands

### Configuration Management

```bash
ccr list              # 📊 List all profiles in table format
ccr current           # 🔍 Show current profile and env status
ccr switch <name>     # 🔄 Switch profile
ccr validate          # ✅ Validate all profiles
ccr add               # ➕ Add new profile
ccr delete <name>     # ❌ Delete profile
```

### Advanced Features

```bash
# Terminal User Interface
ccr tui               # 🖥️ Launch interactive TUI

# Web Interface
ccr ui                # 🎨 Launch full CCR UI application (recommended web interface)
ccr web               # 🌐 Launch lightweight legacy web API server (for programmatic access / compatibility)

# History and Audit
ccr history           # 📚 View operation history
ccr history -l 10     # Show last 10 operations

# Cloud Sync
ccr sync config       # ☁️ Configure WebDAV sync
ccr sync push         # 🔼 Upload config to cloud
ccr sync pull         # 🔽 Download config from cloud

# Maintenance
ccr clean             # 🧹 Clean old backups (default 7 days)
ccr update            # ⚡ Update CCR from GitHub
```

## Multi-Platform Usage

CCR supports managing multiple AI CLI platforms from one tool:

```bash
# List all supported platforms
ccr platform list

# Initialize other platforms
ccr platform init codex      # GitHub Copilot
ccr platform init gemini     # Google Gemini CLI

# Switch between platforms
ccr platform switch codex    # Switch to Codex
ccr add                      # Add Codex profile
ccr switch my-github-token   # Use specific profile

ccr platform switch claude   # Back to Claude
ccr switch anthropic         # Use Claude profile

# Each platform maintains independent profiles and history
```

**Supported Platforms:**

| Platform | Status | Description |
|----------|--------|-------------|
| **Claude Code** | ✅ Fully Implemented | Anthropic's official CLI |
| **Codex** | ✅ Fully Implemented | GitHub Copilot CLI |
| **Gemini CLI** | ✅ Fully Implemented | Google Gemini CLI |
| **Qwen CLI** | 🚧 Planned | Alibaba Qwen CLI |
| **iFlow CLI** | 🚧 Planned | iFlow AI CLI |

## Configuration Modes

### Unified Mode (Default)

Multi-platform configuration management:

```
~/.ccr/
├── config.toml              # Platform registry
└── platforms/
    ├── claude/              # Claude Code platform
    ├── codex/               # GitHub Copilot platform
    └── gemini/              # Gemini CLI platform
```

**Advantages:**
- ✅ Manage multiple platforms from one tool
- ✅ Platform-specific profiles and history
- ✅ Complete isolation between platforms
- ✅ Future-proof for new platforms

### Legacy Mode

Traditional single-platform configuration (backward compatible with CCS):

```bash
export CCR_LEGACY_MODE=1
ccr init
```

Creates `~/.ccs_config.toml` (compatible with shell-based CCS)

## Next Steps

- 📖 [Command Reference](/en/reference/commands/) - Learn all available commands
- 🎨 [Web Guide](/en/web-guide) - Explore the web interface
- 🔧 [Configuration Guide](/en/configuration) - Advanced configuration options
- 💡 [Examples](/en/examples/) - Real-world usage examples
- 🚀 [Multi-Platform Setup](/en/examples/multi-platform-setup) - Detailed multi-platform guide
- 🔁 [Migration Guide](/en/reference/migration) - Old-to-new path and command map

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Config file not found | Run `ccr init` to create configuration structure |
| Lock timeout | Check for zombie processes: `ps aux \| grep ccr`<br>Clean locks: `rm -rf ~/.claude/.locks/*` |
| Permission denied | Fix permissions:<br>`chmod 600 ~/.claude/settings.json`<br>`chmod 644 ~/.ccs_config.toml` |
| Settings not found | Created automatically on first switch: `ccr switch <profile>` |

For more troubleshooting tips, see the [Troubleshooting Guide](/en/examples/troubleshooting).
