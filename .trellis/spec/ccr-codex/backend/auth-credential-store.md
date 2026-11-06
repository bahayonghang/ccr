# Codex Credential Store And Account Save

## 1. Scope / Trigger

Read when changing Codex credential-store defaults, parsing, auth-state admission,
or `ccr codex auth save`. Domain ownership is `ccr-codex`; CLI presentation and
error propagation belong to `ccr-cli`. Official CLI `file` is the omitted-setting
default, as documented in the [official sample](https://learn.chatgpt.com/docs/config-file/config-sample).

## 2. Signatures

- `CredentialStoreKind::from_config_value(Option<&str>) -> CredentialStoreKind`.
- `CredentialStoreKind::from_toml_value(Option<&toml::Value>) -> CredentialStoreKind`.
- `CodexAuthService::get_auth_state(&self) -> AuthState`.
- `CodexAuthService::save_current(&self, name: &str, description: Option<String>, force: bool) -> Result<()>`.
- CLI: `ccr codex auth save <name> [--description <text>] [--force]`.

## 3. Contracts

- Missing config file or missing top-level store setting selects `File`.
  An unreadable or malformed config is not a missing config.
- TOML callers pass the optional value to `from_toml_value`; do not erase
  non-string values through `and_then(Value::as_str)` before default selection.
- Explicit `auto`/`keyring` stays unsupported for CCR saved-account management,
  even when `auth.json` exists. File existence does not establish which source
  official auto mode selected; do not copy potentially stale file credentials.
- The auth service rejects invalid store types/unknown values and read/parse
  errors before snapshot writes. Diagnostics must not include raw config values,
  TOML source snippets, or auth JSON string values. Raw auth parsing errors used
  by save preflight expose only fixed text and line/column.
  `AuthState.status == Invalid` is authoritative on
  detection errors; the DTO's fallback store field is not an observed backend.
- `save` reports success only after snapshot and registry persistence succeeds.
  Errors propagate to the existing dispatcher, yielding a nonzero process exit
  and one error report. Keep safe duplicate-account overwrite suggestions.
- Do not suggest `codex logout` as recovery for unsupported storage. Missing
  auth may suggest login; invalid config/auth must identify the actual problem.
- Preserve runtime config/auth bytes, extension fields, account registry rules,
  locking, atomic secret writes, and private permissions. Tests use synthetic
  credentials and explicit isolated `CCR_CODEX_DIR`/`CCR_ROOT` paths.
- Shared default users include runtime/profile resolution and auth-off routing.
  Audit these consumers together. Full official layered/managed-policy resolution
  and OS credential-store reading are not implemented by these helpers.

## 4. Validation & Error Matrix

| Input / state | Save outcome |
| --- | --- |
| Missing config or omitted setting, valid file auth | Snapshot and registry saved, exit 0 |
| Explicit file, valid auth | Existing successful behavior |
| Explicit auto/keyring plus any file auth | Unsupported, nonzero, no account write |
| Unknown value, non-string value, malformed/unreadable config | Invalid, nonzero, no account write |
| Missing auth | Nonzero with login guidance |
| Invalid auth JSON | Nonzero with auth diagnostic, not a false logged-out claim |
| Existing account without force | Nonzero, existing snapshot unchanged, safe force suggestion |
| Persistence failure | Nonzero; no success result |

## 5. Good / Base / Bad Cases

- Good: official file login with no explicit setting saves `khanh` without editing config.
- Base: explicit file continues using the established secret writer.
- Bad: infer file mode from a leftover `auth.json` while config explicitly says auto.

## 6. Tests Required

Run the default-store unit regression and binary save regression red before the
fix and green afterward. Cover absent config/setting, explicit file/auto/keyring,
invalid type/value/TOML/read, missing/invalid auth, duplicate/force, and persistence
failure. Assert actual snapshot payload plus registry entry, original bytes,
nonzero error exits, and no credential output. Preserve default test parallelism
and `--skip export_bindings`. Synthetic save evidence is not live token validation.

## 7. Wrong vs Correct

```rust
// Wrong: a boolean value becomes None and accidentally selects file storage.
CredentialStoreKind::from_config_value(root.get("cli_auth_credentials_store").and_then(toml::Value::as_str));

// Correct: absence and an invalid type remain distinguishable.
CredentialStoreKind::from_toml_value(root.get("cli_auth_credentials_store"));
```
