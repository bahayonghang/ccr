# UI Module Map

This page groups the pages currently exposed by `ccr-ui/src/shell/routeCatalog.ts` by user task. Redirects are not presented as separate features.

## Platform Workspaces

| Platform | Route | Current sub-pages |
|---|---|---|
| Claude Code | `/claude-code` | settings, profiles, auth |
| Codex | `/codex` | MCP, profiles, agents, sessions, slash commands, auth, settings |
| Antigravity CLI | `/antigravity` | MCP, agents, slash commands, plugins; old `/gemini-cli` paths redirect |
| OpenCode | `/opencode` | providers, MCP, agents, commands, plugins, settings; skills join the shared skills page |

Factory Droid (`legacy_adapter` compatibility adapter) and Qwen (reserved stub) remain CLI platform-registry concepts. CLI recognition must not be interpreted as a dedicated UI platform home page.

## Configuration And Extensions

| Capability | Route | Notes |
|---|---|---|
| Configuration sets | `/configs` | browse, filter, and manage shared configurations |
| MCP | `/mcp-manager` | unified MCP management; old `/mcp` and `/mcp/unified` paths redirect here |
| Slash commands | `/slash-commands` | shared command-resource management |
| Agents | `/agents` | shared list and detail routes |
| Skills | `/skills` | unified skills migration page; manager, hub, market, and detail aliases redirect |
| Extensions | `/plugins`, `/hooks` | plugin and hook management |
| Output | `/output-styles`, `/statusline` | output-style and statusline configuration |

## Data And Operations

| Capability | Route | Notes |
|---|---|---|
| Usage | `/usage` | usage dashboard; old `/stats` redirects here |
| Monitoring | `/monitoring` | monitoring feed; old generic `/sessions` redirects here |
| Cost controls | `/budget`, `/pricing` | budgets and model pricing |
| Check-in | `/checkin` | account list, execution state, and account dashboards |

The Codex session page remains available at `/codex/sessions`; it is distinct from the removed generic SessionsView.

## Tools And Environments

| Capability | Route | Notes |
|---|---|---|
| Commands | `/commands/ccr` and peers | client-scoped command workspace; old `/ccr-control` redirects |
| Converter | `/converter` | configuration format conversion |
| Sync | `/sync` | WebDAV console for fixed configuration assets |
| WSL | `/wsl` | WSL environment management |
| SSH | `/ssh` | SSH environment management |
| App settings | `/settings` | UI appearance, behavior, and diagnostics |

## SSH Boundaries In The Native GPUI Desktop

`ccr-desktop` uses native destinations and platform tabs rather than the Tauri URL routes above. The status bar publishes one active execution environment:

| Destination | SSH behavior |
|---|---|
| Platform settings/profiles/accounts, instructions/extensions, MCP | Captured remote HOME reads/writes; unsupported detailed actions fail explicitly |
| Agent sessions | Per-host private read-only snapshots; missing remote sources never use local files |
| Commands/installation | Captured-session remote exec; missing CLI leads to installation guidance |
| Usage/overview | Remote llmusage import, Python3 sqlite3 consistent snapshot, ccr-usage read-only projections; schema 10 minimum, provider features require 14 |
| Sync | Current-host configuration, four fixed file assets and bounded ordinary folders; profile databases/SQLite sidecars are refused, preserving original encryption and remote preimage backups |
| Check-ins/global settings/operation record storage | Environment-independent and available with SSH offline |

Usage snapshots are limited to 64MiB with a ten-minute transfer deadline. Ordinary sync allows 2MiB per file, 16MiB total, 128 nodes and 12 levels. SSH WebDAV disconnect and creating new empty destination directories remain unavailable; multi-file publication is not a cross-file transaction. Real SSH, system credential stores, external services, and other platforms require separate verification.

## Tauri Route Maintenance Rules

- Product docs describe stable capability groups rather than every internal route.
- When a route is added, removed, or converted to a redirect, update this page and its locale mirror.
- Platform capability and UI route registration are separate contracts; call something a UI page only after the route exists.
- Keep exact CLI flags in the [Command Reference](/en/reference/commands/) and internal Vue/Tauri boundaries in [Architecture](/en/reference/architecture).

## Related Pages

- [UI Overview](./ui-overview)
- [CLI Workflows](./cli-workflows)
- [Platform Support](/en/reference/platforms/)
