# ccr-codex Backend Guidelines

> Dedicated Codex and OpenCode domain crate.

## Scope

`crates/ccr-codex` owns Codex/OpenCode auth, profiles, session visibility, quota, runtime services, usage records, and history sync. CLI commands and UI backends should call this crate instead of reading `~/.codex` or OpenCode files directly.

Reference files:

- `crates/ccr-codex/src/lib.rs`
- `crates/ccr-codex/src/utils.rs`
- `crates/ccr-codex/src/services/`
- `crates/ccr-codex/src/managers/codex_config.rs`

## Structure

Keep this split:

- `models/` for serialized Codex/OpenCode contracts.
- `managers/` for local config/auth stores.
- `platforms/` for `CodexPlatform`.
- `services/` for auth, runtime, quota, session, usage, and history workflows.
- `utils.rs` for path resolution, private permissions, and small encoding helpers.

## Filesystem And Security

Use `CodexPaths`/`OpenCodePaths` instead of direct home-directory joins. Preserve `CCR_CODEX_DIR`, `CCR_DATA_DIR`, and `CCR_LOCK_DIR` overrides for tests and controlled environments.

Auth files and exported account snapshots are security-sensitive. Preserve masking, private-file permissions, backup-before-destructive-change behavior, and repair/sync flows.

## Error Handling

Return `ccr_core::Result<T>` and map missing auth/config/session state to the existing actionable `CcrError` variants — the variant set is frozen, do not add new ones (see `../../ccr-core/backend/ccr-error-freeze.md`). Avoid panics in runtime discovery and session restore paths; unreadable records should become diagnostics or skipped records with context.

## Logging

Use `tracing` for diagnostics. Never log access tokens, refresh tokens, provider API keys, OAuth payloads, or raw auth JSON.

## Testing

Tests that mutate Codex-related env vars must use `test_support::TestCodexEnv`. Prefer temp homes and fixture files over touching real `~/.codex` state.

## Verification

For Codex/OpenCode domain changes, run:

- `just fmt-check`
- `cargo test -p ccr-codex -- --test-threads=1`
- Relevant `cargo test -p ccr --test commands -- --test-threads=1` when CLI surfaces change
- `just lint-strict`
