# Grok Profile Runtime

## 1. Scope / Trigger

- Trigger: changing Grok profile fields, `$GROK_HOME/config.toml` switching,
  `off`/delete behavior, current-profile detection, or credential presentation.
- Applies to `crates/ccr-cli/src/platforms/grok.rs`, shared profile TOML
  parsing in `ccr-config`, CLI/TUI callers, and Grok integration tests.

## 2. Signatures

- `GrokPlatform::new() -> Result<GrokPlatform>`
- `GrokPlatform::{apply_profile, delete_profile, get_current_profile}` through
  `PlatformConfig`
- `GrokPlatform::clear_active_profile_runtime() -> Result<()>`
- `GrokPlatform::profile_auth_mode(&ProfileConfig) -> Result<GrokProfileAuthMode>`
- `GrokPlatform::safe_base_url_for_display(&str) -> String`
- Cross-process operation lock resource: `grok_profile_operation`
- Runtime path: `$GROK_HOME/config.toml`, default `~/.grok/config.toml`
- CCR profile path: `<CCR_ROOT>/platforms/grok/profiles.toml`

## 3. Contracts

- Third-party profiles require non-empty `base_url`, `model`, and exactly one
  of inline `auth_token` or single-string `platform_data.env_key`. They replace
  `[model.custom]` and set `[models].default = "custom"`.
- Official profiles reject `base_url` and credentials. They restore the entry
  `[model.custom]`; an explicit model sets `models.default`, while no model
  removes that key.
- `reasoning_effort` is stored in profile platform data and must be one of the
  Grok Build `ReasoningEffort` values: `none`, `minimal`, `low`, `medium`,
  `high`, `xhigh`, or `max`. Values are trimmed and normalized to lowercase.
  Model-menu option ids are not valid substitutes for their canonical values.
- Third-party profiles with `reasoning_effort` write
  `model.custom.supports_reasoning_effort = true`,
  `model.custom.reasoning_effort`, and `models.default_reasoning_effort`.
  Official profiles write only `models.default_reasoning_effort` and never add
  reasoning fields to the restored built-in model table.
- Entry state captures the exact original `models.default_reasoning_effort`
  TOML value. A profile without `reasoning_effort`, and `off`, restore that
  value or remove the key when it was originally absent. Legacy entry-state
  JSON without this field remains readable and recovers the value from its
  captured original `content`.
- The first apply captures the original runtime configuration in
  `profile_entry_config_state.json`. Creation is create-if-absent CAS: a later
  or concurrent capture may observe `Conflict` but must never replace the
  baseline.
- Runtime writes preserve unknown TOML tables and use content-token CAS with
  one retry. A second conflict returns an actionable error without writing.
- `apply_profile`, `clear_active_profile_runtime`, and `delete_profile` hold the
  same cross-process operation lock across their complete multi-file sequence.
  CAS still handles external Grok/user edits that do not honor this CCR lock.
- Write order is entry state, runtime config, profiles `current_config`, then
  registry pointer. Runtime config is the truth source; a late registry failure
  remains retryable through the profiles pointer and runtime comparison.
- Runtime config writes use `secret: true` and `BackupPolicy::None`. The entry
  state owns restoration; creating same-directory runtime backups would add an
  undisclosed plaintext credential location.
- `off` restores runtime state before clearing pointers. If the entry state is
  missing while activation intent or CCR-managed runtime shape remains, it
  fails closed and leaves pointers/runtime unchanged for manual recovery.
- Delete checks raw registry and profiles intent plus runtime equality without
  calling drift detection. A drifted active profile still requires `off`.
- `auth.json` and `mcp_credentials.json` are never read, written, backed up, or
  validated. Grok owns session authentication.
- Errors and logs never contain tokens, complete credential-bearing TOML
  parser errors, or unsafe base URLs. CLI/TUI URL display uses the shared safe
  helper; inline tokens are not rendered, even masked.

## 4. Validation & Error Matrix

- Third-party missing model/base URL/credential -> `ValidationError`.
- Both `auth_token` and `env_key`, env-key array, invalid env name, unsupported
  backend, non-positive context window, or non-boolean backend-search flag ->
  `ValidationError`.
- Official profile with base URL or either credential -> `ValidationError`.
- Empty, non-string, or non-canonical `reasoning_effort` -> `ValidationError`.
- First runtime CAS conflict -> reload, rebuild, and retry; second conflict ->
  `ValidationError` containing `请重试` and no overwrite.
- Concurrent CCR apply/off/delete -> serialize on `grok_profile_operation`;
  lock timeout -> propagate the lock error without starting a mutation.
- Missing entry state with active intent/managed shape -> `ConfigError`; do not
  clear pointers or delete a profile.
- Malformed runtime/profile TOML -> sanitized `ConfigFormatInvalid`; never echo
  the offending credential line.
- Active or drifted-active delete -> `ValidationError` directing the caller to
  `off` or switch first.

## 5. Good/Base/Bad Cases

- Good: an `env_key` relay switches through `[model.custom]`, preserves `[ui]`
  and `[session]`, then `off` restores the original custom entry and default.
- Good: a relay with `reasoning_effort = "high"` writes all three managed
  reasoning values; switching to an unset profile restores the entry default.
- Good: a registry write fails after runtime apply; `get_current_profile`
  recovers through profiles/runtime, and a retry converges the registry.
- Base: an official profile without a model removes `models.default` and lets
  Grok choose its upstream default.
- Bad: delete through `get_current_profile()` after drift, because that helper
  clears the registry and can bypass the active-intent guard.
- Bad: interpolate `toml::de::Error` into a terminal error; its display text can
  include an `api_key` source line.
- Bad: back up runtime `config.toml` beside itself on each switch; that creates
  extra plaintext copies outside the approved disclosure matrix.

## 6. Tests Required

- `cargo test -p ccr-cli grok -- --test-threads=1`
  - Assert official/third-party validation, field mapping, unmanaged TOML
    preservation, reasoning-effort mapping, round-trip restoration, legacy
    entry-state compatibility, and entry-state non-overwrite.
  - Assert first/second CAS conflict behavior and registry-failure recovery.
  - Assert a second multi-file mutation cannot acquire the operation lock while
    the first owner holds it.
  - Assert missing-state `off` fails closed and drifted inline profiles cannot
    be deleted before `off`.
  - Assert runtime/profile malformed-TOML sentinel values are absent from
    returned errors and unsafe URL forms lose userinfo/query/fragment.
- Run `just fmt-check`, `just lint-strict`, and `just test` before delivery.

## 7. Wrong vs Correct

### Wrong

```rust
if self.get_current_profile()?.as_deref() != Some(name) {
    delete_profile_file_entry(name)?;
}
```

Drift detection can clear the registry before this check, leaving an inline
runtime credential orphaned.

### Correct

```rust
let active_by_intent = registry_current.as_deref() == Some(name)
    || profiles_current.as_deref() == Some(name);
if active_by_intent || runtime_matches_profile(name)? {
    return Err(active_profile_error(name));
}
```

This keeps deletion fail-closed until runtime restoration has completed.

## Scenario: Grok CLI Profile Surface

### 1. Scope / Trigger

- Trigger: changing `ccr grok profile`, shared profile CRUD fields, Grok JSON
  output, force-delete composition, help, or copy-ready Grok examples.

### 2. Signatures

- `Commands::Grok { action: Option<GrokAction> }`
- `GrokProfileAction::{Current,List,Switch,Create,SetField,Enable,Disable,Delete,Off}`
- Create-only fields: `api_backend: Option<String>`,
  `env_key: Option<String>`, `context_window: Option<u64>`, and
  `supports_backend_search: Option<bool>`, and
  `reasoning_effort: Option<String>`.
- Editable platform-data fields: `api_backend`, `env_key`, `context_window`,
  `supports_backend_search`, and `reasoning_effort`.

### 3. Contracts

- The supported command path is `ccr grok profile ...`; retired
  `ccr platform switch/profile` commands continue to return migration errors.
- `api_backend` persists as a lowercase string, `env_key` as one string,
  `context_window` as a positive JSON/TOML integer, and
  `supports_backend_search` as a boolean. `--clear` removes any of them.
- `reasoning_effort` persists as one of the 7 canonical levels, trimmed and
  normalized to lowercase. `--clear` removes it and JSON summaries expose it
  when present.
- `current --json` and `list --json` omit `auth_token`. They expose only the
  stable auth-mode identifier and a URL passed through
  `safe_base_url_for_display`.
- `delete --force` first attempts normal deletion. Only the core
  active-profile rejection authorizes `off` followed by a second delete; do
  not take an unrelated active profile offline when force-deleting an inactive
  item.
- Handwritten `profiles.toml` uses the shared `ConfigSection` encoding:
  `provider_type` is omitted or is `official_relay`/`third_party_model`.
  Grok route selection still comes from `base_url`, not provider type.
- Copy-ready examples use `example.com` and `env_key`; inline secrets are
  disclosure documentation, not example values.

### 4. Validation & Error Matrix

- Unsupported `api_backend` -> Chinese `ValidationError` listing the three
  accepted values.
- Array/comma-shaped `env_key` -> Chinese single-environment-variable error.
- Zero/non-integer `context_window` -> Chinese positive-integer error.
- Backend-search outside `true|false|1|0` -> Chinese boolean error.
- Empty, JSON/non-string, or non-canonical reasoning effort -> Chinese
  validation error listing the allowed values.
- `auth_token` plus `env_key` -> Clap conflict on create or core validation on
  stored/set-field profiles.
- Force delete receives a non-active validation/config error -> propagate it;
  do not run `off`.

### 5. Good/Base/Bad Cases

- Good: create an `env_key` relay, switch, inspect masked JSON, update typed
  fields including `reasoning_effort`, off, and delete.
- Base: official profile with only `model` reports `session` auth mode.
- Bad: serialize `ProfileConfig` directly in CLI JSON because it can expose
  `auth_token` or an unsafe base URL.
- Bad: unconditionally run `off` for every `delete --force`; the target may be
  inactive while another profile is active.

### 6. Tests Required

- `cargo test -p ccr-cli platform -- --test-threads=1`
  - Assert typed parsing, clearing, invalid values, and Grok editable fields.
- `cargo test -p ccr --test commands grok_profile -- --test-threads=1`
  - Assert create/switch/current/list/set/off/delete, output redaction, entry
    restoration, reasoning-effort mapping/clearing, drift detection, drifted
    force deletion, and legacy-route rejection.
- Parse `docs/examples/grok-profiles.toml` through `ccr grok profile list`.
- Run local Grok `inspect --json` with a temporary `GROK_HOME` containing
  `docs/examples/grok-cli-config.toml`; its `configSources` must name that
  isolated file.
- Run the docs build plus `just fmt-check`, `just lint-strict`, and `just test`.

### 7. Wrong vs Correct

#### Wrong

```rust
if force {
    platform.clear_active_profile_runtime()?;
}
platform.delete_profile(name)?;
```

This can turn off a different active profile when the target is inactive.

#### Correct

```rust
match platform.delete_profile(name) {
    Ok(()) => {}
    Err(error) if is_active_profile_error(&error) => {
        platform.clear_active_profile_runtime()?;
        platform.delete_profile(name)?;
    }
    Err(error) => return Err(error),
}
```

The core guard remains the authority for whether force-delete needs runtime
restoration.
