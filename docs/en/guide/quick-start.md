# Quick Start

This page covers the shortest first-run path: install CCR, initialize platform profile catalogs, create your first profile safely, switch runtimes explicitly, and understand how `ccr`, `ccr ui`, and `ccr-ui` fit together.

## Requirements

- Rust 1.90+
- Optional: Node.js 18+ and Bun 1.0+ when developing `ccr-ui` locally
- Recommended: `just`

## Install

### Install from GitHub

```bash
cargo install --git https://github.com/bahayonghang/ccr ccr
```

### Install from source

```bash
git clone https://github.com/bahayonghang/ccr.git
cd ccr
cargo install --path crates/ccr
```

## Initialize platform catalogs

In CCR 7.x, each supported platform stores its live profile catalog in SQLite at `~/.ccr/platforms/<platform>/profiles.db`. Initialize the platforms you use from their embedded templates:

```bash
ccr claude profile init
ccr codex profile init
ccr grok profile init
```

> To initialize a `mini-trellis` or `trellis` workflow inside a project repository, run `ccr project init`.

After initialization, inspect the runtime overview and platform registry:

```bash
ccr current
ccr platform list
```

## Create and switch your first Claude profile

Always pipe credentials via `--auth-token-stdin` so plaintext tokens never appear in process arguments or shell history, or use `profile open` to edit a projection in `$VISUAL` / `$EDITOR`:

```bash
printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile create work \
  --base-url https://api.example.com \
  --auth-token-stdin

ccr claude profile list
ccr claude profile switch work
ccr claude profile current
ccr claude profile off
```

## Manage Codex and Grok profiles

```bash
# Codex
ccr codex auth current
ccr codex profile list
ccr codex profile switch <name>
ccr codex profile off

# Grok (bind an environment variable name with --env-key or pipe via --api-key-stdin)
ccr grok profile create xai --env-key XAI_API_KEY
ccr grok profile list
ccr grok profile switch xai
ccr grok profile off
```

## Smallest daily loop

```bash
ccr current
ccr validate
ccr doctor
ccr history --limit 20
```

## Graphical entrypoint

```bash
ccr ui -p 15173 --backend-port 38081
```

- `ccr`: main CLI / terminal TUI entrypoint
- `ccr ui`: graphical entrypoint (default frontend port `15173`, backend port `38081`)
- `ccr-ui`: project directory for frontend development and Tauri runtime work

## Upgrade and migration note

CCR 7.x consolidates profile routing onto explicit platform-scoped commands (`ccr <claude|codex|grok> profile <action>`) backed by SQLite `profiles.db` catalogs rather than implicit global platform switches or legacy top-level commands. For the complete legacy-to-7.x mapping, see the [Migration Guide](/en/reference/migration).

## Where to go next

- [CLI Workflows](/en/guide/cli-workflows)
- [Configuration Model](/en/guide/configuration)
- [Entrypoints](/en/guide/entrypoints)
- [Command Reference](/en/reference/commands/)
