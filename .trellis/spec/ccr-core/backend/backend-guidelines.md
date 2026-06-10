# ccr-core Backend Guidelines

> Shared infrastructure primitives used by the Rust workspace.

## Scope

`crates/ccr-core` owns infrastructure that many crates share: `CcrError`, logging setup, file locking, file I/O helpers, atomic writes, masking helpers, and small validation traits. Keep domain behavior out of this crate unless it is truly cross-cutting.

Reference files:

- `crates/ccr-core/src/lib.rs`
- `crates/ccr-core/src/core/error.rs`
- `crates/ccr-core/src/core/atomic_writer.rs`
- `crates/ccr-core/src/utils/mask.rs`

## File And Config Writes

Use `AtomicWriter`/`AsyncAtomicWriter` or existing `fileio` helpers for config/runtime file replacement. The local pattern is same-directory temp file plus replacement; Windows replacement has retry logic for sharing violations.

Do not hand-roll `fs::write` for durable CCR config/auth/history state unless a nearby helper already owns the same semantics. Preserve backup, masking, locking, and atomic-write behavior when changing config flows.

## Error Handling

Use `CcrError` for shared application errors and keep variants actionable for CLI users. Add a new variant only when callers need to distinguish a domain of failure or a stable exit code.

Recover from poisoned test/runtime locks with `unwrap_or_else(|poisoned| poisoned.into_inner())` where the existing fixture pattern does so. Do not introduce `unwrap`/`expect` in production paths.

## Logging

Use `tracing` in infrastructure helpers. Logging setup is centralized in `init_logger()` and `init_file_only_logger()`. Respect `CCR_LOG_LEVEL` and avoid printing directly from shared primitives.

Internal implementation comments may be Chinese; public API docs should remain English.

## Testing

Process environment mutations must be serialized with the crate-local fixture lock pattern in `test_support::TestLogEnv`. Hold the guard until Drop restores every changed env var.

Reference:

- `crates/ccr-core/src/lib.rs`

## Verification

For `ccr-core` changes, run:

- `just fmt-check`
- `cargo test -p ccr-core -- --test-threads=1`
- `just lint-strict`

Escalate to `just test` when public helpers or error contracts are reused by multiple crates.
