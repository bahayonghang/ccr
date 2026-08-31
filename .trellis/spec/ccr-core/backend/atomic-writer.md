# Atomic Writer & Guarded Write

> Crash-safe file replacement and guarded persistence contracts for `ccr-core`.

---

## Scenario: Windows atomic replacement without deleting the target first

### 1. Scope / Trigger

- Trigger: changing `AtomicWriter`, `AsyncAtomicWriter`, or helpers used by `fileio` atomic TOML/JSON writes.
- Applies to user configuration, auth, registry, and cache files written through `ccr-core::core::atomic_writer`.
- The writer must preserve the old target file when replacement fails.

### 2. Signatures

- `AtomicWriter::write(&self, content: &[u8]) -> Result<()>`
- `AtomicWriter::write_string(&self, content: &str) -> Result<()>`
- `AtomicWriter::secret(self, secret: bool) -> Self` (builder; default `false`)
- `AsyncAtomicWriter::write_async(&self, content: &[u8]) -> Result<()>`
- `AsyncAtomicWriter::write_string_async(&self, content: &str) -> Result<()>`
- `AsyncAtomicWriter::options(self, AsyncAtomicWriterOptions) -> Self`
- `AsyncAtomicWriterOptions { secret: bool, preserve_mode: bool }`
- Windows helper: `MoveFileExW(source, target, MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)`.

### 3. Contracts

- Create the parent directory before creating the temporary file.
- Write the full payload to a temp file in the same directory as the target.
- `fsync` (`sync_all`) the temp file content BEFORE the rename, on both sync and async paths. Rename durability alone does not guarantee content durability.
- With `secret(true)`, set the temp file policy BEFORE writing content. On Unix,
  new or broadly-readable targets become `0o600`, while an existing stricter
  owner-only mode such as `0o400` is preserved.
- On Windows, secret replacement captures the target DACL and applies the same
  ACE set to the temp file before content is written. Compare the parsed DACL,
  not the complete security descriptor: Windows may normalize control flags.
- On Unix-like platforms, keep using atomic rename/persist semantics.
- On Windows, do not delete the target before replacement.
- On Windows, use `MOVEFILE_REPLACE_EXISTING` so the OS replaces the target in one operation.
- Use `MOVEFILE_WRITE_THROUGH` to request flush-through behavior for the rename operation.
- Async Windows writes must call the blocking WinAPI replacement (and fsync) from `spawn_blocking`.
- After Unix rename/persist, open and `sync_all()` the parent directory. Windows
  uses `MOVEFILE_WRITE_THROUGH`; unsupported platforms must log the documented
  durability downgrade rather than claim a directory fsync.
- Failed replacement may remove the temporary file, but must leave the old target content intact.

### 4. Validation & Error Matrix

- Temp file creation fails -> return I/O error; target remains unchanged.
- Temp file write fails -> return I/O error; target remains unchanged.
- Windows replacement returns retryable sharing/permission error -> retry without deleting target.
- Windows replacement exhausts retries -> return I/O error; target remains unchanged.
- Async blocking task fails to join -> return I/O error; target remains unchanged.
- Existing mode/DACL cannot be read or applied -> fail before writing secret
  bytes; target remains unchanged.
- Parent directory fsync fails after rename -> return an I/O error and do not
  report durable success (the new complete file may already be visible).

### 5. Good/Base/Bad Cases

- Good: overwriting `config.toml` on Windows uses `MoveFileExW` with replace and write-through flags.
- Good: a simulated retry exhaustion leaves the original file content readable.
- Good: replacing `0o400` keeps `0o400`; replacing `0o644` credential state
  produces `0o600`.
- Base: writing a new file still creates the parent directory and produces the requested content.
- Bad: `remove_file(target)` before `rename(temp, target)`.
- Bad: async writer doing a delete-then-rename because the sync writer was fixed separately.
- Bad: writing credentials with `async_fs::write` and applying permissions only
  after secret bytes are already on disk.

### 6. Tests Required

- Unit test for basic sync atomic write.
- Unit test for repeated sync overwrite.
- Unit test for basic async atomic write.
- Unit test for TOML/JSON fileio wrappers.
- Windows-only regression: simulated retry exhaustion keeps the original target file.
- Windows filesystem regression: async secret replacement preserves parsed
  DACL ACE bytes.
- Unix regressions (single test thread): umask `000`/`022`/`077` all create
  `0o600`; overwrite `0o400`/`0o600`/`0o644` produces
  `0o400`/`0o600`/`0o600`.
- `python scripts/quality/check_secret_writes.py` rejects direct async writes and
  sensitive writer chains without `.secret(true)`.
- Run `cargo test -p ccr-core` and `cargo clippy -p ccr-core --all-targets --all-features -- -D warnings` after changes.

### 7. Wrong vs Correct

#### Wrong

```rust
async_fs::write(temp, secret_bytes).await?;
async_fs::set_permissions(temp, owner_only).await?; // too late
```

#### Correct

```rust
AsyncAtomicWriter::new(target)
    .secret(true)
    .preserve_mode(true)
    .write_async(secret_bytes)
    .await?;
```

---

## Scenario: Guarded write for durable CCR state (lock → backup rotation → atomic write → permissions)

### 1. Scope / Trigger

- Trigger: adding or changing any code that persists durable CCR state (config, auth, sync, budget/pricing, crypto keys) in `ccr-core`, `ccr-config`, `ccr-sync`, `ccr-store`, or `ccr-checkin`.
- `ccr-core::core::guarded_write` is the single persistence-policy entry point. Do NOT hand-roll `fs::write`/`fs::rename`, ad-hoc backup copies, or per-module lock directories on these paths.
- `fileio::write_toml/write_json` (and async variants) delegate to guarded write with default options, so every fileio caller gets locking + fsync for free.

### 2. Signatures

- `guarded_write::write_guarded(path: &Path, bytes: &[u8], opts: &WriteOptions) -> Result<()>`
- `guarded_write::write_guarded_async(path: &Path, bytes: Vec<u8>, opts: WriteOptions) -> Result<()>` (`spawn_blocking` wrapper)
- `guarded_write::backup_guarded(path: &Path, policy: &BackupPolicy) -> Result<Option<PathBuf>>` (explicit decision-point backup; `Ok(None)` when source missing or policy is `None`)
- `WriteOptions { backup: BackupPolicy, secret: bool, lock_timeout: Duration }` — `Default` = `{None, false, 10s}`
- `BackupPolicy::{None, SameDir { tag: Option<String> }, Dir { dir: PathBuf, prefix: String }}`
- `fileio::write_toml_opts / write_json_opts (+ _async)` — serialization wrappers that forward `WriteOptions`
- `pub(crate) lock_resource_name(path) -> String` — `gw_{sanitized_stem}_{fnv1a64(lowercased absolute path):016x}`

### 3. Contracts

- Internal order per write: derive path lock (`LockManager::with_default_path()`, i.e. `~/.claude/.locks`, `CCR_LOCK_DIR` override) → backup + rotate (`BACKUP_KEEP = 10`, skipped when source missing) → `AtomicWriter::new(path).secret(opts.secret).write(bytes)` → RAII unlock.
- The derived path lock is a LEAF lock: callers never acquire it directly, so lock order is always `{caller RMW locks (ccr_config / platform_profiles_* / CONFIG_LOCK)} → {gw path lock}` — acyclic. guarded write must NEVER touch `CONFIG_LOCK` (std Mutex, non-reentrant; `config_service.lock_config()` already holds it when saving).
- guarded write guarantees write-write mutual exclusion and crash-safe replacement only. Read-modify-write transactionality remains the CALLER's responsibility (keep named RMW locks such as `platform_profiles_{name}`).
- Lock names must be stable across processes: FNV-1a 64 inline hash (std `DefaultHasher` is per-process seeded and forbidden for lock names); `std::path::absolute` (target may not exist yet); lowercase before hashing (NTFS case-insensitive).
- Backup naming is frozen for discoverability (existing `list_backups` filters must keep finding old and new names):
  - `SameDir{tag}` → `{full_filename}.{tag}_{ts}.bak` / `{full_filename}.{ts}.bak` (`%Y%m%d_%H%M%S`), rotation matches `starts_with(full_filename) && ends_with(".bak")`.
  - `Dir{dir, prefix}` → `{prefix}.{ts}.{ext}.bak` (ext falls back to `bak`), rotation matches `starts_with(prefix) && ends_with(".bak")`.
- `secret: true` → owner-only `0o600` set on the temp file before content is written (Unix; Windows no-op). Required for WebDAV credentials (`sync.toml`), checkin crypto keys, and any new API-key/token file.
- `fs4 1.x` exposes `try_lock()` as `Ok(())` = acquired, `TryLockError::WouldBlock` = held elsewhere, and `TryLockError::Error` = real I/O failure. `ccr-core` normalizes that through a local `io::Result<bool>` adapter so the established acquisition loop remains `Ok(true)` = acquired, `Ok(false)` = contended, `Err` = I/O failure. Treating any non-error result as acquired silently disables cross-process locking; keep the contention, release-and-retry, and adapter error regressions.

### 4. Validation & Error Matrix

- Lock not acquired within `lock_timeout` -> `CcrError::LockTimeout`; target unchanged, no backup taken.
- Backup copy fails -> error returned BEFORE any temp write; target unchanged.
- Temp create/write/fsync fails -> I/O error; target unchanged (see atomic replacement scenario above).
- Source missing with backup policy set -> backup skipped, write proceeds.
- `backup_guarded` on missing source -> `Ok(None)`, never an error.

### 5. Good/Base/Bad Cases

- Good: `sync/config.rs` saves `sync.toml` via `fileio::write_toml_opts(.., secret: true)` — locked, fsynced, `0o600`.
- Good: `platforms/base.rs` holds `platform_profiles_{name}` RMW lock, then issues ONE `write_guarded` with `Dir` backup policy.
- Base: plain `fileio::write_toml` — locked + fsynced atomic write, no backup, not secret.
- Bad: hand-rolled `fs::write` / temp+`fs::rename` for durable state (torn files, no lock, umask perms).
- Bad: module-private lock directory (e.g. `<config_dir>/.locks`) — lock split-brain; two lock dirs never exclude each other.
- Bad: chmod to `0o600` AFTER writing secret content (world-readable window).

### 6. Tests Required

- Backup naming byte-format + keep-10 rotation for `SameDir` and `Dir` (assert oldest deleted).
- Lock contention: pre-hold the derived lock, `write_guarded(lock_timeout=100ms)` returns `LockTimeout`.
- Cross-process lock regression: a child test process holds the file lock, the parent observes `LockTimeout`, then acquires it after the child exits; two handles in one process are not sufficient evidence.
- Multi-thread stress: final file content equals one complete payload (no tearing).
- `#[cfg(unix)]`: `secret: true` target mode `& 0o777 == 0o600` (Windows: skip with comment).
- Crash-safety proxy: parent-is-file → error, old sibling content intact.
- Isolate lock dirs in tests via `CCR_LOCK_DIR` + `TestLockDirEnv` fixture; run with `--test-threads=1`.

### 7. Wrong vs Correct

#### Wrong

```rust
// 手搓持久化：无锁、无备份、非原子、权限靠 umask
let content = toml::to_string_pretty(&config)?;
fs::create_dir_all(parent)?;
fs::write(&self.config_path, content)?;
```

#### Correct

```rust
// 单一入口：锁 → 备份轮换 → temp+fsync+rename → 按需 0o600
ccr_core::core::fileio::write_toml_opts(
    &self.config_path,
    &config,
    &WriteOptions { secret: true, ..Default::default() },
)?;
```

---

## Scenario: Content-versioned guarded write (lock-held CAS)

### 1. Scope / Trigger

- Trigger: a caller reads editable file content, later writes the edited content, and must reject intervening external changes.
- Applies to raw config/prompt/profile editors, Tauri Claude MCP state
  mutations, and any future compare-and-swap persistence built on
  `guarded_write`.

### 2. Signatures

- `content_version_token(bytes: &[u8]) -> String` returns lowercase BLAKE3 hex.
- `write_guarded_versioned(path, bytes, expected_token, opts) -> Result<VersionedWriteOutcome>`.
- `write_guarded_versioned_async(path, bytes, expected_token, opts) -> Result<VersionedWriteOutcome>`.
- `VersionedWriteOutcome::{Written, Conflict}`; conflict is not a `CcrError` variant.

### 3. Contracts

- One path lock covers current-byte read, token comparison, backup, and atomic replacement.
- `expected_token == ""` means the caller expects the target not to exist. An existing empty file has the normal BLAKE3 token for empty bytes.
- A matching token delegates to the same lock-held backup and atomic-write body as `write_guarded`.
- A mismatch returns `Ok(Conflict)` before backup or temp-file creation.
- I/O, lock timeout, and async join failures remain errors. Do not extend the frozen `CcrError` enum for expected conflicts.
- `ccr-cli::SettingsManager::update_atomic` is the managed Claude settings consumer: it reads bytes and a token, applies a deterministic mutation, and retries the complete read/mutate/CAS cycle at most three times.
- `ccr-ui::commands::claude_mcp_config::update_root_for_scope` applies the same
  at-most-three-attempt read/mutate/CAS loop to Claude MCP user/local state and
  project `.mcp.json`. Each replay relocates the current project key and
  changes only the requested MCP subtree on the latest full JSON object.
- A replayable mutation must not perform external side effects. Prepare external data before entering the closure and clone only the prepared values during replay.
- Managed Claude settings writes use `secret: true` plus `BackupPolicy::Dir { prefix: "settings" }`. They must not combine CAS with the legacy fixed `claude_settings` lock or create same-directory backups.
- Claude MCP user/local state writes use `secret: true` and
  `BackupPolicy::None`; project `.mcp.json` uses `secret: false` and
  `BackupPolicy::None`. State backups are forbidden because unrelated fields
  can contain credentials such as `primaryApiKey`.
- The path lock coordinates CCR writers only. A non-cooperating Claude Code
  process can still change the file between CCR's lock-held comparison and
  replacement, or overwrite it after replacement. CAS narrows the overwrite
  window and exposes observed conflicts; it does not provide a cross-process
  transaction or a guarantee that external updates cannot be lost.
- Full replacement remains a distinct recovery operation; ordinary production read-modify-write call sites must not fall back to `load` followed by an unconditional replace.

### 4. Validation & Error Matrix

- Target missing + empty expected token -> `Written`.
- Target exists + matching BLAKE3 token -> backup according to policy, then `Written`.
- Target state differs from expected token -> `Conflict`; target and backup set remain unchanged.
- A replaying caller observes three consecutive conflicts -> return an
  actionable retry error; never convert the last attempted mutation into an
  unconditional write or report success.
- Lock timeout / read / backup / replacement failure -> existing `CcrError`; never map to `Conflict`.

### 5. Good/Base/Bad Cases

- Good: a raw editor returns the token from its read command and passes it unchanged on save.
- Good: a Claude MCP mutation rereads the complete JSON object after conflict,
  preserves `oauthAccount`, `primaryApiKey`, unknown fields, and unrelated
  projects, then replays only its target subtree.
- Base: first creation uses an empty token and succeeds only while the file remains absent.
- Bad: compare bytes before acquiring the guarded-write path lock, then call `write_guarded` separately.
- Bad: treat conflict as an I/O error or add `CcrError::WriteConflict`.
- Bad: claim that CCR's path lock prevents a Claude Code process from writing
  concurrently when that process does not participate in the lock protocol.

### 6. Tests Required

- Stable/content-sensitive token test.
- Matching write produces `Written`, requested backup, and exact new bytes.
- Stale token produces `Conflict`, preserves external bytes, and creates no backup.
- Empty-token first creation succeeds; a second empty-token write conflicts.
- Four concurrent writers using one token yield exactly one `Written` and three `Conflict` outcomes.
- `SettingsManager` conflict injection preserves both writers' independent fields and unknown user-owned JSON; exhausting three conflicts returns an actionable retry error.
- Claude MCP tests cover unknown-field round trips, one-conflict replay,
  three-conflict failure, two concurrent deterministic mutations, and Unix
  owner-only permissions without same-directory backups.

### 7. Wrong vs Correct

#### Wrong

```rust
if content_version_token(&fs::read(path)?) != expected_token {
    return Err(CcrError::ValidationError("conflict".into()));
}
write_guarded(path, bytes, opts)?;
```

#### Correct

```rust
match write_guarded_versioned(path, bytes, expected_token, opts)? {
    VersionedWriteOutcome::Written => save_result(),
    VersionedWriteOutcome::Conflict => conflict_result(),
}
```

---

## Known Debt (out of guarded-write task scope, tracked 2026-07)

- `ccr-cli/src/sync/commands.rs` non-atomic `tokio::fs::write` of config during pull; `ccr-cli/platforms/{gemini,droid}.rs` bare `fs::write` of settings; `ccr-codex`/`ccr-skills` direct `AtomicWriter` / hand-rolled temp+rename call sites — migrate to guarded write incrementally.
- `folder_manager.add_folder` RMW loads outside the lock (pre-existing race, orthogonal to write mutual exclusion).
- `AsyncAtomicWriter` manual temp path gets umask perms on Unix (direct users are all in out-of-scope crates; it now fsyncs).
- Reviewer-noted: `platform_config` untagged backups (prefix `config`) share the keep-10 pool with tagged ones (`config_{tag}`); frequent untagged backups can evict old tagged backups.
