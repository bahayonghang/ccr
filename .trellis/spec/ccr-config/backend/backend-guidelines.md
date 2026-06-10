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

## Error Handling

Return `ccr_core::Result<T>` and map missing config, invalid platform, or TOML failures to `CcrError` with user-actionable messages. Avoid `unwrap`/`expect` in production config paths; missing sections should become validation or config errors.

## Logging

Use `tracing::debug!` for path decisions and autofix behavior. Do not log profile secrets or raw provider keys.

## Testing

Tests that mutate `CCR_ROOT` or `CCR_LOCK_DIR` must use `test_support::TestCcrEnv`, which holds a process-wide lock and restores env vars on Drop.

## Verification

For config changes, run:

- `just fmt-check`
- `cargo test -p ccr-config -- --test-threads=1`
- `just lint-strict`
