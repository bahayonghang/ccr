# Codex Provider Bearer Runtime

> Executable contracts for provider-scoped bearer credentials, DeepSeek-compatible root fields, secret persistence, cleanup, and runtime diagnosis.

## Scenario: Provider bearer profile apply and reconciliation

### 1. Scope / Trigger

- Trigger: changing Codex profile auth modes, `SwitchSpec`, provider table generation, `model_catalog_json` / `preferred_auth_method`, config or backup writers, Tauri Codex settings/MCP writes, or bearer diagnosis.
- The initial consumer is DeepSeek Responses API, but the runtime contract is provider-generic.
- `model_provider` remains the fixed CCR runtime id `custom`; provider branding does not create a new runtime provider id.

### 2. Signatures

- `CodexProfileAuthMode::ProviderBearerToken` serializes as `provider_bearer_token`.
- `AuthSelection::WriteProviderBearerToken(Secret)` carries plaintext; `RouteSelection` remains credential-free and may derive `Debug`.
- `SwitchSpec` includes non-secret `model_catalog_json: Option<String>` and `preferred_auth_method: Option<String>`.
- Profile `platform_data` keys: `auth_mode`, `model_catalog_json`, `preferred_auth_method`, and `forced_login_method`.
- Runtime provider field: `[model_providers.custom].experimental_bearer_token`.
- Tauri `EXPLICIT_PLATFORM_STRING_FIELDS` and profile projection include all four platform keys; projected `extra` excludes their duplicates.

### 3. Contracts

- Bearer mode requires a non-empty `auth_token` and a third-party `base_url`; official OpenAI profiles reject it.
- `normalize_auth_fields` derives `preferred_auth_method = "apikey"`, `forced_login_method = "api"`, and `requires_openai_auth = false`, then clears `env_key` and `openai_login_method`. Explicit supported values win.
- `apply_switch_spec` rebuilds `[model_providers.custom]` and writes the bearer only from `AuthSelection`. It clears OpenAI tokens, `OPENAI_API_KEY`, and provider keys from `auth.json` instead of duplicating the bearer there.
- Plaintext may exist only in the runtime config and backups, the CCR runtime secret store, the explicit typed profile-editor prefill, and the explicit Raw Source editor. Logs, errors, diagnosis/status payloads, dashboard DTOs, and `extra` must not contain it.
- `CodexConfigManager` config/auth writes and backups, the runtime secret store, Tauri Codex config writes, and Unified MCP Codex writes use `AtomicWriter.secret(true)`. Secret policy is applied to the temporary file before content is written.
- `model_catalog_json` is written unchanged. Only absolute paths and `~/` or `~\` paths are checked for existence; a missing file warns after a successful switch and never blocks or rewrites the path.
- Applying another profile rebuilds the provider table and managed root fields. `ccr codex profile off` removes `model_catalog_json`, `preferred_auth_method`, `forced_login_method`, and the bearer before restoring the entry auth state.
- Runtime diagnosis compares root/route fields separately from the bearer credential. It reports only `config:experimental_bearer_token`, and `--repair-runtime` replays the saved profile through the normal atomic apply path.
- `ccr profile current --verbose` falls back to the profile auth source when `auth.json` reports `NoAuth`; config bearer credentials are not observable from `CodexAuthService::get_auth_state()` alone.

### 4. Validation & Error Matrix

- Bearer mode without `auth_token` -> `ValidationError`; runtime and profile pointers remain unchanged.
- Bearer mode without `base_url` or on an official profile -> `ValidationError`.
- `preferred_auth_method` outside `apikey | chatgpt` -> `ValidationError`.
- Missing absolute or expanded-home model catalog -> visible warning; switch still succeeds.
- Missing or different runtime bearer -> credential `missing` / `mismatch`, `repairable = true`; no secret value appears in the diagnostic.
- Pointer conflict, missing saved secret, or unreadable profile -> repair stays disabled; diagnosis does not guess.
- `--dry-run --repair-runtime` -> report only; config, auth, registry, profile store, and secret store remain byte-for-byte unchanged.

### 5. Good / Base / Bad Cases

- Good: a DeepSeek profile writes `model_provider = "custom"`, the derived root auth fields, and one provider bearer; `auth.json` contains no provider secret.
- Good: switching bearer -> official -> bearer is idempotent and restores the same expected runtime each time.
- Base: a non-bearer profile may use `model_catalog_json` or an explicit supported `preferred_auth_method` without a provider bearer.
- Bad: add the plaintext bearer to `RouteSelection`, a `Debug` DTO, `extra`, a log message, or a provider template.
- Bad: write config or backup bytes first and chmod afterward.
- Bad: infer provider validity from a local field match; `provider_auth_validity` remains `not_checked`.

### 6. Tests Required

- `cargo test -p ccr-codex -- --test-threads=1`: apply/clear/idempotency, auth cleanup, route/credential diagnosis, repair, secret-free Debug/JSON, path warning, and Unix owner-only files.
- `cargo test -p ccr-cli render_auth_source_uses_profile_bearer_source_when_auth_json_is_empty -- --test-threads=1`: verbose current output uses the config bearer source.
- `cargo test -p ccr --test commands codex_ -- --test-threads=1`: binary switch/off and repair behavior, including no secret in stdout/stderr.
- Tauri Codex/Unified MCP tests: named field projection does not duplicate `extra`; typed config round trips preserve unknown bearer/root fields.
- `cd ccr-ui && bun run test:smoke -- tests/codex-profile-editor.smoke.test.ts tests/provider-templates.smoke.test.ts`: form round trip, derived defaults, cleanup on mode change, and template secret exclusion.
- `python scripts/check-secret-writes.py`, `just tauri-bindings-check`, and final `just ci` must pass.

### 7. Wrong vs Correct

#### Wrong

```rust
#[derive(Debug)]
enum RouteSelection {
    ThirdParty { bearer_token: String },
}

std::fs::write(&config_path, content)?;
ensure_private_permissions(&config_path);
```

#### Correct

```rust
enum AuthSelection {
    WriteProviderBearerToken(Secret),
}

AtomicWriter::new(&config_path)
    .secret(true)
    .write_string(&content)?;
```
