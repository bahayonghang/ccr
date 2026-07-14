# `claude` - Claude Auth And Profile Runtime

`ccr claude` manages two distinct Claude Code runtimes: official subscription account snapshots and API-key profiles.

## Entrypoints

```bash
ccr claude
ccr claude help
ccr help claude auth
ccr help claude profile
```

Without a subcommand, TUI-enabled builds enter the Claude Auth tab. Calls without a TUI launcher fall back to the account list.

## Official Auth

| Command | Purpose |
|---|---|
| `ccr claude auth save <name>` | save the current official login snapshot |
| `ccr claude auth list` | list saved accounts |
| `ccr claude auth switch <name>` | switch official accounts |
| `ccr claude auth delete <name>` | delete an account; `--force` skips confirmation |
| `ccr claude auth current` | show the current official login; supports `--json` |

`save` accepts `--description <text>` and `--force`. An auth snapshot is not an API-token profile.

## Profile Runtime

| Command | Purpose |
|---|---|
| `ccr claude profile current` | show the current profile/runtime; supports `--json` |
| `ccr claude profile list` | list profiles; supports `--json` |
| `ccr claude profile switch <name>` | apply a profile |
| `ccr claude profile create <name>` | create a profile |
| `ccr claude profile set-field <name> <field>` | update or clear one field |
| `ccr claude profile enable <name>` | enable a profile |
| `ccr claude profile disable <name>` | disable a profile; the current item requires `--force` |
| `ccr claude profile delete <name>` | delete a profile; supports `--force` |
| `ccr claude profile off` | leave profile mode and return to official auth runtime |

Create a third-party API profile:

```bash
ccr claude profile create work \
  --base-url https://api.example.com \
  --auth-token "$ANTHROPIC_AUTH_TOKEN" \
  --model claude-sonnet-4-5 \
  --auth-mode api_key

ccr claude profile switch work
ccr claude profile current --json
```

`create` also supports description, small-fast-model, provider, provider-type, account, repeated `--tag`, `--disabled`, and `--json`. Run `ccr help claude profile create` for the current complete option set.

`set-field` accepts `--value`, array-oriented `--value-json`, or `--clear`; they are mutually exclusive.

## Auth Mode Boundary

- A `subscription` profile clears CCR-managed `ANTHROPIC_*` and related Claude Code environment overrides.
- An `api_key` profile writes typed fields into `~/.claude/settings.json.env`.
- Third-party profiles should use `api_key`; when both base URL and auth token exist, CCR applies runtime normalization to obviously mismatched older configurations.
- Output and diagnostics must not print full tokens.

## Related Pages

- [Configuration Model](/en/guide/configuration)
- [CLI Workflows](/en/guide/cli-workflows)
- [`current`](./current)
- [`doctor`](./doctor)
