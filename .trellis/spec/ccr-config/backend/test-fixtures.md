# Test Fixtures

> Process-wide environment and filesystem fixtures for `ccr-config` tests.

---

## Scenario: TestCcrEnv for CCR config environment isolation

### 1. Scope / Trigger
- Trigger: adding or changing `ccr-config` tests that resolve default CCR paths through process environment variables.
- Applies to tests touching `CCR_ROOT` or `CCR_LOCK_DIR`.
- This fixture is crate-local because `ccr-config` cannot depend on `ccr-cli::test_support`.
- Do not remove global serial test gates until env-heavy tests across crates are migrated and measured.

### 2. Signatures
- `crate::test_support::TestCcrEnv::new() -> TestCcrEnv`
- `TestCcrEnv::root(&self) -> &Path`
- `TestCcrEnv::lock_dir(&self) -> &Path`
- `TestCcrEnv::home(&self) -> &Path`

### 3. Contracts
- `TestCcrEnv::new()` acquires the crate-local process-wide test env lock and holds it until `Drop`.
- `TestCcrEnv::new()` creates an isolated temp home plus `.ccr` and `.locks` directories.
- `TestCcrEnv::new()` sets `CCR_ROOT` and `CCR_LOCK_DIR` to the isolated paths.
- `Drop` restores every captured environment variable in reverse order while the env lock is still held.
- The fixture is intentionally narrow; it must not grow to cover install-detection keys such as `PATH` or `CARGO_HOME`.

### 4. Validation & Error Matrix
- Temp home cannot be created -> test setup fails immediately.
- Fixture directories cannot be created -> test setup fails immediately.
- Test panics -> Rust still drops `TestCcrEnv` and restores captured env vars.
- A test mutates `CCR_ROOT` / `CCR_LOCK_DIR` without `TestCcrEnv` or the shared env lock -> unsafe for concurrent test execution.

### 5. Good/Base/Bad Cases
- Good: `let _env = TestCcrEnv::new();` before constructing `PlatformConfigManager::with_default()`.
- Good: use `env.root().join("config.toml")` for direct config manager tests.
- Base: explicit path tests that do not read env vars can keep using `tempfile::tempdir()` directly.
- Bad: saving/restoring `CCR_ROOT` by hand inside each test.
- Bad: using this fixture for `PATH` / `CARGO_HOME` tests.

### 6. Tests Required
- Unit test that `TestCcrEnv` sets and restores `CCR_ROOT` / `CCR_LOCK_DIR`.
- Migrated env-heavy tests should pass using `TestCcrEnv`.
- Run targeted `ccr-config` tests after fixture changes.
- Run `cargo test -p ccr-config` and `cargo clippy -p ccr-config --all-targets --all-features -- -D warnings`.

### 7. Wrong vs Correct
#### Wrong
```rust
let previous_root = std::env::var("CCR_ROOT").ok();
unsafe { std::env::set_var("CCR_ROOT", temp.path()) };
// ...
unsafe { restore(previous_root) };
```

#### Correct
```rust
let _env = crate::test_support::TestCcrEnv::new();
let manager = PlatformConfigManager::with_default().unwrap();
```
