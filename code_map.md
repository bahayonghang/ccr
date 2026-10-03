# Code Map

Minimal navigation for this repository. Keep behavior rules in `AGENTS.md` / `CLAUDE.md`; this file is only for orientation and verification anchors.

## Top-level areas

- `crates/` — Rust workspace with the CLI/TUI entry point in `crates/ccr` and shared crates such as `ccr-core`, `ccr-config`, `ccr-codex`, `ccr-db`, `ccr-usage`, and `ccr-types`.
- `crates/ccr-usage/` — sole owner of usage SQL and read-only SQLite projections consumed by CLI/TUI/Tauri. Contracts: `.trellis/spec/ccr/backend/llmusage-provider-adapter.md` and `.trellis/spec/ccr/backend/usage-job-lifecycle.md`. Desktop `llmusage_adapter/` owns installed-CLI sync, NDJSON, and DTO/error mapping only; do not link the upstream `llmusage` Rust crate.
- `ccr-ui/` — React 19 + Tauri desktop UI (`src/shell`, `src/features`, `src/api`, `src-tauri/`, `tests/`).
- `ccr-vscode/` — VS Code extension (`src/providers`, `src/services`, extension tests).
- `docs/` — VitePress documentation site. Agent harness routing: `docs/agents/harnesses.md`.
- `scripts/` — version synchronization checks and repo automation.
- `.github/skills/` — tracked shared skill sources (several are five-tool; local clients install them into gitignored paths such as `.codex/skills/`; see the harnesses page).

## Verification anchors

- `justfile` — root repo checks and aggregate gates.
- `ccr-ui/justfile` — Tauri UI local checks.
- `ccr-vscode/justfile` — VS Code extension local checks.
- `docs/package.json` — VitePress docs scripts.

## Generated / ignored paths

Skip generated, local-runtime, reference, or large static output unless the task explicitly targets them: `target/`, `node_modules/`, `dist/`, `.omx/state/`, `.omx/tmp/`, `outputs/`, `ccr-ui/ref/`, and `ccr-ui/public/fonts/`.
