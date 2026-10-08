# Clean Storage

## Scenario: `ccr clean storage`

### 1. Scope / Trigger

- Trigger: adding or changing `ccr clean storage`, its help text, or the paths it deletes.
- The command removes rebuildable caches and retired database copies under the CCR root.
- It does not change `ccr clean planfiles`, `ccr clean backups`, or the bare `ccr clean` menu.

### 2. Signatures

```text
ccr clean storage [--dry-run] [--force]
ccr -y clean storage
```

- Handler: `ccr_cli::commands::clean_storage_command(dry_run: bool, force: bool) -> Result<()>`
- Dispatch passes `auto_yes || storage_args.force` as `force`.
- Root order: `CCR_DATA_DIR`, then `CCR_ROOT`, then `~/.ccr`.

### 3. Contracts

Delete only these non-symlink paths:

| Kind | Path |
| --- | --- |
| Cargo cache | `<root>/ccr-ui/backend/target` directory |
| Frontend dependencies | `<root>/ccr-ui/frontend/node_modules` directory |
| Migration snapshot | regular file directly under `<root>/analytics/` named `usage.db.pre-migration-*.bak` |
| Retired usage home | `<root>/llmusage` directory |

Keep `analytics/usage.db`, `usage.db-wal`, `usage.db-shm`, `data.db`, configuration, platform credentials, check-in data, `skills/`, `locks/`, `history/`, `backups/`, `logs/`, `~/.llmusage`, and `~/.ccr-ui/`.

`LLMUSAGE_HOME` protects `<root>/llmusage` only when the two paths are the same directory. Compare with `canonicalize` when both exist. Otherwise compare the absolute path after removing `.` and `..`. An empty `LLMUSAGE_HOME` does not protect the directory.

Missing candidates are omitted. A symlink candidate is reported and not followed. Directory byte totals count regular files only and do not enter symlink directories.

`--dry-run` prints paths and bytes, then returns success without deleting. A non-empty plan asks for confirmation unless `--force`, global `--yes`, or `settings.skip_confirmation` is set. Missing config still asks. A corrupt config returns before any delete. Cancel returns success and deletes nothing. An empty plan returns success.

One candidate's delete error is recorded. Later candidates still run. Any recorded delete error makes the command return `CcrError::FileIoError`.

### 4. Validation & Error Matrix

- No candidates -> success, message `没有可清理的存储项`.
- User answers no -> success, message `已取消清理操作`, files unchanged.
- `LLMUSAGE_HOME` matches `<root>/llmusage` -> that directory stays, other candidates may delete, success.
- Corrupt config -> `CONFIG_FORMAT_INVALID`, nothing deleted.
- One `remove_dir_all` or `remove_file` error -> other candidates continue, then `FileIoError`.
- Candidate path outside the resolved root -> `ValidationError` before that path is added.

### 5. Good/Base/Bad Cases

- Good: `--dry-run` lists the four kinds and leaves the temp root unchanged.
- Good: `--force` deletes only those kinds and leaves `usage.db`, `data.db`, and `platforms/` in place.
- Base: bare `ccr clean` still shows `planfiles` and `backups` only.
- Bad: deleting `analytics/usage.db` or scanning every `target` and `node_modules` directory under the root.

### 6. Tests Required

- `cargo test -p ccr-cli --lib commands::lifecycle::clean_storage -- --test-threads=1` covers root order, name matching, the four-kind plan, `LLMUSAGE_HOME`, and the failure aggregator.
- `cargo test -p ccr --test commands -- clean -- --test-threads=1` covers dry-run, force, cancel, keepers, `CCR_DATA_DIR`, corrupt config, skip-confirmation, the unchanged menu, and the existing planfiles and backups tests.
- The symlink integration test is `#[cfg(unix)]`.

### 7. Wrong vs Correct

#### Wrong

```rust
fs::remove_dir_all(root.join("ccr-ui"))?;
fs::remove_file(root.join("analytics/usage.db"))?;
```

#### Correct

```rust
// Plan only the four named kinds, skip symlinks, then delete each candidate independently.
let plan = plan_storage_cleanup(&root, llmusage_home.as_deref())?;
let result = delete_storage_candidates(&plan.candidates);
storage_result_status(&result)?;
```
