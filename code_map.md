# Code Map

Minimal navigation for this repository. Keep behavior rules in `AGENTS.md` / `CLAUDE.md`; this file is only for orientation and verification anchors.

## Top-level areas

- `crates/` — Rust workspace with the CLI/TUI entry point in `crates/ccr` and shared crates such as `ccr-core`, `ccr-config`, `ccr-codex`, `ccr-db`, and `ccr-types`.
- `ccr-ui/` — Vue 3 + Tauri desktop UI (`src/`, `src-tauri/`, `tests/`).
- `ccr-vscode/` — VS Code extension (`src/providers`, `src/services`, extension tests).
- `docs/` — VitePress documentation site.
- `scripts/` — version synchronization checks and repo automation.
- `.codex/skills/` — project-local Codex skills.

## Verification anchors

- `justfile` — root repo checks and aggregate gates.
- `ccr-ui/justfile` — Tauri UI local checks.
- `ccr-vscode/justfile` — VS Code extension local checks.
- `docs/package.json` — VitePress docs scripts.

## Generated / ignored paths

Skip generated, local-runtime, reference, or large static output unless the task explicitly targets them: `target/`, `node_modules/`, `dist/`, `.omx/state/`, `.omx/tmp/`, `outputs/`, `ccr-ui/ref/`, and `ccr-ui/public/fonts/`.
