# Quick Start

Use this page for the first-run path only: install CCR, initialize the workspace, create one profile, and understand how `ccr`, `ccr ui`, and `ccr-ui` fit together.

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

Workspace notes:
- The installable CLI crate lives in `crates/ccr`
- `crates/ccr-db` and `crates/ccr-types` provide supporting crates
- `docs/`, `scripts/`, and `examples/` stay at repository root

## Initialize CCR

CCR defaults to Unified Mode:

```bash
ccr init
```

Core layout:

```text
~/.ccr/
├── config.toml
├── platforms/
│   ├── claude/
│   ├── codex/
│   ├── gemini/
│   ├── qwen/
│   ├── iflow/
│   └── droid/
├── history/
└── backups/
```

If you still need legacy single-file mode:

```bash
export CCR_LEGACY_MODE=1
ccr init
```

## Create Your First Profile

```bash
ccr platform list
ccr add
ccr list
ccr switch <name>
```

Smallest daily loop:

```bash
ccr current
ccr validate
ccr history -l 20
```

## Graphical Entry Point

```bash
ccr ui -p 15173 --backend-port 38081
```

- `ccr`: primary CLI/TUI entrypoint
- `ccr ui`: launches the standalone `ccr-ui` graphical interface
- `ccr-ui`: project directory for frontend development and Tauri desktop runtime

## Where To Go Next
- Day-to-day command organization: [CLI Workflows](/en/guide/cli-workflows)
- Runtime modes and startup chain: [UI Overview](/en/guide/ui-overview)
- Full command surface: [Command Reference](/en/reference/commands/)
