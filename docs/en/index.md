---
layout: home

hero:
  name: "CCR"
  text: "Claude Code Configuration Switcher"
  tagline: Multi-platform configuration switcher with CLI/TUI/Web/API/UI
  image:
    src: /logo.svg
    alt: CCR
  actions:
    - theme: brand
      text: Quick Start
      link: /en/guide/quick-start
    - theme: alt
      text: Commands
      link: /en/reference/commands/
    - theme: alt
      text: 中文
      link: /

features:
  - icon: 🚀
    title: Multi-interface
    details: "CLI first; optional TUI, legacy Axum Web API (`ccr web`), and full CCR UI (`ccr ui`, Vue3 + Axum + Tauri)."
  - icon: 🔐
    title: Safe writes
    details: "File locks + in-process mutex + atomic writes to `settings.json` and config files."
  - icon: 🔀
    title: Unified registry
    details: "Default Unified mode with `config.toml` + per-platform profiles; Legacy `~/.ccs_config.toml` still works."
  - icon: 🧭
    title: Direct Claude settings
    details: "Writes `~/.claude/settings.json`, auto-backup/audit; supports temporary token/base_url/model override."
  - icon: ☁️
    title: WebDAV multi-folder sync
    details: "Folder registry/enablement, batch or single push/pull/status, interactive allow-list, smart filters for backups/history/locks/ccr-ui."
  - icon: 📊
    title: Stats & history
    details: "`ccr stats` (web feature) for cost/usage with JSON output; full history with masked env diffs."
---

## Installation

Current version: **3.9.0** (Rust 2024). Requirements: Rust 1.85+; optional Node.js 18+ + Bun 1.0+ for CCR UI development, `just` for scripts.

### Quick Install

```bash
cargo install --git https://github.com/bahayonghang/ccr ccr
```

### Build from Source

```bash
git clone https://github.com/bahayonghang/ccr.git
cd ccr
cargo build --release
cargo install --path .
```

## Quick Usage

```bash
# Unified mode init & platforms
ccr init
ccr platform list
ccr platform switch codex

# Profile lifecycle
ccr add && ccr list && ccr switch <name>
ccr validate
ccr export --no-secrets
ccr import configs.toml --merge
ccr clean --days 30
ccr temp-token set sk-xxx

# Sync
ccr sync config
ccr sync folder add claude ~/.claude -r /ccr-sync/claude
ccr sync push -i
ccr sync all status

# Interfaces
ccr ui -p 3000 --backend-port 38081
ccr tui
ccr web --port 8080 --no-browser
```

## File Structure

```
~/.ccr/
  config.toml               # Unified platform registry
  platforms/<name>/profiles.toml
  history/<name>.json
  backups/<name>/
  ccr-ui/                   # UI cache/downloads
~/.ccs_config.toml          # Legacy mode (compatible)
~/.claude/settings.json     # Claude settings (atomic write target)
```

## Highlights

- Direct Claude settings writes with backups and masked audit history.
- Unified multi-platform registry (Claude, Codex, Gemini, Qwen, iFlow stubs) with `platform` commands.
- WebDAV sync with folder registry, batch/all commands, allow-list and smart filters.
- Full interfaces: CLI/TUI/legacy Web API + CCR UI (auto-detect local/user dir/download).
- Stats & cost analysis (`ccr stats`, web feature), JSON-friendly outputs.

## Differences from CCS

| Feature | CCS (Shell) | CCR (Rust) |
|---------|-------------|------------|
| Configuration switching | ✅ | ✅ |
| Unified multi-platform registry | ❌ | ✅ |
| Direct write to settings.json | ❌ | ✅ |
| File locking / atomic writes | ❌ | ✅ |
| Operation history + masking | ❌ | ✅ |
| Auto backup & cleanup | ❌ | ✅ |
| Web/TUI/Full UI | ❌ | ✅ |
| WebDAV multi-folder sync | ❌ | ✅ |

## License

MIT License

## Contributing

Issues and Pull Requests are welcome!
