# Command Reference

This page groups the current top-level `Commands` by task. Use the linked page and `ccr help` output for nested commands.

## Help

```bash
ccr --help
ccr help
ccr help claude profile
ccr <command> --help
```

`ccr help [COMMAND_PATH...]` is a real top-level command, but it does not need a separate page. It resolves nested command paths.

## Runtime And Profiles

| Command | Purpose |
|---|---|
| [`current`](./current) | Claude/Codex runtime overview |
| [`claude`](./claude) | Claude official auth and profile runtime |
| [`codex`](./codex) | Codex auth, profiles, quota, and history sync |
| [`opencode`](./opencode) | OpenCode auth compatibility and Codex import |
| [`platform`](./platform) | platform registry; `list` is the primary stable operation |
| [`switch`](./switch) | legacy profile switch and migration guidance |

## Configuration And Temporary Overrides

| Command | Purpose |
|---|---|
| [`init`](./init) | initialize user-level CCR configuration (`~/.ccr/`) |
| [`list`](./list) | list configurations |
| [`add`](./add) / [`delete`](./delete) | create or delete configurations |
| [`enable`](./enable) / [`disable`](./disable) | change configuration availability |
| [`temp`](./temp) | interactive temporary configuration |
| [`temp-token`](./temp-token) | command-driven temporary token override |
| [`clear`](./clear) | clear CCR-managed settings |
| [`optimize`](./optimize) | normalize configuration structure |

## Project Bootstrap

| Command | Purpose |
|---|---|
| [`project init`](./project-init) | initialize Git, Trellis, and Agent ignore rules in the current directory |

`ccr init` manages user-level CCR configuration. `ccr project init` manages the current project workflow; neither replaces the other.

## Data, Sync, And Operations

| Command | Purpose |
|---|---|
| [`history`](./history) | inspect masked operation history |
| [`export`](./export) / [`import`](./import) | export or import configuration |
| [`clean`](./clean) | clean backup or plan files |
| [`sync`](./sync) | synchronize configuration assets through WebDAV |
| [`sessions`](./sessions) | index, search, resume, and summarize sessions |
| [`provider`](./provider) | test and verify provider connectivity |
| [`stats`](./stats) | summarize, import, export, or clear usage |
| [`budget`](./budget) / [`pricing`](./pricing) | budgets and model pricing |

## Extensions, Diagnostics, And Interfaces

| Command | Purpose |
|---|---|
| [`skills`](./skills) | skill sources, scans, installs, and inventory |
| [`prompts`](./prompts) | prompt preset management |
| [`validate`](./validate) | configuration and runtime validation |
| [`doctor`](./doctor) | environment, profile, auth, and optional online diagnostics |
| [`check`](./check) | cross-platform configuration conflict checks |
| [`ui`](./ui) | launch or update CCR UI |
| [`tui`](./tui) | no-subcommand and platform interactive entrypoints |
| [`update`](./update) | check for or install CCR updates |
| [`version`](./version) | version and build information |

## Recommended Start

```bash
ccr init                    # user-level CCR configuration
ccr project init            # current project workflow
ccr current
ccr claude profile list
ccr codex auth current
ccr validate
ccr doctor
```

## Related Pages

- [CLI Workflows](/en/guide/cli-workflows)
- [Configuration Model](/en/guide/configuration)
- [Migration Guide](/en/reference/migration)
