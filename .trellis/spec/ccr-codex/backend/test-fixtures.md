# Test Fixtures

> Process-wide environment and filesystem fixtures for `ccr-codex` tests.

---

## Scenario: TestCodexEnv for Codex environment isolation

### 1. Scope / Trigger
- Trigger: adding or changing `ccr-codex` tests that resolve Codex or CCR paths through process environment variables.
- Applies to tests touching `CCR_ROOT`, `CCR_DATA_DIR`, `CCR_CODEX_DIR`, or `CCR_LOCK_DIR`.
- This fixture is crate-local because `ccr-codex` must not depend on `ccr-cli::test_support` or `ccr-config` test internals.
- Do not remove global serial test gates until env-heavy tests across crates are migrated and measured.

### 2. Signatures
- `crate::test_support::TestCodexEnv::new() -> TestCodexEnv`
- `TestCodexEnv::home(&self) -> &Path`
- `TestCodexEnv::root(&self) -> &Path`
- `TestCodexEnv::ccr_codex_dir(&self) -> &Path`
- `TestCodexEnv::codex_dir(&self) -> &Path`
- `TestCodexEnv::lock_dir(&self) -> &Path`
- `TestCodexEnv::set_env(&mut self, key, value)`
- `TestCodexEnv::remove_env(&mut self, key)`

### 3. Contracts
- `TestCodexEnv::new()` acquires the crate-local process-wide test env lock and holds it until `Drop`.
- `TestCodexEnv::new()` creates an isolated temp home, `.ccr`, `.ccr/platforms/codex`, `.codex`, and `.locks` directories.
- `TestCodexEnv::new()` sets `CCR_ROOT` and `CCR_DATA_DIR` to the isolated `.ccr` path.
- `TestCodexEnv::new()` sets `CCR_CODEX_DIR` to the isolated `.codex` runtime path.
- `TestCodexEnv::new()` sets `CCR_LOCK_DIR` to the isolated `.locks` path.
- `Drop` restores every captured environment variable in reverse order while the env lock is still held.
- `set_env` / `remove_env` may add narrow Codex-runtime keys such as `CODEX_HOME` while the same lock is held.
- The fixture is intentionally narrow; it must not grow to cover install-detection keys such as `PATH` or `CARGO_HOME`.

### 4. Validation & Error Matrix
- Temp home cannot be created -> test setup fails immediately.
- Fixture directories cannot be created -> test setup fails immediately.
- Test panics -> Rust still drops `TestCodexEnv` and restores captured env vars.
- A test mutates `CCR_ROOT` / `CCR_DATA_DIR` / `CCR_CODEX_DIR` / `CCR_LOCK_DIR` without `TestCodexEnv` or the shared env lock -> unsafe for concurrent test execution.
- A test needs provider install detection or host binary discovery -> keep it as a separate fixture strategy; do not hide host env keys inside `TestCodexEnv`.

### 5. Good/Base/Bad Cases
- Good: `let _env = TestCodexEnv::new();` before constructing `CodexPlatform`, `CodexConfigManager`, or `CodexAuthService` paths from defaults.
- Good: use `env.codex_dir().join("config.toml")` for Codex runtime config fixture files.
- Good: use `env.ccr_codex_dir().join("auth")` for CCR-managed Codex auth registry fixture files.
- Base: tests that only pass explicit temp paths and do not read env vars can keep using `tempfile::tempdir()` directly.
- Bad: manually saving/restoring `CCR_CODEX_DIR` in each test.
- Bad: calling `std::env::set_var("CCR_ROOT", ...)` without the shared test env lock.
- Bad: using this fixture for `PATH` / `CARGO_HOME` tests.

### 6. Tests Required
- Unit test that `TestCodexEnv` sets and restores `CCR_ROOT`, `CCR_DATA_DIR`, `CCR_CODEX_DIR`, and `CCR_LOCK_DIR`.
- Migrated env-heavy tests should pass using `TestCodexEnv`.
- Run targeted `ccr-codex` tests after fixture changes.
- Run `cargo test -p ccr-codex`.
- Run `cargo clippy -p ccr-codex --all-targets --all-features -- -D warnings`.

### 7. Wrong vs Correct
#### Wrong
```rust
let previous = std::env::var_os("CCR_CODEX_DIR");
unsafe { std::env::set_var("CCR_CODEX_DIR", temp.path()) };
// ...
unsafe { restore(previous) };
```

#### Correct
```rust
let env = crate::test_support::TestCodexEnv::new();
std::fs::write(env.codex_dir().join("config.toml"), "...").unwrap();
let platform = CodexPlatform::new().unwrap();
```
