# UI Overview

`ccr-ui` is CCR's full graphical product surface. It organizes multi-platform configuration management, extensions, and operational views into a single browser or desktop experience.

## Boundaries You Should Keep in Mind

- `ccr`: the CLI/TUI entrypoint for configuration management
- `ccr ui`: the CLI entrypoint that launches the recommended graphical workflow
- `ccr-ui/`: the standalone Vue 3 + Tauri project directory

## How To Start It

```bash
ccr ui
ccr ui -p 15173 --backend-port 38081
```

Defaults:

- frontend port: `15173`
- backend port: `38081`

Startup discovery order:

1. `ccr-ui/` in the current directory
2. `ccr-ui/` in the parent directory
3. `~/.ccr/ccr-ui/`
4. prompt to download from GitHub when missing

## Runtime Modes

| Mode | Entry | Notes |
|------|-------|-------|
| Browser mode | `ccr ui` | Recommended daily path |
| Web development | `cd ccr-ui && bun run dev` | Frontend development and integration testing |
| Desktop shell | `cd ccr-ui && bun run tauri:dev` | Runs the Tauri desktop shell |

> In the docs, “UI” means the full `ccr-ui` product surface, not only the `ccr ui` wrapper command.

## Why the UI Exists

- Platform modules: Claude, Codex, Gemini, Droid, plus visible reserved groups for Qwen, iFlow, and OpenCode
- Configuration and extension management: MCP, Skills, Prompts, Plugins, Hooks, Output Styles, Statusline, Provider Health
- Data and operations views: Usage, Monitoring, Sessions, Budget, Pricing, Sync, Commands
- Specialized surfaces: Checkin, WSL, SSH, OpenCode

## Suggested Reading Order

1. [UI Module Map](/en/guide/ui-modules)
2. [`ccr ui` command](/en/reference/commands/ui)

## Relationship to the Configuration Model

- UI and CLI share the same `~/.ccr/` data model.
- The UI can expose more surfaces than the CLI, but it must not become a second source of truth.
- Platform status, defaults, and API routes still come from the codebase definitions.
