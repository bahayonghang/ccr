# Synchronous Scoped Lock Directory

## 1. Scope / Trigger

Applies to explicit-path profile lifecycles and nested repository/guarded writers.
Injecting a catalog path alone does not isolate its default lock directory.
`ccr-core::core::lock` remains the lock owner; this scope selects the directory,
not a second locking mechanism.

## 2. Signatures

```rust
LockManager::with_scoped_directory<T>(
    directory: &Path,
    operation: impl FnOnce() -> Result<T>,
) -> Result<T>
```

The directory uses the existing normalized resource identity. Unscoped
`LockManager::with_default_path()` retains its production environment/home resolution.

## 3. Contracts

- Run the complete synchronous operation on one blocking worker. The scope is
  thread-local and must not cross an await, spawn, or detached callback.
- The scope is RAII: success, error and panic unwinding restore the prior value.
- A nested scope may reuse the normalized directory. Changing to another
  directory while scoped returns an error before running the nested operation.
- Resolving/constructing a manager does not create directories. Acquiring a
  file lock may create its own parent, as before.
- Production callers resolve their existing directory; fixtures bind an
  explicit owned directory. Do not modify global HOME/USERPROFILE or expose a
  UI control that selects arbitrary lock roots for the same live resource.
- Resource hashes, timeout/retry, file metadata, lock hierarchy and release
  semantics remain owned by their existing implementations.

## 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Same-directory nested scope | Allowed; outer scope restored afterward |
| Different-directory nested scope | FileLockError; nested operation not run |
| Operation returns error or unwinds | Previous scope restored |
| Two threads with different scopes | Independent directories; no env mutation |
| Missing directory, path-only lookup | No filesystem creation |

## 5. Good / Base / Bad Cases

- Good: the injected lifecycle scope covers application, catalog and guarded
  leaf writes on the same worker.
- Base: legacy unscoped callers retain their default lock resolution.
- Bad: scope a constructor, drop the scope, then run its writer elsewhere.
- Bad: select different fixture roots for cooperating writers of the same
  production resource, defeating the lock identity.

## 6. Tests Required

- `scoped_lock_directory_restores_after_error_panic_and_rejects_switch`: assert
  nested identity, rejection, error restoration and panic restoration.
- `scoped_lock_directories_are_thread_local_and_cover_guarded_writes`: exercise
  the real guarded writer on two threads and assert fixture-local lock files.
- Injected lifecycle tests assert constructors/missing reads create nothing,
  all auxiliary writes remain fixture-local, and compensation/CAS still work.
- Keep default Rust test parallelism and `--skip export_bindings`. Test results
  belong in task evidence; this spec does not claim a command was run.

## 7. Wrong vs Correct

Wrong: redirect the catalog but allow nested `LockManager::with_default_path()` to resolve
ambient home; or move scoped work onto another thread.

Correct:

```rust
LockManager::with_scoped_directory(&fixture_locks, || {
    // Complete existing synchronous domain operation, including nested writes.
    lifecycle_operation()
})
```
