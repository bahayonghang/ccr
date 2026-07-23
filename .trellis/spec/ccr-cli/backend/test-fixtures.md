# Test Fixtures

> Process-wide environment and filesystem fixtures for `ccr-cli` tests.

---

## Scenario: TestHome for CCR CLI environment isolation

### 1. Scope / Trigger
- Trigger: adding or changing `ccr-cli` tests that mutate CCR-related environment variables or write user-like config paths.
- Applies to tests touching `CCR_ROOT`, `CCR_LOCK_DIR`, `CCR_SETTINGS_PATH`, `CCR_BACKUP_DIR`, `CCR_CODEX_DIR`, or `CCR_CONFIG_PATH`.
- The fixture is a first step toward reducing the need for whole-workspace serial test execution; do not remove global serial gates until env-heavy tests are migrated and measured.

### 2. Signatures
- `crate::test_support::env_lock() -> MutexGuard<'static, ()>`
- `crate::test_support::TestHome::new() -> TestHome`
- `crate::test_support::TestHome::new_with_home_env() -> TestHome`
- `crate::test_support::TestHostEnv::new() -> TestHostEnv`
- `TestHome::home(&self) -> &Path`
- `TestHome::root(&self) -> &Path`
- `TestHome::settings_path(&self) -> &Path`
- `TestHome::backup_dir(&self) -> &Path`
- `TestHome::lock_dir(&self) -> &Path`
- `TestHome::codex_dir(&self) -> &Path`
- `TestHome::set_env(&mut self, key: &'static str, value: &OsStr)`
- `TestHostEnv::home(&self) -> &Path`
- `TestHostEnv::set_env(&mut self, key: &'static str, value: &OsStr)`
- `TestHostEnv::remove_env(&mut self, key: &'static str)`

### 3. Contracts
- `TestHome::new()` acquires the process-wide test env lock and holds it until `Drop`.
- `TestHome::new()` creates an isolated temp home plus `.ccr`, `.claude`, `.claude/backups`, `.locks`, and `.codex` directories.
- `TestHome::new()` sets common CCR env vars to the isolated paths and removes `CCR_CONFIG_PATH` so default path resolution cannot escape the fixture.
- `TestHome::new_with_home_env()` additionally sets `HOME` and `USERPROFILE` to the isolated temp home for tests that exercise user-home discovery.
- `TestHome::set_env()` may add fixture-specific env vars under the same lock; use it for narrow service-specific keys such as `CLAUDE_CONFIG_DIR`, not for broad install-detection keys like `PATH` or `CARGO_HOME`.
- `TestHostEnv::new()` acquires the same process-wide test env lock and holds it until `Drop`.
- `TestHostEnv` owns host-tool discovery env tests for keys such as `PATH`, `CARGO_HOME`, and `HOME`.
- `TestHostEnv::set_env()` and `TestHostEnv::remove_env()` capture previous values before mutation and restore them in reverse order on `Drop`.
- `Drop` restores every captured environment variable in reverse order while the env lock is still held.
- Tests that also need `CONFIG_LOCK` should create `TestHome` while the relevant test scope owns both locks and should avoid nested `env_lock()` calls.
- Child-process fixtures that replace `PATH` with a fake-tool directory must not let Unix fake scripts resolve helper programs through that stripped `PATH`; use shell builtins or explicit paths available on the supported CI platforms.

### 4. Validation & Error Matrix
- Temp home cannot be created -> test setup fails immediately.
- Fixture directories cannot be created -> test setup fails immediately.
- Test panics -> Rust still drops `TestHome` and restores captured env vars.
- A test calls `env_lock()` and then calls `TestHome::new()` in the same thread -> deadlock risk; use one fixture owner per test.
- A test mutates CCR env vars without `TestHome` or `env_lock()` -> unsafe for concurrent test execution.
- A test needs `HOME` / `USERPROFILE` path discovery -> use `TestHome::new_with_home_env()` rather than saving/restoring those variables by hand.
- A test needs `PATH` / `CARGO_HOME` install detection -> use `TestHostEnv`; do not hide those keys inside `TestHome`.
- A Unix fake executable calls `mkdir`, `touch`, or another helper by name after the child `PATH` was replaced -> the fixture can fail before it exercises product behavior.

### 5. Good/Base/Bad Cases
- Good: `let _home = TestHome::new();` before constructing services that read CCR env paths.
- Good: use `home.root()` / `home.settings_path()` for assertions and fixture file writes.
- Good: `let mut home = TestHome::new_with_home_env(); home.set_env("CLAUDE_CONFIG_DIR", claude_dir.as_os_str());` for service-specific home discovery tests.
- Good: `let mut env = TestHostEnv::new(); env.set_env("PATH", OsStr::new(""));` for install detection tests.
- Good: `env.remove_env("CARGO_HOME")` when testing fallback from `HOME/.cargo/bin`.
- Good: a Unix fake executable under an isolated `PATH` uses shell builtins and `/bin/mkdir` for its fixture-only filesystem setup.
- Base: keep `CONFIG_LOCK` for read-modify-write config tests while `TestHome` owns env restoration.
- Bad: manually saving/restoring `CCR_ROOT` in each test.
- Bad: calling `std::env::set_var("CCR_ROOT", ...)` without the shared test env lock.
- Bad: using `TestHome` as a catch-all fixture for `PATH` / `CARGO_HOME` tests.
- Bad: declaring a second local env mutex inside `install_detect` tests instead of reusing `TestHostEnv`.
- Bad: replacing a child process's `PATH` with only fake tools, then calling `mkdir` by name inside a Unix fake script.

### 6. Tests Required
- Unit test that `TestHome` sets `CCR_ROOT`, removes `CCR_CONFIG_PATH`, creates path roots, and restores previous env values on drop.
- Unit test that `TestHome::new_with_home_env()` sets and restores `HOME` / `USERPROFILE`.
- Unit test that `TestHostEnv` scopes and restores `PATH`, `CARGO_HOME`, and `HOME`.
- A migrated env-heavy test should pass using `TestHome` instead of hand-written env restoration.
- Install detection tests should pass using `TestHostEnv` instead of hand-written host-env restoration.
- Run `cargo test -p ccr-cli --lib` after fixture changes.
- Run `cargo clippy -p ccr-cli --all-targets --all-features -- -D warnings`.

### 7. Wrong vs Correct
#### Wrong
```rust
let previous = std::env::var("CCR_ROOT").ok();
unsafe { std::env::set_var("CCR_ROOT", temp.path()) };
// ...
unsafe { restore(previous) };
```

#### Correct
```rust
let _home = crate::test_support::TestHome::new();
// service/test code now reads isolated CCR paths
```

#### Correct for home discovery tests
```rust
let mut home = crate::test_support::TestHome::new_with_home_env();
home.set_env("CLAUDE_CONFIG_DIR", home.home().join(".claude").as_os_str());
```

#### Correct for host tool discovery tests
```rust
let mut env = crate::test_support::TestHostEnv::new();
env.set_env("PATH", std::ffi::OsStr::new(""));
env.remove_env("CARGO_HOME");
```
