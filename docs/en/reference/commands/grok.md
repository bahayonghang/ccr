# `grok` - Grok Build Profile Runtime

`ccr grok` manages model and third-party provider profiles for Grok Build. CCR only manages `[model.custom]` and `[models].default` in `~/.grok/config.toml`. It never reads or writes `auth.json` or `mcp_credentials.json`.

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
| `ccr grok profile off` | restore the Grok configuration from before CCR profile mode |

## Create A Profile

An official profile only selects a model:

```bash
ccr grok profile create official \
  --model grok-example
```

For a third-party provider, prefer an environment variable reference:

```bash
ccr grok profile create relay \
  --base-url https://api.example.com/v1 \
  --model grok-example \
  --env-key GROK_RELAY_API_KEY \
  --api-backend responses \
  --context-window 1000000 \
  --supports-backend-search

ccr grok profile switch relay
ccr grok profile current --json
```

`api_backend` accepts `chat_completions`, `responses`, or `messages`. `set-field` supports `api_backend`, `env_key`, `context_window`, and `supports_backend_search`; use `--clear` to remove one.

## Credential Boundary

- Prefer `env_key`: CCR stores only the variable name.
- `--auth-token` stores plaintext in CCR profiles, rotating backups, and Grok `config.toml`. Command output still masks or omits it.
- Official profiles reject `auth_token` and `env_key`. Grok owns its login session and `XAI_API_KEY`.
- Displayed URLs omit userinfo, query, and fragment components.

## Examples

- [CCR Grok profiles](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-profiles.toml)
- [Grok config.toml](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-cli-config.toml)
- [Platform migration map](./platform)
