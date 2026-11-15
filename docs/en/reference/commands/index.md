# Command Reference

This page lists all **public canonical top-level commands** in CCR grouped by responsibility. For nested flags and JSON output schemas, open the linked command page or run `ccr help`.

## Help (`ccr help`)

```bash
ccr --help
ccr help
ccr help claude profile
ccr <command> --help
```

`ccr help [COMMAND_PATH...]` is a public top-level command that resolves multi-level subcommand paths directly (for example `ccr help codex profile import`) without needing a separate reference page.

## Platforms & Init

| Command | Purpose |
|---|---|
| [`platform`](./platform) | Inspect the recognized platform registry (`ccr platform list [--json]`) and legacy migration mappings |
| [`claude`](./claude) | Claude official subscription account snapshots (`auth`) and profile runtime routing (`profile` — 13 actions) |
| [`codex`](./codex) | Codex official accounts (`auth`), profile routing (`profile`), quota, session trash, `fix`, and `sync-history` |
| [`grok`](./grok) | Grok Build official OAuth accounts (`auth`) and model / third-party provider profile routing (`profile`) |
| [`project init`](./project-init) | Initialize Git, workflow engine (`mini-trellis` / `trellis`), ignore rules, and local `ref/` in the current repo via the four-stage wizard |

User-level platform profile catalogs are initialized per platform via `ccr <claude|codex|grok> profile init`; repository workflow bootstrapping uses `ccr project init`. Neither replaces the other.

## Status, Diagnostics & Overrides

| Command | Purpose |
|---|---|
| [`current`](./current) | Side-by-side Claude and Codex runtime overview (visible aliases: `status`, `show`) |
| [`validate`](./validate) | Cross-platform read-only validation for the registry and Claude/Codex/Grok profiles & runtimes (`--json` supported) |
| [`doctor`](./doctor) | Unified local environment, profile, auth, conflict, and optional `--online` health diagnostics |
| [`check`](./check) | Cross-platform environment variable and configuration conflict checks (`ccr check conflicts`) |
| [`temp`](./temp) | Interactive temporary runtime override without modifying persistent profile catalogs |
| [`temp-token`](./temp-token) | Command-driven temporary token override management (`set` / `show` / `clear`) |

## Data, Sync & Sessions

| Command | Purpose |
|---|---|
| [`history`](./history) | Inspect masked configuration operation audit history |
| [`clean`](./clean) | Clean planning files (`planfiles`), expired backups (`backups`), and rebuildable storage caches (`storage`) |
| [`sync`](./sync) | Synchronize configuration assets over WebDAV (`config` / `status` / `push` / `pull`) |
| [`sessions`](./sessions) | Index, search, resume, and summarize CLI sessions |
| [`provider`](./provider) | Test provider endpoint connectivity and verify credentials |

For profile catalog backup and exchange, use `ccr <claude|codex|grok> profile export` and `ccr <claude|codex|grok> profile import`.

## UI, Cost & Maintenance

| Command | Purpose |
|---|---|
| [`ui`](./ui) | Launch or update CCR UI (Web / Tauri graphical workbench) |
| [`tui`](./tui) | Terminal UI launcher (`ccr` with no subcommand and platform TUI entrypoints) |
| [`stats`](./stats) | Summarize, import, export, or clear token usage and cost analytics |
| [`budget`](./budget) | Monitor and configure daily/monthly API cost budgets |
| [`pricing`](./pricing) | Manage custom per-model input/output pricing tables |
| [`skills`](./skills) | Scan, install, and manage Agent Skills inventories |
| [`prompts`](./prompts) | Manage and apply system prompt presets |
| [`update`](./update) | Check for or install CCR updates |
| [`version`](./version) | Display detailed version, build metadata, and enabled features (alias: `ver`) |

## Recommended Start

```bash
ccr claude profile init                     # initialize user-level Claude profile catalog (or codex / grok)
ccr project init                            # current repo workflow wizard (recommends mini-trellis)
ccr project init --workflow mini-trellis    # explicitly select a project workflow engine
ccr status                                  # cross-platform runtime overview (alias for ccr current)
ccr claude profile list                     # list Claude profiles
ccr codex auth current                      # check Codex official login status
ccr grok profile list                       # list Grok profiles
ccr validate                                # read-only cross-platform validation
ccr doctor                                  # comprehensive environment diagnostics
```

## Deprecated & Retired Entrypoints (7.x Compatibility)

The following 11 legacy top-level entrypoints are hidden from `--help` in `7.x` (emitting deprecation warnings on `stderr` or returning retirement errors, and scheduled for removal in `8.0`) and are documented solely as migration references:

| Legacy entrypoint | `7.x` status | Canonical replacement |
|---|---|---|
| [`switch`](./switch) | Retired (returns error) | `ccr <claude\|codex\|grok> profile switch <name>` |
| [`init`](./init) | Deprecated & hidden | `ccr <claude\|codex\|grok> profile init` / `ccr project init` |
| [`list`](./list) | Deprecated & hidden | `ccr <claude\|codex\|grok> profile list` / `ccr current` |
| [`add`](./add) | Deprecated & hidden | `ccr <claude\|codex\|grok> profile create <name>` or interactive `ccr` |
| [`delete`](./delete) | Deprecated & hidden | `ccr <claude\|codex\|grok> profile delete <name>` |
| [`enable`](./enable) | Deprecated & hidden | `ccr <claude\|codex\|grok> profile enable <name>` |
| [`disable`](./disable) | Deprecated & hidden | `ccr <claude\|codex\|grok> profile disable <name>` |
| [`clear`](./clear) | Deprecated & hidden | `ccr <claude\|codex\|grok> profile off` |
| [`optimize`](./optimize) | Deprecated & hidden | SQLite `profiles.db` maintains canonical order automatically; use `ccr <platform> profile open` to edit |
| [`export`](./export) | Deprecated & hidden | `ccr <claude\|codex\|grok> profile export` |
| [`import`](./import) | Deprecated & hidden | `ccr <claude\|codex\|grok> profile import` |

## Related Pages

- [CLI Workflows](/en/guide/cli-workflows)
- [Configuration Model](/en/guide/configuration)
- [Migration Guide](/en/reference/migration)
