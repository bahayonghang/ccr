# Entrypoints

This page documents only the currently supported entrypoints. The built-in Web API, `ccr web`, and the old global platform-switching model are no longer the recommended path.

## Current entrypoints

| Entrypoint | Role | Best for |
|---|---|---|
| `ccr <command>` | main CLI surface | automation, scripts, precise command execution |
| `ccr` | default TUI entrypoint | interactive browsing and switching in a terminal |
| `ccr ui` | recommended graphical entrypoint | module browsing, status inspection, and day-to-day visual management |
| `ccr-ui` | standalone UI project directory | frontend development and Tauri desktop work |

## How to choose

### Choose the CLI

```bash
ccr current
ccr claude profile list
ccr codex auth list
ccr sync all status
```

### Choose the TUI

```bash
ccr
```

### Choose CCR UI

```bash
ccr ui
ccr ui -p 15173 --backend-port 38081
```

## Boundaries

- `ccr current` is the runtime-overview entrypoint.
- `ccr claude profile ...` and `ccr codex profile ...` are the main auth/profile routes.
- `ccr platform list` remains a registry compatibility view.
- `ccr switch <name>`, `ccr <name>`, and `ccr platform switch/current/...` are retired.

## Related docs

- [CLI Workflows](/en/guide/cli-workflows)
- [Configuration Model](/en/guide/configuration)
- [UI Overview](/en/guide/ui-overview)
- [TUI Mode](/en/reference/commands/tui)
