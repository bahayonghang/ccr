# Antigravity CLI Platform Guide

> [!IMPORTANT]
> **Retained Compatibility Adapter (`legacy_adapter`)**
> In CCR 7.x, `gemini` (displayed as **Antigravity CLI**; recognized input aliases: `gemini-cli`, `antigravity`, `antigravity-cli`, `agy`) is a **retained compatibility adapter (`legacy_adapter`)**.
> `Platform::auth_profile_supported()` includes only `claude`, `codex`, and `grok` — **`gemini` has no `ccr gemini profile ...` or `ccr gemini auth ...` CLI/TUI subcommands**, and `ccr validate` checks only the registry plus `claude`, `codex`, and `grok`.

## Overview

CCR keeps the persisted platform key as `gemini` while targeting **Antigravity CLI** (`agy`) for user-facing Google CLI integration, preserving existing sync folders, historical session indexing, skill directories, and desktop UI workspace routes across Google's Gemini CLI -> Antigravity CLI transition.

Official migration references:

- Google Developers Blog: <https://developers.googleblog.com/an-important-update-transitioning-gemini-cli-to-antigravity-cli/>
- Antigravity CLI Overview: <https://antigravity.google/docs/cli-overview>
- Gemini CLI migration docs: <https://antigravity.google/docs/gcli-migration>

## Platform Information

| Item | Value |
|------|-------|
| Persisted CCR platform key | `gemini` |
| Recognized input aliases | `gemini-cli`, `antigravity`, `antigravity-cli`, `agy` |
| Display name | Antigravity CLI |
| CLI capability tier | `legacy_adapter` (no CLI/TUI `profile` or `auth` subcommands) |
| Recommended host binary | `agy` |
| Host settings file | `~/.gemini/antigravity-cli/settings.json` |
| Host MCP config | `~/.gemini/antigravity-cli/mcp_config.json` |
| Global skills directory | `~/.gemini/antigravity-cli/skills` |
| Legacy/shared Gemini skills | `~/.gemini/skills` |
| Workspace MCP config | `.agents/mcp_config.json` |
| Workspace skills directory | `.agents/skills` |

Legacy `/gemini-cli` UI routes redirect to `/antigravity`; the persisted platform key remains `gemini`.

## Supported CCR Capabilities

### 1. Platform Registry Recognition

`ccr platform list` and `ccr platform list --json` surface the `gemini` registry entry and its status:

```bash
ccr platform list
ccr platform list --json
```

### 2. Read-Only Health Inspection (`ccr doctor`)

Run `ccr doctor` for side-effect-free local checks against `~/.gemini/antigravity-cli/settings.json` and `GEMINI_API_KEY`:

```bash
ccr doctor --platform gemini
ccr doctor --platform gemini --verbose
ccr doctor --all-platforms
```

### 3. Ecosystem Tooling

The `gemini` platform key is supported across CCR's shared ecosystem commands:

```bash
# Session indexing and search (import-compatible with legacy ~/.gemini/tmp/*/chats/session-*.json logs)
ccr sessions list --platform gemini
ccr sessions search "keyword" --platform gemini

# Skills management
ccr skills list --platform gemini
ccr skills install <skill-name> --platform gemini

# Prompt presets (writes target files under ~/.gemini/)
ccr prompts list --platform gemini
ccr prompts current gemini --platform gemini

# WebDAV folder sync registration
ccr sync folder list
```

> Note: Legacy Gemini CLI session logs remain import-compatible; native Antigravity session import is deferred until upstream local log formats stabilize.

### 4. Graphical Workspace (`ccr ui` -> `/antigravity`)

Launch `ccr ui` and open the `/antigravity` module to manage local Antigravity settings, MCP servers (`mcp_config.json`), slash commands, custom agents, and plugins visually:

```bash
ccr ui -p 15173 --backend-port 38081
```

## Host Configuration File Reference

### `~/.gemini/antigravity-cli/settings.json`

`ccr doctor --platform gemini` inspects the `google` object inside `~/.gemini/antigravity-cli/settings.json` read-only via the `GeminiSettings` schema, ignoring unrelated top-level JSON fields during inspection (`GeminiSettings` itself does not flatten extra top-level fields; `ccr ui` `/antigravity` preserves unrelated top-level fields via raw JSON editing):

```json
{
  "google": {
    "api_key": "AIzaSyExampleKeyWithAtLeast30Chars",
    "project_id": "my-project",
    "region": "us-central1",
    "api_version": "v1"
  }
}
```

Validation and credential rules:

- `google.api_key`: **required** in `GeminiSettings` (`GoogleConfig::api_key: String`); must start with `AIza` and be at least `30` characters long.
- `google.project_id`, `google.region`, `google.api_version`: optional Google Cloud / Vertex metadata fields.
- Environment variable fallback: `ccr doctor --platform gemini` also inspects the optional `GEMINI_API_KEY` environment variable.
- Keep strict file permissions (for example `chmod 600`) on `~/.gemini/antigravity-cli/settings.json` whenever it contains a plaintext key, and never commit it to version control.

### MCP and Skills Paths

Antigravity MCP servers are stored separately from `settings.json` at `~/.gemini/antigravity-cli/mcp_config.json` (workspace-level config: `.agents/mcp_config.json`). CCR writes remote MCP servers with `serverUrl` while reading legacy `url` and `httpUrl` fields for compatibility:

```json
{
  "mcpServers": {
    "example": {
      "serverUrl": "https://example.com/mcp",
      "type": "http"
    }
  }
}
```

Skills lookup order includes:

1. Workspace `.agents/skills`
2. Antigravity global `~/.gemini/antigravity-cli/skills`
3. Legacy/shared `~/.gemini/skills`

Legacy paths (`~/.gemini/skills`, `~/.gemini/commands`, `.gemini/commands`, and `~/.gemini/tmp/*/chats/session-*.json`) remain compatibility sources only and should not be treated as primary Antigravity settings or MCP locations.

## See Also

- [Platform Support](./index)
- [`doctor` unified diagnostics](/en/reference/commands/doctor)
- [UI Module Map](/en/guide/ui-modules)
- [Claude Platform Guide](./claude)
- [Codex Platform Guide](./codex)
