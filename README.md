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
| 🌐 **Web UI** | 11 complete RESTful API endpoints, browser-based management |
| 🏗️ **Modern Architecture** | Service layer pattern, modular design, 95%+ test coverage |
| ⚡ **Smart Update** | Real-time progress display during auto-update |
| 🔄 **CCS Compatible** | Shares `~/.ccs_config.toml` - seamlessly coexist with shell version |

## 📦 Installation

First, you need to install Rust and Cargo, then execute the following commands:

**One-line install from GitHub:**

```bash
cargo install --git https://github.com/bahayonghang/ccr
```

**From source:**

```bash
git clone https://github.com/bahayonghang/ccr.git
cd ccr
cargo install --path .
```

**Requirements:** Rust 1.85+ (for edition 2024 features)

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
- `GET /api/configs` - List all
- `POST /api/switch` - Switch config
- `GET /api/history` - View history
- `POST /api/validate` - Validate all
- `POST /api/clean` - Clean backups
- `POST/PUT/DELETE /api/config` - CRUD operations

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
├── web/              # 🌐 Web Layer (HTTP server + API)
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
```

**Commands:**
```bash
cargo test            # 🧪 Run tests
cargo clippy          # 🔍 Lint
cargo fmt             # 💅 Format
cargo build --release # 🏗️ Production build
```

## 🏗️ Architecture

CCR v1.1.0 features a strict layered architecture:

```
CLI/Web Layer → Services → Managers → Core/Utils
```

- **Service Layer**: 4 services (Config, Settings, History, Backup) - 26 methods
- **Web Module**: Modular design (models, server, handlers, routes) - 11 API endpoints
- **Infrastructure**: Atomic writer, file manager trait, validation trait
- **Test Coverage**: 95%+ (77/81 tests passed)

For detailed architecture docs, see [ARCHITECTURE.md](ARCHITECTURE.md).

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

