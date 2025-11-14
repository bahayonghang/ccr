<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# Repository Guidelines

## Project Structure & Module Organization
CCR is a Cargo workspace: CLI and shared library code live in `src/`, integration suites in `tests/`, and shared manifests in the root `Cargo.toml`. Reference docs stay in `docs/`, runnable walkthroughs in `examples/`, and protocol specs under `openspec/`. `ccr-ui/backend/` hosts the Axum API server, `ccr-ui/frontend/` contains the Vue 3 + TypeScript client, and automation scripts sit in `scripts/`.

## Build, Test, and Development Commands
- `cargo build --workspace` or `just build` compiles every crate in debug mode.
- `just release` performs the optimized `cargo build --release` path for distribution.
- `just dev` chains `cargo check` with the default test suite.
- `just run -- <args>` executes the CLI locally; append `--help` for UX copy reviews.
- Inside `ccr-ui/`, run `just quick-start` once to install dependencies and `just s` to serve the Vue dev client plus Axum backend.

## Coding Style & Naming Conventions
Use Rust 2024 defaults: four-space indentation, snake_case functions, PascalCase types, and SCREAMING_SNAKE_CASE constants. Run `cargo fmt` and `cargo clippy -- -D warnings` (or `just lint`) before submitting, and keep modules single-responsibility to preserve SOLID boundaries. The frontend follows Vue Composition API practices with ESLint + Prettier triggered by `just c`/`just f`, PascalCase component exports, and kebab-case filenames.

## Testing Guidelines
`just test` enforces the expected order: it runs `cargo test --test platform_integration_tests -- --test-threads=1` first because it mutates global config paths, then executes library tests plus `integration_test` and `manager_tests`. Keep fast unit tests beside their modules in `src/`, and place scenario or CLI workflows inside `tests/` with descriptive names such as `sync_workflow.rs`. Maintain the ~95% coverage level in the README and pair each feature with success and failure assertions.

## Commit & Pull Request Guidelines
Follow the Conventional Commit style already used (`feat(web): …`, `refactor(tests): …`) with summaries under 72 characters and meaningful scopes. Each commit must pass `just lint` and `just test` and include any doc or schema updates touched by the change. Pull requests should describe motivation, link related issues, note reproduction steps for fixes, and attach CLI output or UI captures when behavior changes; surface migrations touching `~/.ccr/` so reviewers can back up configs.

## Security & Configuration Tips
Never commit contents from `~/.claude/` or `~/.ccr/`; rely on `ccr temp-token` for throwaway credentials during testing. Use disposable WebDAV endpoints and scrub hostnames or secrets before sharing sync logs.
