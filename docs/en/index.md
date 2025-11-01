---
layout: home

hero:
  name: "CCR"
  text: "Claude Code Configuration Switcher"
  tagline: Powerful configuration management tool for Claude Code
  image:
    src: /logo.svg
    alt: CCR
  actions:
    - theme: brand
      text: Quick Start
      link: /en/quick-start
    - theme: alt
      text: Commands
      link: /en/commands/
    - theme: alt
      text: GitHub
      link: https://github.com/bahayonghang/ccr

features:
  - icon: 🚀
    title: Fast Configuration Switching
    details: Directly manipulates settings.json for immediate effect, no restart or manual environment variable setup required

  - icon: 📊
    title: Beautiful Table UI
    details: Display configuration info with comfy-table, compare configs at a glance with color highlights and icons

  - icon: 🖥️
    title: Interactive TUI
    details: Full-featured terminal interface with 3 tabs (Configs/History/System), keyboard navigation, Vim shortcuts support

  - icon: 🔐
    title: Concurrency Safe
    details: File locking ensures multi-process safety, atomic operations prevent data corruption

  - icon: 📝
    title: Complete Audit Trail
    details: Record all operation history, track environment variable changes, auto-mask sensitive information

  - icon: 💾
    title: Smart Backup Management
    details: Automatically keep the latest 10 backups, no manual cleanup needed, support restore from backups, timestamped backup files

  - icon: ✅
    title: Configuration Validation
    details: Automatically validate configuration integrity, check required fields, verify URL formats

  - icon: 🌐
    title: Web Interface
    details: Built-in lightweight Axum API server, 14 complete RESTful API endpoints, smart port binding, support --no-browser option, modern glassmorphism design

  - icon: 📊
    title: Statistics & Cost Analysis
    details: Complete usage statistics and cost tracking system, support multi-dimensional analysis by time/model/project, provide CLI commands, Web API and visual dashboard

  - icon: 🎨
    title: CCR UI Application
    details: Complete Vue.js 3 + Axum application (ports 5173/8081), visual dashboard and statistical analysis, support multi-CLI tool management, auto-download from GitHub on first use

  - icon: 🔄
    title: CCS Fully Compatible
    details: Share ~/.ccs_config.toml configuration file, consistent command-line interface, can coexist with CCS

  - icon: ⚡
    title: High Performance
    details: Rust implementation, excellent performance, fast response, low resource usage
---

## Installation

### Quick Install (Recommended)

Install directly from GitHub using cargo:

```bash
cargo install --git https://github.com/bahayonghang/ccr ccr
```

### Build from Source

```bash
# Clone repository
cd ccs/ccr

# Build release version
cargo build --release

# Install to system path (optional)
cargo install --path .
```

## Quick Usage

```bash
# Initialize configuration file
ccr init

# View all configurations
ccr list

# Switch configuration
ccr switch anthropic

# View current status
ccr current

# Launch interactive TUI
ccr tui

# Launch lightweight Web API server
ccr web

# Launch complete CCR UI application
ccr ui

# View operation history
ccr history

# Launch Web interface (auto-open browser)
ccr web

# Launch Web interface (no browser, smart port binding)
ccr web --port 8080 --no-browser
```

## File Structure

```
~/.ccs_config.toml          # Configuration file (shared with CCS)
~/.claude/settings.json     # Claude Code settings file
~/.claude/backups/          # Auto backup directory
~/.claude/ccr_history.json  # Operation history log
~/.claude/.locks/           # File lock directory
```

## Feature Highlights

### 🎯 Direct Claude Code Settings Control

CCR directly modifies `~/.claude/settings.json` file, no manual environment variable configuration needed, configuration takes effect immediately.

### 🔒 Multi-Process Safety Guarantee

Ensure concurrent operation safety through file locking mechanism, support timeout protection to avoid deadlock, atomic write prevents data corruption.

### 📊 Operation Audit Trail

Record complete information for each operation:
- Operation ID (UUID)
- Timestamp
- Operator (system username)
- Operation type
- Environment variable changes (masked)
- Operation result and remarks

### 💡 Smart Backup Management

- Auto backup before switching configuration
- **Automatically keep the latest 10 backups**, no manual cleanup needed
- Backup files include timestamp and configuration name
- Support cleaning older backups to free up space
- Can restore configuration from backups

## Differences from CCS

| Feature | CCS (Shell) | CCR (Rust) |
|---------|-------------|-----------|
| Configuration switching | ✅ | ✅ |
| Environment variable setup | ✅ | ✅ |
| Direct write to settings.json | ❌ | ✅ |
| File locking mechanism | ❌ | ✅ |
| Operation history | ❌ | ✅ |
| Auto backup | ❌ | ✅ |
| Configuration validation | Basic | Complete |
| Concurrency safe | ❌ | ✅ |
| Web interface | ❌ | ✅ |
| Performance | Fast | Extremely Fast |

## License

MIT License

## Contributing

Issues and Pull Requests are welcome!

## Related Links

- [GitHub Repository](https://github.com/bahayonghang/ccr)
- [Issue Tracker](https://github.com/bahayonghang/ccr/issues)
- [CCS Project](https://github.com/bahayonghang/ccs)
