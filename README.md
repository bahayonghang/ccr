# CCR (Claude Code Configuration Switcher)

**High-performance, multi-platform configuration management tool written in Rust.**  
Unified management for specific AI CLI tools including **Claude Code**, **Codex**, **Gemini**, **Qwen**, and more.

![Version](https://img.shields.io/badge/version-3.17.3-blue.svg) ![License](https://img.shields.io/badge/license-MIT-green.svg) ![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)

---

## ✨ Features

- **Multi-Platform Support**: Unified management for Claude, Codex, Gemini, Qwen, and iFlow. Each platform has independent profiles, history, and backups.
- **Enterprise-Grade Safety**: Atomic writes, file locking (`fs4`), comprehensive audit logs, and automatic backups before every modification.
- **Multi-Interface**: 
  - **CLI**: Powerful command-line interface for all operations.
  - **TUI**: Interactive terminal configuration selector with Tab navigation.
  - **Web API**: Embedded Axum server for external integration.
  - **Desktop UI**: Full-stack application built with Vue 3 + Tauri.
- **Smart Sync**: WebDAV-based multi-folder synchronization (`web` feature) to keep your configs consistent across machines.
- **Secure**: Sensitive data (API keys, tokens) is automatically masked in outputs.

## 📦 Installation

### One-Line Install
```bash
cargo install --git https://github.com/bahayonghang/ccr ccr
```

### From Source
```bash
git clone https://github.com/bahayonghang/ccr.git
cd ccr
cargo install --path .
```

### Build Requirements
- **Rust**: 1.85+ (Edition 2024)
- **Node.js**: 18+ (For UI development)

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
├── src/            # Core Rust logic (CLI, TUI, Web API)
├── ccr-ui/         # Full-stack Web/Desktop App (Vue 3 + Tauri)
├── tests/          # Integration tests
└── justfile        # Task runner configuration
```

## 📄 License
MIT License
