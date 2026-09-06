# Code Map

Minimal navigation for this repository. Keep behavior rules in `AGENTS.md` / `CLAUDE.md`; this file is only for orientation and verification anchors.

## Top-level areas

- `crates/` — Rust workspace with the CLI/TUI entry point in `crates/ccr` and shared crates such as `ccr-core`, `ccr-config`, `ccr-codex`, `ccr-db`, `ccr-usage`, and `ccr-types`.
- `ccr-ui/` — React 19 + Tauri desktop UI (`src/shell`, `src/features`, `src/api`, `src-tauri/`, `tests/`).
- `ccr-vscode/` — VS Code extension (`src/providers`, `src/services`, extension tests).
- `docs/` — VitePress documentation site. Agent harness routing: `docs/agents/harnesses.md`.
- `scripts/` — version synchronization checks and repo automation.
- `.codex/skills/` — project-local Codex skills (several are five-tool; see the harnesses page).

## Verification anchors

- `justfile` — root repo checks and aggregate gates.
- `ccr-ui/justfile` — Tauri UI local checks.
- `ccr-vscode/justfile` — VS Code extension local checks.
- `docs/package.json` — VitePress docs scripts.

## Generated / ignored paths

Skip generated, local-runtime, reference, or large static output unless the task explicitly targets them: `target/`, `node_modules/`, `dist/`, `.omx/state/`, `.omx/tmp/`, `outputs/`, `ccr-ui/ref/`, and `ccr-ui/public/fonts/`.
