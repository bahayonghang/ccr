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
  - icon: 💸
    title: Budget guardrails
    details: "Track monthly/weekly budgets with `ccr budget status/set/reset`; keep usage in check alongside pricing tables."
  - icon: 📒
    title: Pricing tables
    details: "Manage per-model prices with `ccr pricing list/set/remove/reset`; clean JSON export for audits."
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
    details: "`ccr stats summary/import/export/clear` with table/JSON/CSV; history with masked env diffs."
---

## Installation

Current version: **3.20.11** (Rust 2024). Requirements: Rust 1.90+; optional Node.js 18+ + Bun 1.0+ for CCR UI development, `just` for scripts.
Current workspace note: the installable CLI crate lives in `crates/ccr`; `crates/ccr-db` and `crates/ccr-types` provide supporting crates for the refactored layout.

### Quick Install

```bash
cargo install --git https://github.com/bahayonghang/ccr ccr
```

### Build from Source

```bash
git clone https://github.com/bahayonghang/ccr.git
cd ccr
cargo build --release -p ccr
cargo install --path crates/ccr
```

Need the old-to-new path map? See the [Migration Guide](/en/reference/migration).

## Workspace Layout

```text
ccr/
├── Cargo.toml              # workspace manifest + shared dependencies
├── crates/
│   ├── ccr/                # installable CLI crate + shared runtime logic
│   ├── ccr-db/             # database services and models
│   └── ccr-types/          # shared types
├── ccr-ui/                 # Vue 3 + Tauri application root
├── docs/                   # VitePress documentation
├── scripts/                # automation and maintenance helpers
├── examples/               # sample configs
└── outputs/                # collected/generated artifacts (when present)
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
ccr web --host 0.0.0.0 --port 19527 --no-browser

# Cost controls & reporting
ccr pricing list --verbose
ccr pricing set claude-3.5-sonnet 15
ccr budget status
ccr budget set --period monthly --limit 120
ccr stats summary --format table
ccr stats export --format json > stats.json
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

- Cost controls: budgets (`ccr budget`) and pricing tables (`ccr pricing`) to monitor spend and keep rates current.
- Direct Claude settings writes with backups and masked audit history.
- Unified multi-platform registry (Claude, Codex, Gemini, Qwen, iFlow stubs) with `platform` commands.
- WebDAV sync with folder registry, batch/all commands, allow-list and smart filters.
- Full interfaces: CLI/TUI/legacy Web API + CCR UI (auto-detect local/user dir/download).
- Stats: `ccr stats summary/import/export/clear` with table/JSON/CSV outputs for reporting.

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
