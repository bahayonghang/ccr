# codex - Codex Multi-Account Management

`ccr codex` is the Codex-specific command group. Its main user-facing surface today is the `auth` subcommand family.

## Usage

```bash
ccr codex
ccr codex auth <ACTION> [OPTIONS]
```

## Supported Subcommands

### `ccr codex`

Without a subcommand, CCR enters the default Codex interaction path; when the TUI feature is enabled, it can act as the Codex account-management entrypoint.

### `ccr codex auth`

| Subcommand | Purpose |
|------------|---------|
| `save <name>` | Save the current `~/.codex/auth.json` as a named account |
| `list` | List saved accounts |
| `switch <name>` | Switch to a saved account |
| `delete <name>` | Delete a saved account |
| `current` | Show the current account |
| `export` | Export accounts to JSON |
| `import` | Import accounts from JSON |

## Examples

```bash
# Save the current login
ccr codex auth save work

# Save with description and expiration
ccr codex auth save personal -d "Personal GitHub account" --expires-at 2026-02-01T00:00:00Z

# List and switch
ccr codex auth list
ccr codex auth switch work
ccr codex auth current

# Import and export
ccr codex auth export --no-secrets
ccr codex auth import --replace
```

## When to Use It

- One developer manages multiple GitHub / Codex identities
- A shared machine needs explicit account switching
- You want to export or import Codex auth state for backup or migration

## Related Docs

- [Platform Support](/en/reference/platforms/)
- [UI Module Map](/en/guide/ui-modules)
