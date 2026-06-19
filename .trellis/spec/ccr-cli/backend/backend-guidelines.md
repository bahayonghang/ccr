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

`ClaudePlatform::apply_profile` branches on auth mode: `Subscription` calls `clear_managed_vars()` and writes **no** `ANTHROPIC_*` / `CLAUDE_CODE_*`; `ApiKey` calls `update_from_config` and writes the overrides. A third-party profile therefore **only works under `api_key`**.

Auth mode has two layers — keep them separate:

- `ClaudeAuthService::resolve_profile_auth_mode` — literal/stored resolution (explicit `platform_data.auth_mode` wins over inference). Do not change this; tests depend on its literal semantics.
- `ClaudeAuthService::effective_auth_mode` — normalization layer on top of resolve: if resolved is `Subscription` **and** `is_api_key_shaped`, return `ApiKey`. `ClaudePlatform::profile_auth_mode` delegates here so apply / validate / `profile_to_json` stay consistent.

`is_api_key_shaped` is intentionally conservative: `provider_type == "third_party_model"`, or `base_url` and `auth_token` both non-empty. **Do not** include model-mapping fields — `ANTHROPIC_DEFAULT_*_MODEL` is valid on official subscription (snapshot pinning), so that would false-positive and fail `section.validate()`.

Correction happens at two points and must stay idempotent: `normalize_profile` (save — persists the corrected `auth_mode`) and `apply_profile` (defensive — self-heals stale on-disk profiles). Each emits a `tracing::warn` on correction; never log `auth_token` / full `base_url`.

Model-mapping fields are typed on `ProfileConfig` / `ConfigSection` and mapped in `ClaudeSettings::update_from_config`; `custom_model_option`(`_name`) → `ANTHROPIC_CUSTOM_MODEL_OPTION`(`_NAME`). New env keys must also be registered in `ClaudePlatform::get_env_var_names`. Typing a previously-untyped key auto-migrates existing TOML (serde captures it into the typed slot instead of `other`/`platform_data`).

## Testing

Use crate-local `test_support::TestHome` and `TestHostEnv` for env/path-sensitive command tests. These fixtures serialize process env mutation and restore variables on Drop.

Command integration behavior also has tests under `crates/ccr/tests/commands/`; update those when command output or compatibility surfaces change.

## Verification

For CLI command changes, run:

- `just fmt-check`
- `cargo test -p ccr-cli -- --test-threads=1`
- Relevant `cargo test -p ccr --test commands -- --test-threads=1`
- `just lint-strict`
