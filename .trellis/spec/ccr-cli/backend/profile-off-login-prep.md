# Profile Off Login-Prep

> Executable contract for `ccr <platform> profile off` as login-prep cleanup.

## Scenario: Shared login-prep off

### 1. Scope / Trigger

- Trigger: changing `profile_off_for_platform`, `needs_login_prep`, Claude/Codex/Grok off CLI, TUI apply/auth switch, or Tauri `*_profile_off`.
- Applies to `crates/ccr-cli/src/application/profile_off.rs`, platform `clear_active_profile_runtime`, CLI/TUI/Tauri callers.

### 2. Signatures

- `needs_login_prep(platform: Platform) -> Result<bool>`
- `profile_off_for_platform(platform: Platform) -> Result<ProfileOffResult>`
- `ProfileOffResult { platform, previous_profile, changed, runtime_mode, auth_outcome, warnings }`
- CLI: `ccr {claude,codex,grok} profile off [--json]`
- Tauri: `claude_profile_off` / `codex_profile_off` / `grok_profile_off`

### 3. Contracts

- One write core. CLI, TUI, and Tauri must call `profile_off_for_platform`. Do not copy field-clear lists into clients.
- `needs_login_prep` is the only `can_off` source for UI. Frontends must not inspect home files.
- Claude: true when registry/`profiles.toml` pointer is set, or `settings.json` has any `CCR_MANAGED_KEYS`. Clear those keys. Keep user `ANTHROPIC_API_KEY`. Report it in `remaining_suppressors`.
- Codex: true when raw pointer, legacy entry auth snapshot, or CCR third-party runtime exists. A non-official `model_providers.custom` shape counts as third-party runtime even without a bearer or `forced_login_method`. New profile switches do not persist an entry auth snapshot. Off removes the root `model_provider`, the CCR-managed `model_providers.custom` entry, and other CCR profile fields, but preserves `model_reasoning_effort` verbatim, deletes runtime `auth.json`, then discards any legacy entry auth snapshot. Official API-key `auth.json` without pointer, snapshot, or third-party runtime stays unchanged.
- Codex login-prep inspects the resolved Codex dir (`CCR_CODEX_DIR` / `CODEX_HOME` / `~/.codex`). `CCR_CODEX_DIR` is an explicit override and stays the only target. When `CODEX_HOME` redirects away from the default home and `CCR_CODEX_DIR` is unset, off also inspects and clears `%USERPROFILE%\.codex` or `$HOME/.codex`. Official `codex login` and Codex Desktop read that default home. CLI JSON may list `runtime_dirs` and `removed_auth_json` as paths only.
- Grok: true when `inspect_activation_state` is not `Inactive`. Off removes `[model.custom]` and `[models].default`, restores the entry `models.default_reasoning_effort`, and preserves unrelated TOML. When entry state is missing, off performs the same bounded route cleanup but leaves `models.default_reasoning_effort` unchanged. Direct profile deletion still fails closed until off completes.
- Backup dir is `$CCR_ROOT/backups/profile-off/` (fallback `~/.ccr`). Snapshots use `AtomicWriter.secret(true)`. Unix backup dir mode is `0o700`. Codex snapshots include `profiles.toml`, registry, `config.toml`, and `auth.json`.
- `ConfigFileHandler::save` writes `profiles.toml` with `secret: true`.
- JSON/DTO/logs contain no credential values.

### 4. Validation & Error Matrix

- No leftover and no pointer -> success, `changed=false`, no backup dir.
- Grok missing entry state -> remove only `[model.custom]` and `[models].default`, preserve other runtime values, then clear pointers.
- TUI apply/auth switch: off `Err` aborts apply/`switch_account`.
- Non-local Tauri env -> `unsupported_environment`, no write.

### 5. Good / Base / Bad Cases

- Good: API-key or bearer profile then off removes the CCR route and runtime `auth.json`, so native `codex login` starts from a clean state.
- Good: Claude leftover managed env with empty pointer is still cleared.
- Base: repeat off is idempotent.
- Bad: delete user `ANTHROPIC_API_KEY` or official Codex API key with no pointer.
- Bad: print tokens in CLI JSON or Tauri errors.

### 6. Tests Required

- `cargo test -p ccr-cli profile_off -- --test-threads=1`: leftover env, Grok missing-state cleanup, backup under `CCR_ROOT`.
- `cargo test -p ccr --test commands -- {claude,codex,grok}_profile -- --test-threads=1`: switch/off, Codex route and auth deletion, inactive official key kept, inherited `CODEX_HOME` still deletes default-home `auth.json`.
- `cargo test -p ccr-tui -- --test-threads=1`: Profile tab shows `o`; Auth tab does not.
- UI smoke: banner visibility and confirm cancel does not invoke off.

### 7. Wrong vs Correct

#### Wrong

Call `ConfigManager::for_platform` inside `needs_login_prep` pointer reads. That helper can create `current_config = "default"` and make an empty home look off-able.

#### Correct

Read existing `profiles.toml` only. Missing file means no pointer.
