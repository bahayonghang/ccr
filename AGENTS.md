# Repository Guidelines

## Project Structure & Module Organization
`crates/` is the Rust workspace. `crates/ccr` is the installable CLI/TUI entry point, while shared logic lives in crates such as `ccr-core`, `ccr-config`, `ccr-codex`, `ccr-db`, and `ccr-types`. `ccr-ui/` contains the Vue 3 + Tauri app (`src/`, `src-tauri/`, `tests/`). `ccr-vscode/` contains the VS Code extension (`src/providers`, `src/services`). `docs/` holds VitePress docs. `scripts/` holds repo automation and version-sync checks. Before broad repo searches, read `./code_map.md` to identify the relevant module and verification anchors. Usage analytics depend on the external [`llmusage`](https://github.com/bahayonghang/llmuasage) crate, declared as a pinned `rev` git dependency in `ccr-ui/src-tauri/Cargo.toml` and wrapped by `ccr-ui/src-tauri/src/llmusage_adapter/`.

## Build, Test, and Development Commands
- `just build` — build the Rust CLI in debug mode.
- `just test` — run Rust workspace tests.
- `just ci` — run the repo-wide CI path: version checks, fmt, clippy, tests, build, audit, and frontend checks.
- `just ui-dev` / `just ui-check` — develop or validate the Tauri UI.
- `cd ccr-ui && bun run dev` — run the web UI locally.
- `cd ccr-ui && bun run test` — run i18n and Vitest smoke tests.
- `cd ccr-vscode && npm run build && npm test` — build and test the VS Code extension.

Quick-check priority:
- General preflight: `just version-check`, then `just fmt-check`.
- Rust changes: `just lint-strict`, then `just test`.
- Frontend fast feedback: `just frontend-check-quick`.
- UI complete checks: `just ui-check` for the Tauri UI, or `just frontend-check` for frontend build/docs coverage.
- VS Code extension: `just vscode-ci`.
- Final full gate: `just ci` for cross-module changes, release readiness, or final acceptance.

## Coding Style & Naming Conventions
Rust code must stay `cargo fmt` and clippy clean. Prefer `Result`-based error handling. Do not add `unwrap` or `expect` in production paths. Rust files and modules use `snake_case`; structs, enums, and traits use `PascalCase`.

Frontend and extension code use 2-space indentation, single quotes, and no semicolons. Vue components use `PascalCase.vue`. Follow existing store, service, and component patterns before adding new abstractions.

## Testing Guidelines
Keep Rust integration tests under `crates/*/tests` and group them by feature area. UI smoke tests belong in `ccr-ui/tests/**/*.smoke.test.ts`. VS Code tests live beside source as `*.test.ts`. Start with the narrowest relevant check, then escalate by impact scope: subsystem checks for local changes, `just frontend-check` or `just vscode-ci` for full frontend/extension coverage, and `just ci` for cross-module changes or delivery-ready validation.

Project-local Codex skills live under `.codex/skills/`; prefer the narrowest failing gate first before escalating to full `just ci`.

## Commit & Pull Request Guidelines
Recent history uses Chinese Conventional Commits with scopes and emoji, for example `feat(认证TUI): ✨ ...`, `docs(帮助文档): 📝 ...`, and `chore(release): 🔧 ...`. Keep commits atomic and scoped to one surface.

PRs should state the affected area (`CLI`, `ccr-ui`, `ccr-vscode`, docs), link the issue when available, list the verification commands you ran, and include screenshots or GIFs for UI and extension changes.

## Security & Configuration Tips
Never commit personal config, tokens, or local runtime files from home-directory toolchains. Use examples and fixtures instead. When editing config flows, preserve backup, masking, and atomic-write behavior.

<!-- TRELLIS:START -->
# Trellis Instructions

These instructions are for AI assistants working in this project.

This project is managed by Trellis. The working knowledge you need lives under `.trellis/`:

- `.trellis/workflow.md` — development phases, when to create tasks, skill routing
- `.trellis/spec/` — package- and layer-scoped coding guidelines (read before writing code in a given layer)
- `.trellis/workspace/` — per-developer journals and session traces
- `.trellis/tasks/` — active and archived tasks (PRDs, research, jsonl context)

If a Trellis command is available on your platform (e.g. `/trellis:finish-work`, `/trellis:continue`), prefer it over manual steps. Not every platform exposes every command.

If you're using Codex or another agent-capable tool, additional project-scoped helpers may live in:
- `.agents/skills/` — reusable Trellis skills
- `.codex/agents/` — optional custom subagents

Managed by Trellis. Edits outside this block are preserved; edits inside may be overwritten by a future `trellis update`.

<!-- TRELLIS:END -->
