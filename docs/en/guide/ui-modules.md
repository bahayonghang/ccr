# UI Module Map

This page groups the current `ccr-ui` surfaces by capability so the docs stay understandable without turning into a raw route dump.

## 1. Platform Modules

| Module group | Representative pages | Notes |
|--------------|----------------------|-------|
| Claude / Codex / Gemini / Droid | platform home pages and sub-pages | Main implemented or clearly productized platform surfaces |
| Qwen / iFlow | platform home pages and sub-pages | Visible in the UI, but still reserved/stub at the core platform layer |
| OpenCode | `opencode/*` | Separate tool-family entrypoint with providers, MCP, and plugins |

Typical sub-pages:

- Codex: MCP, Profiles, Slash Commands, Auth, Settings
- Gemini / Qwen / iFlow: MCP, Agents, Slash Commands, Plugins
- Droid: MCP, Agents, Slash Commands, Plugins, Models, Profiles, Droids

## 2. Configuration and Extension Modules

| Module group | Representative pages | Purpose |
|--------------|----------------------|---------|
| Configs | `configs` | Inspect and switch configuration sets |
| MCP | `mcp`, `mcp/unified` | Manage unified or platform-level MCP servers |
| Slash Commands / Agents | `slash-commands`, `agents` | Extend commands and agent-oriented settings |
| Skills / Market | `skills`, `skills/add`, `market` | Browse, install, and manage skills |
| Plugins / Hooks / Output Styles / Statusline | matching pages | Manage extension points and output presentation |
| Provider Health | `provider-health` | Browser-oriented view on provider connectivity |

## 3. Data and Operations Modules

| Module group | Representative pages | Purpose |
|--------------|----------------------|---------|
| Commands / Converter | `commands/:client?`, `converter` | Command execution and config translation |
| Sync | `sync` | WebDAV synchronization view |
| Usage / Monitoring | `usage`, `monitoring` | Usage reporting and monitoring |
| Budget / Pricing | `budget`, `pricing` | Cost and budget control |
| Sessions | `sessions` | Session search and resume workflows |

## 4. Specialized Tools and Environment Modules

| Module group | Representative pages | Purpose |
|--------------|----------------------|---------|
| Checkin | `checkin`, `checkin/manage/:accountId` | Account check-in and dashboard flows |
| Environment | `wsl`, `ssh` | Local and remote environment helper surfaces |

## 5. Documentation Strategy

To avoid turning the docs into a route inventory:

- describe UI in capability groups, not one page per route
- document user-visible modules, not every internal page
- keep defaults, CLI flags, and HTTP details in CLI / API reference pages

## 6. Suggested Navigation Order

If you are entering the UI for the first time:

1. start with [UI Overview](/en/guide/ui-overview)
2. move into a platform module or an operations module
3. return to [Command Reference](/en/reference/commands/) or [Web API Reference](/en/reference/api) when you need exact flags or routes
