# Test Fixtures

> Process-wide environment and filesystem fixtures for root `ccr` integration tests.

---

## Scenario: CcrIntegrationTestEnv for root integration test environment isolation

### 1. Scope / Trigger
- Trigger: adding or changing root `crates/ccr/tests/**` integration tests that resolve default CCR paths through process environment variables.
- Applies to tests touching `CCR_ROOT` or `CCR_LOCK_DIR` through root `ccr` re-exported managers, platform helpers, or command helpers.
- This fixture is test-binary local because root integration tests compile as separate binaries (`platforms`, `commands`, `managers`) and cannot share runtime state across those binaries.
- Do not remove global serial test gates until env-heavy tests across crates are migrated, measured, and the remaining serial list is explicit.

### 2. Signatures
- `crates/ccr/tests/support/env.rs`
- `CcrIntegrationTestEnv::new() -> CcrIntegrationTestEnv`
- `CcrIntegrationTestEnv::path(&self) -> &Path`
- `platforms.rs`: `type PlatformTestEnv = env::CcrIntegrationTestEnv`
- `platforms.rs`: `setup_platform_test_env() -> PlatformTestEnv`
- `commands.rs`: `setup_ccr_test_env() -> env::CcrIntegrationTestEnv`
- `managers.rs`: `setup_ccr_test_env() -> env::CcrIntegrationTestEnv`

### 3. Contracts
- `CcrIntegrationTestEnv::new()` acquires a process-wide mutex for the current integration test binary and holds it until `Drop`.
- `CcrIntegrationTestEnv::new()` creates an isolated temp root plus `.locks` directory.
- `CcrIntegrationTestEnv::new()` sets `CCR_ROOT` to the isolated temp root and `CCR_LOCK_DIR` to the isolated `.locks` path.
- `Drop` restores every captured environment variable in reverse order while the env lock is still held.
- The shared support module exports only the core fixture type; each test binary defines only the aliases/setup functions it actually uses to keep `clippy -D warnings` clean.
- The fixture is intentionally narrow; subprocess CLI tests that pass env vars directly to `Command` should keep doing so instead of mutating process-global env.

### 4. Validation & Error Matrix
- Temp root cannot be created -> test setup fails immediately.
- Lock directory cannot be created -> test setup fails immediately.
- Test panics -> Rust still drops `CcrIntegrationTestEnv` and restores captured env vars.
- A test mutates `CCR_ROOT` / `CCR_LOCK_DIR` without this fixture or an equivalent lock -> unsafe for future concurrent test execution.
- A support helper is exported from a binary that does not use it -> `cargo clippy -p ccr --all-targets --all-features -- -D warnings` fails with dead-code warnings.

### 5. Good/Base/Bad Cases
- Good: `let env = setup_ccr_test_env(); fs::write(env.path().join("config.toml"), "...")` for command/manager tests that read default paths.
- Good: `let temp_dir = setup_platform_test_env();` for platform integration tests that construct default `PlatformConfigManager` or platform paths.
- Good: keep support functions binary-local (`commands.rs`, `managers.rs`, `platforms.rs`) when only one test binary needs them.
- Base: explicit-path tests that do not read process env can keep using `tempfile::tempdir()` directly.
- Base: subprocess command integration tests can set `Command.env("CCR_ROOT", ...)` / `Command.env("CCR_LOCK_DIR", ...)` directly, because they do not mutate parent process globals.
- Bad: saving/restoring `CCR_ROOT` by hand inside each integration test.
- Bad: placing unused aliases/functions in `support/env.rs` and suppressing warnings with broad `#[allow(dead_code)]`.
- Bad: removing global `--test-threads=1` solely because this fixture exists; other crates still have separate env-heavy candidates.

### 6. Tests Required
- `cargo test -p ccr --test platforms -- --nocapture`
- `cargo test -p ccr --test commands -- sync_content --nocapture`
- `cargo test -p ccr --test managers -- --nocapture`
- `cargo clippy -p ccr --all-targets --all-features -- -D warnings`
- `just fmt-check`
- `git diff --check`

### 7. Wrong vs Correct
#### Wrong
```rust
let previous = std::env::var_os("CCR_ROOT");
unsafe { std::env::set_var("CCR_ROOT", temp.path()) };
// ...
unsafe { restore(previous) };
```

#### Correct
```rust
let env = setup_ccr_test_env();
std::fs::write(env.path().join("config.toml"), "default_platform = 'claude'").unwrap();
```

#### Correct for platform integration tests
```rust
let temp_dir = setup_platform_test_env();
std::fs::create_dir_all(temp_dir.path()).unwrap();
```
