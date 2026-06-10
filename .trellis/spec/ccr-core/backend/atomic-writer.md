# Atomic Writer

> Crash-safe file replacement contracts for `ccr-core`.

---

## Scenario: Windows atomic replacement without deleting the target first

### 1. Scope / Trigger
- Trigger: changing `AtomicWriter`, `AsyncAtomicWriter`, or helpers used by `fileio` atomic TOML/JSON writes.
- Applies to user configuration, auth, registry, and cache files written through `ccr-core::core::atomic_writer`.
- The writer must preserve the old target file when replacement fails.

### 2. Signatures
- `AtomicWriter::write(&self, content: &[u8]) -> Result<()>`
- `AtomicWriter::write_string(&self, content: &str) -> Result<()>`
- `AsyncAtomicWriter::write_async(&self, content: &[u8]) -> Result<()>`
- `AsyncAtomicWriter::write_string_async(&self, content: &str) -> Result<()>`
- Windows helper: `MoveFileExW(source, target, MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)`.

### 3. Contracts
- Create the parent directory before creating the temporary file.
- Write the full payload to a temp file in the same directory as the target.
- On Unix-like platforms, keep using atomic rename/persist semantics.
- On Windows, do not delete the target before replacement.
- On Windows, use `MOVEFILE_REPLACE_EXISTING` so the OS replaces the target in one operation.
- Use `MOVEFILE_WRITE_THROUGH` to request flush-through behavior for the rename operation.
- Async Windows writes must call the blocking WinAPI replacement from `spawn_blocking`.
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
