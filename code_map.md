# Code Map

Minimal navigation for this repository. Keep behavior rules in `AGENTS.md` / `CLAUDE.md`; this file is only for orientation and verification anchors.

## Top-level areas

- `crates/` — Rust workspace with the CLI/TUI entry point in `crates/ccr` and shared crates such as `ccr-core`, `ccr-config`, `ccr-codex`, `ccr-db`, `ccr-usage`, and `ccr-types`.
- `crates/ccr-usage/` — sole owner of usage SQL and read-only SQLite projections consumed by CLI/TUI/Tauri. Contracts: `.trellis/spec/ccr/backend/llmusage-provider-adapter.md` and `.trellis/spec/ccr/backend/usage-job-lifecycle.md`. Desktop `llmusage_adapter/` owns installed-CLI sync, NDJSON, and DTO/error mapping only; do not link the upstream `llmusage` Rust crate.
- `crates/ccr-app/` — framework-independent configuration/resource/MCP policy, Codex agent/source orchestration, SSH workspace and bounded background runtime; native and Tauri adapters share these owners.
- `crates/ccr-sync/` — WebDAV transfer, sensitive-asset v2 encryption and transactional pull/backup owners. Native sync adapters must preserve this protocol rather than implement another one.
- `crates/ccr-checkin/` — provider/account/check-in/balance services; native operations capture database access and crypto roots for isolated fixtures.
- `ccr-ui/` — React 19 + Tauri desktop UI (`src/shell`, `src/features`, `src/api`, `src-tauri/`, `tests/`).
- `ccr-desktop/` — independent GPUI native workspace (`ccr-gpui`), own lock/target; profiles, platform settings/OAuth, usage/sessions, resources/MCP, retained command/install/sync/check-in operations, Local/SSH selection and independent desktop preferences. Feature adapters are under `src/runtime/`, native controls under `src/features/`, window lifecycle under `src/shell/`. Migration and native acceptance remain in progress.
- `ccr-vscode/` — VS Code extension (`src/providers`, `src/services`, extension tests).
- `docs/` — VitePress documentation site. Agent harness routing: `docs/agents/harnesses.md`.
- `scripts/` — version synchronization checks and repo automation.
- `.github/skills/` — tracked shared skill sources (several are five-tool; local clients install them into gitignored paths such as `.codex/skills/`; see the harnesses page).

## Verification anchors

- `justfile` — root repo checks and aggregate gates.
- `ccr-desktop/justfile` — independent `desktop-*` checks and portable packaging with a matching hashed CLI sidecar; native input probe and command/clean boundary tests.
- `ccr-ui/justfile` — Tauri UI local checks.
- `ccr-vscode/justfile` — VS Code extension local checks.
- `docs/package.json` — VitePress docs scripts.

## Generated / ignored paths

Skip generated, local-runtime, reference, or large static output unless the task explicitly targets them: `target/`, `node_modules/`, `dist/`, `.omx/state/`, `.omx/tmp/`, `outputs/`, `ccr-ui/ref/`, and `ccr-ui/public/fonts/`.
