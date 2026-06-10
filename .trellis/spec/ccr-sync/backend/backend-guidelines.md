# ccr-sync Backend Guidelines

> WebDAV sync domain and sync folder registry.

## Scope

`crates/ccr-sync` owns WebDAV sync configuration, folder selection, remote push/pull, and sync path expansion. CLI command wrappers should call this crate instead of implementing WebDAV behavior directly.

Reference files:

- `crates/ccr-sync/src/lib.rs`
- `crates/ccr-sync/src/sync/service.rs`
- `crates/ccr-sync/src/sync/folder_manager.rs`
- `crates/ccr-sync/src/sync/content_selector.rs`

## Structure

The crate intentionally exposes the `sync/` module as its public surface. Keep new sync behavior near the existing domain:

- `config.rs` for WebDAV config file handling.
- `folder_manager.rs` for configured sync folder metadata.
- `content_selector.rs` for allowed sync content.
- `service.rs` for network transfer.

## Error Handling

Return `ccr_core::Result<T>` and map WebDAV/network/path failures to `CcrError::SyncError`. Preserve context in messages, including the local or remote path that failed.

Do not treat every WebDAV error as "not found"; `service.rs` already maps `reqwest_dav` status and decode errors more specifically.

## File And Network Boundaries

Use `tokio::fs` for async transfer paths. Honor `allowed_paths` filters and `should_exclude_from_sync` when uploading directories; do not broaden sync scope by default.

WebDAV uses `reqwest_dav` with `native-tls`. Do not add another HTTP client stack for this crate without a dependency-governance review.

## Logging

Use `tracing::debug!` for detailed path decisions and `tracing::info!` for user-meaningful sync milestones. Never log WebDAV passwords or full secret-bearing URLs.

## Testing

Tests that mutate sync env vars must use `test_support::TestSyncEnv`. For path matching and selector changes, add focused unit tests before broad integration tests.

## Verification

For sync changes, run:

- `just fmt-check`
- `cargo test -p ccr-sync -- --test-threads=1`
- `just lint-strict`
