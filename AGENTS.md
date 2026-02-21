# Repository Guidelines
请使用中文回复。

## Project Structure & Module Organization

- `src/`: main Rust crate (`ccr`) containing the CLI and shared library.
- `tests/`: Rust integration tests (`cargo test --test <file_stem>`).
- `ccr-ui/backend/`: Axum backend (workspace member) used by the CCR UI.
- `ccr-ui/frontend/`: Vue 3 + Vite + TypeScript + Tailwind + Tauri frontend.
- `docs/` and `ccr-ui/docs/`: VitePress documentation sites.
- `examples/`, `scripts/`, `web/`: examples, helper scripts, and a small static web page.

## Build, Test, and Development Commands

Rust (workspace):
- `cargo build --workspace`: build all Rust workspace members.
- `cargo test --workspace --all-features -- --test-threads=1`: run the full Rust test suite (matches CI).
- `cargo fmt --all`: format Rust code; CI enforces `--check`.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: lint (CI treats warnings as errors).

Task runner (optional):
- `just --list`: list helper tasks from `justfile` (e.g. `just lint`, `just ci`, `just dev`).

Frontend (Bun recommended):
- `cd ccr-ui/frontend && bun install --frozen-lockfile`
- `bun run dev` / `bun run build` / `bun run type-check` / `bun run lint`

Docs:
- `cd docs && bun install && bun run dev` (or `cd ccr-ui/docs ...`)

## Coding Style & Naming Conventions

- Rust: follow `rustfmt` output; prefer `clippy`-clean code (CI also warns on `unwrap()` usage).
- TypeScript/Vue: 2-space indentation; format via Prettier (`ccr-ui/frontend/.prettierrc`) and lint via ESLint (`bun run lint`).
- Naming: Rust modules/functions `snake_case`, types `CamelCase`; Vue components `PascalCase.vue`.

## Testing Guidelines

- Rust tests live in `tests/*.rs`; add/extend integration coverage when changing managers/services.
- Run a single file: `cargo test --test manager_tests`; run by name: `cargo test <keyword>`.

## Commit & Pull Request Guidelines

- Commits generally follow Conventional Commits with optional emoji prefixes, e.g. `feat(core): ...`, `fix(backend): ...`, `refactor(ui): ...`, `docs: ...`, `chore: ...`.
- PRs should include: a clear summary, rationale, and any linked issues. Include screenshots/GIFs for UI changes and note any config migrations or breaking behavior.

## Security & Configuration Tips

- Do not commit secrets (API keys/tokens) or local profiles. When sharing exported configs, prefer “no secrets” options and redact logs when needed.
