# Test Fixtures

> Process-wide environment and filesystem fixtures for `ccr-codex` tests.

---

## Scenario: TestCodexEnv for Codex environment isolation

### 1. Scope / Trigger
- Trigger: adding or changing `ccr-codex` tests that resolve Codex or CCR paths through process environment variables.
- Applies to tests touching `CCR_ROOT`, `CCR_DATA_DIR`, `CCR_CODEX_DIR`, or `CCR_LOCK_DIR`.
- This fixture is crate-local because `ccr-codex` must not depend on `ccr-cli::test_support` or `ccr-config` test internals.
- Keep default Rust test parallelism and `--skip export_bindings` for behavior tests. Retain the fixture's existing crate-local lock; HOME fallback cases run as independent test subprocesses, not a serial whole-suite gate.

### 2. Signatures
- `crate::test_support::TestCodexEnv::new() -> TestCodexEnv`
- `TestCodexEnv::home(&self) -> &Path`
- `TestCodexEnv::root(&self) -> &Path`
- `TestCodexEnv::ccr_codex_dir(&self) -> &Path`
- `TestCodexEnv::codex_dir(&self) -> &Path`
- `TestCodexEnv::lock_dir(&self) -> &Path`
- `TestCodexEnv::set_env(&mut self, key, value)`
- `TestCodexEnv::remove_env(&mut self, key)`
- Test-only `crate::home_process::run_current_test() -> bool` and `child_root() -> PathBuf`, included from `scripts/test-support/home_process.rs` without depending on CLI.

### 3. Contracts
- `TestCodexEnv::new()` acquires the crate-local process-wide test env lock and holds it until `Drop`.
- `TestCodexEnv::new()` creates an isolated temp home, `.ccr`, `.ccr/platforms/codex`, `.codex`, and `.locks` directories.
- In a guarded home-discovery child, `new()` uses the parent's canonical, ownership-marked root instead of creating another home. The parent retains its TempDir until that exact child test finishes.
- At a default-home test entry, return immediately if `home_process::run_current_test()` returns true. In the isolated child it returns false and the original test body runs. The helper sets HOME/USERPROFILE only through `Command::env` before spawn, checks the parent values remain unchanged, and requires successful exit plus one passing child test.
- `CCR_TEST_HOME_PROCESS_CASE`, `CCR_TEST_HOME_PROCESS_ROOT`, and `.ccr-home-process-fixture` identify and validate the test root; missing/invalid roots fail setup. Do not set/remove process-global HOME/USERPROFILE in either process.
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
- A test clears the default-home runtime after CODEX_HOME redirection -> guard the entire test before constructing its fixture. Removing explicit CCR_CODEX_DIR must resolve only to the synthetic child home.
- Child failure or no matching passing test -> parent test fails; a subprocess's successful empty test run is not coverage.

### 5. Good/Base/Bad Cases
- Good: `let _env = TestCodexEnv::new();` before constructing `CodexPlatform`, `CodexConfigManager`, or `CodexAuthService` paths from defaults.
- Good: use `env.codex_dir().join("config.toml")` for Codex runtime config fixture files.
- Good: use `env.ccr_codex_dir().join("auth")` for CCR-managed Codex auth registry fixture files.
- Good: preserve redirected CODEX_HOME plus default-home fallback assertions in a guarded child whose home matches `env.home()`.
- Base: tests that only pass explicit temp paths and do not read env vars can keep using `tempfile::tempdir()` directly.
- Bad: manually saving/restoring `CCR_CODEX_DIR` in each test.
- Bad: calling `std::env::set_var("CCR_ROOT", ...)` without the shared test env lock.
- Bad: using this fixture for `PATH` / `CARGO_HOME` tests.
- Bad: calling `env.set_env("HOME", ...)` / `env.set_env("USERPROFILE", ...)` to simulate fallback; those values must be scoped to a child `Command`.

### 6. Tests Required
- Unit test that `TestCodexEnv` sets and restores `CCR_ROOT`, `CCR_DATA_DIR`, `CCR_CODEX_DIR`, and `CCR_LOCK_DIR`.
- Migrated env-heavy tests should pass using `TestCodexEnv`.
- Preserve `login_prep_dirs_include_default_home_when_codex_home_redirects`, `login_prep_dirs_stay_on_explicit_ccr_codex_dir`, and `clear_runtime_scrubs_default_home_when_codex_home_is_redirected`. Each must execute its original assertions in one successful exact-name child, with no global HOME mutation.
- Run targeted `ccr-codex` tests after fixture changes.
- Run `cargo test -p ccr-codex --locked -- --skip export_bindings`.
- Run `cargo clippy -p ccr-codex --locked --all-targets --all-features -- -D warnings`.

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

#### Correct for default-home discovery and cleanup
```rust
if crate::home_process::run_current_test() {
    return;
}
let mut env = crate::test_support::TestCodexEnv::new();
env.remove_env("CCR_CODEX_DIR");
let sandbox = env.home().join("sandbox-codex");
std::fs::create_dir_all(&sandbox).unwrap();
env.set_env("CODEX_HOME", sandbox.as_os_str());
// Keep the original sandbox + env.home().join(".codex") assertions.
```
