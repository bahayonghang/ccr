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

Use `CodexPaths` instead of direct Codex home-directory joins. The retired OpenCode Auth `OpenCodePaths` helper has been removed; this does not remove other OpenCode configuration or usage consumers. Preserve `CCR_CODEX_DIR`, `CCR_DATA_DIR`, and `CCR_LOCK_DIR` overrides for tests and controlled environments.

Auth files and exported account snapshots are security-sensitive. Preserve masking, private-file permissions, backup-before-destructive-change behavior, and repair/sync flows.

Pending OAuth login credentials are an explicit no-backup domain.
`services::CodexOAuthPendingStore` owns `oauth_pending.json` under
`CodexPaths::ccr_codex_dir`. CLI/desktop consumers use `save`, `load(now)`, and
`clear`; the desktop owns its listener and events. The existing desktop
adapter passes its published `PlatformPaths` / `CCR_ROOT` path to `with_path`
so a distinct `CCR_DATA_DIR` does not silently move pending credentials.
No credential copies or path migration are part of this change. A store operation lock
serializes save with cancellation and expiry cleanup; the guarded path lock
remains a leaf lock. Saves explicitly use `secret: true` and
`BackupPolicy::None`. Never copy verifier/state to a backup or history file.
Creation/replacement permissions are established by the existing secret writer
before payload bytes; read, permission, write, and cleanup errors propagate.
Callers publish an in-memory state only after persistence succeeds.

`CodexOAuthPendingState` uses `Secret` for verifier/state and URLs that contain
credentials. Default Debug/serialization is masked. Only the private disk DTO
opts into plaintext serialization; the login-start response explicitly exposes
the browser authorization URL. Parse errors report line/column without quoting
input values. Preserve the existing camelCase disk format.

Pending-store tests cover create/replace/cancel/expiry, permission denial,
no-copy filesystem scans, legacy plaintext load, and redacted errors. Run
native Windows ACL/cleanup-denial tests and Unix mode tests on their respective
platforms; source checks cannot replace native permission checks.

The desktop OAuth controller is `commands/codex_auth/oauth.rs`. It owns the
already-bound listener, pending state, cancel token, one completion request,
and terminal cleanup. Bind and secret persistence must succeed before an ID
or authorization URL is published. Startup restore follows the same path.
An active ID can be reused without binding or saving again.

One monotonic deadline and cancellation token cover accept, accepted socket
reads/writes, the token request, and the complete HTTP body. A callback must
match the current ID, state, loopback port, and callback path; duplicate or
expired callbacks cannot replace the accepted callback. A restored callback
is validated before listener publication. Account commit is completion-aware:
the controller cannot detach a secret writer through cancellation. A cancel
request during commit returns `oauth_commit_in_progress`; admission stays held
until the commit and cleanup finish. Pending
cleanup failure is visible and blocks a new login until cleanup succeeds.
Cleanup retry first verifies the same owner still occupies the slot and claims
one retry. Disk cleanup runs without holding the slot or login-state mutex;
a concurrent retry returns `oauth_cleanup_in_progress`. A stale retry must not
clear a replacement login's pending record. Restore re-reads persisted state
after admission and rejects a record cancelled during queueing with
`oauth_saved_login_missing`; restore never recreates a captured old snapshot.

OAuth token errors report status or a fixed diagnostic, never response bodies
or credential values. Existing callback-received and timeout events retain
their payload shape. A callback-received event does not claim token exchange
or account commit succeeded. Port release cancels the controller's own
listener and proven ProcessGateway owners only; unrelated PIDs remain
report-only. Tests use synthetic accounts and loopback servers, never real
OpenAI endpoints or user credentials.

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
