# Diagnosis and baseline evidence

## Root cause

CCR treats an omitted `cli_auth_credentials_store` as `auto`, then rejects non-file account management before looking at `auth.json`. The official CLI sample documents `file` as the default. This is independent of whether the user has already signed in.

- Source: `crates/ccr-codex/src/models/codex_auth.rs`, `CredentialStoreKind::from_config_value`.
- Source: `crates/ccr-codex/src/services/codex_auth_service.rs`, `detect_credential_store`, `get_auth_state`, `ensure_managed_auth_supported`.
- CLI: `crates/ccr-cli/src/commands/codex/auth/save.rs`, unsupported/missing/save-error branches return `Ok(())` in baseline.
- Official definition: https://learn.chatgpt.com/docs/config-file/config-sample — Authentication & Login: `file (default) | keyring | auto`.
- Official storage distinction: https://learn.chatgpt.com/docs/auth — auto prefers OS credential storage, otherwise file; an existing file alone does not establish that explicit auto is using it.

## Observed local state (read-only metadata/status)

- `ccr --version`: 7.5.0. `codex --version`: 0.167.0.
- `codex login status` and `codex -c 'cli_auth_credentials_store="file"' login status`: both `Logged in using ChatGPT`.
- TOML parsed successfully; top-level store setting absent; auth.json exists; CODEX_HOME and CCR_CODEX_DIR overrides absent.
- No personal credential contents were printed or copied. No login/logout/save was run against real accounts.
- These status checks prove local credential recognition, not a fresh network token validity check or the eventual installed CCR behavior.

## Baseline checks

- `just version-check`: PASS.
- `just fmt-check`: PASS (JSON formatting checks plus root and Tauri rustfmt).
- Initial unrelated change: untracked `ccr-desktop/tests/runtime_shutdown.rs`; concurrent GPUI task changes remain outside ownership.
- Build uses the host's managed mbx cache outside the repo; sandbox access may require a scoped command escalation. Do not change global build-cache configuration to bypass this.

## Delivery boundary

Test synthetic credentials with the compiled CLI. Do not overwrite installed CCR or edit actual Codex config/credentials as part of verification. Give the user the verified binary invocation and distinguish source changes from installed behavior.
