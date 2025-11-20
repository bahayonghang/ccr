# CCR - Claude Code Configuration Switcher

Rust workspace for managing Claude Code and other AI IDE configs (Claude, Codex/GitHub Copilot, Gemini CLI, Qwen, iFly). Version 3.4.1. CLI-first with optional TUI, Web API, and the full-stack CCR UI (Vue 3 + Axum + Tauri).

## Highlights
- Safe settings management: atomic writes, file locks, audit trail, and auto backups for settings and config files.
- Unified multi-platform mode by default (`~/.ccr/` with per-platform profiles/history/backups); compatible with legacy `~/.ccs_config.toml`.
- Complete config lifecycle: `init`, `add`, `list/current/switch`, `enable/disable`, `validate`, `history`, `optimize`, with `--yes` to skip prompts.
- Import/export/clean: export with or without secrets, import in merge/replace mode with backups, prune old backups.
- WebDAV sync (feature `web`): folder registry, enable/disable, folder/all push/pull/status, interactive selection for conflict handling.
- Platform controls: `ccr platform list/switch/current/info/init` with optional JSON output.
- Temp tokens and updates: `ccr temp-token set/show/clear`, `ccr update --check`.
- Observability: detailed history, JSON output options, cost stats (`ccr stats ...`, feature `web`).
- Interfaces: CLI everywhere, optional TUI (`--features tui`), legacy lightweight API server (`ccr web`), modern CCR UI (`ccr ui`, Vue 3 + Axum backend, Tauri desktop build).

## Installation
Requirements: Rust 1.85+, Cargo. For CCR UI development you also need Node.js 18+ (npm) and optionally `just`.

One-line install:
```bash
cargo install --git https://github.com/bahayonghang/ccr ccr
```

From source:
```bash
git clone https://github.com/bahayonghang/ccr.git
cd ccr
cargo install --path .
```

Build options:
- `cargo build --no-default-features` (CLI only, fastest)
- `cargo build --features web` (CLI + Web API + sync + UI entrypoint)
- `cargo build --features tui` (CLI + TUI)
- `cargo build --all-features`
- Workspace build/test: `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`

## Workspace layout
```
ccr/
|-- src/                # CLI + library (platforms, services, sync, web, tui)
|-- ccr-ui/
|   |-- backend/        # Axum server (workspace member)
|   `-- frontend/       # Vue 3 + Vite + Pinia + Tauri
|-- tests/              # Integration suites
|-- docs/               # Documentation site
|-- examples/           # Runnable walkthroughs
`-- justfile            # Common dev tasks
```

## CLI quick start
1) Initialize unified mode:
```bash
ccr init
```
Creates `~/.ccr/config.toml` plus per-platform directories under `~/.ccr/platforms/`. For legacy single-file mode, set `CCR_LEGACY_MODE=1` before `init`.

2) Inspect platforms and switch:
```bash
ccr platform list
ccr platform switch claude   # or codex/gemini/qwen/iflow
```

3) Create and manage configs:
```bash
ccr add                      # guided setup
ccr list
ccr current
ccr switch <name>            # alias: ccr <name>
ccr enable <name> | ccr disable <name> [--force]
ccr validate
ccr history -l 50
ccr optimize
```

4) Import/export and backups:
```bash
ccr export -o configs.toml --no-secrets
ccr import configs.toml --merge --backup
ccr clean --days 30 --dry-run
```

5) WebDAV multi-folder sync (feature `web`):
```bash
ccr sync config                    # configure WebDAV endpoint
ccr sync folder add claude ~/.claude -r /ccr-sync/claude
ccr sync folder enable claude
ccr sync all status
ccr sync all push --force
ccr sync claude pull               # external_subcommand style
```

6) Run interfaces:
```bash
ccr ui -p 3000 --backend-port 8081   # full UI (Vue 3 + Axum), auto-detects local ~/.ccr/ccr-ui or downloads from GitHub
ccr tui                              # optional TUI (feature tui)
ccr web -p 8080                      # legacy lightweight API server
```

Other useful commands:
- `ccr temp-token set sk-xxx [--base-url ... --model ...]`
- `ccr update --check`
- `ccr stats cost --today` (feature `web`)
- `ccr version`

## CCR UI (Vue 3 + Axum + Tauri)
Features:
- Visual config management, validation, history, and backups.
- Command executor for all CLI commands with real-time output.
- WebDAV multi-folder sync dashboard (add/enable/disable/push/pull/status/all).
- System info, platform overview, and health checks.
- Supports web mode (HTTP API) and desktop mode (Tauri invoke).

Start via CLI (recommended):
```bash
ccr ui                     # auto-detect workspace -> ~/.ccr/ccr-ui -> download from GitHub
# default ports: frontend 3000, backend 8081 (configurable with -p/--backend-port)
```

Develop from repo:
```bash
cd ccr-ui
just s                     # start frontend + backend in dev mode
just quick-start           # first-time setup (prereqs + install + start)
```

Manual dev (if you prefer explicit commands):
```bash
# Backend (workspace member)
cd ccr-ui/backend
cargo run -- --port 8081

# Frontend
cd ../frontend
npm install
npm run dev                # default http://localhost:5173
```

Production builds:
```bash
cd ccr-ui
just build                 # build backend + frontend
just run-prod              # run compiled backend with built frontend
```

Desktop (Tauri):
```bash
cd ccr-ui
just tauri-dev             # desktop dev window
just tauri-build           # package desktop app
```

## Development workflow
- Fast checks: `cargo check`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Tests: `cargo test --workspace`, `cargo test --test platform_integration_tests -- --test-threads=1`
- just helpers: `just dev`, `just watch`, `just ci`, `just build`, `just release`
- Sync-specific tests/ops require `--features web`.

## Troubleshooting
- Enable logs: `export CCR_LOG_LEVEL=debug` then rerun (`trace`/`debug`/`info`/`warn`/`error`).
- Legacy configs: set `CCR_LEGACY_MODE=1` and run `ccr init` to stay in single-file mode.
- Conflicts during sync: rerun with `--force` or `--interactive` as appropriate.
- Permissions: ensure config and settings files are writable by your user.

## License
MIT
