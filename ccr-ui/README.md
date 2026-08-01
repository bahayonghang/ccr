# CCR UI - Vue 3 + Tauri Desktop

> CCR Desktop UI for advanced AI CLI configuration, runtime monitoring, sync, and usage insight workflows.

[![Version](https://img.shields.io/badge/version-7.1.2-blue.svg)](../Cargo.toml)
[![Vue](https://img.shields.io/badge/Vue-3.5-4FC08D.svg)](https://vuejs.org/)
[![Tauri](https://img.shields.io/badge/Tauri-2.10-FFC131.svg)](https://tauri.app/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.9-3178C6.svg)](https://www.typescriptlang.org/)

## Current contract

- **Primary runtime**: Tauri desktop app. UI commands call the Rust backend through Tauri invoke APIs.
- **Web runtime**: Vite dev/preview surface for frontend iteration, visual checks, and smoke tests. Tauri-only invoke paths may be unavailable in a plain browser.
- **Package manager**: Bun is the only maintained frontend package manager. bun.lock is the dependency source of truth, and npm lockfiles are not maintained.
- **Design direction**: calm, precise, editorial workbench for power users; avoid generic SaaS, loud glassmorphism, mascot, or anime styling.

## Requirements

| Tool | Required version / source |
| --- | --- |
| Bun | `bun@1.3.10` from `package.json#packageManager` |
| Node.js | Use the version bundled/required by the installed Bun and tooling |
| Rust | `>= 1.95` from `src-tauri/Cargo.toml#rust-version` |
| Rust edition | Edition 2024 |
| Tauri | 2.x, pinned in `src-tauri/Cargo.toml` and `package.json` |

Platform desktop dependencies still follow the official Tauri 2 prerequisites for Linux, macOS, and Windows.

## Install

```bash
cd ccr-ui
bun install --frozen-lockfile
```

Do not generate or commit npm lockfiles for this package.

## Development

### Desktop app

```bash
cd ccr-ui
bun run tauri:dev
```

### Web preview

```bash
cd ccr-ui
bun run dev:web -- --host 127.0.0.1 --strictPort
```

Open <http://127.0.0.1:5173/> for browser-based UI inspection. Treat failures in Tauri-only API calls as browser runtime limitations unless the task explicitly targets web compatibility.

## Verification

Run the smallest relevant check first:

```bash
# frontend
bun run type-check
bun run lint
bun run test
bun audit --audit-level=high

# Tauri backend from ccr-ui/
bun run tauri:check
bun run tauri:test

# root-level quick checks from repository root
just version-check
just frontend-check-quick
just ui-check
```

`bun run lint` is no-fix and safe for CI. Use `bun run lint:fix` only when intentionally mutating files locally.

## Build

```bash
# web bundle
bun run build:web

# desktop bundle
bun run build:desktop
```

Desktop artifacts are emitted under `src-tauri/target/release/bundle/`.

## Project map

```text
ccr-ui/
├── src/                 # Vue application
│   ├── api/             # Tauri/domain API wrappers and compatibility facades
│   ├── components/      # Shared Vue components
│   ├── views/           # Route-level views
│   ├── router/          # Vue Router setup
│   ├── store/           # Pinia stores
│   ├── styles/          # Global styles and tokens
│   └── main.ts          # Frontend entrypoint
├── src-tauri/           # Tauri Rust backend and desktop configuration
├── tests/               # Frontend smoke/i18n tests
├── scripts/             # UI automation and bundle checks
├── bun.lock             # Maintained JS dependency lockfile
├── package.json         # Bun scripts and frontend manifest
└── justfile             # UI-local command runner
```

## Core modules

- **Dashboard / Home**: runtime overview and navigation.
- **Claude Code**: profiles, settings, MCP, agents, plugins, usage insight.
- **Codex**: profiles, MCP, agents, slash commands, plugins, usage insight.
- **Gemini CLI / Antigravity compatibility**: configuration and migration-aware workflows.
- **Command center**: controlled CLI command execution surface.
- **Sync / WebDAV**: configuration backup and restore workflows.
- **Usage analytics**: llmusage-backed local usage import and dashboards.

## API boundary

The current UI API layer is Tauri-first:

- `src/api/index.ts` is the main import surface for UI code.
- Domain APIs live under `src/api/domains/*`.
- `src/api/tauri.ts` remains a compatibility facade; avoid adding new domain APIs there unless the compatibility boundary is explicitly part of the task.

## Related docs

- Root repository guide: [`../AGENTS.md`](../AGENTS.md)
- UI agent notes: [`./AGENTS.md`](./AGENTS.md)
- UI developer notes: [`./README.dev.md`](./README.dev.md)
- Root docs site: [`../docs/`](../docs/)

---

Made with Vue 3, Tauri 2, TypeScript 5.9, Bun, and a restrained editorial surface system.

**Version**: 7.1.1
