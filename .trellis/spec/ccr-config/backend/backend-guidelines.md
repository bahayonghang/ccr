# ccr-config Backend Guidelines

> Shared platform/profile configuration contracts and TOML registry helpers.

## Scope

`crates/ccr-config` owns platform profile models, platform path resolution, config managers, and profile serialization helpers. Do not duplicate platform/profile TOML parsing in `ccr-cli`, `ccr`, or UI-facing code.

Reference files:

- `crates/ccr-config/src/lib.rs`
- `crates/ccr-config/src/managers/config/manager.rs`
- `crates/ccr-config/src/platforms/base.rs`
- `crates/ccr-config/src/services/config_service.rs`

## Directory And Module Layout

Keep the existing domain split:

- `models/` for `Platform`, `PlatformConfig`, `PlatformPaths`, and `ProfileConfig`.
- `platforms/` for platform-specific profile load/save helpers.
- `managers/` for config file and registry persistence.
- `services/` for higher-level list/import/validation operations.

New platform config behavior should flow through `PlatformPaths` and the manager/service layer rather than ad hoc path joins in command code.

## Persistence Rules

Unified config is rooted at `CCR_ROOT` when set, otherwise under `~/.ccr`. Platform profiles live under `~/.ccr/platforms/<platform>/profiles.toml`. Preserve `ConfigManager::for_platform` for per-platform callers and `with_default` for current-platform callers.

Use `ccr_core::fileio`/manager save methods for TOML writes. Do not bypass manager helpers for profile writes because they preserve autofix and path conventions.

## Managed Env Mapping Contract

`ConfigSection::to_managed_env_pairs()` (in `managers/config/types.rs`) is the **only** `ConfigSection -> settings.json env` mapping in the workspace. It references `ccr_types::env_keys` constants — never string literals — so key names cannot drift, and `None` fields produce no pair (the clear-first semantics of `ClaudeSettings::apply_managed_env` then drop the stale key on profile switch).

When adding a typed env field: add the constant to `ccr_types::env_keys` (plus `NON_ANTHROPIC_MANAGED_KEYS` if unprefixed), map it here, and extend the mapping tests in the same file (per-key assertion + stale-key-cleared-on-switch combination test). `to_anthropic_env_status` stays a 4-key display preview; its values must match `to_managed_env_pairs` for those keys (guarded by `test_env_status_preview_matches_managed_pairs`).

## Error Handling

Return `ccr_core::Result<T>` and map missing config, invalid platform, or TOML failures to `CcrError` with user-actionable messages. Avoid `unwrap`/`expect` in production config paths; missing sections should become validation or config errors.

## Logging

Use `tracing::debug!` for path decisions and autofix behavior. Do not log profile secrets or raw provider keys.

## Testing

Tests that mutate `CCR_ROOT` or `CCR_LOCK_DIR` must use `test_support::TestCcrEnv`, which holds a process-wide lock and restores env vars on Drop.

## Scenario: TUI Preference Config

### 1. Scope / Trigger

- Trigger: changing user-facing TUI preferences that are stored under the unified CCR root.
- Applies to `<CCR_ROOT>/tui.toml` (default `~/.ccr/tui.toml`) and the `TuiConfigManager` API.

### 2. Signatures

- `TuiConfigManager::with_default() -> Result<TuiConfigManager>`
- `TuiConfigManager::load_or_default(&self) -> TuiConfig`
- `TuiConfig { tab_order: Vec<TuiTabId> }`

### 3. Contracts

- `tab_order` is a complete ordered list of known tab ids.
- Current tab ids: `codex_profile`, `claude_profile`, `codex_auth`, `claude_auth`, `opencode_auth`.
- Missing files return the built-in default order and must not block TUI startup.

### 4. Validation & Error Matrix

- Missing `tui.toml` -> return default config.
- Missing `tab_order`, duplicate ids, unknown ids, or incomplete lists -> return the full default order.
- TOML parse failure -> return the full default order and let the TUI continue.

### 5. Good/Base/Bad Cases

- Good: `tab_order = ["codex_profile", "claude_profile", "codex_auth", "claude_auth", "opencode_auth"]`
- Base: no `tui.toml` exists, so the same default order is used.
- Bad: `tab_order = ["claude_auth"]`, because partial overrides are intentionally rejected.

### 6. Tests Required

- Unit tests for missing file, valid custom order, duplicate ids, missing ids, and unknown ids.
- Tests that resolve default paths through `CCR_ROOT` must use `test_support::TestCcrEnv`.

### 7. Wrong vs Correct

#### Wrong

```rust
let path = home_dir().unwrap().join(".ccr").join("tui.toml");
```

#### Correct

```rust
let manager = TuiConfigManager::with_default()?;
let config = manager.load_or_default();
```

## Verification

For config changes, run:

- `just fmt-check`
- `cargo test -p ccr-config -- --test-threads=1`
- `just lint-strict`
