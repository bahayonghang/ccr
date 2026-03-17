# CCR

**Unified entrypoint for AI CLI configuration management, written in Rust.**  
CLI-first workflow with TUI, legacy Web API, and the full CCR UI for Claude Code, Codex, Gemini, Qwen, Droid, and more.

> Historical note: CCR started as `Claude Code Configuration Switcher`. The repository now tracks a broader multi-platform AI CLI workspace.

![Version](https://img.shields.io/badge/version-5.1.4-blue.svg) ![License](https://img.shields.io/badge/license-MIT-green.svg) ![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)

---

## ✨ Features

- **Unified Platform Registry**: Manage Claude, Codex, Gemini, Qwen, iFlow, Droid, and related AI CLI platforms with isolated profiles, history, and backups.
- **Enterprise-Grade Safety**: Atomic writes, file locking (`fs4`), comprehensive audit logs, and automatic backups before every modification.
- **Multi-Interface**:
  - **CLI**: Powerful command-line interface for all operations.
  - **TUI**: Interactive terminal configuration selector with Tab navigation.
  - **Legacy Web API**: Embedded Axum server for automation, CI, and compatibility workflows.
  - **CCR UI**: Full-stack browser/desktop experience built with Vue 3 + Tauri.
- **Smart Sync**: WebDAV-based multi-folder synchronization keeps your configs consistent across machines.
- **Secure**: Sensitive data (API keys, tokens) is automatically masked in outputs.

## 📦 Installation

### One-Line Install
```bash
cargo install --git https://github.com/bahayonghang/ccr ccr
```

### Install from dev branch (Recommended for latest features)
```bash
cargo install --git https://github.com/bahayonghang/ccr --branch dev ccr
```

### From Source
```bash
git clone https://github.com/bahayonghang/ccr.git
cd ccr
cargo install --path crates/ccr
```

> Workspace note: the installable CLI crate now lives in `crates/ccr`. See `docs/reference/migration.md` for the old-to-new path map.

### Build Requirements
- **Rust**: 1.90+ (Edition 2024, for the installable CLI crate)
- **Node.js**: 18+
- **Bun**: 1.3+ (recommended for `ccr-ui` and embedded web asset builds; npm remains a compatibility fallback only for the legacy embedded web build)

## 🚀 Quick Start

### 1. Initialize
Initialize the unified configuration structure in `~/.ccr/`:
```bash
ccr init
```

### 2. Select Platform
Switch to your desired platform (default is usually `claude`):
```bash
# List available platforms
ccr platform list

# Switch to Gemini (for example)
ccr platform switch gemini
```

### 3. Manage Configurations
```bash
# Interactive wizard to add a new config
ccr add

# List all configs for current platform
ccr list

# View current configuration status
ccr status

# Switch to a specific config
ccr switch my-work-config

# Quick switch (shorthand)
ccr my-work-config

```


### 4. Interactive TUI
Launch the Terminal UI configuration selector:
```bash
# Simply run ccr without arguments
ccr
```

**Keyboard Shortcuts:**
| Key | Action |
|-----|--------|
| `Tab` | Switch between Claude/Codex platforms |
| `←` / `→` | Navigate pages (when >20 configs) |
| `↑` / `↓` / `j` / `k` | Select configuration |
| `Enter` | Apply selected configuration and exit |
| `Space` | Apply selected configuration (stay in TUI) |
| `q` / `Esc` | Quit |

**Features:**
- Dual-tab interface for Claude Code and Codex CLI
- Pagination support (20 configs per page)
- Real-time status messages at the bottom
- Platform-specific color themes (Orange for Claude, Purple for Codex)

## 🖥️ CCR UI

A modern graphical interface is available for managing your configurations.

```bash
# Launch the UI (auto-detects workspace or downloads release)
ccr ui

# Specify custom port
ccr ui -p 3000
```

## 🔐 Codex Multi-Account Management

CCR provides powerful multi-account management for Codex CLI, allowing you to easily switch between different GitHub accounts.

### Basic Commands

```bash
# Save current login as a named account
ccr codex auth save work

# Save with description
ccr codex auth save personal -d "Personal GitHub account"

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
- **Replace**: Always overwrite accounts with the same name

### Interactive TUI

Launch the Codex account management interface:
```bash
ccr codex
```

**Features:**
- Visual account list with token freshness indicators
- 🟢 Fresh (<1 day) | 🟡 Stale (1-7 days) | 🔴 Old (>7 days)
- Process detection warnings before switching
- Email masking for privacy (e.g., `use***@example.com`)

## 🔄 Auto Update

CCR supports automatic updates from GitHub to the latest version.

> Note: `ccr update` compiles the default `web` feature and prefers Bun when building embedded web assets. Node.js 18+ with npm remains a compatibility fallback if Bun is unavailable.

```bash
# Update from main branch (stable)
ccr update

# Update from dev branch (latest features)
ccr update dev

# Check for updates only, without installing
ccr update --check

# Check for dev branch updates
ccr update dev --check
```

| Command | Description |
|---------|-------------|
| `ccr update` | Update to the latest stable version from `main` branch |
| `ccr update dev` | Update from `dev` branch to get the latest features |
| `ccr update --check` | Preview the update command without executing |


## 🛠️ Development

This project uses `just` for task automation.

```bash
# Build all features
just build

# Run tests
just test

# Check code quality
just check
just lint
```

## 📂 Project Structure
overview
```text
ccr/
├── Cargo.toml      # Workspace manifest + shared dependencies
├── crates/
│   ├── ccr/        # Installable CLI crate + library
│   ├── ccr-db/     # Database services and models
│   └── ccr-types/  # Shared type definitions
├── ccr-ui/         # Full-stack Web/Desktop App (Vue 3 + Tauri)
├── docs/           # VitePress documentation
├── scripts/        # Repository automation and maintenance helpers
├── examples/       # Sample configs and usage examples
├── outputs/        # Collected/generated artifacts (when present)
└── justfile        # Task runner configuration
```

## 📄 License
MIT License
