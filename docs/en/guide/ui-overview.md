# UI Overview

CCR UI is the Vue 3 and Tauri graphical interface over the shared `~/.ccr/` configuration and runtime state. It uses the same Rust domain crates as the CLI and TUI; it is not a separate configuration system.

## Launch Entrypoints

```bash
ccr ui
ccr ui -p 15173 --backend-port 38081
```

The default frontend port is `15173`; the backend port is `38081`. `ccr ui` looks for a development checkout, then `~/.ccr/ccr-ui/`, and enters the download or update path when the UI is missing.

For `ccr-ui` development:

```bash
cd ccr-ui
bun run dev:web -- --host 127.0.0.1 --strictPort
```

Use `bun run tauri:dev` when native windows and Tauri invokes are required. Plain browser preview verifies routes and presentation but cannot complete every desktop command.

## Relationship To Other Entrypoints

| Entrypoint | Best for |
|---|---|
| `ccr <command>` | automation, exact flags, scripts, and diagnostic output |
| `ccr` without a subcommand | fast terminal profile/auth operations |
| `ccr ui` | visual configuration, platform management, usage, monitoring, and desktop tools |
| `ccr-ui/` checkout | UI development, testing, and Tauri builds |

## Current Capability Surface

- Platform workspaces: Claude Code, Codex, Antigravity CLI, and OpenCode.
- Configuration and extensions: profiles, auth, settings, MCP, agents, slash commands, plugins, hooks, output styles, statusline, and skills.
- Data and operations: usage, monitoring, budget, pricing, and check-in.
- Tools and environments: commands, converter, WebDAV sync, WSL, and SSH.

Factory Droid is retained in the CLI as a compatibility adapter (`legacy_adapter`) and Qwen remains partial/stub support; the current router does not provide dedicated platform home pages for them. Use [Platform Support](/en/reference/platforms/) for platform status and the [UI Module Map](./ui-modules) for actual UI pages.

## Execution Environments In The Native GPUI Desktop

The independent `ccr-desktop` (`ccr-gpui`) switches the active Local/SSH execution environment from the bottom status bar. Global Settings has Appearance, Workbench preferences, Execution environments, and Diagnostics pages. Platform configuration, MCP, Agent sessions, commands/installation, sync, and usage follow the selected environment. A failed connection retains the SSH selection and never falls back to local data. Check-ins and global settings remain available offline. Tauri keeps its separate system OpenSSH implementation.

SSH usage requires installed llmusage and Python3 with sqlite3. Explicit imports run on that host. Reports use a read-only SQLite backup including committed WAL pages, downloaded over SFTP into a private snapshot (64MiB maximum, ten-minute transfer deadline), then schema/feature-gated by `ccr-usage`. CCR never migrates the remote database. Missing tools, disconnection, snapshot failure, or unconfirmed cleanup show unavailable data rather than local data or invented zeros.

SSH WebDAV supports four fixed file assets and bounded ordinary folders within the captured remote HOME. Sensitive fixed assets retain v2 encryption. Remote profile databases/SQLite sidecars, WebDAV disconnect, and creating new empty destination directories remain unsupported. Multi-file publication can partially complete and retains remote preimage backups. Real-host and cross-platform acceptance require separate verification.

## Tauri Data Boundary

```text
Vue view/store
  -> src/api/domains/*
  -> Tauri invoke
  -> src-tauri/src/commands/*
  -> workspace domain crate
```

`src/api/tauri.ts` is a legacy compatibility facade. New frontend business APIs belong in domain modules. Usage data is read through `ccr-usage` and the desktop llmusage adapter; Vue does not parse transcripts directly.

## When To Prefer The UI

Prefer the UI when you need to:

- browse and compare state across platform workspaces;
- inspect usage, monitoring, check-in, or cost dashboards;
- manage MCP, agents, skills, plugins, and sync assets together.

Prefer the CLI for:

- CI and shell automation;
- repeatable profile/auth scripting;
- diagnostics that require JSON or explicit exit codes.

## Related Pages

- [Choosing An Entrypoint](./entrypoints)
- [UI Module Map](./ui-modules)
- [`ui` command](/en/reference/commands/ui)
- [Architecture](/en/reference/architecture)
