# Factory Droid Platform Guide

> [!IMPORTANT]
> **Retained Compatibility Adapter (`legacy_adapter`)**
> In CCR 7.x, `droid` (displayed as **Factory Droid**; recognized input aliases: `droid`, `factory`) is a **retained compatibility adapter (`legacy_adapter`)**.
> `Platform::auth_profile_supported()` includes only `claude`, `codex`, and `grok` — **`droid` has no `ccr droid profile ...` or `ccr droid auth ...` CLI/TUI subcommands**, no dedicated CCR UI platform workspace, and `ccr validate` checks only the registry plus `claude`, `codex`, and `grok`.

## Overview

Factory Droid is Factory AI's terminal coding assistant. CCR retains the `droid` compatibility adapter in the shared platform registry and diagnostic layer for registry visibility, read-only host settings inspection, and ecosystem utilities.

Upstream installation:

```powershell
# Windows
irm https://app.factory.ai/cli/windows | iex
```

```bash
# macOS / Linux
curl -fsSL https://app.factory.ai/cli/install | bash
```

## Platform Information

| Item | Value |
|------|-------|
| Persisted CCR platform key | `droid` |
| Recognized input aliases | `droid`, `factory` |
| Display name | Factory Droid |
| CLI capability tier | `legacy_adapter` (no CLI/TUI `profile` or `auth` subcommands) |
| Host settings file | `~/.factory/settings.json` |

## Supported CCR Capabilities

### 1. Platform Registry Recognition

Inspect `droid` status in the unified platform registry via `ccr platform list`:

```bash
ccr platform list
ccr platform list --json
```

### 2. Read-Only Health Inspection (`ccr doctor`)

Run `ccr doctor` for side-effect-free local checks against `~/.factory/settings.json`:

```bash
ccr doctor --platform droid
ccr doctor --platform droid --verbose
ccr doctor --all-platforms
```

### 3. Ecosystem Tooling

The `droid` platform key works with shared cross-platform tools where applicable (such as session filtering, prompt presets, and custom sync folder registration):

```bash
# Filter indexed sessions by platform
ccr sessions list --platform droid

# Inspect registered WebDAV sync folders (e.g., if syncing ~/.factory)
ccr sync folder list
```

> `examples/droid/profiles.toml` in the repository is kept strictly as a historical schema reference and is not an active CLI import or switching target.

## Host Configuration File Reference (`~/.factory/settings.json`)

Factory Droid reads `~/.factory/settings.json` directly. CCR's `DroidPlatform` read-only inspector parses the `customModels` array using the `DroidSettings` schema while preserving unrelated top-level JSON fields:

```json
{
  "customModels": [
    {
      "model": "claude-sonnet-4-5-20250929",
      "displayName": "Anthropic Sonnet",
      "baseUrl": "https://api.anthropic.com/v1",
      "apiKey": "sk-ant-...",
      "provider": "anthropic",
      "maxOutputTokens": 8192
    },
    {
      "model": "gpt-4o",
      "displayName": "OpenAI GPT-4o",
      "baseUrl": "https://api.openai.com/v1",
      "apiKey": "sk-...",
      "provider": "openai",
      "maxOutputTokens": 4096
    }
  ]
}
```

### `customModels` Entry Fields

| JSON Field | Required | Description |
|------------|----------|-------------|
| `model` | ✅ | Upstream model identifier |
| `displayName` | ❌ | Human-readable label shown in Factory Droid |
| `baseUrl` | ✅ | Provider API endpoint (`http://` or `https://`) |
| `apiKey` | ✅ | Provider API credential |
| `provider` | ✅ | Provider protocol type: `anthropic`, `openai`, or `generic-chat-completion-api` |
| `maxOutputTokens` | ❌ | Maximum output token limit per response |

### Security Notes

- `apiKey` values in `~/.factory/settings.json` are stored in plaintext; keep strict file permissions (for example `chmod 600 ~/.factory/settings.json`).
- Never commit `settings.json` with real credentials to Git or attach it unredacted to issue reports.

## See Also

- [Platform Support](./index)
- [`doctor` unified diagnostics](/en/reference/commands/doctor)
- [Claude Platform Guide](./claude)
- [Codex Platform Guide](./codex)
- [Antigravity CLI Platform Guide](./gemini)
