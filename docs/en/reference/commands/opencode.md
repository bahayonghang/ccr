# opencode - OpenCode auth migration and entrypoint

`ccr opencode` is the OpenCode-specific command group. Its current user-facing roles are:

- opening the OpenCode Auth tab in terminal interactive mode
- incrementally importing compatible saved Codex accounts into OpenCode's saved-account registry

## Usage

```bash
ccr opencode
ccr opencode auth import-codex [--dry-run] [--json]
```

## Supported Subcommands

### `ccr opencode`

Without a subcommand, the default TUI-enabled build opens the OpenCode Auth tab directly. Use it when you want an interactive view for inspection, switching, or import preview.

### `ccr opencode auth import-codex`

Imports compatible Codex accounts already saved in CCR into OpenCode's saved-account registry.

Supported options:

| Option | Purpose |
|---|---|
| `--dry-run` | Preview the migration result without writing any OpenCode snapshots or registry entries |
| `--json` | Emit a machine-readable migration report |

## Behavioral Guarantees

- reads only Codex accounts already saved in CCR, not an unsaved runtime login
- imports only compatible ChatGPT OAuth-backed accounts
- skips API-key-only entries, missing snapshots, and invalid snapshots
- checks conflicts by both OpenCode account name and `accountId`
- never overwrites, renames, or deletes existing OpenCode accounts
- never switches the current OpenCode runtime login as part of the import
- CLI and TUI share the same structured migration report

## Examples

```bash
# Preview importable accounts
ccr opencode auth import-codex --dry-run

# Perform the import
ccr opencode auth import-codex

# Emit a JSON report
ccr opencode auth import-codex --json

# Open the OpenCode Auth tab directly
ccr opencode
```

Inside the OpenCode Auth tab, press `i` to preview and confirm importing compatible saved Codex accounts.

## When To Use It

- you already saved multiple Codex accounts with `ccr codex auth save`
- you want OpenCode to reuse those accounts instead of reauthenticating one by one
- you need additive import behavior without affecting the current OpenCode runtime login

## Related Docs

- [`codex`](./codex)
- [`tui`](./tui)
- [CLI Workflows](/en/guide/cli-workflows)
