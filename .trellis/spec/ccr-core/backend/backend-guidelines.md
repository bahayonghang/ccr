# ccr-core Backend Guidelines

> Shared infrastructure primitives used by the Rust workspace.

## Scope

`crates/ccr-core` owns infrastructure that many crates share: `CcrError`, logging setup, file locking, file I/O helpers, atomic writes, the `Secret` credential newtype and masking helpers, and small validation traits. Keep domain behavior out of this crate unless it is truly cross-cutting.

Reference files:

- `crates/ccr-core/src/lib.rs`
- `crates/ccr-core/src/core/error.rs`
- `crates/ccr-core/src/core/atomic_writer.rs`
- `crates/ccr-core/src/core/secret.rs`
- `crates/ccr-core/src/utils/mask.rs`

## Secrets And Masking

The masking _algorithm_ has exactly one implementation: `ccr_core::utils::mask::mask_sensitive` (frozen public re-export via the `ccr` facade). `ColorOutput::mask_sensitive` and `Secret`'s Debug/Display/default-Serialize are delegating callers, not second implementations. Do not add another masking rule anywhere in the workspace — the previous four divergent copies (`mask_api_key`, `mask_cookies_json`/`mask_value`, ccr-ui `mask_token`) were deleted in 07-03-arch-secret-newtype.

Credential fields in persistent or IPC-visible structs must be typed `ccr_core::Secret` (or `Option<Secret>`), never bare `String`:

- `Debug`/`Display`/default `Serialize` are always masked; `Deserialize` accepts plaintext transparently, so existing files load losslessly.
- `expose()` is the only plaintext read channel. Legitimate consumption points: env injection, HTTP auth headers, settings/auth.json writes, encryption input, explicit plaintext export, edit-form backfill. Never let an `expose()` result flow into `format!`/`tracing`/error strings.
- Plaintext persistence is opt-in per field via `#[serde(serialize_with = "ccr_core::expose_plaintext")]` (or `expose_plaintext_option`). `rg 'expose_plaintext'` therefore lists every plaintext-on-disk field — use it as the review entry point when auditing credential flows.

```rust
// ❌ Wrong: bare String credential — Debug/logs/default serde leak plaintext,
//    and masking depends on every caller remembering to call mask_sensitive.
pub struct MyConfig {
    pub api_key: Option<String>,
}

// ✅ Correct: Secret field + explicit plaintext opt-in for the disk format.
pub struct MyConfig {
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "ccr_core::expose_plaintext_option"
    )]
    pub api_key: Option<ccr_core::Secret>,
}
```

Failure-mode tradeoff (deliberate): a persistence struct that _forgets_ the `expose_plaintext` annotation writes masked text to disk — loud, caught by round-trip tests and first save. That replaces the old silent failure where a display path forgot to mask and leaked plaintext. Keep per-file round-trip tests (`legacy plaintext → load → save → reload lossless`) when migrating or adding credential fields.

Decisions recorded in 07-03-arch-secret-newtype (do not re-propose without new requirements):

- Sync WebDAV passwords stay plaintext-on-disk with 0o600 files (`sync.toml`, `sync_folders.toml` via guarded write `secret: true`); AES-GCM-encrypting them adds no protection while the key sits on the same disk with the same permissions. A real upgrade would be the OS keychain.
- No zeroize-on-drop: tokens legitimately live in `String` copies (env construction, HTTP headers, serde buffers); newtype-only zeroize would be security theater without a full-chain audit.
- `Secret` is not generic (`Secret<T>`): masking needs a `&str` view and no non-String credential consumer exists.
- Known plaintext IPC responses (claude/codex profile detail, checkin cookies backfill) intentionally call `expose()` for edit-form prefill and are comment-marked; converting them to a `has_password` pattern belongs to 07-03-arch-typed-ipc.

Remaining bare-String whitelist (input boundaries only, wrap into `Secret` on first assignment into a model): clap CLI arg structs, `ClaudeSettings`-style env maps (07-03-arch-claude-settings scope), ciphertext fields (`cookies_json_encrypted`), and the codex profile secret store internals. Known same-class debt, out of scope so far: WAF session cookies stored plaintext in `checkin_waf_cookies` (SQLite), and the SSH environment password cache (`ccr-ui/src-tauri/src/ssh/connection.rs`).

## File And Config Writes

Use `AtomicWriter`/`AsyncAtomicWriter` or existing `fileio` helpers for config/runtime file replacement. The local pattern is same-directory temp file plus replacement; Windows replacement has retry logic for sharing violations.

Do not hand-roll `fs::write` for durable CCR config/auth/history state unless a nearby helper already owns the same semantics. Preserve backup, masking, locking, and atomic-write behavior when changing config flows.

## Error Handling

Use the existing `CcrError` variants for shared application errors and keep messages actionable for CLI users. The variant set is frozen (25 variants) — do not add domain variants; new domain errors live in the owning crate as self-owned types, and primitive-tier additions need case review plus an intentional guard-test update. See [CcrError Freeze](./ccr-error-freeze.md).

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
