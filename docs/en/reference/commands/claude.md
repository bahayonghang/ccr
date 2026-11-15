# `claude` - Claude Auth and Profile Runtime

`ccr claude` manages two independent Claude Code runtime paths: official subscription account snapshots (`auth`) and SQLite-backed profile routing (`profile`).

## Entrypoints

```bash
ccr claude
ccr claude help
ccr help claude auth
ccr help claude profile
```

Without a subcommand, TUI-enabled builds open the Claude Auth tab directly; builds without a TUI launcher fall back to listing saved accounts.

## Official Auth

| Command | Purpose |
|---|---|
| `ccr claude auth save <name> [-d <text>] [-f\|--force]` | Save the current official login credential snapshot |
| `ccr claude auth list` | List saved official subscription accounts |
| `ccr claude auth switch <name>` | Switch to a saved official account |
| `ccr claude auth delete <name> [-f\|--force]` | Delete a saved account snapshot; `--force` skips confirmation |
| `ccr claude auth current [--json]` | Show the current official login state |
| `ccr claude auth off [--json]` | Log out the current official runtime session |

An auth snapshot is distinct from an API-key profile. `auth off` is independent from `profile off`: it never deletes saved CCR account snapshots, never modifies profile pointers, and never invokes `profile off`. On Windows and Linux it deletes `.credentials.json`; on macOS it runs `claude auth logout`. Saved snapshots remain restorable via `auth switch`.

## Profile Runtime (13 Public Actions)

Authoritative profile storage lives at `<CCR_ROOT>/platforms/claude/profiles.db` (default `~/.ccr/platforms/claude/profiles.db`).

| Command | Purpose |
|---|---|
| `ccr claude profile init [--json]` | Initialize `profiles.db` from the embedded inactive template and register the platform in `config.toml` |
| `ccr claude profile open [--json]` | Render `profiles.db` to a `0600` temp TOML projection, open `$VISUAL` / `$EDITOR`, and import changes back |
| `ccr claude profile current [--json]` | Inspect the current Claude profile and runtime state |
| `ccr claude profile list [--json]` | List all Claude profiles (`--json` includes top-level `revision`) |
| `ccr claude profile switch <name> [--json]` | Activate and apply the named profile to the runtime |
| `ccr claude profile create <name> [...] [--json]` | Create a new inactive profile (`current_config` unchanged) |
| `ccr claude profile edit <name> [...] [--json]` | Atomically update multiple fields, rename, or rotate/clear secrets |
| `ccr claude profile enable <name> [--json]` | Validate and atomically enable a profile in `profiles.db` |
| `ccr claude profile disable <name> [--force] [--json]` | Disable a profile; the active profile requires `--force` |
| `ccr claude profile delete <name> [--force] [--json]` | Delete a profile; the active profile requires `--force` (runs `off` first) |
| `ccr claude profile off [--json]` | Exit profile mode, clean CCR runtime overrides, and return to official auth |
| `ccr claude profile export [NAME]... [...] [--json]` | Pure read-only export to `ExchangeV1` interchangeable TOML |
| `ccr claude profile import [<FILE>\|-] [...] [--json]` | Merge or full-replace import from an exchange or legacy TOML file |

### Initialization and Editor Projection (`init` / `open`)

- **`init [--json]`**: Populates `profiles.db` from the embedded example template when the catalog does not yet exist (`current_config = ""`, keeping all template entries inactive) and ensures `claude` is registered in `~/.ccr/config.toml`. It never creates a live `profiles.toml` file and never modifies `~/.claude/settings.json`. `--json` outputs `{ ok, platform, profiles_file, created, registered }`.
- **`open [--json]`**: Requires `$VISUAL` or `$EDITOR` (fails without creating any files if neither is set). Renders `profiles.db` (initializing from the template first if absent) into an owner-only (`0600`) temporary TOML file in the system temp directory outside the platform directory; once the editor exits cleanly with modifications, CCR validates and imports the updated catalog into `profiles.db` against the base revision and deletes the temp file. `--json` outputs `{ ok, platform, profiles_file, created, registered, editor }`.

### Inspection, Switching, and Lifecycle (`current` / `list` / `switch` / `enable` / `disable` / `delete` / `off`)

- **`current [--json]`**: Displays the Claude runtime status card; `--json` outputs the structured runtime summary matching the `claude` node of `ccr current --json` (without exposing full secrets).
- **`list [--json]`**: Prints status, name, provider, model, auth source, and enabled state in a table; `--json` outputs:
  ```json
  {
    "current_profile": "work",
    "revision": "sha256:...",
    "profiles": [
      {
        "name": "work",
        "is_current": true,
        "enabled": true,
        "provider": "Example",
        "model": "claude-sonnet-4-5",
        "auth_source": "api_key"
      }
    ]
  }
  ```
  The top-level `revision` (`null` when uninitialized) can be passed to `--expected-revision <REV>` on `create`, `edit`, or `import` for optimistic concurrency control.
- **`switch <name> [--json]`**: Validates the target profile and atomically updates `~/.claude/settings.json` and active pointers. `--json` prints a single-line receipt `{ ok, platform, profile, previous_profile, status, revision, warnings, recovery_paths? }`.
- **`enable <name> [--json]`**: Validates profile completeness under the platform operation lock and sets `enabled = true` without changing the active profile (rejects profiles missing required secrets—such as `--no-secrets` imports—with guidance to populate the secret first). `--json` outputs `{ ok, platform, name, message, enabled, current_profile }`.
- **`disable <name> [--force] [--json]`**: Sets `enabled = false`; disabling the currently active profile is rejected unless `--force` is passed.
- **`delete <name> [--force] [--json]`**: Removes the profile from `profiles.db`; deleting the currently active profile is rejected unless `--force` is passed (`--force` deactivates the active runtime via `profile off` first).
- **`off [--json]`**: Exits CCR profile mode, clears `current_profile` pointers, and removes CCR-managed `ANTHROPIC_*` overrides from `~/.claude/settings.json` (snapshotting touched files under `~/.ccr/backups/profile-off/` with automatic rollback on failure). `ccr claude profile off --json` contract:
  ```json
  {
    "ok": true,
    "changed": true,
    "previous_profile": "work",
    "runtime_mode": "official_auth",
    "remaining_suppressors": [
      {
        "kind": "process_env",
        "location": "ANTHROPIC_API_KEY",
        "confidence": "high",
        "evidence": "present_non_empty",
        "ownership": "external"
      }
    ],
    "warnings": []
  }
  ```
  `remaining_suppressors` reports external auth sources (such as shell environment variables) that CCR never deletes automatically and that may still suppress official Claude login; `warnings` is omitted when empty.

### Creating and Editing Profiles (`create` / `edit`)

Create a third-party API profile (secrets are read exclusively via stdin so they never leak into shell history or process argument lists):

```bash
printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile create relay \
  --base-url https://api.example.com \
  --model claude-sonnet-4-5 \
  --small-fast-model claude-haiku-4-5 \
  --auth-mode api_key \
  --provider Example \
  --tag work \
  --auth-token-stdin

ccr claude profile switch relay --json
ccr claude profile current --json
```

- **`create` flags**: `<name>`, `--description <TEXT>`, `--base-url <URL>`, `--auth-token-stdin`, `--model <MODEL>`, `--small-fast-model <MODEL>`, `--auth-mode <subscription|api_key>`, `--provider <NAME>`, `--provider-type <TYPE>`, `--account <ACCOUNT>`, repeatable `--tag <TAG>`, `--disabled`, repeatable `--set FIELD=VALUE` (rejects inline secret fields), `--expected-revision <REV>`, `--json`. Leaves `current_config` unchanged; `--json` outputs `{ ok, platform, name, message, enabled, current_profile, revision, set_fields }`.
- **`edit` flags**: `<name>`, `--rename <NEW_NAME>`, `--description <TEXT>`, `--model <MODEL>`, `--base-url <URL>`, `--provider <NAME>`, repeatable `--tag <TAG>` (replaces all tags), mutually exclusive `--enable` / `--disable` (disabling the active profile is rejected), repeatable `--set FIELD=VALUE`, repeatable `--clear FIELD`, mutually exclusive `--auth-token-stdin` / `--clear-auth-token`, `--expected-revision <REV>`, `--no-apply`, `--json`.

```bash
ccr claude profile edit relay \
  --description "Relay primary" \
  --set small_fast_model=claude-haiku-4-5 \
  --rename relay-2026

printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile edit relay-2026 --auth-token-stdin
ccr claude profile edit relay-2026 --no-apply --description "storage only"
```

Editing the currently active profile re-applies the runtime automatically unless `--no-apply` is passed. `--json` reports changed field names, old/new names, catalog revision, and runtime replay status without exposing secrets.

### Exporting and Importing Profiles (`export` / `import`)

```bash
# Read-only export: redacted template (safe to share) or private 0600 backup (-o)
ccr claude profile export --no-secrets -o claude-team.toml
ccr claude profile export relay-2026 -o relay-backup.toml
ccr claude profile export relay-2026 --stdout | ssh dev-box ccr claude profile import -

# Preview and merge-import (defaults to --on-conflict skip)
ccr claude profile import claude-team.toml --dry-run
ccr claude profile import claude-team.toml --on-conflict overwrite --dedup --json

# Destructive full replacement (non-TTY or --json requires -y/--yes)
ccr -y claude profile import relay-backup.toml --replace --json
```

- **`export [NAME]... [-t|--tag <TAG>] [-o|--output <FILE>] [--stdout] [--no-secrets] [-y|--yes] [--json]`**:
  - Pure read-only `ExchangeV1` export (`schema_version = 1`, `platform = "claude"`, `secrets_mode = "included" | "redacted"`, `[profiles.<name>]`) with zero lock or migration side effects on `profiles.db`.
  - Positional `[NAME]...` (errors if any name is missing) and repeatable `-t, --tag <TAG>` are mutually exclusive. Exports include inline secrets by default for private local backups; default output path is `./ccr-claude-profiles-<YYYYMMDD-HHMMSS>.toml` (written with owner-only `0600` permissions; rejects symlinks and reparse points).
  - `--no-secrets` strips inline secrets, sensitive extension fields, and URL userinfo while preserving explicit `auth_mode`—always pass `--no-secrets` before sharing or committing files.
  - `--stdout` conflicts with `-o` and `--json` (use `--stdout` only for trusted pipes or with `--no-secrets`; avoid bare shell `> file.toml` redirects, which bypass `0600` permissions); `--json` writes the export file to disk and prints `{ ok, platform, output_path, schema_version, secrets_mode, contains_secrets, exported_count, exported_names, warnings }` to `stdout` (overwriting an existing `-o` target in non-TTY or `--json` mode without `-y/--yes` exits with code `64`).
- **`import [<FILE>|-] [--stdin] [--on-conflict skip|overwrite|rename] [--dedup] [--replace] [-y|--yes] [--dry-run] [--no-apply] [--expected-revision <REV>] [--json]`**:
  - Accepts three input formats: `ExchangeV1` (strictly enforces `platform = "claude"`), `LegacyCatalog` (`default_config` / `current_config`), and `LegacySimplified` (profile sections only), rejecting masked secret placeholders.
  - Defaults to non-destructive **Merge** (`--on-conflict skip`; `overwrite` updates matching profiles while preserving `usage_count`; `rename` adds `<name>-imported`; `--dedup` skips incoming entries whose normalized spec matches an existing profile).
  - `--replace` performs full-catalog replacement, conflicts with `--on-conflict` and `--dedup`, requires interactive confirmation or `-y/--yes` (mandatory with `--stdin`, `--json`, or non-TTY), and protects the active profile from removal or disablement.
  - New entries missing required inline secrets are imported with `enabled = false` (`missing_secrets`); compatible overwrites preserve local secrets (`preserved_local_secrets`); commits create an owner-only (`0600`) backup first and re-apply the active profile unless `--no-apply` is set.
  - `--json` returns 6+1 bucket `counts` (`total_input`, `total_incoming`, `added`, `updated`, `unchanged`, `renamed`, `skipped_conflict`, `skipped_duplicate`, `skipped`, `removed`, `disabled_missing_secret`), per-bucket name lists, `missing_secrets`, `preserved_local_secrets`, `reapplied_profile`, `backup_path`, `base_revision`, `revision`, and `warnings`.

## Auth Mode Boundary

- A `subscription` profile clears CCR-managed `ANTHROPIC_*` and related Claude Code environment overrides.
- An `api_key` profile writes typed fields into the `env` block of `~/.claude/settings.json`.
- Third-party endpoints should use `api_key`; when both `base_url` and an auth token exist, CCR normalizes obviously mismatched legacy configurations at runtime.
- Terminal output, diagnostics, and `--json` payloads never print full tokens.

## Related Pages

- [Configuration Model](/en/guide/configuration)
- [CLI Workflows](/en/guide/cli-workflows)
- [`current`](./current)
- [`validate`](./validate)
- [`doctor`](./doctor)
- [`platform`](./platform)
