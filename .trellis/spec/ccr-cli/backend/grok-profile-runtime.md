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
    preservation, round-trip restoration, and entry-state non-overwrite.
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
