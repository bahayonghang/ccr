# UI Module Map

This page groups the pages currently exposed by `ccr-ui/src/shell/routeCatalog.ts` by user task. Redirects are not presented as separate features.

## Platform Workspaces

| Platform | Route | Current sub-pages |
|---|---|---|
| Claude Code | `/claude-code` | settings, profiles, auth |
| Codex | `/codex` | MCP, profiles, agents, sessions, slash commands, auth, settings |
| Antigravity CLI | `/antigravity` | MCP, agents, slash commands, plugins; old `/gemini-cli` paths redirect |
| OpenCode | `/opencode` | providers, MCP, agents, commands, plugins, settings; skills join the shared skills page |

Factory Droid and Qwen remain CLI platform-domain concepts. CLI support must not be interpreted as a dedicated UI platform home page.

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

## Route Maintenance Rules

- Product docs describe stable capability groups rather than every internal route.
- When a route is added, removed, or converted to a redirect, update this page and its locale mirror.
- Platform capability and UI route registration are separate contracts; call something a UI page only after the route exists.
- Keep exact CLI flags in the [Command Reference](/en/reference/commands/) and internal Vue/Tauri boundaries in [Architecture](/en/reference/architecture).

## Related Pages

- [UI Overview](./ui-overview)
- [CLI Workflows](./cli-workflows)
- [Platform Support](/en/reference/platforms/)
