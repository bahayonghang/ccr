# Test Fixtures

> Process-wide environment and filesystem fixtures for `ccr-cli` tests.

---

## Scenario: TestHome for CCR CLI environment isolation

### 1. Scope / Trigger
- Trigger: adding or changing `ccr-cli` tests that mutate CCR-related environment variables or write user-like config paths.
- Applies to tests touching `CCR_ROOT`, `CCR_LOCK_DIR`, `CCR_SETTINGS_PATH`, `CCR_BACKUP_DIR`, `CCR_CODEX_DIR`, or `CCR_CONFIG_PATH`.
- Keep the default Rust test parallelism and `--skip export_bindings` for behavior suites. Existing crate-local locks still protect the CCR/host keys they own; HOME discovery uses a separate test process rather than a whole-suite serial gate.

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
- `crate::home_process::run_current_test() -> bool`
- `crate::home_process::run_case(case: &str, configure: impl FnOnce(&mut Command, &Path))`
- `crate::home_process::{is_child, child_root, case, reset_child_home}`; shared test-only implementation: `scripts/test-support/home_process.rs`.

### 3. Contracts
- `TestHome::new()` acquires the process-wide test env lock and holds it until `Drop`.
- `TestHome::new()` creates an isolated temp home plus `.ccr`, `.claude`, `.claude/backups`, `.locks`, and `.codex` directories.
- `TestHome::new()` sets common CCR env vars to the isolated paths and removes `CCR_CONFIG_PATH` so default path resolution cannot escape the fixture.
- `TestHome::new_with_home_env()` requires an isolated child created by the test-entry guard. It constructs CCR paths inside that child's canonical, ownership-marked root; it never sets or restores process-global HOME/USERPROFILE. Calling it without a valid child fixture fails immediately.
- `run_current_test()` returns true in the parent only after its exact named test has passed in a fresh subprocess; return from the parent test at that point. In the child it returns false, allowing the original body and assertions to execute. The parent snapshots HOME/USERPROFILE and checks they are unchanged; the child must exit successfully and report one passing test, not zero matching tests.
- Only `Command::env` / `Command::env_remove` configures HOME/USERPROFILE for that child. `CCR_TEST_HOME_PROCESS_CASE` and `CCR_TEST_HOME_PROCESS_ROOT` identify its case/root; `.ccr-home-process-fixture` proves parent ownership. Use `run_case` for empty/missing-HOME and USERPROFILE fallback cases, preserving each precedence assertion without global setters.
- Repeated `TestHome` lifecycle fixtures may reset only this marked root's children. Cleanup verifies the absolute canonical root and that each resolved directory stays within it, retaining the ownership marker. The parent TempDir owns final cleanup.
- `TestHome::set_env()` may add fixture-specific env vars under the same lock; use it for narrow service-specific keys such as `CLAUDE_CONFIG_DIR`, not for broad install-detection keys like `PATH` or `CARGO_HOME`.
- `TestHostEnv::new()` acquires the same process-wide test env lock and holds it until `Drop`.
- `TestHostEnv` owns host-tool discovery keys such as `PATH` and `CARGO_HOME`. It uses the marked child root when entered through `home_process`; HOME variations belong exclusively to the child command environment, never `set_env` / `remove_env`.
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
- A test needs HOME/USERPROFILE discovery -> guard its entry with `home_process::run_current_test()`, then use `TestHome::new_with_home_env()`; missing root/marker fails fixture construction.
- Child exit fails or exact test selection matches no passing test -> parent assertion fails with child output; do not report the case passed.
- A test needs `PATH` / `CARGO_HOME` install detection -> use `TestHostEnv`; do not hide those keys inside `TestHome`.
- A Unix fake executable calls `mkdir`, `touch`, or another helper by name after the child `PATH` was replaced -> the fixture can fail before it exercises product behavior.

### 5. Good/Base/Bad Cases
- Good: `let _home = TestHome::new();` before constructing services that read CCR env paths.
- Good: use `home.root()` / `home.settings_path()` for assertions and fixture file writes.
- Good: a guarded child test uses `TestHome::new_with_home_env()` plus narrow service-specific overrides.
- Good: `let mut env = TestHostEnv::new(); env.set_env("PATH", OsStr::new(""));` for install detection tests.
- Good: `env.remove_env("CARGO_HOME")` when testing fallback from `HOME/.cargo/bin`.
- Good: a Unix fake executable under an isolated `PATH` uses shell builtins and `/bin/mkdir` for its fixture-only filesystem setup.
- Base: keep `CONFIG_LOCK` for read-modify-write config tests while `TestHome` owns env restoration.
- Bad: manually saving/restoring `CCR_ROOT` in each test.
- Bad: calling `std::env::set_var("CCR_ROOT", ...)` without the shared test env lock.
- Bad: using `TestHome` as a catch-all fixture for `PATH` / `CARGO_HOME` tests.
- Bad: declaring a second local env mutex inside `install_detect` tests instead of reusing `TestHostEnv`.
- Bad: `std::env::set_var` or fixture `set_env` / `remove_env` for HOME/USERPROFILE, including inside the child. Set those keys on its `Command` before spawn.
- Bad: replacing a child process's `PATH` with only fake tools, then calling `mkdir` by name inside a Unix fake script.

### 6. Tests Required
- Unit test that `TestHome` sets `CCR_ROOT`, removes `CCR_CONFIG_PATH`, creates path roots, and restores previous env values on drop.
- Guarded test that `new_with_home_env().home()` equals the child HOME/USERPROFILE while the parent environment remains unchanged; require one actually passing child test.
- Unit test that `TestHostEnv` scopes/restores PATH/CARGO_HOME while its child HOME remains unchanged.
- Retain the Gemini HOME / empty HOME / missing HOME to USERPROFILE cases, install-detection home hints, and Codex redirected/default-home cleanup assertions. Dispatch varying environments as separate children, never remove assertions to avoid isolation work.
- Preserve downstream CLI/TUI/Tauri profile contract tests. The explicit `CCR_PROFILE_CONTRACT_ROOT` Tauri fixture is a separate path-injected seam and does not require HOME overrides; legacy TestHome consumers require the entry guard.
- A migrated env-heavy test should pass using `TestHome` instead of hand-written env restoration.
- Install detection tests should pass using `TestHostEnv` instead of hand-written host-env restoration.
- Run `cargo test -p ccr-cli --all-features --locked --lib -- --skip export_bindings` after fixture changes, plus affected dispatch/TUI/Tauri contract tests.
- Run `cargo clippy -p ccr-cli --locked --all-targets --all-features -- -D warnings`.

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
if crate::home_process::run_current_test() {
    return;
}
let mut home = crate::test_support::TestHome::new_with_home_env();
home.set_env("CLAUDE_CONFIG_DIR", home.home().join(".claude").as_os_str());
```

#### Correct for host tool discovery tests
```rust
let mut env = crate::test_support::TestHostEnv::new();
env.set_env("PATH", std::ffi::OsStr::new(""));
env.remove_env("CARGO_HOME");
```

#### Correct for a missing-HOME fallback case
```rust
if !crate::home_process::is_child() {
    crate::home_process::run_case("without-home", |command, _| {
        command.env_remove("HOME");
    });
    return;
}
let mut env = crate::test_support::TestHostEnv::new();
// Original host-tool fallback assertions execute here against env.home().
```
