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

## Scenario: Parse and persist profile TOML safely

### 1. Scope / Trigger

- Trigger: validating raw `profiles.toml` content or changing any structured profile write path.
- Applies to `parse_profiles_from_str`, `load_profiles_from_toml`, `save_profiles_to_toml`, and `update_current_config`.

### 2. Signatures

- `parse_profiles_from_str(content: &str) -> Result<IndexMap<String, ProfileConfig>>`
- `load_profiles_from_toml(path: &Path) -> Result<IndexMap<String, ProfileConfig>>`
- Structured profile writes use `ccr_core::fileio::write_toml_opts(..., WriteOptions { secret: true, .. })`.

### 3. Contracts

- `parse_profiles_from_str` is the single semantic parser for both full `CcsConfig` TOML and the legacy simplified profile map.
- `load_profiles_from_toml` preserves its missing-file behavior, reads bytes from disk, then delegates semantic parsing to `parse_profiles_from_str`.
- Parsing an empty document may return an empty collection at the core-library boundary. A UI workflow that forbids clearing all profiles must enforce that policy after parsing rather than changing shared parser compatibility.
- Both profile registry saves and current-profile updates set `secret: true`; on Unix, resulting files must not expose credential-bearing content beyond owner read/write permissions.
- Parser and persistence errors must not log or embed raw profile source or credential values.

### 4. Validation & Error Matrix

- Full config with `[profiles.<name>]` entries -> return those profiles.
- Simplified top-level profile map -> return the equivalent profile collection.
- Invalid TOML or incompatible profile fields -> return `CcrError`; do not partially accept entries.
- Missing file through `load_profiles_from_toml` -> preserve the established empty/default result.
- Empty parsed collection -> return empty from the shared parser; caller-specific destructive-action policy decides whether to reject it.
- Structured write failure -> propagate the guarded-write error and preserve the previous file.

### 5. Good/Base/Bad Cases

- Good: a raw editor parses with `parse_profiles_from_str`, applies its empty/activation policies, then writes the original bytes through a secret guarded-write path.
- Base: an existing caller loads a simplified registry without behavior changes after parser extraction.
- Bad: duplicate the full-versus-simplified fallback parser inside a Tauri command.
- Bad: serialize credential-bearing profiles with default `secret: false` permissions.

### 6. Tests Required

- Parse equivalent full and simplified fixtures and assert matching profile fields.
- Assert empty input preserves the shared parser's empty-collection behavior.
- Keep existing missing-file and malformed-file load tests passing.
- Exercise both structured write paths and assert secret file permissions on Unix.
- Run `cargo test -p ccr-config -- --test-threads=1` and clippy for `ccr-config` with warnings denied.

### 7. Wrong vs Correct

#### Wrong

```rust
let profiles: IndexMap<String, ProfileConfig> = toml::from_str(content)?;
write_toml(path, &profiles)?;
```

#### Correct

```rust
let profiles = parse_profiles_from_str(content)?;
write_toml_opts(
    path,
    &profiles,
    WriteOptions {
        secret: true,
        ..Default::default()
    },
)?;
```

## Scenario: TUI Preference Config

### 1. Scope / Trigger

- Trigger: changing user-facing TUI preferences that are stored under the unified CCR root.
- Applies to `<CCR_ROOT>/tui.toml` (default `~/.ccr/tui.toml`) and the `TuiConfigManager` API.

### 2. Signatures

- `TuiConfigManager::with_default() -> Result<TuiConfigManager>`
- `TuiConfigManager::load_or_default(&self) -> TuiConfig`
- `TuiConfigManager::save(&self, config: &TuiConfig) -> Result<()>`
- `TuiLanguage::{English, SimplifiedChinese}`
- `TuiTheme::{Mocha, Latte}`
- `TuiConfig { language: TuiLanguage, theme: TuiTheme, tab_order: Vec<TuiTabId> }`

### 3. Contracts

- `tab_order` is a complete ordered list of known tab ids.
- `language` serializes as `en` or `zh_cn`; a missing value defaults to English.
- `theme` serializes as `mocha` or `latte`; a missing value defaults to Mocha.
- An unsupported or non-string `language` falls back to English independently
  of `tab_order`, so an otherwise valid custom order is preserved.
- An unsupported or non-string `theme` falls back to Mocha independently of
  `language` and `tab_order`, preserving all other valid preferences.
- Current tab ids: `codex_profile`, `claude_profile`, `codex_auth`, `claude_auth`, `opencode_auth`.
- Deprecated id `usage` (standalone Usage tab retired 2026-07) stays parse-tolerant: the enum variant is kept `#[doc(hidden)]`, `load()` filters it out with a `tracing::warn!` **before** validation, and the user's custom order of the remaining tabs is preserved — never fall back to defaults just because `usage` appears.
- Missing files return the built-in default order and must not block TUI startup.
- `save` validates the complete tab order before calling
  `ccr_core::fileio::write_toml`; callers must pass the full loaded config so a
  language or theme change does not discard the other preferences.

### 4. Validation & Error Matrix

- Missing `tui.toml` -> return default config.
- Missing `language` -> English, preserving the loaded tab order.
- Unknown or non-string `language` -> warn and use English, preserving the
  loaded tab order.
- Missing `theme` -> Mocha, preserving language and tab order.
- Unknown or non-string `theme` -> warn and use Mocha, preserving language and
  tab order.
- `tab_order` containing deprecated `usage` -> filter + warn, then validate the remaining list normally (custom order preserved).
- Missing `tab_order`, duplicate ids, unknown ids, or incomplete lists (after `usage` filtering) -> return the full default order.
- TOML parse failure -> return the full default order and let the TUI continue.
- Invalid tab order passed to `save` -> return an error before writing; keep the
  existing file unchanged.
- Filesystem/lock/serialization failure during `save` -> propagate the
  `ccr_core` error; guarded write behavior keeps the previous file intact.

### 5. Good/Base/Bad Cases

- Good: `language = "zh_cn"`, `theme = "latte"`, and a complete custom
  `tab_order` round-trip through `load` / `save`.
- Good (legacy): a 6-item order containing `usage` loads with `usage` dropped and the custom order intact.
- Base: no `tui.toml` exists, so English, Mocha, and the default order are used.
- Bad: `language = "fr"` with a valid order must fall back only the language;
  it must not replace the valid order.
- Bad: `theme = "solarized"` must not discard a valid Chinese language or
  custom tab order.
- Bad: `tab_order = ["claude_auth"]`, because partial overrides are intentionally rejected.

### 6. Tests Required

- Unit tests for missing/English/Chinese/unknown/non-string language values,
  including assertions that valid custom ordering survives language fallback.
- Unit tests for default/Latte/unknown/non-string theme values, including
  assertions that language and custom ordering survive theme fallback.
- Unit tests for missing file, valid custom order, duplicate ids, missing ids, unknown ids, and legacy orders containing `usage` (order preserved, `usage` filtered).
- Save tests must assert language/theme/order round-trip and that validation
  failure does not overwrite an existing valid file.
- Tests that resolve default paths through `CCR_ROOT` must use `test_support::TestCcrEnv`.

### 7. Wrong vs Correct

#### Wrong

```rust
let language_only = "language = \"zh_cn\"";
std::fs::write("~/.ccr/tui.toml", language_only)?;
```

#### Correct

```rust
let manager = TuiConfigManager::with_default()?;
let mut config = manager.load_or_default();
config.language = TuiLanguage::SimplifiedChinese;
config.theme = TuiTheme::Latte;
manager.save(&config)?;
```

## Verification

For config changes, run:

- `just fmt-check`
- `cargo test -p ccr-config -- --test-threads=1`
- `just lint-strict`
