# ccr Backend Guidelines

> Root CLI/library package and compatibility facade.

## Scope

`crates/ccr` is the installable `ccr` binary and the historical public library facade. Keep it thin:

- `src/main.rs` parses the Clap command and delegates to `ccr::cli::CommandDispatcher::dispatch`.
- `src/lib.rs` re-exports domain crates and maintains the compatibility prelude.
- CLI behavior should live in `ccr-cli`; shared infrastructure in `ccr-core`; config contracts in `ccr-config`; session/database logic in `ccr-store` or `ccr-db`.

Reference files:

- `crates/ccr/src/main.rs`
- `crates/ccr/src/lib.rs`
- `crates/ccr/tests/public_api_compat.rs`

## Public API Boundary

Prefer `crate::prelude` for new integration examples and public docs. In the
7.x line, the broad root module paths in `src/lib.rs` remain compatibility-only
and are deprecated with an actionable narrow-crate replacement. Keep them
callable until 8.0.0 at the earliest; removal still requires a separately
reviewed breaking-change inventory.

When adding a new cross-crate type that must be public, first ask whether it belongs in the domain crate (`ccr-types`, `ccr-config`, `ccr-codex`, etc.) and only then re-export it from `ccr`.

## Error Handling

The binary catches errors at the dispatcher boundary and routes them through `ccr::cli::dispatch::handle_error`. Do not add panics or process exits inside root glue code. Production paths should return `ccr_core::CcrError`/`Result` through the lower-level crate boundary.

The only expected `unwrap_or_else(|err| err.exit())` pattern is Clap parse handling in `src/main.rs`, where Clap owns the process exit.

## Persistence And Database Boundaries

Do not add direct SQLite, file mutation, or config persistence logic to `crates/ccr`. Use:

- `ccr-config` for platform/profile TOML config.
- `ccr-store` for CLI-side session/history persistence.
- `ccr-db` for desktop/check-in/usage archive SQLite storage.
- `ccr-core::AtomicWriter` and domain managers for file writes.

## Logging

Initialize logging once in `src/main.rs`:

- TUI mode uses `init_file_only_logger()` so logs do not corrupt the terminal UI.
- Non-TUI mode uses `init_logger()`.

New runtime logs should normally be emitted in the crate that owns the behavior, not in the root facade.

## Quality And Verification

Root changes can affect downstream users even when behavior looks unchanged. For `crates/ccr` changes, run at least:

- `just fmt-check`
- `just lint-strict`
- `cargo test -p ccr -- --test-threads=1`

Run `just test` or `just ci` for cross-crate re-export, feature, or release changes.
