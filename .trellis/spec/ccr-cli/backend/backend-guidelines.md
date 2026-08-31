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

## Scenario: CLI Table Style And Truncation Compatibility

### 1. Scope / Trigger

- Trigger: adding or changing a `comfy-table` table, table preset, width constraint, ANSI color, or command snapshot.
- Applies because `comfy-table 8` replaced positional presets with `TableStyle` and changed its default truncation indicator from `...` to `…`.

### 2. Signatures

- Default ASCII table: `crate::commands::common::new_table() -> comfy_table::Table`.
- Full UTF-8 table: `crate::commands::common::new_utf8_table() -> comfy_table::Table`.
- Style load owned by the shared helper: `Table::load_style(presets::UTF8_FULL)`.

### 3. Contracts

- Command handlers construct tables through `new_table` or `new_utf8_table`; do not call `Table::new` directly.
- Both helpers explicitly retain the CLI's `...` truncation indicator so an upstream default change cannot silently alter script-visible text.
- `new_utf8_table` is the only owner of the full UTF-8 preset. Callers still own headers, column order, padding/constraints, alignment, and content arrangement.
- Captured/non-TTY output remains free of ANSI styling. Explicit TTY styling continues to respect `NO_COLOR`.
- A dependency migration must not change human output intentionally unless the command contract and its tests are updated in the same batch.

### 4. Validation & Error Matrix

- Direct `Table::new` in a CLI table path -> replace it with the matching shared helper before accepting the change.
- `load_preset` or another removed pre-8 style API remains -> compile failure; migrate the style only in the shared helper.
- Narrow table renders `…` instead of `...` -> compatibility regression.
- Captured/non-TTY table contains an ANSI escape -> output regression for pipes, snapshots, and Windows test processes.
- Border, column order, or alignment changes without an explicit command requirement -> reject as an unrelated output change.

### 5. Good/Base/Bad Cases

- Good: a command starts with `let mut table = new_utf8_table();` and configures only its own header, rows, and widths.
- Base: an intentionally ASCII table uses `new_table()` and otherwise keeps the default ASCII style.
- Bad: calling `Table::new().load_style(UTF8_FULL)` locally, because it bypasses the shared truncation contract.

### 6. Tests Required

- `cargo test -p ccr-cli commands::common::table::tests` asserts exact UTF-8 borders/column order, 12-column truncation with `...`, non-TTY plain text, and ANSI/`NO_COLOR` behavior.
- Run `cargo test -p ccr-cli --all-features`, `cargo test -p ccr-tui --all-features`, and `cargo test -p ccr --test commands -- --test-threads=1` after table or dependency changes.
- Run Rust 1.95 check, `just lint-strict`, `just test`, `just tauri-ci`, and both Cargo-lock audits before accepting a `comfy-table` upgrade.

### 7. Wrong vs Correct

#### Wrong

```rust
let mut table = Table::new();
table.load_style(comfy_table::presets::UTF8_FULL);
```

#### Correct

```rust
let mut table = crate::commands::common::new_utf8_table();
```

## Error Handling

Command handlers should return `ccr_core::Result<T>` or `anyhow::Result<T>` where the local command already uses it. Preserve actionable errors from shared crates and handle them at the dispatcher boundary.

Do not use panics for invalid user input. Clap validation, typed command enums, and `CcrError` should carry invalid states.

## Scenario: Project Workflow Bootstrap

### 1. Scope / Trigger

- Trigger: adding or changing `ccr project init`, project-level external-tool orchestration, or the fixed Agent directory ignore rules.
- Applies to the Clap command tree, dispatcher, `commands/project`, command integration tests, and bilingual command reference.

### 2. Signatures

- `Commands::Project { action: ProjectAction }`
- `ProjectAction::Init`
- `project_init_command(auto_yes: bool) -> ccr_core::Result<()>`
- Minimum Trellis postcondition: `<cwd>/.trellis/workflow.md` and `<cwd>/.trellis/scripts/task.py` are files.
- Fixed ignore rules: `.agents/`, `.claude/`, `.codex/`.

### 3. Contracts

- The process current directory is the only project root; do not add upward/downward project discovery or a path argument implicitly.
- Run stages in order: Git -> Trellis -> `.gitignore`. A failed stage stops later stages and never reports overall success.
- `git rev-parse --show-toplevel` detects both a repository root and membership in a parent worktree. Either result skips `git init`; parent membership must not create a nested repository.
- Delegate username and Agent selection to native `trellis init` with inherited stdin/stdout/stderr. Global CCR `--yes` maps only to `trellis init --yes`; CCR must not copy Trellis's platform registry.
- A successful Trellis exit status is necessary but insufficient: validate the minimum postcondition before touching `.gitignore`.
- Merge only missing fixed ignore rules. Preserve existing text and LF/CRLF style, skip the write when unchanged, and use `AtomicWriter` when changed.
- Partial Git/Trellis results are not rolled back. Idempotent retry is the recovery model.

### 4. Validation & Error Matrix

- Git executable missing or `git init` non-zero -> `ExternalCommandError`; do not run Trellis or write `.gitignore`.
- Trellis executable missing or non-zero -> `ExternalCommandError`; keep Git result, do not write `.gitignore`.
- Trellis exits zero but either minimum file is missing -> `ValidationError`; do not write `.gitignore`.
- `.gitignore` read/write fails -> `FileIoError`; keep completed Git and Trellis results.
- All fixed rules already exist -> success without rewriting `.gitignore`.
- Bare `ccr project` -> Clap missing-subcommand failure; never initialize implicitly.

### 5. Good/Base/Bad Cases

- Good: a new directory runs `git init`, native Trellis, then an atomic ignore merge.
- Good: a monorepo child reports the parent Git root, skips nested `git init`, and initializes Trellis in the child directory.
- Base: rerunning after success delegates re-init behavior to Trellis and leaves a complete `.gitignore` byte-for-byte unchanged.
- Bad: checking only `<cwd>/.git` and creating a nested repository inside a parent worktree.
- Bad: treating Trellis exit code zero as complete without checking its minimum files.
- Bad: hard-coding Claude/Codex or any other Trellis platform list in CCR.

### 6. Tests Required

- `cargo test -p ccr-cli project -- --test-threads=1`
  - Assert command parsing stays distinct from legacy top-level `ccr init`.
  - Assert ignore merging covers empty/partial/complete content, missing final newline, CRLF, and no-write idempotence.
- `cargo test -p ccr --test commands project_init -- --test-threads=1`
  - Assert stage order, cwd, parent-worktree behavior, `--yes` forwarding, all failure boundaries, postcondition checks, and retry-safe file results.
  - Fake tools under an isolated child `PATH` must follow `test-fixtures.md`; Unix scripts cannot depend on helpers hidden by that PATH.
- `cargo test -p ccr --test commands help -- --test-threads=1`
  - Assert nested help parity and legacy `ccr init` compatibility.
- Run `just docs-check`, `just lint-strict`, and the final task-appropriate aggregate gate.

### 7. Wrong vs Correct

#### Wrong

```rust
if !root.join(".git").exists() {
    Command::new("git").arg("init").status()?;
}
Command::new("trellis").args(["init", "--claude", "--codex"]).status()?;
fs::write(root.join(".gitignore"), ".agents/\n.claude/\n.codex/\n")?;
```

This can create a nested repository, freezes Trellis platform choices in CCR, discards user ignore rules, and accepts partial Trellis initialization.

#### Correct

```rust
ensure_git_repository(root)?;          // detects parent worktrees
run_trellis_init(root, auto_yes)?;     // native TTY + postcondition
ensure_project_gitignore(root)?;       // merge + no-op + AtomicWriter
```

This preserves ownership boundaries and makes partial failures safely retryable.

## Claude Profile Auth Mode Contract

`ClaudePlatform::apply_profile` branches on auth mode: `Subscription` calls `clear_ccr_managed_vars()` and writes no key from `CCR_MANAGED_KEYS`; `ApiKey` calls `settings.apply_managed_env(section.to_managed_env_pairs())` and writes the registered overrides. Keys outside the registry are user-owned and survive both branches. A third-party profile therefore **only works under `api_key`**.

Auth mode has two layers — keep them separate:

- `ClaudeAuthService::resolve_profile_auth_mode` — literal/stored resolution (explicit `platform_data.auth_mode` wins over inference). Do not change this; tests depend on its literal semantics.
- `ClaudeAuthService::effective_auth_mode` — normalization layer on top of resolve: if resolved is `Subscription` **and** `is_api_key_shaped`, return `ApiKey`. `ClaudePlatform::profile_auth_mode` delegates here so apply / validate / `profile_to_json` stay consistent.

`is_api_key_shaped` is intentionally conservative: `provider_type == "third_party_model"`, or `base_url` and `auth_token` both non-empty. **Do not** include model-mapping fields — `ANTHROPIC_DEFAULT_*_MODEL` is valid on official subscription (snapshot pinning), so that would false-positive and fail `section.validate()`.

Correction happens at two points and must stay idempotent: `normalize_profile` (save - persists the corrected `auth_mode`) and `apply_profile` (defensive - self-heals stale on-disk profiles). Defensive apply must persist the corrected profile through the guarded profiles writer **before** loading or modifying runtime settings. If persistence fails, return an actionable error naming the profile and leave `settings.json` byte-for-byte unchanged. Each correction emits a `tracing::warn`; never log `auth_token` / full `base_url`.

Model-mapping fields are typed on `ProfileConfig` / `ConfigSection` and mapped in `ConfigSection::to_managed_env_pairs` (ccr-config, keyed by `ccr_types::env_keys` constants); `custom_model_option`(`_name`) -> `ANTHROPIC_CUSTOM_MODEL_OPTION`(`_NAME`). New env keys must be added to `ccr_types::env_keys::CCR_MANAGED_KEYS`; `ClaudePlatform::get_env_var_names` derives from that registry. Typing a previously-untyped key auto-migrates existing TOML (serde captures it into the typed slot instead of `other`/`platform_data`).

`ClaudeSettings` itself is `ccr_types::ClaudeSettings` (single workspace shape); `managers/settings.rs` is a pure IO adapter (`SettingsManager`: load/save/backup/restore) plus a re-export, and must not grow local settings types or env-mutation logic.

## Scenario: Claude API-Key Profile Runtime Env

### 1. Scope / Trigger

- Trigger: adding or changing Claude profile fields that write Claude Code environment variables, profile apply behavior, doctor diagnostics, or Claude state-file ownership.
- Applies to `ProfileConfig`, `ConfigSection`, `ccr_config::profile_to_section`, `ccr_config::section_to_profile`, `ClaudeSettings`, `ClaudePlatform`, Tauri Claude profile JSON, and command integration tests.

### 2. Signatures

- `ConfigSection::{default_fable_model, default_*_model_name, claude_code_auto_compact_window, api_timeout_ms, claude_code_disable_nonessential_traffic}`
- `ProfileConfig::{default_fable_model, default_*_model_name, claude_code_auto_compact_window, api_timeout_ms, claude_code_disable_nonessential_traffic}`
- `ConfigSection::to_managed_env_pairs()` (ccr-config)
- `env_keys::CCR_MANAGED_KEYS` (ccr-types)
- `ClaudeSettings::{apply_managed_env, clear_ccr_managed_vars, has_managed_overrides, managed_env_entries}` (ccr-types)
- `ClaudePlatform::get_env_var_names()`
- `ClaudePlatform::apply_profile(name)`
- `ccr_config::ClaudeRuntimePaths::{from_env, resolve_with}` owns all
  user-level Claude settings/credentials/state/backup path priority.
- Test fixtures: `TestHome` must isolate `CLAUDE_CONFIG_DIR`, `CLAUDE_JSON_PATH`, `CCR_SETTINGS_PATH`, and `CCR_BACKUP_DIR`.

### 3. Contracts

- `SettingsManager`, `ClaudePlatform`, `ClaudeAuthService`, and doctor consume
  `ClaudeRuntimePaths`; they must not reimplement environment priority or
  default `.claude` joins. The authoritative contract is in
  `ccr-config/backend/backend-guidelines.md`.
- API-key Claude profiles write only typed, managed env keys into the `env`
  object stored at `ClaudeRuntimePaths::settings_file`; do not add ad hoc env
  writes in command handlers.
- Subscription apply, auth switch, profile off, and lifecycle clear call `clear_ccr_managed_vars()` and remove every key in `CCR_MANAGED_KEYS`, including non-Anthropic runtime keys such as `CLAUDE_CODE_AUTO_COMPACT_WINDOW`, `API_TIMEOUT_MS`, and `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`.
- User-owned keys outside `CCR_MANAGED_KEYS`, including `ANTHROPIC_API_KEY` and `ANTHROPIC_CUSTOM_HEADERS`, are never written or deleted by profile operations. Unknown prefix keys may still affect Claude Code; doctor owns the warning path rather than cleanup guessing ownership.
- `managed_env_entries()` is the single source for lifecycle clear preview, empty-state detection, confirmation count, and removal scope.
- Every new typed env field must be added to both `ProfileConfig` and `ConfigSection`, both conversion directions, an `ccr_types::env_keys` constant, `CCR_MANAGED_KEYS` (and `NON_ANTHROPIC_MANAGED_KEYS` when unprefixed), `ConfigSection::to_managed_env_pairs`, Tauri JSON parse/serialize, UI form state, and provider template mappers when templates can fill it. Registry-to-mapping equality tests must fail if any path drifts.
- Profile apply, auth switching, and doctor must not create or modify
  `ClaudeRuntimePaths::state_file`. In particular, `hasCompletedOnboarding` is
  private Claude Code state; API-key apply leaves an existing state file
  byte-for-byte unchanged and does not create a missing one.
- Tauri Claude MCP user/local mutations are the only CCR-owned state-file
  write surface. They resolve the file through `ClaudeRuntimePaths`, replay
  only the target `mcpServers` subtree on the latest JSON object, and use
  guarded-write CAS for at most three attempts. Top-level fields such as
  `oauthAccount` and `primaryApiKey`, unknown fields, and unrelated project
  entries must round-trip unchanged.
- User/local MCP state writes use `secret: true` with no backup. Project-scope
  `.mcp.json` writes use the same CAS loop with `secret: false` and no backup.
- `ccr doctor` checks API-key profiles for placeholder-looking tokens, active-profile env mismatches, and GLM 1M profiles missing compact-window configuration; it does not diagnose onboarding state.

### 4. Validation & Error Matrix

- Profile token is placeholder-like -> `doctor` warning; do not print or infer a real token.
- Profile expected env differs from `settings.json.env` -> `doctor` warning recommending re-apply.
- GLM model contains `[1m]` and `claude_code_auto_compact_window` is empty -> `doctor` warning recommending `1000000`.
- `.claude.json` missing, unparsable, unreadable, or lacking
  `hasCompletedOnboarding` -> no profile-apply or onboarding-doctor failure;
  the profile/auth path does not own this file.
- An MCP state root is not a JSON object -> return an actionable error and
  leave the file unchanged.
- One or two MCP CAS conflicts -> reread, replay the deterministic subtree
  mutation, and retry. Three conflicts -> return an actionable retry error and
  do not report the operation as successful.
- A mismarked `subscription` profile with API-key shape -> use effective `api_key` in apply, auth switch, and runtime summary; persist the corrected literal before runtime mutation.
- Corrected profile persistence fails -> return an actionable error and do not modify `settings.json`.
- A registered env key is written but not cleared on subscription/off switch -> regression; add it to the registry/mapping equality fixture and switch-cleanup tests.
- An unregistered pair reaches `apply_managed_env` -> ignore it and preserve any existing user-owned value.

### 5. Good/Base/Bad Cases

- Good: typed GLM profile writes `ANTHROPIC_DEFAULT_FABLE_MODEL`, all `*_MODEL_NAME` vars, `CLAUDE_CODE_AUTO_COMPACT_WINDOW`, `API_TIMEOUT_MS`, and `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`, then switching to subscription clears them.
- Good: apply/off/auth switch/clear removes all registered keys while preserving `ANTHROPIC_API_KEY`, `ANTHROPIC_CUSTOM_HEADERS`, and unrelated env entries.
- Good: applying a stale API-key-shaped profile persists `auth_mode = "api_key"` before settings mutation; a retry is idempotent.
- Good: a custom `CLAUDE_CONFIG_DIR` makes profile apply, auth switching, and
  doctor observe the same settings, credentials, and state files.
- Good: API-key apply leaves a state file containing `oauthAccount`,
  `primaryApiKey`, and unknown fields byte-for-byte unchanged.
- Good: a Tauri user/local MCP mutation changes only the selected MCP subtree
  while preserving the rest of the latest state-file object.
- Base: API-key apply succeeds without creating a missing resolved state file.
- Bad: `retain(|key, _| !key.starts_with("ANTHROPIC_"))` in a normal mutation path, because it deletes keys CCR does not own.
- Bad: writing `hasCompletedOnboarding` into either `settings.json` or the
  Claude state file.
- Bad: describing the MCP path lock as protection against Claude Code itself;
  non-cooperating external processes do not acquire CCR's lock.
- Bad: checking `CLAUDE_JSON_PATH` or joining `.claude/settings.json` directly
  in a CLI consumer instead of using `ClaudeRuntimePaths`.
- Bad: storing runtime env keys only in `platform_data` or UI-only state, because apply and doctor cannot round-trip them reliably.

### 6. Tests Required

- `cargo test -p ccr-cli platforms::claude -- --test-threads=1`
  - Assert API-key apply writes expected registered env, preserves unregistered Anthropic keys, leaves existing state bytes unchanged, and does not create a missing state file.
  - Assert corrupt `.claude.json` is unchanged and does not block `settings.json` apply.
  - Assert subscription apply clears every registered key and preserves user-owned keys.
  - Assert stale auth mode persists before runtime mutation and write failure leaves settings bytes unchanged.
- `cargo test -p ccr-cli claude_auth -- --test-threads=1`
  - Assert auth switch and runtime summary use effective auth mode and explicit managed override detection.
- `cargo test -p ccr-config -- --test-threads=1`
  - Assert each typed field maps to the correct env key in `to_managed_env_pairs` and stale keys clear on profile switch (combined with `apply_managed_env`).
- `cargo test -p ccr-cli managers::settings -- --test-threads=1`
  - Assert disk-level read→apply→write→read keeps unknown fields and non-managed env intact.
- `cargo test -p ccr --test commands doctor -- --test-threads=1`
  - Assert placeholder, mismatch, and compact-window warnings without an onboarding warning.
- `cargo test -p ccr-desktop --manifest-path ccr-ui/src-tauri/Cargo.toml claude_mcp -- --test-threads=1`
  - Assert user/local add, update, and delete preserve unrelated state, replay a single conflict, fail after three conflicts, and do not silently lose concurrent mutations.
- `cargo test -p ccr --test commands claude_profile -- --test-threads=1`
  - Assert command-level switch/off behavior remains compatible.

### 7. Wrong vs Correct

#### Wrong

```rust
settings.env.retain(|key, _| !key.starts_with("ANTHROPIC_"));
```

This deletes user-owned Anthropic variables and still misses CCR-managed non-Anthropic runtime keys.

#### Correct

```rust
let preview = settings.managed_env_entries();
settings.clear_ccr_managed_vars();
assert_eq!(preview.len(), removed_count);
```

This keeps preview, count, and cleanup on the same explicit ownership registry while preserving user keys.

## Scenario: Claude Credential Snapshots And Settings CAS

### 1. Scope / Trigger

- Trigger: changing Claude official-auth save/switch/list behavior or any local `settings.json` mutation in CLI/Tauri.
- Applies to `ClaudeAuthService`, `SettingsManager`, Claude profile apply/off/clear/temp flows, and Tauri Claude agents/hooks/plugins/slash/settings/statusline commands.

### 2. Contracts

- Windows and Linux official credentials use `ClaudeRuntimePaths::credentials_file`. macOS save/switch returns an explicit Keychain-not-supported error before touching a credentials file.
- `.credentials.json`, `auth/<name>.json`, and `auth_registry.toml` write through `write_guarded` with `secret: true` and `BackupPolicy::None`; credential fields are `Secret` with explicit plaintext persistence serializers.
- Before switch overwrites an existing credentials file, its stable in-memory identity token must match a valid registry snapshot. Missing current credentials may switch; corrupt or unmatched credentials must be preserved and return guidance to run `ccr claude auth save`.
- Runtime identity comes from the matching snapshot's `oauth_account`. Only an unmatched login may fall back to the current state-file `oauthAccount`; identity display never rewrites Claude's state file.
- A switch writes target credentials, clears CCR-managed settings through `SettingsManager::update_atomic`, and only then updates the registry. Settings failure restores the previously matched credentials snapshot; combined errors contain categories only, never token content.
- `SettingsManager::{update_atomic,update_atomic_async}` own local managed RMW. They use the guarded-write path lock, `secret: true`, centralized `settings` backups, and at most three deterministic conflict replays.
- `save_atomic` is reserved for validated complete replacement and restore paths. Production mutation flows must not use `load -> save_atomic`.
- Tauri Local mutations call the same `SettingsManager` API. SSH/WSL retain their environment-specific read/write protocol and are outside the local cross-process guarantee.
- `LocalEnvironment::write_config` uses the same secret/central-backup policy when performing a complete local Claude settings replacement; it must not leave `*.bak` beside `settings.json`.

### 3. Validation

- `cargo test -p ccr-cli claude_auth -- --test-threads=1`
- `cargo test -p ccr-cli managers::settings -- --test-threads=1`
- `cargo test -p ccr-desktop --manifest-path ccr-ui/src-tauri/Cargo.toml claude -- --test-threads=1`
- Assert A/B/A snapshot metadata follows credentials even while the state file remains stale.
- Inject a real CAS conflict between independent CLI and local Tauri mutations; both fields and unknown user JSON must survive.
- On Unix, assert auth durable files and settings replacements are owner-only. On Windows, verify inherited user-directory ACL behavior without claiming a Unix mode result.

## Scenario: Retired Platform Command Discovery Boundary

### 1. Scope / Trigger

- Trigger: changing `ccr platform`, root/platform help text, or migration
  behavior for the retired global platform-routing commands.
- Applies to `PlatformAction`, `dispatch_platform`, `help_config`, and command
  integration tests.

### 2. Signatures

- Supported discovery actions: `PlatformAction::{Help,List { json }}`.
- Compatibility-only actions:
  `PlatformAction::{Switch,Current,Info,Init,Profile}`.
- Compatibility errors:
  `legacy_platform_command_error(command: &str) -> CcrError` and the dedicated
  `legacy_platform_init_error() -> CcrError`.
- Current profile entry points: `ccr claude profile ...`,
  `ccr codex profile ...`, and `ccr grok profile ...`.

### 3. Contracts

- `ccr platform --help` and `ccr help platform` expose only `help` and `list`;
  they must not advertise retired actions in the Commands list or examples.
- Compatibility-only variants remain in the Clap tree with hidden help
  metadata. A syntactically valid old invocation must still parse and reach
  `dispatch_platform`, which returns the shared migration error.
- Do not delete the hidden variants or restore writes to legacy
  `current_platform` / `default_platform` state.
- Root and platform help direct status checks to `ccr current`, registry
  discovery to `ccr platform list`, and profile work to the explicit
  Claude/Codex/Grok command trees.

### 4. Validation & Error Matrix

- `ccr platform list [--json]` -> execute the supported registry view.
- `ccr platform --help` or `ccr help platform` -> success with identical,
  migration-safe help.
- Valid retired invocation such as `ccr platform init grok` -> non-zero
  `legacy command retired` error containing all three supported
  `ccr <platform> profile init` replacements.
- Retired action with malformed/missing arguments -> normal Clap argument
  error; do not initialize or mutate platform state.
- Unknown platform subcommand -> normal Clap unknown-subcommand error.

### 5. Good/Base/Bad Cases

- Good: an old script receives an actionable migration error while a new user
  cannot discover the retired action through help.
- Base: `ccr platform list` remains visible and unchanged.
- Bad: removing `PlatformAction::Init` makes old calls fail as unknown commands
  and loses the platform-specific migration guidance.
- Bad: leaving a hidden action in custom after-help examples still recommends
  an operation that always fails.

### 6. Tests Required

- `cargo test -p ccr --test commands -- --test-threads=1`
  - Assert root help contains `ccr current`, `ccr platform list`, and all three
    explicit profile help paths.
  - Assert direct and nested platform help are equal and omit every retired
    action from both Commands and examples.
  - Execute `ccr platform init grok`; assert non-zero status, the legacy error,
    and Grok profile migration guidance.
- `cargo test -p ccr-cli --test dispatch_routing -- --test-threads=1`
  keeps supported platform-list routing green.

### 7. Wrong vs Correct

#### Wrong

```rust
pub enum PlatformAction {
    List { json: bool },
}
```

This removes the compatibility parse surface and downgrades old calls to a
generic unknown-subcommand error.

#### Correct

```rust
pub enum PlatformAction {
    List { json: bool },
    #[command(hide = true)]
    Init { platform_name: String },
}
```

The dispatcher then returns `legacy_platform_init_error()` for init and the
shared compatibility error for the other retired actions.

## Testing

Use crate-local `test_support::TestHome` and `TestHostEnv` for env/path-sensitive command tests. These fixtures serialize process env mutation and restore variables on Drop.

Command integration behavior also has tests under `crates/ccr/tests/commands/`; update those when command output or compatibility surfaces change.

## Verification

For CLI command changes, run:

- `just fmt-check`
- `cargo test -p ccr-cli -- --test-threads=1`
- Relevant `cargo test -p ccr --test commands -- --test-threads=1`
- `just lint-strict`
