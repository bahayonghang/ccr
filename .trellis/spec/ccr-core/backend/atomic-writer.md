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
- Windows helper: `MoveFileExW(source, target, MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)`.

### 3. Contracts

- Create the parent directory before creating the temporary file.
- Write the full payload to a temp file in the same directory as the target.
- `fsync` (`sync_all`) the temp file content BEFORE the rename, on both sync and async paths. Rename durability alone does not guarantee content durability.
- With `secret(true)`, chmod the temp file to `0o600` BEFORE writing content (Unix; Windows no-op). Never write secret bytes to a file that is not yet owner-only. Rename preserves source permissions, so the final target is `0o600`.
- On Unix-like platforms, keep using atomic rename/persist semantics.
- On Windows, do not delete the target before replacement.
- On Windows, use `MOVEFILE_REPLACE_EXISTING` so the OS replaces the target in one operation.
- Use `MOVEFILE_WRITE_THROUGH` to request flush-through behavior for the rename operation.
- Async Windows writes must call the blocking WinAPI replacement (and fsync) from `spawn_blocking`.
- Failed replacement may remove the temporary file, but must leave the old target content intact.

### 4. Validation & Error Matrix

- Temp file creation fails -> return I/O error; target remains unchanged.
- Temp file write fails -> return I/O error; target remains unchanged.
- Windows replacement returns retryable sharing/permission error -> retry without deleting target.
- Windows replacement exhausts retries -> return I/O error; target remains unchanged.
- Async blocking task fails to join -> return I/O error; target remains unchanged.

### 5. Good/Base/Bad Cases

- Good: overwriting `config.toml` on Windows uses `MoveFileExW` with replace and write-through flags.
- Good: a simulated retry exhaustion leaves the original file content readable.
- Base: writing a new file still creates the parent directory and produces the requested content.
- Bad: `remove_file(target)` before `rename(temp, target)`.
- Bad: async writer doing a delete-then-rename because the sync writer was fixed separately.

### 6. Tests Required

- Unit test for basic sync atomic write.
- Unit test for repeated sync overwrite.
- Unit test for basic async atomic write.
- Unit test for TOML/JSON fileio wrappers.
- Windows-only regression: simulated retry exhaustion keeps the original target file.
- Run `cargo test -p ccr-core` and `cargo clippy -p ccr-core --all-targets --all-features -- -D warnings` after changes.

### 7. Wrong vs Correct

#### Wrong

```rust
if target.exists() {
    let _ = fs::remove_file(target);
}
fs::rename(temp, target)?;
```

#### Correct

```rust
replace_path_windows(temp, target)?;
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
- `FileLock` acquisition semantics (fs4 ≥ 0.12): `try_lock_exclusive()` returns `Ok(true)` = acquired, `Ok(false)` = held elsewhere. Treating `Ok(_)` as acquired silently disables all cross-process locking (regression fixed 2026-07; keep the contention regression test).

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

## Known Debt (out of guarded-write task scope, tracked 2026-07)

- `ccr-cli/src/sync/commands.rs` non-atomic `tokio::fs::write` of config during pull; `ccr-cli/platforms/{gemini,droid}.rs` bare `fs::write` of settings; `ccr-codex`/`ccr-skills` direct `AtomicWriter` / hand-rolled temp+rename call sites — migrate to guarded write incrementally.
- `folder_manager.add_folder` RMW loads outside the lock (pre-existing race, orthogonal to write mutual exclusion).
- `AsyncAtomicWriter` manual temp path gets umask perms on Unix (direct users are all in out-of-scope crates; it now fsyncs).
- Reviewer-noted: `platform_config` untagged backups (prefix `config`) share the keep-10 pool with tagged ones (`config_{tag}`); frequent untagged backups can evict old tagged backups.
