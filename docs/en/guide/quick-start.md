# Quick Start

This page covers the shortest first-run path: install CCR, initialize it, create the first profile, and understand how `ccr`, `ccr ui`, and `ccr-ui` fit together.

## Requirements
- Rust 1.90+
- Optional: Node.js 18+ and Bun 1.0+ when developing `ccr-ui`
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

## Initialize CCR

```bash
ccr init
```

After initialization, inspect the runtime overview first:

```bash
ccr current
ccr platform list
```

## Create and switch your first Claude profile

```bash
ccr add
ccr claude profile list
ccr claude profile switch <name>
ccr claude profile current
ccr claude profile off
```

## Switch your first Codex profile

```bash
ccr codex auth current
ccr codex profile list
ccr codex profile switch <name>
ccr codex profile current
ccr codex profile off
```

## Smallest daily loop

```bash
ccr current
ccr validate
ccr history -l 20
```

## Graphical entrypoint

```bash
ccr ui -p 15173 --backend-port 38081
```

- `ccr`: main CLI / TUI entrypoint
- `ccr ui`: graphical entrypoint
- `ccr-ui`: project directory for frontend development and Tauri runtime work

## Migration note

These legacy commands are retired:

- `ccr switch <name>`
- `ccr <name>`
- `ccr platform switch <platform>`
- `ccr platform current`

Use these instead:

- `ccr claude profile switch <name>`
- `ccr codex profile switch <name>`
- `ccr current`

## Where to go next

- [CLI Workflows](/en/guide/cli-workflows)
- [Configuration Model](/en/guide/configuration)
- [Entrypoints](/en/guide/entrypoints)
- [Command Reference](/en/reference/commands/)
