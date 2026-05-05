# CLI Workflows

This page groups CCR's CLI into task-oriented workflows instead of repeating command reference content.

## Workflow 1: Initialize and create the first profile

```bash
ccr init
ccr platform list
ccr platform switch claude
ccr add
ccr list
ccr switch my-profile
```

Use this flow for first-run setup, Unified Mode migration, and creating the first usable profile on a platform.

## Workflow 2: Temporarily change the active settings

### Override token / base URL / model on top of an existing profile

```bash
ccr switch work
ccr temp-token set sk-temp-xxx --base-url https://api.example.com --model claude-sonnet-4-5
ccr current
```

### Write a temporary configuration without relying on a profile

```bash
ccr temp
```

Use this for short-lived tests, provider validation, or quick reproduction steps.

## Workflow 3: Sync and backup

```bash
ccr sync config
ccr sync folder add claude ~/.claude -r /ccr-sync/claude
ccr sync folder enable claude
ccr sync claude push
ccr sync all status
```

Pair it with:

```bash
ccr export -o configs.toml --no-secrets
ccr clean backups --days 30 --dry-run
```

## Workflow 4: Diagnose problems

```bash
ccr validate
ccr provider test --all
ccr provider verify my-provider
ccr history --limit 20
```

If the problem is related to restoring conversations:

```bash
ccr sessions list
ccr sessions search "keyword"
ccr sessions show <id>
ccr sessions resume <id> --dry-run
```

## Workflow 5: Cost and budget control

```bash
ccr stats summary --range week --by-model
ccr pricing list --verbose
ccr budget status
ccr budget set --monthly 200 --warn-at 90 --enable
```

Use this for weekly reporting, billing reviews, and budget threshold checks.

## Workflow 6: Codex multi-account auth

```bash
ccr codex auth save work
ccr codex auth list
ccr codex auth switch work
ccr codex auth current
```

This path is Codex-specific and should be documented as a first-class workflow rather than hidden inside generic profile docs.

## Workflow 7: Migrate saved Codex auth into OpenCode

```bash
# Preview which accounts are importable
ccr opencode auth import-codex --dry-run

# Import compatible saved Codex accounts into OpenCode
ccr opencode auth import-codex

# Emit a machine-readable migration report
ccr opencode auth import-codex --json
```

Use this when:

- you already saved multiple Codex accounts in CCR
- you want OpenCode to reuse those accounts without reauthenticating one by one
- you want additive import semantics instead of overwriting existing OpenCode entries

Behavior boundaries:

- imports only Codex accounts already saved in CCR
- accepts only compatible ChatGPT OAuth-backed accounts
- never overwrites existing OpenCode accounts
- never switches the current OpenCode runtime login
- reports skip reasons for conflicts, missing snapshots, and invalid snapshots

If you want to inspect the OpenCode auth view interactively first:

```bash
ccr opencode
```

Then press `i` on the OpenCode Auth tab to preview and confirm the import inside TUI.

## Workflow 8: Switch to the graphical interface

```bash
ccr ui
ccr
```

Move from CLI to UI when:

- you need the full module map
- you want to jump across multiple capability areas
- you want browser access to skills, monitoring, sessions, statusline, checkin, or opencode

Use `ccr` directly when you want the terminal interactive mode instead of the graphical UI.

## Related Docs

- [Command Overview](/en/reference/commands/)
- [Entrypoints](/en/guide/entrypoints)
- [UI Overview](/en/guide/ui-overview)
