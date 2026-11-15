# `codex` - Codex Runtime and Multi-Account Management

`ccr codex` is the Codex-specific command group covering official OAuth multi-account snapshots, SQLite-backed profile routing, quota checks, session trash recovery, stale `app-server` / runtime drift repair, and history provider-namespace bridging:

- `ccr codex auth ...`: official OAuth account snapshots, sync, repair, relogin, and JSON export/import
- `ccr codex profile ...`: Codex profile catalog and runtime routing (all 13 public actions)
- `ccr codex env [name]` / `quota` / `sessions`: shell export script, quota lookup, and session trash
- `ccr codex fix`: stale `app-server` cleanup and local profile/runtime consistency diagnosis
- `ccr codex sync-history ...`: history visibility repair across `openai` / `custom` provider namespaces

## Entrypoints

```bash
ccr codex
ccr codex help
ccr help codex auth
ccr help codex profile
```

Without a subcommand, TUI-enabled builds launch the Codex TUI directly.

## Official Auth (`auth`)

| Command | Purpose |
|---|---|
| `ccr codex auth save <name> [-d <text>] [-f\|--force]` | Save the current `~/.codex/auth.json` as a named account snapshot |
| `ccr codex auth update <name> [-d <text> \| --clear-description] [--json]` | Update or clear an account description without overwriting credentials |
| `ccr codex auth list` | List saved accounts and current login state |
| `ccr codex auth sync` | Write current runtime OAuth tokens back to the matching saved account |
| `ccr codex auth repair <name>` | Scan `~/.codex/auth.json` and backups to repair rotated OAuth tokens |
| `ccr codex auth switch <name> [--restart-daemon]` | Switch to a saved official account; optionally restart `app-server` |
| `ccr codex auth delete <name> [-f\|--force]` | Delete a saved account snapshot without logging out the active runtime |
| `ccr codex auth rename <old> <new> [-f\|--force] [--json]` | Atomically rename account snapshot, registry entry, and usage attribution |
| `ccr codex auth current [--json]` | Inspect the current `~/.codex/auth.json` login state |
| `ccr codex auth off [--json]` | Log out the current official runtime session |
| `ccr codex auth relogin` | Remove local `auth.json`, stop `app-server`, and run `codex login` |
| `ccr codex auth export [--no-secrets]` | Export saved official accounts to a JSON file |
| `ccr codex auth import [--replace] [-f\|--force]` | Import saved official accounts from a JSON file |

`ccr codex auth off` is independent from `profile off` and leaves the profile pointer and `config.toml` route untouched. Even in third-party profile mode, it clears runtime `auth.json` (for file store) or runs `codex logout` (for keyring / auto). `--json` may report a remaining `profile_pointer` warning; run `profile switch` again to rewrite the profile credential.

`ccr codex auth relogin` switches or adds an official account safely: because `codex login`, `codex logout`, and the Codex TUI `/logout` revoke the active refresh token on the server, `relogin` deletes the local `auth.json` first (without calling `codex logout`), stops the `app-server` daemon, and then runs `codex login` so previously saved snapshots keep valid refresh tokens. Supported only when `cli_auth_credentials_store = "file"` (rejected for keyring / auto); aborts if the daemon cannot be stopped. Save the new session afterward with `ccr codex auth save <name>`.

## Profile Runtime (13 Public Actions)

Authoritative profile storage lives at `<CCR_ROOT>/platforms/codex/profiles.db` (default `~/.ccr/platforms/codex/profiles.db`), paired with the platform secret store (`profile_secrets.json`) for inline credentials.

| Command | Purpose |
|---|---|
| `ccr codex profile init [--json]` | Initialize `profiles.db` from the embedded inactive template and register the platform in `config.toml` |
| `ccr codex profile open [--json]` | Render `profiles.db` to a `0600` temp TOML projection, open `$VISUAL` / `$EDITOR`, and import changes back |
| `ccr codex profile current [--json]` | Inspect the current Codex profile and runtime state |
| `ccr codex profile list [--json]` | List all Codex profiles (`--json` includes top-level `revision`) |
| `ccr codex profile switch <name> [--json]` | Activate and apply the named profile into the Codex runtime |
| `ccr codex profile create <name> [...] [--json]` | Create a new inactive profile (`current_config` unchanged) |
| `ccr codex profile edit <name> [...] [--json]` | Atomically update multiple fields, rename, set reasoning effort, or rotate/clear secrets |
| `ccr codex profile enable <name> [--json]` | Validate and atomically enable a profile in `profiles.db` |
| `ccr codex profile disable <name> [--force] [--json]` | Disable a profile; the active profile requires `--force` |
| `ccr codex profile delete <name> [--force] [--json]` | Delete a profile; the active profile requires `--force` (runs `off` first) |
| `ccr codex profile off [--json]` | Exit profile mode and clean the Codex profile route and runtime `auth.json` |
| `ccr codex profile export [NAME]... [...] [--json]` | Pure read-only export to `ExchangeV1` interchangeable TOML |
| `ccr codex profile import [<FILE>\|-] [...] [--json]` | Merge or full-replace import from an exchange or legacy TOML file |

### Initialization and Editor Projection (`init` / `open`)

- **`init [--json]`**: Initializes `~/.ccr/platforms/codex/profiles.db` from the embedded inactive template (`current_config = ""`) and registers `codex` in `~/.ccr/config.toml` without creating a live `profiles.toml` or touching `~/.codex/`. `--json` outputs `{ ok, platform, profiles_file, created, registered }`.
- **`open [--json]`**: Renders `profiles.db` to an owner-only (`0600`) temporary TOML file outside the platform directory, opens `$VISUAL` or `$EDITOR`, validates and imports modified TOML back into `profiles.db` on clean exit, and removes the temp file. `--json` outputs `{ ok, platform, profiles_file, created, registered, editor }`.

### Inspection, Switching, and Lifecycle (`current` / `list` / `switch` / `enable` / `disable` / `delete` / `off`)

- **`current [--json]`**: Shows the Codex runtime status card; `--json` outputs the structured runtime state matching the `codex` node of `ccr current --json`.
- **`list [--json]`**: Outputs `{ "current_profile": "..." | null, "revision": "..." | null, "profiles": [...] }`, where top-level `revision` supports optimistic concurrency via `--expected-revision`.
- **`switch <name> [--json]`**: Atomically switches and applies Codex runtime routing and credentials; `--json` outputs `{ ok, platform, profile, previous_profile, status, revision, warnings, recovery_paths? }`.
- **`enable <name> [--json]`** / **`disable <name> [--force] [--json]`** / **`delete <name> [--force] [--json]`**: Atomically manage enabled state and catalog deletion; `disable` and `delete` protect the active profile unless `--force` is passed (`delete --force` runs `off` first).
- **`off [--json]`**: Exits Codex profile mode, removing CCR profile routing from `~/.codex/config.toml` and CCR-managed runtime `auth.json` files. `--json` outputs `{ ok, changed, previous_profile, runtime_mode, runtime_dirs, removed_auth_json }`.

### Creating and Editing Profiles (`create` / `edit`)

Secrets are read exclusively via stdin (`--auth-token-stdin`) and never passed as plaintext command-line arguments:

```bash
printf '%s' "$OPENAI_API_KEY" | ccr codex profile create relay \
  --base-url https://api.example.com/v1 \
  --model gpt-5.4 \
  --auth-mode openai_api_key \
  --set model_reasoning_effort=high \
  --tag work \
  --auth-token-stdin

ccr codex profile edit relay \
  --description "Work relay" \
  --model gpt-5.4 \
  --base-url https://api.example.com/v1 \
  --reasoning-effort high

printf '%s' "$OPENAI_API_KEY" | ccr codex profile edit relay --auth-token-stdin
ccr codex profile switch relay --json
```

- **`create` flags**: `<name>`, `--description <TEXT>`, `--base-url <URL>`, `--auth-token-stdin`, `--model <MODEL>`, `--small-fast-model <MODEL>`, `--auth-mode <MODE>`, `--provider <NAME>`, `--provider-type <TYPE>`, `--account <ACCOUNT>`, repeatable `--tag <TAG>`, `--disabled`, repeatable `--set FIELD=VALUE` (e.g. `model_reasoning_effort=high`, `wire_api=responses`), `--expected-revision <REV>`, `--json`. Newly created profiles remain inactive.
- **`edit` flags**: `<name>`, `--rename <NEW_NAME>`, `--description <TEXT>`, `--model <MODEL>`, `--base-url <URL>`, `--provider <NAME>`, repeatable `--tag <TAG>`, mutually exclusive `--enable` / `--disable`, `--reasoning-effort <LEVEL>` (writes `model_reasoning_effort`), repeatable `--set FIELD=VALUE`, repeatable `--clear FIELD`, mutually exclusive `--auth-token-stdin` / `--clear-auth-token`, `--expected-revision <REV>`, `--no-apply`, `--json`. Editing the active profile re-applies the runtime unless `--no-apply` is set; `--json` never includes secrets.

### Exporting and Importing Profiles (`export` / `import`)

```bash
ccr codex profile export --no-secrets -o codex-team.toml
ccr codex profile import codex-team.toml --dry-run
ccr codex profile import codex-team.toml --on-conflict skip --dedup --json
```

- **`export [NAME]... [-t|--tag <TAG>] [-o|--output <FILE>] [--stdout] [--no-secrets] [-y|--yes] [--json]`**: Overlays `profile_secrets.json` in memory without disk or lock side effects and emits `ExchangeV1` (`schema_version = 1`, `platform = "codex"`, `secrets_mode = "included" | "redacted"`, `[profiles.<name>]`). Default file `./ccr-codex-profiles-<YYYYMMDD-HHMMSS>.toml` (`0600`); `--stdout` conflicts with `-o` and `--json`; `--json` writes the file and prints the metadata receipt (overwriting an existing `-o` file in non-TTY or `--json` mode without `-y/--yes` exits with code `64`).
- **`import [<FILE>|-] [--stdin] [--on-conflict skip|overwrite|rename] [--dedup] [--replace] [-y|--yes] [--dry-run] [--no-apply] [--expected-revision <REV>] [--json]`**: Accepts `ExchangeV1` (strictly enforces `platform = "codex"`), `LegacyCatalog`, and `LegacySimplified`. Defaults to non-destructive **Merge** (`--on-conflict skip`); `--replace` conflicts with `--on-conflict` and `--dedup` and requires interactive confirmation or `-y/--yes`. Missing-secret additions are imported with `enabled = false` (`missing_secrets`), compatible overwrites preserve local secrets (`preserved_local_secrets`), and commits atomically synchronize `profiles.db` + `profile_secrets.json` after creating a `0600` pre-write backup. `--json` returns 6+1 bucket `counts`, `missing_secrets`, `preserved_local_secrets`, `reapplied_profile`, `backup_path`, `revision`, and `warnings`.

## Environment Script, Quota, and Session Trash (`env` / `quota` / `sessions`)

| Command | Purpose |
|---|---|
| `ccr codex env [<name>]` | Print shell environment export statements for the current or named profile |
| `ccr codex quota [-a\|--account <name>] [--refresh] [--json]` | Query API quota balance (5h window / weekly limit) across all or one saved account |
| `ccr codex sessions trash <session_id>... [--codex-home <dir>]` | Move sessions into the CCR-managed trash |
| `ccr codex sessions trash-list [--codex-home <dir>]` | List restorable sessions in the CCR trash |
| `ccr codex sessions restore <session_id>... [--codex-home <dir>]` | Restore trashed sessions back to Codex |

## `fix`

```bash
# Switch the target profile first; bare fix neither writes runtime files nor runs upstream doctor
ccr codex profile switch future
ccr codex fix

# Explicitly repair local drift that can be handled safely
ccr codex fix --repair-runtime

# Preview process cleanup and runtime replay without signals or file writes
ccr codex fix --dry-run --repair-runtime

# Run upstream codex doctor only when extra evidence is needed
ccr codex fix --doctor
```

The process stage only matches native Codex or the Node Codex wrapper running `app-server` under the current user—never `codex exec`, `codex resume`, `codex login`, or unrelated tools mentioning `codex app-server` in arguments. Cleanup sends TERM first, re-polls every 300 ms for up to ~3 seconds, and sends KILL to remaining matches at the deadline; after any signal, CCR waits ~1 second for a settle snapshot and records newly spawned identities under `respawned` (exit code `2`). Owner, PID start time, and argv are re-verified before every signal; if they cannot be read safely, CCR reports `process_state = unavailable` and stops signaling.

Diagnostics separate profile pointers, route, credential consistency, and provider validity, comparing only locally stored secrets against the active credential source without printing keys, masked fragments, lengths, or fingerprints. Upstream `codex doctor` is not invoked unless `--doctor` is passed (`provider_auth_validity = not_checked` is not a failure), and `--repair-runtime` does not imply `--doctor`.

| Exit code | Meaning |
|---|---|
| `0` | No confirmed local drift; provider validity may still be unchecked |
| `1` | CCR runtime inspection or repair failed |
| `2` | An `app-server` remains, or process discovery/cleanup could not complete safely |
| `3` | Local profile/runtime drift remains, or the snapshot changed during doctor |
| `127` | `--doctor` was passed and `codex` is not on `PATH` |

## `sync-history`

Repairs old-session visibility in Codex CLI / Desktop after `openai` / `custom` provider-namespace changes, and provides `status`, `restore <backup_dir> [--restore-state]`, and `prune-backups [--keep <N>]` subcommands.

```bash
# Explicitly target one provider, defaulting to the last 7 days (adjust via --max-age-days <N>)
ccr codex sync-history --provider custom --dry-run
ccr codex sync-history --provider openai

# Bridge mode: bridge openai/custom/missing-provider history into the current runtime provider
ccr codex sync-history --bridge official-custom --dry-run
ccr codex sync-history --bridge official-custom --all-history

# Diagnose provider, SQLite, preview, cwd, Desktop first-page limits, and encrypted_content
ccr codex sync-history status
```

- `--provider` and `--bridge` are mutually exclusive; `--bridge official-custom` resolves `openai` for official/implicit OpenAI runtimes and `custom` for third-party profiles.
- `--all-history` disables the 7-day age filter; SQLite repair only touches `openai`, `custom`, and missing-provider rows unless repeatable `--include-provider <name>` is passed.
- Write mode backs up rollout first lines, `state_5.sqlite`, and `.codex-global-state.json` (prunable via `--keep <N>`); `--dry-run` prints the plan only; `encrypted_content` is counted and warned about without decrypting or modifying message bodies.

## Related Pages

- [Configuration Model](/en/guide/configuration)
- [CLI Workflows](/en/guide/cli-workflows)
- [`current`](./current)
- [`validate`](./validate)
- [`doctor`](./doctor)
- [`platform`](./platform)
