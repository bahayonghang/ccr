# Test Fixtures

> Process-wide environment fixtures for `ccr-core` tests.

---

## Scenario: TestLogEnv for log-level environment isolation

### 1. Scope / Trigger
- Trigger: adding or changing `ccr-core` tests that mutate logging-related process environment variables.
- Applies to tests touching `CCR_LOG_LEVEL` or `RUST_LOG`.
- This fixture is crate-local because `ccr-core` owns logging infrastructure and should not depend on CLI/config test helpers.
- Do not remove global serial test gates until env-heavy tests across crates are migrated and measured.

### 2. Signatures
- `crate::test_support::TestLogEnv::new() -> TestLogEnv`
- `TestLogEnv::set_env(&mut self, key: &'static str, value: &OsStr)`
- `TestLogEnv::remove_env(&mut self, key: &'static str)`

### 3. Contracts
- `TestLogEnv::new()` acquires the crate-local process-wide test env lock and holds it until `Drop`.
- `set_env()` records the previous value before setting a logging env var.
- `remove_env()` records the previous value before removing a logging env var.
- `Drop` restores captured environment variables in reverse order while the env lock is still held.
- The fixture is intentionally narrow; it must not grow to cover filesystem locks, `CCR_LOCK_DIR`, Qwen runtime env, or unrelated host env keys.

### 4. Validation & Error Matrix
- Test panics -> Rust still drops `TestLogEnv` and restores captured env vars.
- A test mutates `CCR_LOG_LEVEL` / `RUST_LOG` without `TestLogEnv` -> unsafe for future concurrent test execution.
- A test needs non-logging environment keys -> create a separate narrow fixture rather than expanding `TestLogEnv`.

### 5. Good/Base/Bad Cases
- Good: `let mut env = TestLogEnv::new(); env.set_env("RUST_LOG", OsStr::new("warn"));`.
- Good: call `env.remove_env("CCR_LOG_LEVEL")` before asserting fallback to `RUST_LOG`.
- Base: tests that only exercise formatting/masking and do not mutate env do not need this fixture.
- Bad: manually saving/restoring `CCR_LOG_LEVEL` and `RUST_LOG` inside each test.
- Bad: relying on global `--test-threads=1` as the only protection around env mutation.

### 6. Tests Required
- Unit test that `TestLogEnv` sets, removes, and restores `CCR_LOG_LEVEL` / `RUST_LOG`.
- Regression test that `resolve_log_filter()` preserves precedence: `CCR_LOG_LEVEL` overrides `RUST_LOG`.
- Run `cargo test -p ccr-core test_log_env_sets_removes_and_restores_log_vars -- --nocapture`.
- Run `cargo test -p ccr-core test_resolve_log_filter_precedence -- --nocapture`.
- Run `cargo test -p ccr-core -- --nocapture`.
- Run `cargo clippy -p ccr-core --all-targets --all-features -- -D warnings`.

### 7. Wrong vs Correct
#### Wrong
```rust
let old_ccr = std::env::var("CCR_LOG_LEVEL").ok();
unsafe { std::env::set_var("CCR_LOG_LEVEL", "debug") };
// ...
unsafe { restore(old_ccr) };
```

#### Correct
```rust
let mut env = crate::test_support::TestLogEnv::new();
env.set_env("RUST_LOG", std::ffi::OsStr::new("warn"));
env.remove_env("CCR_LOG_LEVEL");
assert_eq!(resolve_log_filter(), "warn");
env.set_env("CCR_LOG_LEVEL", std::ffi::OsStr::new("debug"));
assert_eq!(resolve_log_filter(), "debug");
```


