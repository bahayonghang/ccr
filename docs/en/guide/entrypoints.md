# Entrypoints

This page documents only the currently supported entrypoints. The built-in Web API, `ccr web`, and the old global platform-switching model are retired and no longer part of the active workflow.

## Current entrypoints

| Entrypoint | Role | Best for |
|---|---|---|
| `ccr <command>` | main CLI surface | automation, scripts, precise command execution, and JSON diagnostics |
| `ccr` | default TUI entrypoint | interactive browsing and switching for Claude, Codex, and Grok in a terminal |
| `ccr ui` | recommended graphical entrypoint | full module browsing, usage/status inspection, and day-to-day visual management |
| `ccr-ui` | standalone UI project directory | frontend development and Tauri desktop work |

## How to choose

### Choose the CLI

```bash
ccr current
ccr claude profile list
ccr codex auth list
ccr grok profile list
ccr project init
ccr sync all status
```

### Choose the TUI

```bash
ccr
ccr codex
ccr grok auth
```

### Choose CCR UI

```bash
ccr ui
ccr ui -p 15173 --backend-port 38081
```

## Boundaries and historical command replacements

- `ccr current` (aliases: `ccr status`, `ccr show`) is the runtime-overview entrypoint.
- `ccr <claude|codex|grok> profile ...` and `ccr <claude|codex|grok> auth ...` are the canonical SQLite catalog (`profiles.db`) and account management surfaces.
- `ccr platform list [--json]` remains the platform-registry inspection command.
- Removed, retired, and hidden legacy entrypoints:
  - `ccr platform switch|current|info|init|profile` have been removed from the CLI parser; use `ccr <claude|codex|grok> profile <action>`, `ccr current`, and `ccr platform list`.
  - `ccr switch <name>` and the `ccr <name>` shortcut are retired and return migration errors; use `ccr <claude|codex|grok> profile switch <name>`.
  - Top-level `ccr init|list|add|delete|enable|disable|clear|optimize|export|import` are hidden and deprecated in 7.x (scheduled for removal in 8.0); use `ccr <claude|codex|grok> profile init|list|create|delete|enable|disable|off|open|export|import` (and `ccr project init` for repository workflow setup).

## Related docs

- [CLI Workflows](/en/guide/cli-workflows)
- [Configuration Model](/en/guide/configuration)
- [UI Overview](/en/guide/ui-overview)
- [Migration Guide](/en/reference/migration)
- [TUI Mode](/en/reference/commands/tui)
