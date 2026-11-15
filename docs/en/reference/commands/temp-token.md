# temp-token - Command-Line Temporary Override

`ccr temp-token` applies temporary command-line overrides for the active Claude host settings (`~/.claude/settings.json`): token, base URL, and model.

## Usage

```bash
ccr temp-token help
ccr temp-token set <token> [--base-url <url>] [--model <model>]
ccr temp-token show
ccr temp-token clear
```

## Subcommands

| Subcommand | Purpose |
|------------|---------|
| `help` | Show command help |
| `set` | Immediately apply a temporary token / base URL / model override |
| `show` | Inspect the current temporary override state (tokens masked) |
| `clear` | Clear the active temporary override |

## Use Cases

- short-lived tests against a different relay endpoint or temporary credential
- overriding active host settings without modifying the persistent `profiles.db` catalog
- local debugging scripts that apply and clear temporary parameters

## Examples

```bash
ccr claude profile switch work
ccr temp-token set "$TEMP_TOKEN" --base-url https://api.example.com --model claude-sonnet-4-5
ccr temp-token show
ccr temp-token clear
```

## Difference from `ccr temp`

| Command | Style | Best for |
|---------|-------|----------|
| `ccr temp` | interactive prompts | manual step-by-step entry and preview of a temporary config |
| `ccr temp-token` | subcommand flags | inspecting, applying, and clearing explicit temporary overrides |

## Notes

- `set` writes directly to active host settings and leaves `~/.ccr/platforms/claude/profiles.db` unchanged.
- Be mindful of shell history when invoking `set` interactively; store long-lived credentials via `printf '%s' "$TOKEN" | ccr claude profile create <name> ... --auth-token-stdin` and switch with `ccr claude profile switch <name>`.

## Related Commands

- [`temp`](./temp.md) - interactive temporary configuration
- [`claude`](./claude.md) - persistent Claude profile catalog management
- [`current`](./current.md) - inspect current runtime status
