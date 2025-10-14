# 🚀 CCR - Claude Code Configuration Switcher

**Rust-powered configuration management tool for Claude Code**

CCR directly manages Claude Code's `settings.json` with atomic operations, file locking, complete audit trails, and automatic backups. The Rust implementation of CCS with enhanced reliability and performance.

## ✨ Why CCR?

| Feature | Description |
|---------|-------------|
| 🎯 **Direct Settings Control** | Directly writes to `~/.claude/settings.json` - changes take effect immediately |
| 🔒 **Concurrency Safe** | File locking + atomic operations prevent corruption across multiple processes |
| 📝 **Complete Audit Trail** | Every operation logged with masked sensitive data (UUID, timestamp, actor) |
| 💾 **Auto Backup** | Automatic backups before changes with timestamped `.bak` files |
| ✅ **Validation** | Comprehensive config validation (URLs, required fields, format) |
| 🔤 **Config Optimization** | Sort configs alphabetically, maintain order after switching |
| 🌐 **Web Server** | Built-in Axum web server exposing 14 RESTful API endpoints (config, history, backups, system info, etc.) |
| 🖥️ **Full-Stack Web UI** | Next.js 16 (React 19) + Actix Web application for visual management |
| 🏗️ **Modern Architecture** | Service layer pattern, modular design, 95%+ test coverage |
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

CCR UI is a modern **Next.js + Actix Web** full-stack application for CCR management!

The App Router frontend delivers a React 19 experience with Tailwind-driven UI, while the Actix backend wraps the CCR CLI and exposes extended management APIs for MCP servers, slash commands, agents, and plugins.

### Features

- ⚛️ **Next.js Frontend**: Next.js 16 (React 19) App Router with TypeScript and Tailwind CSS
- 🦀 **Actix Web Backend**: High-performance Rust async web server
- 🖥️ **Config Management**: Visual config switching and validation
- 💻 **Command Executor**: Execute all 13 CCR commands with visual output
- 📊 **Syntax Highlighting**: Terminal-style output with color coding
- ⚡ **Real-time Execution**: Async command execution with progress display
- 🧩 **Extensible Control**: Manage MCP servers, slash commands, agents, and plugins via dedicated APIs

### Super Quick Start

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

**🎯 CLI vs Web Server vs CCR UI**:
- **CLI Tool**: Best for scripting, automation, and quick operations
- **Web Server** (`ccr web`): Built-in lightweight Axum server for API access
- **CCR UI** (Actix + Next.js): Full-featured web application for visual management

## 🚀 Quick Start

**1️⃣ Initialize config file:**

```bash
ccr init  # Creates ~/.ccs_config.toml with examples
```

**2️⃣ Edit your configuration:**

```toml
# ~/.ccs_config.toml
default_config = "anthropic"
current_config = "anthropic"

[anthropic]
description = "Anthropic Official API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-your-api-key"
model = "claude-sonnet-4-5-20250929"

[anyrouter]
description = "AnyRouter Proxy"
base_url = "https://api.anyrouter.ai/v1"
auth_token = "your-anyrouter-token"
model = "claude-sonnet-4-5-20250929"
```

**3️⃣ Use CCR:**

```bash
ccr list              # 📋 List all configs
ccr switch anthropic  # 🔄 Switch to a config (or just: ccr anthropic)
ccr current           # 🔍 Show current status
ccr validate          # ✅ Validate all configs
ccr history           # 📚 View operation history
ccr web               # 🌐 Launch web UI (port 8080)
```

## 📚 Commands

| Command | Aliases | Description |
|---------|---------|-------------|
| `ccr init [--force]` | - | 🎬 Initialize config from template |
| `ccr list` | `ls` | 📜 List all configurations with validation status |
| `ccr current` | `show`, `status` | 🔍 Show current config and env variables |
| `ccr switch <name>` | `<name>` | 🔄 Switch to configuration (5-step atomic operation) |
| `ccr validate` | `check` | ✅ Validate all configs and settings |
| `ccr optimize` | - | 🔤 Sort config sections alphabetically |
| `ccr history [-l N] [-t TYPE]` | - | 📚 Show operation history (limit/filter by type) |
| `ccr web [-p PORT]` | - | 🌐 Launch web UI (default port 8080) |
| `ccr export [-o FILE] [--no-secrets]` | - | 📤 Export configs (with/without API keys) |
| `ccr import FILE [--merge]` | - | 📥 Import configs (merge or replace) |
| `ccr clean [-d DAYS] [--dry-run]` | - | 🧹 Clean old backups (default 7 days) |
| `ccr update [--check]` | - | ⚡ Update CCR from GitHub (with real-time progress) |
| `ccr version` | `ver` | ℹ️ Show version and features |

**Switch operation flow:**
1. 📖 Read and validate target config
2. 💾 Backup current settings.json
3. ✏️ Update ~/.claude/settings.json (atomic write with lock)
4. 📝 Update current_config marker
5. 📚 Record to history with masked env changes

## 📁 Files & Directories

```
~/.ccs_config.toml          # 📝 Config file (shared with CCS)
~/.claude/settings.json     # 🎯 Claude Code settings (CCR manages this)
~/.claude/backups/          # 💾 Auto backups (timestamped .bak files)
~/.claude/ccr_history.json  # 📚 Operation audit log
~/.claude/.locks/           # 🔒 File locks (auto-cleanup)
```

## 🔧 Key Features

### 🌍 Environment Variables

CCR manages these variables in `settings.json`:
- `ANTHROPIC_BASE_URL` - API endpoint
- `ANTHROPIC_AUTH_TOKEN` - Auth token (auto-masked in display/logs)
- `ANTHROPIC_MODEL` - Default model
- `ANTHROPIC_SMALL_FAST_MODEL` - Fast model (optional)

### 📚 History & Audit

Every operation logged with:
- UUID + timestamp + system username
- Operation type (switch/backup/restore/validate/update)
- Environment variable changes (masked)
- From/to config + backup path
- Result (success/failure/warning)

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
├── backend/          # 🦀 Actix Web server
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
└── frontend/         # ⚛️ Next.js 16 App Router
    ├── src/
    │   ├── app/              # Route segments (configs, commands, agents, ...)
    │   ├── components/       # Reusable UI components
    │   └── lib/              # API clients & helpers
    ├── package.json
    └── next.config.mjs
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
