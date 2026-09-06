# Repository Guidelines

## Project Structure & Module Organization
This file governs `crates/**`, the Rust workspace packages for CCR. Read `./code_map.md` before broad grep or repo-wide code search inside `crates/`. Run workspace commands from the repository root (`D:\Documents\Code\Github\ccr`) because `Cargo.toml` and `justfile` live there. `ccr` is the installable CLI/TUI entry point. Shared domains live in `ccr-core`, `ccr-config`, `ccr-codex`, `ccr-db`, `ccr-store`, `ccr-sync`, `ccr-skills`, `ccr-checkin`, `ccr-cli`, `ccr-tui`, `ccr-usage`, and `ccr-types`. `ccr-usage` owns read-only llmusage SQL projections; do not copy usage SQL into Tauri or TUI. Keep cross-crate API types in `ccr-types` and avoid duplicating config or storage logic across crates.

## Build, Test, and Development Commands
- `just build` — build the default Rust CLI package in debug mode.
- `just check-workspace` — typecheck all Rust workspace crates without producing binaries.
- `just test` — run the Rust workspace test suite.
- `just fmt-check` — verify `cargo fmt` without rewriting files.
- `just clippy` — run clippy with warnings treated as errors.
- `just ci` — run version checks, formatting, clippy, tests, build, audit, and frontend checks.
- `cargo test -p <crate-name>` — iterate on one crate, for example `cargo test -p ccr-config`.

## Coding Style & Naming Conventions
Keep Rust code `cargo fmt` clean and clippy-clean. Use `snake_case` for files, modules, functions, and variables; use `PascalCase` for structs, enums, and traits. Prefer `Result`-returning APIs with `thiserror` or `anyhow` as already used by the workspace. Do not add `unwrap` or `expect` in production paths; convert errors into actionable messages instead.

## Testing Guidelines
Place integration tests under `crates/<crate>/tests/` and focused unit tests beside the implementation. Name tests by observable behavior, not implementation detail. Prefer the narrowest relevant `cargo test -p ...` command while iterating, then run `just test` or `just ci` before handing off larger changes.

## Commit & Pull Request Guidelines
Recent history uses Chinese Conventional Commits with scope, `[AI]` when applicable, and emoji, such as `feat(签到): [AI] ✨ ...` or `chore(版本): [AI] 🔧 ...`. Keep commits atomic by crate or feature boundary. PRs should identify the affected Rust crates, summarize behavior changes, link issues when available, and list the verification commands run.

## Security & Configuration Tips
Never commit local credentials, tokens, home-directory runtime files, or generated secrets. When touching config, auth, sync, or database code, preserve masking, backup, atomic-write, migration, and read-only safety boundaries. Keep destructive cleanup flows dry-run or confirmation-gated unless a test fixture explicitly owns the temporary directory.
