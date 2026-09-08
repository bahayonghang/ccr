# Auth Off Official Logout

> Executable contract for `ccr <platform> auth off` as official-runtime logout.

## Scenario: Shared official auth off

### 1. Scope / Trigger

- Trigger: changing `auth_off_for_platform`, `needs_auth_off`, Claude/Codex/Grok auth off CLI, TUI Auth `o`, or Tauri `*_auth_off`.
- Applies to `crates/ccr-cli/src/application/auth_off.rs`, Grok auth current, CLI/TUI/Tauri callers.
- Does not apply to `profile_off_for_platform`. The two cores must not call each other.

### 2. Signatures

- `needs_auth_off(platform: Platform) -> Result<bool>`
- `auth_off_for_platform(platform: Platform) -> Result<AuthOffResult>`
- `AuthOffResult { platform, changed, path: File | NativeLogout, profile_pointer, warnings }`
- CLI: `ccr {claude,codex,grok} auth off [--json]`
- CLI: `ccr grok auth current [--json]`
- Tauri: `claude_auth_off` / `codex_auth_off` / `grok_auth_off` / `grok_auth_current`
- UI capability field: `can_auth_off` (never reuse profile `can_off`)

### 3. Contracts

- One write core. Clients must not copy delete lists or spawn official logout themselves.
- Grok account operations serialize on the CCR store operation lock; switch/off
  additionally hold the nontruncating official `auth.json.lock`. Shared Grok off
  calls the service coordinator exactly once, then the already-locked deletion
  kernel. Recognized saved credentials are persisted before deleting the entire
  runtime file. Saved accounts remain; unknown/malformed runtime may be explicitly
  cleared. Store corruption or failed outgoing persistence blocks deletion.
- `needs_auth_off` is the only `can_auth_off` source. Frontends must not inspect home files.
- Claude Win/Linux: true when credentials file exists as a non-empty object or diagnosis says logged in. Off deletes `ClaudeRuntimePaths.credentials_file`. Do not change `~/.claude.json` onboarding fields.
- Claude macOS: `needs_auth_off` is always true. Off spawns `claude auth logout`. Do not read or write Keychain.
- Codex file: true when any directory from `CodexPlatform::login_prep_codex_dirs()` contains `auth.json`. Off deletes `auth.json` in every directory from that same list. Do not change `config.toml` or profile pointers.
- Codex keyring/auto: `needs_auth_off` is always true. Off spawns `codex logout`.
- Grok: true when `$GROK_HOME/auth.json` exists (default `~/.grok/auth.json`). Off may existence-read, secret-backup, and delete that file. `mcp_credentials.json` is never read, written, backed up, or validated.
- File `changed`: true only if this run deleted at least one credential file. No file → success, `changed=false`, no backup, no spawn.
- Native `changed`: true when this run spawned official logout and the process exited 0. Repeat runs may still report `changed=true`.
- File backup lives under `$CCR_ROOT/backups/auth-off/` (fallback `~/.ccr`), Unix dir `0o700`, `AtomicWriter.secret(true)`. Drop restores unless `commit()`. After successful commit, delete that snapshot directory. Native path creates no backup.
- Spawn: inherit PATH, no shell, stdin closed, do not echo stdout/stderr, 15s timeout, no flags, never `login`. Missing binary, timeout, or non-zero exit → `Err`.
- JSON/DTO/logs contain no credential values. `path` is `"file"` or `"native_logout"`.
- Codex with a profile pointer still logs out runtime credentials; JSON may include `profile_pointer` plus a warning that `profile switch` is required to rewrite the key. That warning is not a failure.
- Tauri non-local: `ensure_local_env` returns `unsupported_environment`; no write and no spawn.
- ccr-ui confirm: Claude/Codex `warning`; Grok `danger`. Cancel must not invoke.

### 4. Validation & Error Matrix

- File store, no credential file → success, `changed=false`, no backup dir.
- File write failure before commit → original file restored; snapshot not left as the live credential.
- File success → live credential gone; `auth-off` snapshot dir for this run gone.
- Native missing CLI → error; CCR account snapshots unchanged.
- Claude/Grok third-party profile → session file gone; profile pointer and Claude managed env / Grok `[model.custom]` remain.

### 5. Good / Base / Bad Cases

- Good: Win/Linux Claude logged in, off deletes `.credentials.json`, current reports not logged in, onboarding fields unchanged.
- Good: Codex `CODEX_HOME` redirect with leftover default-home `auth.json`, `needs_auth_off` is true and both dirs are cleared.
- Base: file off twice → second `changed=false`.
- Bad: call `profile_off_for_platform` from auth off.
- Bad: reuse `can_off` for auth off.
- Bad: print tokens, keep successful file backups under CCR_ROOT, or write Grok `mcp_credentials.json`.

### 6. Tests Required

- `cargo test -p ccr-cli auth_off -- --test-threads=1`
- `cargo test -p ccr --test commands -- {claude,codex,grok}_auth -- --test-threads=1` as named after implementation
- Codex login-prep second directory coverage
- Backup rollback and post-success snapshot removal
- Native missing-binary error
- UI: `can_auth_off`; Grok confirm `danger`; cancel does not invoke

### 7. Wrong vs Correct

#### Wrong

Reuse `needs_login_prep` / `can_off` to decide whether to show auth logout.

#### Correct

`needs_auth_off` → `can_auth_off`. Profile leftover and official login are independent flags.

## Scenario: Grok saved-account service

### 1. Scope / Trigger

Applies to local Grok OAuth account save/switch/delete and coordinated logout in
`services/grok_auth_service/accounts.rs`; profile and MCP ownership stays separate.

### 2. Signatures

`ccr_cli::services::grok_auth_service` exports `GrokAuthSnapshot`,
`GrokAuthAccount`, `GrokAuthSource`, opaque `GrokAuthRevision`, and
`GrokAuthMutation { warnings, outgoing_saved }`. Public account fields contain
only alias/scope, local email/team metadata, saved/expiry time and local-match
flags; revisions redact their Debug output. No DTO exposes credential objects.

Mutation signatures are `save_current(name: &str, source_scope: &str,
revision: &GrokAuthRevision, replace: bool) -> Result<GrokAuthMutation>`,
`switch_account(name: &str, revision: &GrokAuthRevision) -> Result<GrokAuthMutation>`,
`delete_account(name: &str, revision: &GrokAuthRevision) -> Result<GrokAuthMutation>`,
and `off_checked(revision: &GrokAuthRevision) -> Result<AuthOffResult>`.

### 3. Contracts

- `read_snapshot() -> Result<GrokAuthSnapshot>` is read-only, including missing
  files. A malformed runtime becomes `runtime_error`; malformed account storage
  is an error rather than an empty library. Runtime existence is not proof of
  authentication. Existing `current().logged_in` remains existence-only.
- `save_current(name, source_scope, &revision, replace)` captures one complete
  official OAuth scope and writes only CCR `platforms/grok/auth/accounts.json`.
  It holds no native lock and permits subsequent native refresh. Duplicate names
  require explicit replace; duplicate known identities require updating the
  original alias. Multiple sources must be selected explicitly by the caller.
- `switch_account(name, &revision)` validates the target before writes. The
  outgoing selected scope must exactly or conservatively uniquely match a saved
  account; email alone, missing/unknown user identity, enrichment differences,
  and ambiguous candidates never authorize inferred ownership. Save outgoing
  before CAS-writing only the selected runtime scope. Same-account switching
  saves latest runtime and never restores stale bytes.
- `delete_account(name, &revision)` updates only the CCR library; it does not log
  out. `off_checked(&revision)` checks the confirmed snapshot then follows the
  shared entire-runtime logout kernel. The unchecked explicit CLI off shares
  the same coordinator and preserves recognizable outgoing saved accounts.

All mutations reread and compare both observed content revisions after the CCR
operation lock. Switch/off then use the shared native lock before rereading;
leaf guarded writes perform CAS. Save/delete never create that native lock.
An external writer that ignores locks remains outside the concurrency guarantee;
switching is for a new session after the user stops Grok writers.

### 4. Validation & Error Matrix

Store failure prevents runtime mutation. If outgoing store replacement is visible
but durability is unconfirmed, retain the new store and stop before runtime
write/delete. A final write error is reread: matching target bytes means applied
but durability unconfirmed; unreadable state is unknown and requires refresh.
Runtime failure after outgoing save retains the latest saved account. No current
pointer or compensating rollback overwrites externally changed runtime data.
Unknown fields and original token times survive; known native field types are
validated. A snapshot without user_id may be saved but cannot be restored until
Grok supplies a native-compatible snapshot. No refresh/login/profile-off occurs.

### 5. Good / Base / Bad Cases

- Good: save A, save B, capture refreshed A before switching to B, and restore that
  latest A on switching back; preserve other runtime scopes and unknown fields.
- Base: missing library is empty, deleting a saved item leaves runtime intact.
- Bad: infer outgoing identity from email, or overwrite unsaved runtime credentials.

### 6. Tests Required

Fixture sources cover captured-save/native-refresh, other scope/config/MCP/profile
preservation, outdated revisions, unsupported outgoing schema, identity enrichment,
store corruption/write failure, runtime CAS failure, outgoing durability warnings,
shared operation locking, duplicate identities, and saved-account-preserving off.
Windows ACL and native-lock metadata fixtures live in core. These tests are not
execution evidence: the 09-08 task explicitly skips tests/builds/lint/runtime.

### 7. Wrong vs Correct

Wrong: treat an outgoing store durability warning as permission to delete runtime.
Correct: retain visible saved bytes, report uncertainty, and require a fresh action.
