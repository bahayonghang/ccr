# temp-token - Command-Line Temporary Override

`ccr temp-token` applies temporary command-line overrides for the currently active settings: token, base URL, and model.

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
| `set` | Immediately apply a temporary token / base URL / model |
| `show` | Show the current temporary override state |
| `clear` | Clear the current temporary override state |

## Use Cases

- short-lived tests against a different token or relay endpoint
- overriding the active settings without editing a permanent TOML profile
- automation scripts that need explicit temporary parameters

## Examples

```bash
ccr switch work
ccr temp-token set sk-temp-xxx --base-url https://api.example.com --model claude-sonnet-4-5
ccr temp-token show
ccr temp-token clear
```

## Difference from `ccr temp`

| Command | Style | Best for |
|---------|-------|----------|
| `ccr temp` | interactive | quick manual input of a temporary configuration |
| `ccr temp-token` | command-line | scripts, copy/paste workflows, explicit flag-driven overrides |

## Notes

- `set` applies directly to the active settings.
- This command is for temporary overrides, not long-lived profile management.
- Permanent configuration should still be managed through `ccr add`, `ccr switch`, and `profiles.toml`.
