# ccr-cli Backend Guidelines

> CLI/application domain crate.

## Scope

`crates/ccr-cli` owns command definitions, command dispatch, CLI presentation, application services, and command-facing managers. Domain logic should call shared crates rather than duplicating lower-level behavior.

Reference files:

- `crates/ccr-cli/src/lib.rs`
- `crates/ccr-cli/src/cli/`
- `crates/ccr-cli/src/commands/mod.rs`
- `crates/ccr-cli/src/application/`

## Structure

Follow the current command split:

- `cli/` for Clap definitions and subcommand enums.
- `commands/` for user-facing command handlers grouped by area.
- `services/`, `managers/`, and `platforms/` for application orchestration that is still CLI-specific.
- `models/` mostly re-exports shared model crates for command use.

Do not put Ratatui rendering in this crate; that belongs in `ccr-tui`. Do not put persistent session SQL here; that belongs in `ccr-store` or `ccr-db`.

## Output And Logging

Human CLI output may use `println!`, tables, and colored output in command handlers. JSON modes should serialize stable output structs. Diagnostic/runtime logs should use `tracing`, not direct stderr, unless a test-only debug helper is explicitly isolated.

Never print secrets. Use masking helpers for tokens, provider keys, auth files, and config values.

## Error Handling

Command handlers should return `ccr_core::Result<T>` or `anyhow::Result<T>` where the local command already uses it. Preserve actionable errors from shared crates and handle them at the dispatcher boundary.

Do not use panics for invalid user input. Clap validation, typed command enums, and `CcrError` should carry invalid states.

## Claude Profile Auth Mode Contract

`ClaudePlatform::apply_profile` branches on auth mode: `Subscription` calls `clear_managed_vars()` and writes **no** `ANTHROPIC_*` / `CLAUDE_CODE_*`; `ApiKey` calls `settings.apply_managed_env(section.to_managed_env_pairs())` and writes the overrides. A third-party profile therefore **only works under `api_key`**.

Auth mode has two layers — keep them separate:

- `ClaudeAuthService::resolve_profile_auth_mode` — literal/stored resolution (explicit `platform_data.auth_mode` wins over inference). Do not change this; tests depend on its literal semantics.
- `ClaudeAuthService::effective_auth_mode` — normalization layer on top of resolve: if resolved is `Subscription` **and** `is_api_key_shaped`, return `ApiKey`. `ClaudePlatform::profile_auth_mode` delegates here so apply / validate / `profile_to_json` stay consistent.

`is_api_key_shaped` is intentionally conservative: `provider_type == "third_party_model"`, or `base_url` and `auth_token` both non-empty. **Do not** include model-mapping fields — `ANTHROPIC_DEFAULT_*_MODEL` is valid on official subscription (snapshot pinning), so that would false-positive and fail `section.validate()`.

Correction happens at two points and must stay idempotent: `normalize_profile` (save — persists the corrected `auth_mode`) and `apply_profile` (defensive — self-heals stale on-disk profiles). Each emits a `tracing::warn` on correction; never log `auth_token` / full `base_url`.

Model-mapping fields are typed on `ProfileConfig` / `ConfigSection` and mapped in `ConfigSection::to_managed_env_pairs` (ccr-config, keyed by `ccr_types::env_keys` constants); `custom_model_option`(`_name`) → `ANTHROPIC_CUSTOM_MODEL_OPTION`(`_NAME`). New env keys must also be registered in `ClaudePlatform::get_env_var_names`. Typing a previously-untyped key auto-migrates existing TOML (serde captures it into the typed slot instead of `other`/`platform_data`).

`ClaudeSettings` itself is `ccr_types::ClaudeSettings` (single workspace shape); `managers/settings.rs` is a pure IO adapter (`SettingsManager`: load/save/backup/restore) plus a re-export, and must not grow local settings types or env-mutation logic.

## Scenario: Claude API-Key Profile Runtime Env

### 1. Scope / Trigger

- Trigger: adding or changing Claude profile fields that write Claude Code environment variables, profile apply behavior, doctor diagnostics, or onboarding state.
- Applies to `ProfileConfig`, `ConfigSection`, `ccr_config::profile_to_section`, `ccr_config::section_to_profile`, `ClaudeSettings`, `ClaudePlatform`, Tauri Claude profile JSON, and command integration tests.

### 2. Signatures

- `ConfigSection::{default_fable_model, default_*_model_name, claude_code_auto_compact_window, api_timeout_ms, claude_code_disable_nonessential_traffic}`
- `ProfileConfig::{default_fable_model, default_*_model_name, claude_code_auto_compact_window, api_timeout_ms, claude_code_disable_nonessential_traffic}`
- `ConfigSection::to_managed_env_pairs()` (ccr-config)
- `ClaudeSettings::apply_managed_env(pairs)` / `ClaudeSettings::clear_managed_vars()` (ccr-types)
- `ClaudePlatform::get_env_var_names()`
- `ClaudePlatform::apply_profile(name)`
- Test fixtures: `TestHome` must isolate `CLAUDE_CONFIG_DIR`, `CLAUDE_JSON_PATH`, `CCR_SETTINGS_PATH`, and `CCR_BACKUP_DIR`.

### 3. Contracts

- API-key Claude profiles write only typed, managed env keys into `~/.claude/settings.json.env`; do not add ad hoc env writes in command handlers.
- Subscription profiles call `clear_managed_vars()` and must remove both `ANTHROPIC_*` keys and non-Anthropic CCR-managed Claude Code keys such as `CLAUDE_CODE_AUTO_COMPACT_WINDOW`, `API_TIMEOUT_MS`, and `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`.
- Every new typed env field must be added to both `ProfileConfig` and `ConfigSection`, both conversion directions, `ccr_types::env_keys` (constant + `NON_ANTHROPIC_MANAGED_KEYS` when unprefixed), `ConfigSection::to_managed_env_pairs`, `ClaudePlatform::get_env_var_names`, Tauri JSON parse/serialize, UI form state, and provider template mappers when templates can fill it. Clearing needs no extra wiring: `clear_managed_vars` covers the `ANTHROPIC_` prefix plus `NON_ANTHROPIC_MANAGED_KEYS`.
- API-key profile apply should try to set `hasCompletedOnboarding = true` in `~/.claude.json` or `CLAUDE_JSON_PATH`. This helper preserves unknown JSON fields. Failure to read/parse/write `.claude.json` is logged as a warning and must not prevent `settings.json` from being saved.
- `ccr doctor` checks API-key profiles for placeholder-looking tokens, active-profile env mismatches, GLM 1M profiles missing compact-window configuration, and missing/corrupt onboarding state.

### 4. Validation & Error Matrix

- Profile token is placeholder-like -> `doctor` warning; do not print or infer a real token.
- Profile expected env differs from `settings.json.env` -> `doctor` warning recommending re-apply.
- GLM model contains `[1m]` and `claude_code_auto_compact_window` is empty -> `doctor` warning recommending `1000000`.
- `.claude.json` missing, unparsable, unreadable, or lacking `hasCompletedOnboarding = true` -> `doctor` warning.
- `.claude.json` is corrupt during API-key apply -> keep applying `settings.json`, emit `tracing::warn`, and let `doctor` surface the onboarding state.
- A new env key is written but not cleared on subscription/off switch -> regression; add a switch-cleanup test before merging.

### 5. Good/Base/Bad Cases

- Good: typed GLM profile writes `ANTHROPIC_DEFAULT_FABLE_MODEL`, all `*_MODEL_NAME` vars, `CLAUDE_CODE_AUTO_COMPACT_WINDOW`, `API_TIMEOUT_MS`, and `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`, then switching to subscription clears them.
- Good: `.claude.json` with `oauthAccount` keeps that object and only adds `hasCompletedOnboarding = true`.
- Base: missing `.claude.json` becomes a minimal JSON object with `hasCompletedOnboarding = true` during API-key apply.
- Bad: writing `hasCompletedOnboarding` into `~/.claude/settings.json`.
- Bad: storing runtime env keys only in `platform_data` or UI-only state, because apply and doctor cannot round-trip them reliably.

### 6. Tests Required

- `cargo test -p ccr-cli platforms::claude -- --test-threads=1`
  - Assert API-key apply writes expected env and onboarding state.
  - Assert corrupt `.claude.json` does not block `settings.json` apply.
  - Assert subscription apply clears non-Anthropic managed keys.
- `cargo test -p ccr-config -- --test-threads=1`
  - Assert each typed field maps to the correct env key in `to_managed_env_pairs` and stale keys clear on profile switch (combined with `apply_managed_env`).
- `cargo test -p ccr-cli managers::settings -- --test-threads=1`
  - Assert disk-level read→apply→write→read keeps unknown fields and non-managed env intact.
- `cargo test -p ccr --test commands doctor -- --test-threads=1`
  - Assert placeholder, mismatch, compact-window, and onboarding warnings.
- `cargo test -p ccr --test commands claude_profile -- --test-threads=1`
  - Assert command-level switch/off behavior remains compatible.

### 7. Wrong vs Correct

#### Wrong

```rust
settings.env.insert("API_TIMEOUT_MS".into(), "3000000".into());
```

This one-off write skips typed TOML migration, UI round-trip, doctor comparison, and cleanup.

#### Correct

```rust
profile.api_timeout_ms = Some("3000000".into());
let section = ClaudePlatform::profile_to_section(&profile)?;
settings.apply_managed_env(section.to_managed_env_pairs());
```

This keeps persistence, apply, diagnostics, and cleanup on the same typed contract.

## Testing

Use crate-local `test_support::TestHome` and `TestHostEnv` for env/path-sensitive command tests. These fixtures serialize process env mutation and restore variables on Drop.

Command integration behavior also has tests under `crates/ccr/tests/commands/`; update those when command output or compatibility surfaces change.

## Verification

For CLI command changes, run:

- `just fmt-check`
- `cargo test -p ccr-cli -- --test-threads=1`
- Relevant `cargo test -p ccr --test commands -- --test-threads=1`
- `just lint-strict`
