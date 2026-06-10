# Test Fixtures

> Process-wide environment and filesystem fixtures for `ccr-sync` tests.

---

## Scenario: TestSyncEnv for sync environment isolation

### 1. Scope / Trigger
- Trigger: adding or changing `ccr-sync` tests that resolve default sync paths through process environment variables.
- Applies to tests touching `CCR_ROOT`, `CCR_SYNC_FOLDERS_CONFIG`, or `CCR_SYNC_CONFIG_PATH`.
- This fixture is crate-local because `ccr-sync` owns sync-specific config path semantics.
- Do not remove global serial test gates until env-heavy tests across crates are migrated and measured.

### 2. Signatures
- `crate::test_support::TestSyncEnv::new() -> TestSyncEnv`
- `TestSyncEnv::home(&self) -> &Path`
- `TestSyncEnv::root(&self) -> &Path`
- `TestSyncEnv::platforms_dir(&self) -> &Path`
- `TestSyncEnv::sync_folders_path(&self) -> &Path`
- `TestSyncEnv::sync_config_path(&self) -> &Path`

### 3. Contracts
- `TestSyncEnv::new()` acquires the crate-local process-wide test env lock and holds it until `Drop`.
- `TestSyncEnv::new()` creates an isolated temp home plus `.ccr` and `.ccr/platforms` directories.
- `TestSyncEnv::new()` sets `CCR_ROOT` to the isolated `.ccr` path.
- `TestSyncEnv::new()` sets `CCR_SYNC_FOLDERS_CONFIG` and `CCR_SYNC_CONFIG_PATH` to isolated config files under the temp home.
- `Drop` restores every captured environment variable in reverse order while the env lock is still held.
- The fixture is intentionally narrow; it must not grow to cover install-detection keys such as `PATH` or `CARGO_HOME`.

### 4. Validation & Error Matrix
- Temp home cannot be created -> test setup fails immediately.
- Fixture directories cannot be created -> test setup fails immediately.
- Test panics -> Rust still drops `TestSyncEnv` and restores captured env vars.
- A test mutates `CCR_ROOT` / `CCR_SYNC_FOLDERS_CONFIG` / `CCR_SYNC_CONFIG_PATH` without `TestSyncEnv` or the shared env lock -> unsafe for concurrent test execution.

### 5. Good/Base/Bad Cases
- Good: `let env = TestSyncEnv::new();` before testing content selection with `CCR_ROOT`.
- Good: use `env.platforms_dir().join("claude")` for platform content fixtures.
- Good: use `env.sync_config_path()` and `env.sync_folders_path()` for migration tests.
- Base: explicit path tests that do not read env vars can keep using `tempfile::tempdir()` directly.
- Bad: manually saving/restoring sync env vars inside each test.
- Bad: using `CONFIG_LOCK` as a substitute for sync env restoration.

### 6. Tests Required
- Unit test that `TestSyncEnv` sets and restores `CCR_ROOT`, `CCR_SYNC_FOLDERS_CONFIG`, and `CCR_SYNC_CONFIG_PATH`.
- Migrated env-heavy tests should pass using `TestSyncEnv`.
- Run targeted `ccr-sync` content selector and folder migration tests after fixture changes.
- Run `cargo test -p ccr-sync`.
- Run `cargo clippy -p ccr-sync --all-targets --all-features -- -D warnings`.

### 7. Wrong vs Correct
#### Wrong
```rust
unsafe { std::env::set_var("CCR_ROOT", temp.path().join(".ccr")) };
// ...
unsafe { std::env::remove_var("CCR_ROOT") };
```

#### Correct
```rust
let env = crate::test_support::TestSyncEnv::new();
std::fs::write(env.root().join("config.toml"), "test").unwrap();
```

