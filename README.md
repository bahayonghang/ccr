# 🚀 CCR - Claude Code Configuration Switcher

**Rust-powered configuration management tool for Claude Code**

CCR directly manages Claude Code's `settings.json` with atomic operations, file locking, complete audit trails, and automatic backups. The Rust implementation of CCS with enhanced reliability and performance.

> **🎉 Version 2.5.0 - Multi-Folder Sync Management**
>
> This version introduces comprehensive multi-folder sync management:
> - 📁 **Independent Folder Management**: Register and sync multiple directories independently (.claude/, .gemini/, config files)
> - 🔄 **Granular Control**: Push/pull individual folders or batch operations
> - ⚡ **Backward Compatible**: Legacy `ccr sync push/pull` commands continue working
> - 🚀 **Auto Migration**: Seamlessly upgrades from v2.4 sync configuration
> - 🌐 **Web UI Support**: 12 new API endpoints for folder management

## ✨ Why CCR?

| Feature | Description |
|---------|-------------|
| 🎯 **Direct Settings Control** | Directly writes to `~/.claude/settings.json` - changes take effect immediately |
| 🌟 **Multi-Platform Support** | Manage configs for Claude Code, Codex (GitHub Copilot), Gemini CLI, and more from one tool |
| 📊 **Beautiful Table UI** | Display config info with comfy-table, compare configs at a glance with color highlights and icons |
| 🖥️ **Interactive TUI** | Full-featured terminal UI with 3 tabs (Configs/History/System) and keyboard navigation |
| 🔒 **Concurrency Safe** | File locking + atomic operations prevent corruption across multiple processes |
| 📝 **Complete Audit Trail** | Every operation logged with masked sensitive data (UUID, timestamp, actor) |
| 💾 **Auto Backup** | Automatic backups before changes with timestamped `.bak` files |
| ☁️ **Cloud Sync** | WebDAV-based multi-folder synchronization - independently sync config, .claude/, .gemini/ and more (Nutstore, Nextcloud, ownCloud supported) |
| ✅ **Validation** | Comprehensive config validation (URLs, required fields, format) |
| 🔤 **Config Optimization** | Sort configs alphabetically, maintain order after switching |
| 🌐 **Web Server** | Built-in Axum web server exposing 14 RESTful API endpoints (config, history, backups, system info, etc.) |
| 🖥️ **Full-Stack Web UI** | Vue.js 3 + Axum application for visual management with support for multi-platform config |
| 🏗️ **Modern Architecture** | Service layer pattern, modular design, 95%+ test coverage |
| ⚡ **Optimized Performance** | Streaming I/O for stats, memory caching, opt-level 1 dev builds |
| 🎯 **Feature Gates** | Optional TUI/Web features for faster compilation (--no-default-features) |
| ⚡ **Smart Update** | Real-time progress display during auto-update |
| 🔄 **CCS Compatible** | Shares `~/.ccs_config.toml` - seamlessly coexist with shell version |

## 📦 Installation

First, you need to install Rust and Cargo, then execute the following commands:

**One-line install from GitHub:**

```bash
cargo install --git https://github.com/bahayonghang/ccr ccr
```

**From source:**

```bash
git clone https://github.com/bahayonghang/ccr.git
cd ccr
cargo install --path .
```

**Requirements:** Rust 1.85+ (for edition 2024 features)

## 🌐 CCR UI - Full-Stack Web Application

CCR UI is a modern **Vue.js 3 + Axum** full-stack application for CCR management!

The Vue.js 3 frontend delivers a reactive experience with TypeScript and Tailwind-driven UI, while the Axum backend wraps the CCR CLI and exposes extended management APIs for MCP servers, slash commands, agents, and plugins.

### Features

- ⚛️ **Vue.js 3 Frontend**: Vue.js 3.5 with Composition API, TypeScript and Tailwind CSS
- 🦀 **Axum Backend**: High-performance Rust async web server
- 🖥️ **Config Management**: Visual config switching and validation
- 💻 **Command Executor**: Execute all 13 CCR commands with visual output
- 📊 **Syntax Highlighting**: Terminal-style output with color coding
- ⚡ **Real-time Execution**: Async command execution with progress display
- 🧩 **Extensible Control**: Manage MCP servers, slash commands, agents, and plugins via dedicated APIs
- 🔄 **GitHub Auto-Download**: Automatically downloads ccr-ui to user directory on first use

### Quick Start

**Method 1: Direct `ccr ui` Command (Recommended)**

```bash
# First time use - auto-download and start
ccr ui

# 💬 Prompt: CCR UI is a full Vue.js 3 + Axum Web application
#    It will be downloaded to:
#    /home/user/.ccr/ccr-ui/
#
# ❓ Download CCR UI from GitHub now? [Y/n]: y
# 📦 Cloning repository: https://github.com/bahayonghang/ccr.git
# ⏳ Downloading (this may take a few minutes)...
# ✅ CCR UI download complete
#
# [Auto-checking dependencies and starting...]
```

**Three-tier Priority Detection System:**
1. **Development Environment** - Detects `ccr-ui/` in current/parent directory (for developers)
2. **User Directory** - Detects `~/.ccr/ccr-ui/` (for daily use)
3. **GitHub Download** - Auto-prompts to download from GitHub (first time use)

**Method 2: Manual ccr-ui Directory**

```bash
cd ccr-ui

# One-letter command - that's it!
just s    # Start development environment

# First time? One command does it all:
just quick-start    # Check prereqs + Install + Start
```

**Available simplified commands:**
- `just s` - Start development (most used!)
- `just i` - Install dependencies
- `just b` - Build production
- `just c` - Check code
- `just t` - Run tests
- `just f` - Format code

**Not sure what to do?** Just run `just` to see help!

**📖 Full Documentation**: See `ccr-ui/START_HERE.md` for ultra-simple guide or `ccr-ui/README.md` for complete docs.

**🎯 CLI vs TUI vs Web Server vs CCR UI**:
- **CLI Tool**: Best for scripting, automation, and quick operations (`ccr switch`, `ccr list`, etc.)
- **TUI** (`ccr tui`): Terminal-based interactive interface with keyboard navigation
- **Web Server** (`ccr web`): Built-in lightweight Axum API server (default port 8080, auto-fallback if occupied) for programmatic access
- **CCR UI** (`ccr ui`): Full-featured Vue.js 3 + Axum web application with visual dashboard (ports 3000/8081)

## 🚀 Quick Start

**1️⃣ Initialize configuration structure:**

```bash
ccr init  # Creates ~/.ccr/ directory structure with multi-platform support
```

This creates the **Unified Mode** directory structure:

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

For traditional single-file configuration, set the environment variable:
```bash
export CCR_LEGACY_MODE=1
ccr init
```
:::

**2️⃣ View available platforms:**

```bash
ccr platform list   # List all supported platforms with status
```

**3️⃣ Add your first API configuration:**

```bash
ccr add             # Interactive wizard to add your API credentials
```

**4️⃣ List and use configurations:**

```bash
ccr list              # 📊 List all configs in table format
ccr switch anthropic  # 🔄 Switch config (shows table with changes, or just: ccr anthropic)
ccr current           # 🔍 Show current config and env status in tables
ccr validate          # ✅ Validate all configs
ccr history           # 📚 View operation history
ccr sync config       # ☁️ Configure WebDAV sync (interactive setup)
ccr sync status       # 📊 Check sync status and remote file
ccr sync push         # 🔼 Upload config to cloud
ccr sync pull         # 🔽 Download config from cloud
ccr tui               # 🖥️ Launch interactive TUI (recommended for visual management!)
ccr web               # 🌐 Launch lightweight web API (port 8080)
ccr ui                # 🎨 Launch full CCR UI application (Vue.js 3 + Axum, ports 3000/8081)
```

**5️⃣ Multi-Platform Usage:**

```bash
# List all supported platforms
ccr platform list

# Initialize other platforms (Codex, Gemini)
ccr platform init codex
ccr platform init gemini

# Switch between platforms
ccr platform switch codex      # Switch to Codex (GitHub Copilot)
ccr add                        # Add Codex profile
ccr platform switch claude     # Back to Claude

# Each platform maintains independent profiles and history
```

**📖 For detailed multi-platform setup and examples, see** [docs/examples/multi-platform-setup.md](docs/examples/multi-platform-setup.md)

## 📚 Commands

| Command | Aliases | Description |
|---------|---------|-------------|
| `ccr init [--force]` | - | 🎬 Initialize config from template |
| `ccr list` | `ls` | 📊 List all configs in table format (status, provider, URL, models, validation) |
| `ccr current` | `show`, `status` | 🔍 Show current config details and env variables in dual tables |
| `ccr switch <name>` | `<name>` | 🔄 Switch config (shows new config table and env changes comparison) |
| `ccr temp-token set <TOKEN> [OPTIONS]` | - | 🎯 Set temporary token override (no toml modification) |
| `ccr temp-token show` | - | 👁️ Show current temporary config status |
| `ccr temp-token clear` | - | 🧹 Clear temporary config override |
| `ccr validate` | `check` | ✅ Validate all configs and settings |
| `ccr optimize` | - | 🔤 Sort config sections alphabetically |
| `ccr history [-l N] [-t TYPE]` | - | 📚 Show operation history (limit/filter by type) |
| `ccr web [-p PORT] [--no-browser]` | - | 🌐 Launch lightweight web API server (default port 8080, auto-fallback) |
| `ccr ui [-p PORT] [--backend-port PORT]` | - | 🎨 Launch full CCR UI app (Next.js + Actix, default 3000/8081) |
| `ccr tui` | - | 🖥️ Launch interactive TUI for visual management |
| `ccr export [-o FILE] [--no-secrets]` | - | 📤 Export configs (with/without API keys) |
| `ccr import FILE [--merge]` | - | 📥 Import configs (merge or replace) |
| `ccr clean [-d DAYS] [--dry-run]` | - | 🧹 Clean old backups (default 7 days) |
| `ccr sync config` | - | ☁️ Configure WebDAV sync (interactive) |
| `ccr sync status` | - | 📊 Check sync status and remote file |
| `ccr sync push [--force]` | - | 🔼 Upload config to cloud |
| `ccr sync pull [--force]` | - | 🔽 Download config from cloud |
| `ccr update [--check]` | - | ⚡ Update CCR from GitHub (with real-time progress) |
| `ccr version` | `ver` | ℹ️ Show version and features |

**Platform Management Commands:**

| Command | Description | Example |
|---------|-------------|---------|
| `ccr platform list` | 🌟 List all platforms with status and current profile | Shows enabled platforms, current platform marker (▶), profiles count |
| `ccr platform current` | ▶️ Display detailed info about current active platform | Shows platform name, current profile, enabled status, last used time |
| `ccr platform switch <name>` | 🔄 Switch to different platform (auto-updates settings path) | `ccr platform switch codex` → switches from Claude to Codex |
| `ccr platform init <name>` | 🎬 Initialize new platform with default profiles.toml | `ccr platform init gemini` → creates `~/.ccr/platforms/gemini/` |
| `ccr platform info <name>` | ℹ️ Show detailed platform information | `ccr platform info claude` → shows all Claude profiles and settings |

**Switch operation flow:**
1. 📖 Read and validate target config
2. 💾 Backup current settings.json
3. ✏️ Update ~/.claude/settings.json (atomic write with lock)
4. 📝 Update current_config marker
5. 📚 Record to history with masked env changes

## 📁 Files & Directories

**Legacy Mode (Single Platform):**
```
~/.ccs_config.toml           # 📝 Config file (shared with CCS)
~/.claude/settings.json      # 🎯 Claude Code settings (CCR manages this)
~/.claude/temp_override.json # 🎯 Temporary config override (temp-token command)
~/.claude/backups/           # 💾 Auto backups (timestamped .bak files)
~/.claude/ccr_history.json   # 📚 Operation audit log
~/.claude/.locks/            # 🔒 File locks (auto-cleanup)
```

**Unified Mode (Multi-Platform):**
```
~/.ccr/                      # 🏠 CCR root directory
  ├── config.toml            # 📝 Platform configuration registry
  ├── backups/               # 💾 Platform config backups
  ├── claude/                # 🤖 Claude Code platform
  │   ├── profiles.toml      # 📋 Claude profiles
  │   ├── settings.json      # ⚙️ Claude settings
  │   ├── history.json       # 📚 Claude operation history
  │   └── backups/           # 💾 Claude backups
  ├── codex/                 # 💻 Codex (GitHub Copilot) platform
  │   ├── profiles.toml      # 📋 Codex profiles
  │   ├── settings.json      # ⚙️ Codex settings
  │   ├── history.json       # 📚 Codex operation history
  │   └── backups/           # 💾 Codex backups
  └── gemini/                # ✨ Gemini CLI platform
      ├── profiles.toml      # 📋 Gemini profiles
      ├── settings.json      # ⚙️ Gemini settings
      ├── history.json       # 📚 Gemini operation history
      └── backups/           # 💾 Gemini backups
```

## 🔧 Key Features

### 🌍 Environment Variables

CCR manages these variables in `settings.json`:
- `ANTHROPIC_BASE_URL` - API endpoint
- `ANTHROPIC_AUTH_TOKEN` - Auth token (auto-masked in display/logs)
- `ANTHROPIC_MODEL` - Default model
- `ANTHROPIC_SMALL_FAST_MODEL` - Fast model (optional)

### 🌟 Multi-Platform Configuration

CCR supports managing configurations for multiple AI CLI platforms from a single tool:

**Supported Platforms:**

| Platform | Status | Description | Settings Path |
|----------|--------|-------------|---------------|
| **Claude Code** | ✅ Fully Implemented | Anthropic's official CLI | `~/.claude/settings.json` |
| **Codex** | ✅ Fully Implemented | GitHub Copilot CLI | `~/.codex/settings.json` |
| **Gemini CLI** | ✅ Fully Implemented | Google Gemini CLI | `~/.gemini/settings.json` |
| **Qwen CLI** | 🚧 Planned | Alibaba Qwen CLI | `~/.qwen/settings.json` |
| **iFlow CLI** | 🚧 Planned | iFlow AI CLI | `~/.iflow/settings.json` |

**Configuration Modes:**

- **Legacy Mode**: Single platform (backward compatible with CCS)
  - Uses `~/.ccs_config.toml`
  - Only manages Claude Code
  - Compatible with shell-based CCS

- **Unified Mode**: Multi-platform (new in v1.4+)
  - Uses `~/.ccr/config.toml` for platform registry
  - Separate `~/.ccr/{platform}/` directories for each platform
  - Platform-specific profiles, history, and backups
  - Complete isolation between platforms

**Platform Features:**

- ✅ **Platform Isolation**: Each platform has separate profiles, history, and backups
- ✅ **Platform Switching**: Switch between platforms with `ccr platform switch`
- ✅ **Profile Management**: Manage platform-specific profiles independently
- ✅ **Platform Detection**: Auto-detect Unified/Legacy mode based on directory structure
- ✅ **Unified History**: Track operations across all platforms in centralized log
- ✅ **Concurrent Safety**: File locks prevent corruption during multi-platform operations
- ✅ **Automatic Migration**: Easy migration from Legacy to Unified mode

**Platform Detection Logic:**

CCR automatically detects which configuration mode to use:

1. **Check `CCR_ROOT` environment variable** → If set, use Unified mode
2. **Check `~/.ccr/config.toml` existence** → If exists, use Unified mode
3. **Fallback to Legacy mode** → Use `~/.ccs_config.toml` (backward compatible)

**Migration from Legacy to Unified:**

```bash
# Check if you should migrate
ccr migrate --check

# Migrate all profiles to Unified mode
ccr migrate

# Migrate specific platform
ccr migrate --platform claude
```

**Example Workflow:**

```bash
# Initialize multiple platforms
ccr platform init claude
ccr platform init codex
ccr platform init gemini

# Work with Claude Code
ccr platform switch claude
ccr add                          # Add Claude profile
ccr switch my-anthropic-api      # Use specific profile

# Work with GitHub Copilot
ccr platform switch codex
ccr add                          # Add Codex profile
ccr switch my-github-token       # Use specific profile

# Work with Gemini CLI
ccr platform switch gemini
ccr add                          # Add Gemini profile
ccr switch my-google-api         # Use specific profile

# View all platforms
ccr platform list
```

### 🎯 Temporary Token Override

Need to test a free token temporarily? CCR provides temporary configuration override without modifying your permanent `~/.ccs_config.toml` file:

```bash
# Set temporary token (one-time use, auto-cleared after switch)
ccr temp-token set sk-free-test-xxx

# Optional: Override additional fields
ccr temp-token set sk-xxx \
  --base-url https://api.temp.com \
  --model claude-opus-4

# View current temporary config
ccr temp-token show

# Apply temporary config (will auto-apply and clear)
ccr switch duck

# Next switch will use permanent config
ccr switch duck
```

**Features:**
- 🔒 Isolated storage (`~/.claude/temp_override.json`)
- 🎯 One-time use (auto-cleared after application)
- 🎯 Partial field override (token only, or token + url, etc.)
- 🔄 Higher priority than permanent config
- 🧹 Auto-cleanup after application
- 👁️ Masked display for security

### 📚 History & Audit

Every operation logged with:
- UUID + timestamp + system username
- Operation type (switch/backup/restore/validate/update)
- Environment variable changes (masked)
- From/to config + backup path
- Result (success/failure/warning)

### 🖥️ TUI - Terminal User Interface

CCR provides an interactive terminal UI for visual configuration management. Launch with:

```bash
ccr tui [--yolo]  # --yolo: Enable YOLO mode (skip confirmations)
```

**Features:**
- **🖥️ Four Tabs**:
  - **Configs Tab** 📋: Browse and manage all configurations
  - **History Tab** 📜: View operation history with timestamps
  - **Sync Tab** ☁️: Check WebDAV sync status and remote file
  - **System Tab** ⚙️: Display system info and file paths

- **⌨️ Keyboard Shortcuts**:
  - `1-4` / `Tab` / `Shift+Tab`: Switch between tabs
  - `↑↓` / `j`/`k`: Navigate lists
  - `Enter`: Switch to selected configuration
  - `d`: Delete selected configuration (requires YOLO mode)
  - `y` / `Y`: Toggle YOLO mode
  - `q` / `Ctrl+C`: Quit TUI

- **🎨 Visual Features**:
  - Color-coded config list (Current=Green, Default=Cyan)
  - Real-time status messages (success/error)
  - Operation history with result indicators (✅❌⚠️)
  - System information display (hostname, OS, paths, version)

- **⚡ YOLO Mode**:
  - Skip all confirmation prompts
  - Required for delete operations in TUI
  - Toggle on/off with `Y` key or start with `--yolo` flag
  - Status displayed in footer (🔴 YOLO / 🟢 SAFE)

**Example workflow:**
```bash
ccr tui              # Launch TUI
# Press '1' → navigate configs → Enter to switch
# Press '2' → view history
# Press '3' → check sync status (P/L/S for push/pull/status in CLI)
# Press '4' → check system info
# Press 'Y' → enable YOLO mode → 'd' to delete config
# Press 'q' → quit
```

### ☁️ Cloud Sync (WebDAV)

CCR supports WebDAV-based configuration synchronization for multi-device management with two sync modes:

**Sync Modes:**
- 📁 **Directory Sync (Unified Mode)** - Syncs entire `~/.ccr/` directory with all platform configurations (recommended)
- 📄 **File Sync (Legacy Mode)** - Syncs single `~/.ccs_config.toml` file (backward compatible)

CCR automatically detects which mode to use based on your configuration structure.

**Supported Services:**
- 🥜 **Nutstore (坚果云)** - Recommended for China users (free tier available)
- 📦 **Nextcloud / ownCloud** - Self-hosted or managed
- 🌐 **Any standard WebDAV server**

**Setup Guide:**

1. **Configure WebDAV connection:**
```bash
ccr sync config
# Interactive prompts:
# - WebDAV server URL (default: https://dav.jianguoyun.com/dav/)
# - Username/Email
# - Password/App password (for Nutstore: Account Settings → Security → Add App)
# - Remote directory path (default: /ccr/)
# - Connection test will run automatically
```

2. **Check sync status:**
```bash
ccr sync status
# Shows:
# - Sync configuration (server, username, remote path)
# - Sync mode (Directory or File)
# - Local path being synced
# - Auto-sync status
# - Remote content existence check
```

3. **Upload config to cloud (first time):**
```bash
ccr sync push
# - Automatically detects directory or file mode
# - Recursively uploads all files and subdirectories
# - Excludes temporary files (.bak, .tmp, .lock, .DS_Store, etc.)
# - Prompts for confirmation if remote content exists
# - Use --force to skip confirmation
```

4. **Download config from cloud:**
```bash
ccr sync pull
# - Automatically detects directory or file mode
# - Recursively downloads all files and subdirectories
# - Applies same exclusion rules as push
# - Backs up local content before overwriting
# - Use --force to skip confirmation
```

**Configuration:**

Sync settings are stored in `~/.ccs_config.toml`:
```toml
[settings.sync]
enabled = true
webdav_url = "https://dav.jianguoyun.com/dav/"
username = "user@example.com"
password = "your-app-password"
remote_path = "/ccr/"  # Directory for Unified Mode or file path for Legacy Mode
auto_sync = false  # Not yet implemented
```

**Sync Features:**
- 🔄 **Automatic Mode Detection** - Detects directory vs file sync automatically
- 📁 **Recursive Directory Sync** - Uploads/downloads entire directory trees
- 🚫 **Smart File Filtering** - Excludes temporary and system files automatically
- 💾 **Safe Operations** - Creates backups before overwriting local content
- 🔒 **Atomic Uploads** - Ensures remote directories exist before uploading files

**Use Cases:**
- 📱 Sync configs across multiple machines
- 🎯 Sync multi-platform configurations (Claude, Codex, Gemini, etc.)
- 💼 Team collaboration with shared configs
- 🔄 Backup configs to cloud storage
- 🚀 Quick setup on new machines

**Security Notes:**
- ✅ Passwords are stored locally in config file
- ✅ Use app passwords instead of account passwords (Nutstore)
- ✅ Ensure proper file permissions: `chmod 600 ~/.ccs_config.toml`
- ⚠️ Remote files are not encrypted by CCR (rely on WebDAV server security)

### 🌐 Web API

RESTful endpoints (run `ccr web`):
The built-in server currently exposes 14 endpoints covering configuration management, backups, and system telemetry.
- `GET /api/configs` – List configurations
- `POST /api/switch` – Switch configuration
- `POST /api/config` – Create configuration section
- `POST /api/config/{name}` – Update configuration section
- `DELETE /api/config/{name}` – Delete configuration section
- `GET /api/history` – View operation history
- `POST /api/validate` – Validate configs and settings
- `POST /api/clean` – Clean backups
- `GET /api/settings` – Inspect Claude Code settings.json snapshot
- `GET /api/settings/backups` – List settings backups
- `POST /api/settings/restore` – Restore settings backup
- `POST /api/export` – Export configuration file
- `POST /api/import` – Import configuration file
- `GET /api/system` – Read cached system information

### 🐛 Debugging

```bash
export CCR_LOG_LEVEL=debug  # trace|debug|info|warn|error
ccr switch anthropic        # See detailed logs
```

## 🆚 CCR vs CCS

| Feature | CCS (Shell) | CCR (Rust) |
|---------|:-----------:|:----------:|
| Config switching | ✅ | ✅ |
| Direct settings.json write | ❌ | ✅ |
| File locking | ❌ | ✅ |
| Audit history | ❌ | ✅ |
| Auto backup | ❌ | ✅ |
| Validation | Basic | Complete |
| Web UI | ❌ | ✅ |
| Performance | Fast | Extremely Fast |

**💡 Fully compatible** - Shares `~/.ccs_config.toml`, can coexist and switch between both tools seamlessly.

## 🛠️ Development

**Project structure:**
```
src/
├── main.rs           # 🚀 CLI entry
├── lib.rs            # 📚 Library entry
├── commands/         # 🎯 CLI Layer (13 commands)
├── web/              # 🌐 Web Layer (Axum server + API)
├── services/         # 🎯 Service Layer (business logic)
├── managers/         # 📁 Manager Layer (data access)
│   ├── config.rs     # ⚙️ Config management
│   ├── settings.rs   # ⭐ Settings management
│   └── history.rs    # 📚 Audit trail
├── core/             # 🏗️ Core Layer (infrastructure)
│   ├── error.rs      # ⚠️ Error types + exit codes
│   ├── lock.rs       # 🔒 File locking
│   ├── logging.rs    # 🎨 Colored output
│   └── ...           # More core modules
└── utils/            # 🛠️ Utils (masking, validation)

ccr-ui/               # 🌐 Full-Stack Web Application
├── backend/          # 🦀 Axum server
│   ├── src/
│   │   ├── main.rs               # Server entry
│   │   ├── executor/             # CCR CLI subprocess executor
│   │   ├── handlers/             # API route handlers (config, command, MCP, etc.)
│   │   ├── models.rs             # Request/response types
│   │   ├── settings_manager.rs   # Claude settings I/O with atomic writes
│   │   ├── plugins_manager.rs    # Plugin repository management
│   │   ├── claude_config_manager.rs # Config file helpers
│   │   └── markdown_manager.rs   # Markdown knowledge base utilities
│   └── Cargo.toml
└── frontend/         # ⚛️ Vue.js 3 with Vite
    ├── src/
    │   ├── views/            # Page views (Dashboard, Configs, Commands, etc.)
    │   ├── components/       # Reusable UI components
    │   ├── router/           # Vue Router configuration
    │   └── store/            # Pinia state management
    ├── package.json
    └── vite.config.ts
```

**Commands:**
```bash
# Development workflow (use justfile)
just dev              # Quick check + test
just watch            # Auto-rebuild on changes
just ci               # Full CI pipeline

# Or use cargo directly
cargo test            # 🧪 Run tests
cargo clippy          # 🔍 Lint
cargo fmt             # 💅 Format
cargo build --release # 🏗️ Production build
```

**⚡ Quick Commands for Fast Development:**

CCR uses optimized build configurations (see `.cargo/config.toml`) for faster iteration:

```bash
# 🚀 Fastest commands for daily development
cargo check           # Syntax check only (no binary) - ~3s
cargo check -q        # Quiet mode - only show errors

# 🎯 Feature-specific builds (optional features: web, tui)
cargo build --no-default-features    # CLI only - saves ~30s build time
cargo build --features web           # CLI + Web API
cargo build --features tui           # CLI + TUI
cargo build --all-features           # Everything (default)

# 🧪 Testing
cargo test --lib                     # Unit tests only
cargo test --all-features            # Full test suite

# 🔍 Quality checks (enforced by CI)
cargo fmt --all --check              # Check formatting
cargo clippy --all-targets -- -D warnings  # Zero warnings policy
RUSTFLAGS="-D warnings" cargo build  # Fail on warnings

# 🎯 Platform tests (must run serially)
cargo test --test platform_tests -- --test-threads=1
cargo test --test platform_integration_tests -- --test-threads=1

# 📊 Coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

**Build Performance Tips:**
- Use `cargo check` during active development (instant feedback)
- Use `--no-default-features` if working on core CLI logic
- Incremental compilation is enabled by default
- Debug info is reduced (`debuginfo=1`) for faster dev builds

## 🏗️ Architecture

CCR v1.1.5 features a **strict layered architecture** with clear separation of concerns:

```
CLI/Web Layer → Services → Managers → Core/Utils
```

**Key Components:**
- **Service Layer**: 4 services (Config, Settings, History, Backup) - 26 methods
- **Manager Layer**: 3 managers (Config, Settings, History) - Data access & file operations
- **Web Module**: Axum-based server with 14 RESTful API endpoints
- **Core Infrastructure**: Atomic writer, file locking, error handling, logging
- **Test Coverage**: 95%+ comprehensive test suite

**Design Patterns:**
- Atomic file operations (temp file + rename)
- Multi-process safety via file locking
- Complete audit trail with UUID tracking
- Automatic backups before destructive operations

For detailed architecture documentation, see [ARCHITECTURE.md](ARCHITECTURE.md).

## 🐛 Troubleshooting

| Issue | Solution |
|-------|----------|
| Config file not found | Run `ccr init` to create `~/.ccs_config.toml` |
| Lock timeout | Check for zombie processes: `ps aux \| grep ccr`<br>Clean locks: `rm -rf ~/.claude/.locks/*` |
| Permission denied | Fix permissions:<br>`chmod 600 ~/.claude/settings.json`<br>`chmod 644 ~/.ccs_config.toml` |
| Settings not found | Created automatically on first switch: `ccr switch <config>` |

## 📄 License & Contributing

- **License:** MIT
- **Issues & PRs:** Welcome! 🤝
- **GitHub:** https://github.com/bahayonghang/ccr
- **Status:** Active development - test thoroughly before production use

---

Made with 💙 in Rust | Part of [CCS Project](https://github.com/bahayonghang/ccs)
