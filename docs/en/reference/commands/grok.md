# `grok` - Grok Build Profile Runtime

`ccr grok` manages model and third-party provider profiles for Grok Build. CCR manages `[model.custom]`, `[models].default`, and `[models].default_reasoning_effort` in `~/.grok/config.toml`. It never reads or writes `auth.json` or `mcp_credentials.json`.

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
| `ccr grok profile off` | restore the Grok configuration from before CCR profile mode |

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
- Displayed URLs omit userinfo, query, and fragment components.

## Examples

- [CCR Grok profiles](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-profiles.toml)
- [Grok config.toml](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-cli-config.toml)
- [Platform migration map](./platform)
