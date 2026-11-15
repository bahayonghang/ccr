# Multi-Platform Setup Guide

> This page covers the recommended CLI workflow for running multiple AI CLI platforms side by side under one `~/.ccr/` registry.

## Platform Capability Overview

| Platform | CLI Capability Tier | Authoritative Catalog | Host Runtime Settings Target |
|----------|---------------------|-----------------------|------------------------------|
| **Claude Code** (`claude`) | Full `profile` + `auth` | `~/.ccr/platforms/claude/profiles.db` | `~/.claude/settings.json`, `~/.claude.json` |
| **Codex** (`codex`) | Full `profile` + `auth` | `~/.ccr/platforms/codex/profiles.db` | `~/.codex/config.toml`, `~/.codex/auth.json` |
| **Grok** (`grok`) | Full `profile` + `auth` | `~/.ccr/platforms/grok/profiles.db` | `~/.grok/config.toml`, `~/.grok/user-settings.json` |
| **Antigravity CLI** (`gemini`) | Retained compatibility adapter (`legacy_adapter`) | No CLI `profile`/`auth` subcommands | `~/.gemini/antigravity-cli/settings.json` (plus `/antigravity` UI) |
| **Factory Droid** (`droid`) | Retained compatibility adapter (`legacy_adapter`) | No CLI `profile`/`auth` subcommands | `~/.factory/settings.json` |
| **Qwen CLI** (`qwen`) | Reserved / Stub | No pre-created catalog | Reserved platform key |

## Directory Structure

```text
~/.ccr/
├── config.toml                            # Platform registry (independent current_profile per platform)
├── platforms/
│   ├── claude/
│   │   └── profiles.db                    # Claude SQLite profile catalog
│   ├── codex/
│   │   └── profiles.db                    # Codex SQLite profile catalog
│   └── grok/
│       └── profiles.db                    # Grok SQLite profile catalog
├── history/                               # Masked audit history
├── backups/                               # Automatic rollback backups
└── logs/                                  # Daily rotated runtime logs
```

## Quick Setup and Side-by-Side Routing

### 1. Initialize supported platform catalogs

```bash
ccr claude profile init
ccr codex profile init
ccr grok profile init
```

Each command initializes that platform's SQLite `profiles.db` from its embedded template without affecting other platforms.

### 2. Create and edit profiles safely

Always pass secrets via standard input (`--auth-token-stdin` / `--api-key-stdin`) or reference an environment variable name (`--env-key`) so plaintext tokens never leak into process arguments:

```bash
# Claude: pipe credential via stdin
printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile create work \
  --base-url https://api.example.com \
  --auth-token-stdin

# Codex: pipe credential via stdin
printf '%s' "$OPENAI_API_KEY" | ccr codex profile create relay \
  --base-url https://api.example.com/v1 \
  --auth-token-stdin

# Grok: bind an environment variable name or pipe a direct key via --api-key-stdin
ccr grok profile create xai --env-key XAI_API_KEY
printf '%s' "$GROK_RELAY_KEY" | ccr grok profile create relay \
  --base-url https://api.example.com/v1 \
  --api-key-stdin
```

To edit multiple fields or tags at once in `$VISUAL` / `$EDITOR`, open the temporary TOML projection for that platform's catalog:

```bash
ccr claude profile open
ccr codex profile open
ccr grok profile open
```

### 3. Switch and inspect runtimes independently

CCR 7.x routes each platform independently — there is no global platform switch step before switching a profile:

```bash
ccr claude profile switch work
ccr codex profile switch relay
ccr grok profile switch xai

# Inspect runtime overview and platform registry
ccr current --verbose
ccr platform list
```

To leave CCR profile mode on any platform and return to its official login runtime, run `profile off`:

```bash
ccr claude profile off
ccr codex profile off
ccr grok profile off
```

## Exporting, Sharing, and Importing Profiles (`ExchangeV1`)

Use platform-scoped `profile export` and `profile import` to share or migrate profiles across machines:

```bash
# Export a sanitized ExchangeV1 TOML bundle without secrets
ccr claude profile export --no-secrets -o claude-team.toml

# Preview the import on another machine, then merge incrementally
ccr claude profile import claude-team.toml --dry-run
ccr claude profile import claude-team.toml --on-conflict skip
```

> Pass `--replace -y` explicitly only when you want the imported bundle to replace the entire target catalog.

## Inspecting Compatibility Adapter Platforms

`gemini` (Antigravity CLI) and `droid` (Factory Droid) do not expose CLI `profile` switching subcommands, but you can inspect their local host configuration health read-only with `ccr doctor`:

```bash
ccr doctor --platform gemini
ccr doctor --platform droid
ccr doctor --all-platforms
```

Antigravity CLI settings, MCP servers (`~/.gemini/antigravity-cli/mcp_config.json`), slash commands, and plugins can also be managed visually in the `/antigravity` workspace inside `ccr ui`.

## See Also

- [Platform Support](/en/reference/platforms/)
- [Examples Overview](./index)
- [Troubleshooting Guide](./troubleshooting)
- [Quick Start](/en/guide/quick-start)
- [UI Overview](/en/guide/ui-overview)
