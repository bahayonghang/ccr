# Codex Session Recovery

> Contracts for Codex history visibility repair and recoverable session trash.

## Native reviewed trash restoration

- `CodexSessionTrashService::prepare_restores` returns metadata and opaque `CodexSessionRestorePlan` proofs captured from the explicit Codex home. `restore_versioned` accepts that plan, holds the same trash resource lock as native trash, and returns false for a stale home/manifest/payload without restoring it.
- GUI listing uses `list_restore_candidates`, never `prepare_restores`: candidates bind home and manifest bytes but do not open/hash rollout bodies. Check cancellation before enumeration/each entry and during bounded manifest reads; return no list on cancellation. One unreadable/oversized rollout cannot prevent other metadata from being listed.
- Only explicit Review calls `prepare_restore(candidate, cancelled)`. Revalidate the original candidate, read/hash only its selected bounded payload, then recheck the original manifest; a changed manifest must not silently acquire fresh authority. Runtime confirmation holds a distinct private-plan `PreparedRestore` only after the matching generation/decision completes. Cancel/leave drops that read; an admitted restore remains app-owned. Commit reads only the selected manifest and exact versioned payload, without scanning other trash entries.
- Native manifest and rollout reads share the existing 64 MiB session-operation budget. The checked rollout bytes are the bytes written; do not validate a token and subsequently reread an unchecked payload. Legacy `restore_sessions` retains its streaming copy behavior and has no new native payload cap.
- Native absolute-path/reparse checks belong to the reviewed restore branch only. Preserve the existing CLI `--codex-home` relative-path behavior in legacy `restore_sessions`; do not apply native-only validation before choosing the branch.
- Reject linked/reparse ancestors in native home, trash, rollout, index and destination paths. Existing rollout files and index session IDs stop restoration. Native rollout creation and index append use guarded CAS with private permissions. An index conflict preserves the concurrent index and removes only the exact rollout just created by this restore.
- Mtime, index, cache invalidation or trash cleanup failure can occur after a source write. Adapters report partial failure, retain the trash where possible, and never claim rollback. A completed source restore with a failed CCR archive refresh is a distinct partial result.
- Native archive refresh uses the same captured provider registry without environment overrides. Trash list and confirmation state contain metadata and opaque tokens, never transcript plaintext.

## Scenario: sync-history session index repair

### 1. Scope / Trigger
- Trigger: changing `CodexHistorySyncService` or CLI rendering for `ccr codex sync-history`.
- Applies to rollout provider repair, `state_5.sqlite` thread repair, `.codex-global-state.json` sidebar sync, and `session_index.jsonl` reconciliation.

### 2. Signatures
- Service: `CodexHistorySyncService::status() -> Result<CodexHistorySyncStatus>`.
- Service: `CodexHistorySyncService::sync(CodexHistorySyncOptions) -> Result<CodexHistorySyncResult>`.
- Service: `CodexHistorySyncService::restore(path) -> Result<CodexHistoryRestoreResult>`.
- CLI: `ccr codex sync-history [--provider <key> | --bridge official-custom] [--dry-run] [--all-history] [--codex-home <path>]`.
- CLI: `ccr codex sync-history status [--codex-home <path>]`.
- CLI: `ccr codex sync-history restore <backup-dir> [--restore-state] [--codex-home <path>]`.

### 3. Contracts
- `status` reports whether `session_index.jsonl` exists and how many session index entries are missing.
- Missing session index entries are detected from SQLite `threads` rows when available, with rollout-derived candidates as a fallback/additional source.
- Repaired index entries use stable Codex shape: `id`, `thread_name`, and `updated_at`.
- `--dry-run` reports planned index entries but must not write `session_index.jsonl` or create backups.
- A write backup captures prior `session_index.jsonl` state when the sync will mutate it.
- Restore must replace the previous index content or remove the created file when the original file did not exist.
- Rollout provider and SQLite repair policy must remain scoped by the existing provider/bridge/all-history rules.

### 4. Validation & Error Matrix
- Invalid `session_index.jsonl` JSON during repair planning -> return `ConfigError` with line number.
- SQLite database missing -> skip SQLite-only index source and continue with rollout source.
- Write failure after index mutation -> restore the captured index snapshot before returning the error.
- Restore backup without session-index metadata -> leave current index untouched.

### 5. Good/Base/Bad Cases
- Good: add a SQLite-only regression when changing index planning.
- Good: keep rollback snapshots for files mutated after backup creation.
- Base: rollout-only fixtures can still prove index append behavior.
- Bad: deriving all index rows only from rollout files; existing SQLite rows can still be hidden if `session_index.jsonl` is incomplete.
- Bad: creating a separate repair command that duplicates `sync-history` backup and lock behavior.

### 6. Tests Required
- `cargo test -p ccr-codex codex_history_sync_service`
- `cargo test -p ccr-cli codex_sync_history`
- `cargo clippy -p ccr-codex --all-targets --all-features -- -D warnings`
- Assertion points: status counts, dry-run no-write, write append, restore reverts existing index, restore removes newly-created index, rollback restores partial mutations.

### 7. Wrong vs Correct
#### Wrong
```rust
let entries = rollout_candidates.iter().map(build_session_index_entry);
```

#### Correct
```rust
let sqlite_rows = self.read_sqlite_thread_rows()?;
let plan = self.prepare_session_index_repair(&rollout_candidates, sqlite_rows.as_deref(), policy)?;
```

## Scenario: recoverable Codex session trash

### 1. Scope / Trigger
- Trigger: changing `CodexSessionTrashService`, `ccr codex sessions ...`, or future UI calls that delete/restore Codex conversations.
- Applies to the active Codex home passed by `--codex-home` or resolved from Codex config.

### 2. Signatures
- Service: `CodexSessionTrashService::trash_sessions(ids) -> Result<CodexSessionTrashSummary>`.
- Service: `CodexSessionTrashService::list_trashed_sessions() -> Result<Vec<CodexTrashedSessionRecord>>`.
- Service: `CodexSessionTrashService::restore_sessions(ids) -> Result<CodexSessionRestoreSummary>`.
- CLI: `ccr codex sessions trash <session-id>... [--codex-home <path>]`.
- CLI: `ccr codex sessions trash-list [--codex-home <path>]`.
- CLI: `ccr codex sessions restore <session-id>... [--codex-home <path>]`.

### 3. Contracts
- Trash root is CCR-owned under `<codex-home>/backups_state/session-trash`.
- Trash manifest records session id, title, cwd, deletion time, original rollout path, original relative path, original Codex home, saved or reconstructed `session_index.jsonl` entry, and rollout mtime.
- Move-to-trash writes the rollout copy and manifest before deleting the active rollout.
- Move-to-trash removes only matching `session_index.jsonl` rows and restores the index if active rollout deletion fails.
- Restore rejects an existing target rollout path and rejects duplicate `session_index.jsonl` ids.
- Restore copies the rollout back, restores mtime, appends the saved index entry, then deletes the trash entry only after success.
- Restore targets the service's active `codex_home` using the manifest relative path; the manifest original home is preserved for display/audit.

### 4. Validation & Error Matrix
- Empty id list -> `ValidationError`.
- Session id not found in active rollouts -> `ResourceNotFound`.
- Trash manifest has unsafe relative path or path outside `sessions` / `archived_sessions` -> `ValidationError`.
- Restore target rollout exists -> `ValidationError`, trash entry remains.
- Restore target index already contains id -> `ValidationError`, trash entry remains.
- Index append fails after rollout copy -> remove copied rollout and restore original index snapshot.

### 5. Good/Base/Bad Cases
- Good: use `with_codex_home` in tests for isolated fixtures.
- Good: preserve file mtime with `filetime`.
- Base: missing original `session_index.jsonl` is valid; restore should create one from the manifest entry.
- Bad: using the OS trash; CCR needs a manifest-backed format for future UI restore.
- Bad: deleting the active rollout before a trash copy and manifest have been written.

### 6. Tests Required
- `cargo test -p ccr-codex codex_session_trash`
- `cargo test -p ccr-cli codex_sessions`
- `cargo clippy -p ccr-cli --all-targets --all-features -- -D warnings`
- Assertion points: list empty, trash removes active rollout/index row, multi-session trash, restore recreates rollout/index/mtime, rollout conflict keeps trash, index conflict keeps trash.

### 7. Wrong vs Correct
#### Wrong
```rust
fs::remove_file(&active_rollout)?;
fs::copy(&active_rollout, trash_rollout)?;
```

#### Correct
```rust
fs::copy(&snapshot.rollout_path, &trash_rollout)?;
write_manifest(&manifest)?;
remove_session_index_entry(&snapshot.session_id)?;
fs::remove_file(&snapshot.rollout_path)?;
```
