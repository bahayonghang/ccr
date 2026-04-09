# Repository Guidelines

## Project Structure & Module Organization
This repository is a Rust workspace with a Vue/Tauri UI. `crates/ccr/src/` contains the main CLI, TUI, web API, and service layers. `crates/ccr-db/` holds database-facing services and models, while `crates/ccr-types/` provides shared types used across crates. `crates/ccr/tests/` stores workspace integration tests, organized by feature areas such as commands, managers, platforms, and workflows. `ccr-ui/` contains the Vue 3 frontend, Tauri shell in `ccr-ui/src-tauri/`, and lightweight frontend tests in `ccr-ui/tests/`. Keep `docs/`, `scripts/`, and `examples/` at the repository root; `outputs/` is a collected-artifacts directory and should not be used as a native build output.

## Build, Test, and Development Commands
Prefer root `just` recipes so local checks match CI:

- `just check` — fast Rust workspace compile check.
- `just test` — full Rust test suite with `--test-threads=1`.
- `just lint-strict` — CI-grade Clippy, including `clippy::unwrap_used`.
- `just frontend-check` — frontend type-check, lint, build, and docs build.
- `just ui-dev` — start the UI development flow.
- `just tauri-dev` — run the desktop app locally.
- `just ci` — full repository validation.

## Coding Style & Naming Conventions
Rust follows `rustfmt`; use `snake_case` for modules/functions and `PascalCase` for types. Prefer `Result`-based error handling and avoid `unwrap`/`expect` in production code. Keep internal implementation comments in Chinese and public API docs in English. Frontend code uses TypeScript, Vue 3, and `<script setup lang="ts">`; keep 2-space indentation, no semicolons, single quotes, and follow `ccr-ui/.prettierrc`, `ccr-ui/eslint.config.js`, and `ccr-ui/.stylelintrc.json`. Name Vue components `PascalCase.vue`; use `camelCase` for composables, stores, and utilities.

## Testing Guidelines
Add tests close to the changed surface. Put Rust integration coverage in `crates/ccr/tests/` and keep frontend checks in `ccr-ui/tests/`. Run focused checks first, then broader suites. Use `cargo test --workspace --all-features -- --test-threads=1` for Rust and `cd ccr-ui && npm test` for frontend smoke tests. No fixed coverage threshold is defined, but each behavior change should include a regression test or a short rationale.

## Commit & Pull Request Guidelines
Git history follows Conventional Commits with scope and optional emoji, for example `feat(tauri): ✨ add profile switch` or `fix(core): 🐛 handle missing config`. Keep commits scoped to one concern when possible. PRs should include a concise summary, impacted areas, verification commands, linked issues, and screenshots or GIFs for `ccr-ui`/Tauri changes. Call out config, schema, or migration impacts explicitly.

## Security & Configuration Tips
Do not commit real tokens, exported auth data, or local machine state. Use `examples/` and `*.example.*` files for shareable samples. When changing config or sync logic, preserve masking, backup, and atomic-write behavior.

## graphify

This project has a graphify knowledge graph at graphify-out/.

Rules:
- Before answering architecture or codebase questions, read graphify-out/GRAPH_REPORT.md for god nodes and community structure
- If graphify-out/wiki/index.md exists, navigate it instead of reading raw files
- After modifying code files in this session, run `python3 -c "from graphify.watch import _rebuild_code; from pathlib import Path; _rebuild_code(Path('.'))"` to keep the graph current
