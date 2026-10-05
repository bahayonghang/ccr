# `grok` - Grok Build Profile Runtime

`ccr grok` manages Grok Build model profiles, official session status and logout, and saved accounts in the CLI and TUI. Profile operations manage `[model.custom]`, `[models].default`, and `[models].default_reasoning_effort` in `~/.grok/config.toml`; the account service reads official OAuth credentials from `$GROK_HOME/auth.json`, defaulting to `~/.grok/auth.json`. UI and command output do not expose tokens. CCR does not read, write, back up, or validate `mcp_credentials.json`.

## Official Auth

| Command | Purpose |
|---|---|
| `ccr grok auth` | open the Grok Auth tab when a TUI launcher is present; otherwise print help |
| `ccr grok auth save <name>` | save an official OAuth source; supports `--scope`, `--force`, and `--json` |
| `ccr grok auth list` | list saved accounts and available runtime sources; supports `--json` |
| `ccr grok auth switch <name>` | restore a saved account for a new Grok session; supports `--json` |
| `ccr grok auth delete <name>` | confirm deletion of a saved entry without logout; supports `--force`, global `-y`, and `--json` |
| `ccr grok auth current` | report whether an official session file exists; supports `--json`; does not print the token |
| `ccr grok auth off` | log out the current official runtime; supports `--json` |

`auth off` is independent from `profile off` and leaves the profile pointer and `[model.custom]` unchanged. Logout deletes the entire `auth.json`, including other scopes and API-key caches. Uniquely identified saved accounts are updated with their latest credentials first, and CCR saved accounts are retained. It does not change `mcp_credentials.json`.

The Grok Auth TUI supports official OAuth personal and team accounts:

- `s`: save a complete copy of the selected current scope under an alias in `<CCR_ROOT>/platforms/grok/auth/accounts.json` (default `~/.ccr/platforms/grok/auth/accounts.json`). Grok may keep running and using the current account. Choose a source when several scopes exist; overwriting an alias requires confirmation.
- `Enter`: confirm switching to the selected account. End the current Grok session yourself first; the written credentials are intended for a new session. Only the target scope is replaced, after preserving the latest credentials of an identified outgoing account. Unsaved or uncertain identities require an explicit save first.
- `d`: confirm deleting a CCR saved entry, leaving runtime credentials unchanged.
- `o`: confirm logout of the entire runtime while retaining CCR saved entries; `r` only reloads local state.

A local match does not verify server authentication. Switching preserves token timestamps, does not refresh or log in, and does not exit a third-party profile. Profiles or environment variables may still determine the authentication route. Enterprise OIDC, external, API-key, and legacy web_login entries are outside saved-account management.

The CLI and TUI share one account store. A single supported OAuth source is selected automatically. With multiple sources, use `list` to discover the exact scope and pass `--scope`; CCR never picks the first source implicitly. Replacing an alias requires `--force`; global `-y` does not authorize overwriting it. Saving and deleting leave runtime credentials unchanged.

```bash
ccr grok auth save gmail
ccr grok auth list --json
ccr grok auth save work --scope '<scope from list>'
ccr grok auth save gmail --force
# End the current Grok session before switching and starting a new one.
ccr grok auth switch gmail
ccr grok auth delete work --force
```

Failures return a nonzero exit code. `list --json` exposes saved accounts, OAuth sources, and local runtime status without tokens. The `logged_in` field in `current --json` still means only that the session file exists. Codex description updates, rename, explicit sync, repair, and import/export are not implemented for Grok. Switching and logout already preserve the latest credentials of identified accounts through the shared service.

A copy with missing identity metadata can be saved explicitly. Switching rejects a copy without the `user_id` required by Grok's native format and never invents an identity. Save again after official login produces complete credentials.

## Commands

| Command | Purpose |
|---|---|
| `ccr grok profile current` | show the current profile; supports `--json` |
| `ccr grok profile list` | list profiles; supports `--json` |
| `ccr grok profile switch <name>` | apply a profile |
| `ccr grok profile create <name>` | create a profile |
| `ccr grok profile set-field <name> <field>` | update or clear one field |
| `ccr grok profile enable <name>` | enable a profile |
| `ccr grok profile disable <name>` | disable a profile |
| `ccr grok profile delete <name>` | delete a profile; active profiles require off or `--force` |
| `ccr grok profile open` | Open profiles.toml in your editor; creates the file from the template if it does not exist |
| `ccr grok profile off` | leave profile mode and remove `[model.custom]` and `[models].default` |

## Create A Profile

An official profile only selects a model:

```bash
ccr grok profile create official \
  --model grok-example
```

For a third-party provider, use Grok Build's `api_key` field directly:

```bash
ccr grok profile create relay \
  --base-url https://api.example.com/v1 \
  --model grok-example \
  --api-key sk-your-grok-relay-api-key \
  --api-backend responses \
  --context-window 1000000 \
  --reasoning-effort high \
  --supports-backend-search

ccr grok profile switch relay
ccr grok profile current --json
```

`api_backend` accepts `chat_completions`, `responses`, or `messages`. `reasoning_effort` accepts Grok Build's canonical `none`, `minimal`, `low`, `medium`, `high`, `xhigh`, and `max` levels; other values are rejected. `set-field` supports `api_backend`, `api_key`, `env_key`, `context_window`, `supports_backend_search`, and `reasoning_effort`; use `--clear` to remove one:

```bash
ccr grok profile set-field relay reasoning_effort --value high
ccr grok profile current --json
```

For third-party profiles, CCR writes `[model.custom].reasoning_effort`, derives `[model.custom].supports_reasoning_effort = true`, and synchronizes `[models].default_reasoning_effort`. Official profiles only set the global default. Switching to a profile without the field or running `off` restores the default reasoning effort captured on entry to profile mode.

## Credential Boundary

- `api_key` is Grok Build's direct credential field. `--api-key` stores plaintext in CCR profiles, rotating backups, and Grok `config.toml`, while command output omits it. The old `--auth-token` spelling remains a compatibility alias.
- `env_key` remains available for an environment variable name; do not put an API key value in it.
- Official profiles reject `api_key`, `auth_token`, and `env_key`. Grok owns its login session and `XAI_API_KEY`.
- The account service can save and restore official OAuth scopes; `auth off` protects identified saved accounts before deleting the entire `auth.json`. Neither changes `mcp_credentials.json`. Profile commands still do not read or write `auth.json`.
- Displayed URLs omit userinfo, query, and fragment components.

## Examples

- [CCR Grok profiles](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-profiles.toml)
- [Grok config.toml](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-cli-config.toml)
- [Platform migration map](./platform)
