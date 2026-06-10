# Test Fixtures

> Process-wide environment fixtures for `ccr-db` tests.

---

## Scenario: TestOpenCodeEnv for OpenCode usage import path isolation

### 1. Scope / Trigger
- Trigger: adding or changing `ccr-db` tests that resolve OpenCode storage through `CCR_OPENCODE_DIR`.
- Applies to usage import tests that call `UsageImportService::list_usage_files("opencode")` or `import_platform("all")` and need a synthetic `opencode.db`.
- This fixture is crate-local because `ccr-db` owns the legacy usage import path discovery and should not depend on CLI/Tauri test helpers.
- Do not remove global serial test gates until env-heavy tests across crates are migrated and measured.

### 2. Signatures
- `crate::test_support::TestOpenCodeEnv::new() -> TestOpenCodeEnv`
- `TestOpenCodeEnv::opencode_dir(&self) -> &Path`

### 3. Contracts
- `TestOpenCodeEnv::new()` acquires the crate-local process-wide test env lock and holds it until `Drop`.
- `TestOpenCodeEnv::new()` creates an isolated temp directory and sets `CCR_OPENCODE_DIR` to that directory.
- `Drop` restores captured environment variables in reverse order while the env lock is still held.
- The fixture is intentionally narrow; it must not grow to cover `CODEX_HOME`, `HOME`, `CCR_ROOT`, or database pool state.
- Database pool/table isolation remains owned by existing usage import test setup (`setup()` / `reset_usage_tables()`).

### 4. Validation & Error Matrix
- Temp directory cannot be created -> test setup fails immediately.
- Test panics -> Rust still drops `TestOpenCodeEnv` and restores `CCR_OPENCODE_DIR`.
- A test mutates `CCR_OPENCODE_DIR` without `TestOpenCodeEnv` -> unsafe for future concurrent test execution.
- A test needs Codex path discovery through `CODEX_HOME` -> create a separate fixture; do not hide unrelated env keys in `TestOpenCodeEnv`.

### 5. Good/Base/Bad Cases
- Good: `let env = TestOpenCodeEnv::new(); let db_path = env.opencode_dir().join("opencode.db");`.
- Good: hold the existing database test guard and then create `TestOpenCodeEnv` before calling `import_platform("all")`.
- Base: tests that pass explicit `db_path` to `import_file("opencode", &db_path)` do not need this fixture.
- Bad: manually saving/restoring `CCR_OPENCODE_DIR` in each test.
- Bad: using `TestOpenCodeEnv` as a catch-all fixture for all usage import environment variables.

### 6. Tests Required
- Unit test that `TestOpenCodeEnv` sets and restores `CCR_OPENCODE_DIR`.
- Regression test that `import_platform("all")` imports from a synthetic OpenCode database through the fixture.
- Run `cargo test -p ccr-db test_open_code_env_sets_and_restores_opencode_dir -- --nocapture`.
- Run `cargo test -p ccr-db test_import_platform_all_includes_opencode -- --nocapture`.
- Run `cargo test -p ccr-db -- --nocapture`.
- Run `cargo clippy -p ccr-db --all-targets --all-features -- -D warnings`.

### 7. Wrong vs Correct
#### Wrong
```rust
let previous = std::env::var("CCR_OPENCODE_DIR").ok();
unsafe { std::env::set_var("CCR_OPENCODE_DIR", temp.path()) };
let result = service.import_platform("all");
unsafe { restore(previous) };
```

#### Correct
```rust
let env = crate::test_support::TestOpenCodeEnv::new();
let db_path = env.opencode_dir().join("opencode.db");
let result = UsageImportService::new(ImportConfig::default()).import_platform("all");
```
