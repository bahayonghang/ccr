# Antigravity CLI Platform Guide

## Overview

CCR keeps the persisted platform key as `gemini`, but the user-facing Google CLI integration now targets **Antigravity CLI**. This preserves existing CCR profiles, history, usage aggregation, sync folders, and UI bookmarks while following Google's Gemini CLI -> Antigravity CLI migration path.

Official migration references:

- Google Developers Blog: <https://developers.googleblog.com/an-important-update-transitioning-gemini-cli-to-antigravity-cli/>
- Antigravity CLI Overview: <https://antigravity.google/docs/cli-overview>
- Gemini CLI migration docs: <https://antigravity.google/docs/gcli-migration>

## Platform Information

| Item | Value |
|------|-------|
| CCR platform key | `gemini` |
| Display name | Antigravity CLI |
| Recommended binary | `agy` |
| Profiles file | `~/.ccr/platforms/gemini/profiles.toml` |
| Antigravity settings | `~/.gemini/antigravity-cli/settings.json` |
| Antigravity MCP config | `~/.gemini/antigravity-cli/mcp_config.json` |
| Global skills | `~/.gemini/antigravity-cli/skills` |
| Legacy/shared Gemini skills | `~/.gemini/skills` |
| Workspace MCP | `.agents/mcp_config.json` |
| Workspace skills | `.agents/skills` |

The old `/gemini-cli` UI route and `gemini` platform key remain compatibility aliases. New documentation and visible UI should prefer Antigravity CLI and `/antigravity` routes.

## Prerequisites

- Antigravity CLI installed when using the local CLI runtime (`agy --help`, `agy --version`).
- A Google API key or enterprise/standard/cloud/API-key access path appropriate for your account.
- Existing CCR `gemini` profiles can be reused; do **not** rename the platform key to `antigravity`.

## Quick Start

```bash
# Initialize the compatibility platform namespace
ccr platform init gemini

# Switch to the Google/Antigravity profile namespace
ccr platform switch gemini

# Add or edit a profile
ccr add

# Verify the local Antigravity CLI binary outside CCR
agy --version
```

Antigravity's own migration command preview:

```bash
agy plugin import gemini
```

## Profile Configuration

CCR profiles remain under `~/.ccr/platforms/gemini/profiles.toml`:

```toml
[google-official]
description = "Google Antigravity / Gemini API"
base_url = "https://generativelanguage.googleapis.com/v1beta"
auth_token = "AIzaSyXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
model = "gemini-2.0-flash-exp"
small_fast_model = "gemini-1.5-flash"
```

When a profile is active, CCR writes Antigravity settings to `~/.gemini/antigravity-cli/settings.json`:

```json
{
  "env": {
    "GOOGLE_API_BASE_URL": "https://generativelanguage.googleapis.com/v1beta",
    "GOOGLE_API_KEY": "AIzaSyXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    "GEMINI_MODEL": "gemini-2.0-flash-exp",
    "GEMINI_SMALL_FAST_MODEL": "gemini-1.5-flash"
  }
}
```

| TOML field | Environment variable | Description |
|------------|----------------------|-------------|
| `base_url` | `GOOGLE_API_BASE_URL` | API endpoint |
| `auth_token` | `GOOGLE_API_KEY` | Google API key, masked in normal output |
| `model` | `GEMINI_MODEL` | Default model |
| `small_fast_model` | `GEMINI_SMALL_FAST_MODEL` | Optional fast model |

## MCP, Skills, and Workspace Paths

Antigravity MCP servers are stored separately from settings:

```text
~/.gemini/antigravity-cli/mcp_config.json
```

CCR writes remote MCP servers with `serverUrl` and reads legacy `url` / `httpUrl` fields for compatibility:

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
3. Legacy/shared Gemini `~/.gemini/skills`

Workspace MCP should use `.agents/mcp_config.json` when a project-specific Antigravity configuration is needed.

## Sessions and Usage Import

CCR keeps historical Gemini session and usage data under the internal `gemini` platform key. Legacy Gemini CLI logs under `~/.gemini/tmp/*/chats/session-*.json` remain import-compatible.

Antigravity session/log import is intentionally marked as pending until its local log format is confirmed. Do not claim Antigravity session import support from the Gemini legacy parser alone.

## Troubleshooting

### Settings file did not change

Check the Antigravity path, not the old Gemini root settings file:

```bash
ls -la ~/.gemini/antigravity-cli/settings.json
chmod 600 ~/.gemini/antigravity-cli/settings.json
```

Then verify CCR profile state:

```bash
ccr platform switch gemini
ccr current
ccr validate
```

### MCP server is missing

Check `mcp_config.json`:

```bash
cat ~/.gemini/antigravity-cli/mcp_config.json
```

Remote servers should prefer `serverUrl`; older `url` and `httpUrl` are read for compatibility.

### Old Gemini paths still appear

Some old paths are intentionally retained as legacy/import-compatible sources:

- `~/.gemini/skills` for shared legacy skills
- `~/.gemini/commands` and project `.gemini/commands` for legacy slash-command files
- `~/.gemini/tmp/*/chats/session-*.json` for legacy session import

They should not be documented as the primary Antigravity settings or MCP paths.

## Security Notes

- API keys in `~/.ccr/platforms/gemini/profiles.toml` are plaintext; keep file permissions strict.
- CCR masks API keys in normal output and history, but backups can still contain secrets.
- Prefer `ccr export --no-secrets` when sharing profile examples.

## Related Commands

```bash
ccr platform init gemini
ccr platform switch gemini
ccr list
ccr switch <profile>
ccr validate
ccr history --platform gemini
```

## See Also

- [Platform Overview](./index)
- [Claude Code Platform](./claude)
- [Codex Platform](./codex)
- [Multi-Platform Setup](/en/examples/multi-platform-setup)
