# WebDAV Hardening Contracts

## Scenario: Bounded transactional WebDAV sync

### 1. Scope / Trigger

- Trigger: changing WebDAV href parsing, push/pull transfer, transport policy,
  pull replacement, or sensitive asset encryption in `crates/ccr-sync`.
- This contract prevents remote path escape, unbounded downloads, partial local
  replacement, insecure transport, and plaintext sensitive assets.

### 2. Signatures

- `SyncService::new(config: &SyncConfig) -> Result<SyncService>`
- `SyncService::new_with_limits(config: &SyncConfig, limits: SyncLimits) -> Result<SyncService>`
- `SyncService::pull_with_backup(local_path: &Path) -> Result<Option<PathBuf>>`
- `SyncService::push_sensitive(local_path, asset_id, passphrase) -> Result<()>`
- `SyncService::pull_sensitive_with_backup(local_path, asset_id, passphrase, allow_plaintext_v1) -> Result<Option<PathBuf>>`
- `validate_webdav_url(raw, allow_insecure_loopback) -> Result<Url>`
- Development override: `CCR_ALLOW_INSECURE_WEBDAV_HTTP=1|true`

### 3. Contracts

- Decode each href segment exactly once. Reject empty segments, `.`, `..`,
  control bytes, `/`, `\\`, drive/UNC forms, and names over the component limit.
- A remote entry name must be exactly one `Component::Normal`. Check lexical
  containment before creation and canonical parent containment when available.
- Track normalized href identities and reject duplicates/cycles.
- Pull writes only into a unique sibling staging path. Commit order is staged
  fsync, `active -> backup`, `staging -> active`, parent fsync. Any commit error
  restores the pre-operation active path; uncommitted staging is removed.
- Stream GET and PROPFIND bodies while enforcing file bytes, total bytes,
  depth, entries, component bytes, and operation deadline. Do not call
  `Response::bytes()` in pull paths.
- Require HTTPS. HTTP is allowed only for a loopback host when the explicit
  development override is enabled. Redirects are limited and revalidated.
- Sensitive bytes use the v2 AES-256-GCM envelope with Argon2id, random salt
  and nonce, and authenticated `asset_id` plus relative path metadata. V2 reads
  accept only the fixed, bounded v2 KDF parameters.
- Plaintext v1 reads require `allow_plaintext_v1=true`. New sensitive writes
  are always v2. Passphrases and derived keys are never logged or persisted;
  derived key buffers are cleared on success and error paths.

### 4. Validation & Error Matrix

- Hostile/ambiguous href -> `sync_path_*`; no active bytes change.
- Duplicate or cyclic href -> `sync_path_cycle`; no active bytes change.
- File/total/depth/entry/deadline limit -> `sync_limit_*`; staging is removed.
- Staging fsync, backup/install rename, parent fsync, or restore fault ->
  `sync_transaction_*`; old active bytes remain readable.
- Non-loopback HTTP or unsafe redirect -> `sync_transport_*`.
- Wrong passphrase, tampered metadata, hostile KDF params, or unsupported
  envelope -> `sync_envelope_*`.
- Plaintext without explicit migration ->
  `sync_envelope_plaintext_v1_requires_migration`.

### 5. Good/Base/Bad Cases

- Good: `/ccr/a%20b.toml` decodes once to `a b.toml` and stays in staging.
- Good: a truncated GET fails and leaves the original active file untouched.
- Base: a non-sensitive file still uses the same bounded pull transaction.
- Bad: normalize `../` and then join it to the local directory.
- Bad: buffer a complete remote body before applying byte limits.
- Bad: trust envelope-provided Argon2 parameters without a v2 bound.

### 6. Tests Required

- Unit corpus for encoded traversal, separators, drive/UNC, empty segments,
  component length, containment, and duplicate href identity.
- Fake DAV tests for list, GET, truncated stream, oversized body, mkdir, write,
  and file fsync failures; assert exact old active bytes after every error.
- Transaction failpoints for staged fsync, both renames, parent fsync, and
  backup restore retry; assert exact old active bytes.
- Envelope round-trip, random salt/nonce, authenticated metadata, plaintext v1
  opt-in, hostile KDF parameter, and fake PUT no-plaintext assertions.
- Run `cargo test -p ccr-sync -- --test-threads=1` and
  `cargo clippy -p ccr-sync --all-targets --all-features -- -D warnings -D clippy::unwrap_used`.

### 7. Wrong vs Correct

#### Wrong

```rust
let bytes = response.bytes().await?;
tokio::fs::write(local_dir.join(extract_filename(href)), bytes).await?;
```

#### Correct

```rust
let name = RemoteEntryName::from_href_with_limit(href, max_component_bytes)?;
let destination = name.join_contained(staging_dir)?;
// Stream chunks through SyncBudget, then commit PullTransaction.
```
