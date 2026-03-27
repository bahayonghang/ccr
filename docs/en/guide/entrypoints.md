# Entrypoints

This page documents the current entrypoints only. The removed built-in Web API and `ccr web` are no longer part of the supported surface.

## Current Entrypoints

| Entrypoint | Role | Best for |
|---|---|---|
| `ccr <command>` | primary CLI surface | automation, scripts, exact command execution |
| `ccr` | default TUI entrypoint | interactive profile switching in a terminal |
| `ccr ui` | recommended graphical entrypoint | module browsing and day-to-day visual workflows |
| `ccr-ui` | standalone UI project | frontend development and Tauri desktop work |

## How To Choose

### Choose the CLI

Use the CLI when:

- you need a stable script surface
- you want one exact operation
- you are wiring CCR into shell aliases, CI, or automation

Typical entrypoints:

```bash
ccr platform list
ccr switch <name>
ccr sync all status
ccr sessions list
```

### Choose the TUI

Use the TUI when:

- you stay inside a terminal
- your main task is browsing and switching profiles
- you want a fast keyboard-driven selector

In the default build, launch it with:

```bash
ccr
```

See [`tui mode`](/en/reference/commands/tui) for the exact behavior.

### Choose CCR UI

Use CCR UI when:

- you need to move across multiple capability areas
- you want visual access to skills, monitoring, sessions, statusline, checkin, and related modules
- you want CCR to use a local `ccr-ui/` checkout during development

Launch it with:

```bash
ccr ui
ccr ui -p 15173 --backend-port 38081
```

## Boundaries

- `ccr` remains the source of truth; its commands are defined in `crates/ccr/src/cli/definitions.rs`
- `ccr ui` is a graphical entrypoint, not a second configuration system
- `ccr-ui` is the project directory name, not the main command ordinary users need to memorize

## Related Pages

- [CLI Workflows](/en/guide/cli-workflows)
- [UI Overview](/en/guide/ui-overview)
- [UI Module Map](/en/guide/ui-modules)
- [TUI Mode](/en/reference/commands/tui)
