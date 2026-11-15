# `grok` - Grok Build Auth and Profile Runtime

`ccr grok` manages Grok Build official OAuth session snapshots (`auth`) and SQLite-backed model / third-party provider profiles (`profile`). Profile operations manage `[model.custom]`, `[models].default`, and `[models].default_reasoning_effort` in `~/.grok/config.toml` (or `$GROK_HOME/config.toml`); the account service reads official OAuth credentials from `$GROK_HOME/auth.json` (default `~/.grok/auth.json`). Terminal and JSON output never expose tokens, and CCR never reads, writes, backs up, or validates `mcp_credentials.json`.

## Entrypoints

```bash
ccr grok help
ccr grok auth
ccr help grok auth
ccr help grok profile
```

## Official Auth (`auth`)

| Command | Purpose |
|---|---|
| `ccr grok auth` | Open the Grok Auth tab when a TUI launcher is present; otherwise print help |
| `ccr grok auth save <name> [--scope <scope>] [-f\|--force] [--json]` | Save an official OAuth source in the CCR account store |
| `ccr grok auth list [--json]` | List saved accounts and available local OAuth sources |
| `ccr grok auth switch <name> [--json]` | Restore a saved account for a new Grok session |
| `ccr grok auth delete <name> [-f\|--force] [--json]` | Delete a saved account snapshot without logging out the runtime (supports global `-y`) |
| `ccr grok auth current [--json]` | Report whether an official session file exists (never prints tokens) |
| `ccr grok auth off [--json]` | Log out the current official runtime session |

`auth off` is independent from `profile off` and leaves the profile pointer and `[model.custom]` unchanged. Logout deletes the entire `auth.json` (including other scopes and API-key caches) after updating uniquely identified saved accounts with their latest credentials; CCR saved accounts are retained.

Grok Auth TUI keys (stored in `<CCR_ROOT>/platforms/grok/auth/accounts.json`):
- `s`: save a copy of the selected OAuth scope under an alias (prompts for scope when several exist; overwriting requires confirmation).
- `Enter`: confirm switching to the selected account (end the running Grok session first; replaces only the target scope after preserving identified outgoing credentials).
- `d`: confirm deleting a saved entry while leaving runtime credentials untouched.
- `o`: confirm logging out the entire runtime while retaining saved entries; `r` reloads local state.

A single supported OAuth source is selected automatically; when multiple sources exist, inspect exact scopes via `ccr grok auth list --json` and pass `--scope`. Overwriting an existing alias requires `--force` (global `-y` does not authorize alias replacement). Switching rejects snapshots lacking the `user_id` required by Grok's native format.

```bash
ccr grok auth save gmail
ccr grok auth list --json
ccr grok auth save work --scope '<scope from list>'
ccr grok auth switch gmail
ccr grok auth delete work --force
```

## Profile Runtime (13 Public Actions)

Authoritative profile storage lives at `<CCR_ROOT>/platforms/grok/profiles.db` (default `~/.ccr/platforms/grok/profiles.db`).

| Command | Purpose |
|---|---|
| `ccr grok profile init [--json]` | Initialize `profiles.db` from the embedded inactive template and register the platform in `config.toml` |
| `ccr grok profile open [--json]` | Render `profiles.db` to a `0600` temp TOML projection, open `$VISUAL` / `$EDITOR`, and import changes back |
| `ccr grok profile current [--json]` | Inspect the current Grok profile and runtime state |
| `ccr grok profile list [--json]` | List all Grok profiles (`--json` includes top-level `revision`) |
| `ccr grok profile switch <name> [--json]` | Activate and apply the named profile into `~/.grok/config.toml` |
| `ccr grok profile create <name> [...] [--json]` | Create a new inactive profile (`current_config` unchanged) |
| `ccr grok profile edit <name> [...] [--json]` | Atomically update multiple fields, rename, or rotate/clear credentials |
| `ccr grok profile enable <name> [--json]` | Validate and atomically enable a profile in `profiles.db` |
| `ccr grok profile disable <name> [--force] [--json]` | Disable a profile; the active profile requires `--force` |
| `ccr grok profile delete <name> [--force] [--json]` | Delete a profile; the active profile requires `--force` (runs `off` first) |
| `ccr grok profile off [--json]` | Exit profile mode, remove `[model.custom]` and `[models].default`, and restore prior reasoning effort |
| `ccr grok profile export [NAME]... [...] [--json]` | Pure read-only export to `ExchangeV1` interchangeable TOML |
| `ccr grok profile import [<FILE>\|-] [...] [--json]` | Merge or full-replace import from an exchange or legacy TOML file |

### Initialization and Editor Projection (`init` / `open`)

- **`init [--json]`**: Initializes `~/.ccr/platforms/grok/profiles.db` from the embedded inactive template (`current_config = ""`) and registers `grok` in `~/.ccr/config.toml` without creating a live `profiles.toml` or modifying `~/.grok/config.toml`. `--json` outputs `{ ok, platform, profiles_file, created, registered }`.
- **`open [--json]`**: Renders `profiles.db` to an owner-only (`0600`) temporary TOML file outside the platform directory, opens `$VISUAL` or `$EDITOR`, validates and imports modified TOML back into `profiles.db` on clean exit, and deletes the temp file. `--json` outputs `{ ok, platform, profiles_file, created, registered, editor }`.

### Inspection, Switching, and Lifecycle (`current` / `list` / `switch` / `enable` / `disable` / `delete` / `off`)

- **`current [--json]`**: Shows the active Grok profile; `--json` outputs `{ platform: "grok", profile, runtime_mode: "profile" | "grok_native", details? }`.
- **`list [--json]`**: Outputs `{ "current_profile": "..." | null, "revision": "..." | null, "profiles": [...] }`, where top-level `revision` can be passed to `--expected-revision`.
- **`switch <name> [--json]`**: Atomically switches and applies the profile; `--json` outputs `{ ok, platform, profile, previous_profile, status, revision, warnings, recovery_paths? }`.
- **`enable <name> [--json]`** / **`disable <name> [--force] [--json]`** / **`delete <name> [--force] [--json]`**: Atomically manage enabled state and catalog deletion; `disable` and `delete` protect the active profile unless `--force` is passed (`delete --force` runs `off` first to restore the pre-profile runtime before deleting).
- **`off [--json]`**: Exits Grok profile mode, removing `[model.custom]` and `[models].default` from `~/.grok/config.toml` and restoring the default reasoning effort captured upon entering profile mode. `--json` outputs `{ ok, changed, previous_profile, runtime_mode }`.

### Creating and Editing Profiles (`create` / `edit`)

Official model-selector profile (leaves authentication to Grok):

```bash
ccr grok profile create official --model grok-4
```

Third-party provider profile (pass inline secrets exclusively via stdin with `--api-key-stdin` or its visible alias `--auth-token-stdin`, or specify an environment variable name with mutually exclusive `--env-key <ENV_VAR>`):

```bash
printf '%s' "$GROK_API_KEY" | ccr grok profile create relay \
  --base-url https://api.example.com/v1 \
  --model grok-4 \
  --api-backend responses \
  --context-window 1000000 \
  --reasoning-effort high \
  --supports-backend-search \
  --api-key-stdin

# Or reference an environment variable name instead (mutually exclusive with --api-key-stdin)
ccr grok profile create relay-env \
  --base-url https://api.example.com/v1 \
  --model grok-4 \
  --env-key GROK_RELAY_KEY

ccr grok profile switch relay --json
ccr grok profile current --json
```

- **`create` flags**: `<name>`, `--description <TEXT>`, `--base-url <URL>`, mutually exclusive `--api-key-stdin` (visible alias `--auth-token-stdin`) and `--env-key <ENV_VAR>`, `--model <MODEL>`, `--provider <NAME>`, `--provider-type <TYPE>`, `--account <ACCOUNT>`, repeatable `--tag <TAG>`, `--api-backend <chat_completions|responses|messages>`, `--context-window <TOKENS>`, `--supports-backend-search [true|false]`, `--reasoning-effort <none|minimal|low|medium|high|xhigh|max>`, repeatable `--set FIELD=VALUE` (use `--env-key` rather than `--set` for `env_key`), `--expected-revision <REV>`, `--disabled`, `--json`. Newly created profiles stay inactive.
- **`edit` flags**: `<name>`, `--rename <NEW_NAME>`, `--description <TEXT>`, `--model <MODEL>`, `--base-url <URL>`, `--provider <NAME>`, repeatable `--tag <TAG>`, mutually exclusive `--enable` / `--disable`, mutually exclusive credential flags `--api-key-stdin` / `--auth-token-stdin` / `--env-key <ENV_VAR>` / `--clear-auth-token` (setting inline `api_key` removes `env_key`, and setting `env_key` removes `api_key`), `--api-backend <BACKEND>`, `--context-window <TOKENS>`, `--supports-backend-search [true|false]`, `--reasoning-effort <LEVEL>`, repeatable `--set FIELD=VALUE`, repeatable `--clear FIELD`, `--expected-revision <REV>`, `--no-apply`, `--json`.

```bash
ccr grok profile edit relay \
  --description "Relay" \
  --model grok-4 \
  --base-url https://api.example.com/v1 \
  --reasoning-effort high

printf '%s' "$GROK_API_KEY" | ccr grok profile edit relay --api-key-stdin
ccr grok profile edit relay --env-key GROK_RELAY_KEY
```

Third-party profiles write `[model.custom].reasoning_effort`, derive `[model.custom].supports_reasoning_effort = true`, and synchronize `[models].default_reasoning_effort`; official profiles only set the global default. Editing the active profile re-applies the runtime automatically unless `--no-apply` is set, and disabling the active profile is rejected.

### Exporting and Importing Profiles (`export` / `import`)

```bash
ccr grok profile export --no-secrets -o grok-team.toml
ccr grok profile import grok-team.toml --dry-run
ccr grok profile import grok-team.toml --on-conflict skip --dedup --json
```

- **`export [NAME]... [-t|--tag <TAG>] [-o|--output <FILE>] [--stdout] [--no-secrets] [-y|--yes] [--json]`**: Pure read-only `ExchangeV1` export (`schema_version = 1`, `platform = "grok"`, `secrets_mode = "included" | "redacted"`, `[profiles.<name>]`). Default file `./ccr-grok-profiles-<YYYYMMDD-HHMMSS>.toml` (`0600`); `--stdout` conflicts with `-o` and `--json`; `--json` writes the file and prints the metadata receipt (overwriting an existing `-o` file in non-TTY or `--json` mode without `-y/--yes` exits with code `64`).
- **`import [<FILE>|-] [--stdin] [--on-conflict skip|overwrite|rename] [--dedup] [--replace] [-y|--yes] [--dry-run] [--no-apply] [--expected-revision <REV>] [--json]`**: Accepts `ExchangeV1` (strictly enforces `platform = "grok"`), `LegacyCatalog`, and `LegacySimplified`. Defaults to non-destructive **Merge** (`--on-conflict skip`); `--replace` requires interactive confirmation or `-y/--yes` and conflicts with `--on-conflict` and `--dedup`. Third-party additions missing inline secrets are imported with `enabled = false` (`missing_secrets`, ready to populate via `ccr grok profile edit <name> --api-key-stdin --enable`), compatible overwrites preserve local secrets (`preserved_local_secrets`), and commits write a `0600` backup first. `--json` returns 6+1 bucket `counts`, per-bucket name lists, `backup_path`, `revision`, and `warnings`.

## Credential Boundary

- `api_key` is Grok Build's direct third-party credential field: when supplied via `--api-key-stdin` (or `--auth-token-stdin`), CCR persists it in the SQLite catalog, rotating backups, and `~/.grok/config.toml` when active, while omitting it from all terminal and JSON output.
- `env_key` holds an environment variable name (such as `GROK_RELAY_KEY`)—never put a raw API key value in `env_key`.
- Official profiles reject inline credentials and `env_key`; Grok owns its native login session and `XAI_API_KEY`.
- Profile commands never read or write `auth.json`, and displayed URLs strip userinfo, query, and fragment components.

## Related Pages

- [CCR Grok profiles example](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-profiles.toml)
- [Grok config.toml example](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-cli-config.toml)
- [`current`](./current)
- [`validate`](./validate)
- [`platform`](./platform)
