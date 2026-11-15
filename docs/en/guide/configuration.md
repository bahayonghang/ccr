# Configuration Model

This page explains CCR 7.x configuration modes, directory layout, SQLite profile catalogs (`profiles.db`), and the explicit `auth` / `profile` source of truth.

## Modes

| Mode | When to use it | Primary location | Notes |
|------|----------------|------------------|-------|
| Unified Mode | Multi-platform workflows, recommended default | `~/.ccr/` | Registry, `profiles.db` catalogs, history, backups, and logs live under one root. |
| Legacy Mode | Historical single-platform compatibility | `~/.ccs_config.toml` | Used only when `CCR_ROOT` is unset and `~/.ccr/config.toml` does not exist. |

Detection order:

1. `CCR_ROOT` is set
2. `~/.ccr/config.toml` exists
3. fall back to Legacy Mode

## Unified layout

```text
~/.ccr/
├── config.toml
├── platforms/
│   ├── claude/
│   │   └── profiles.db
│   ├── codex/
│   │   └── profiles.db
│   ├── grok/
│   │   └── profiles.db
│   ├── gemini/
│   ├── droid/
│   └── qwen/
├── history/
├── backups/
├── logs/
└── ccr-ui/
```

Key storage rules:

- `config.toml`: platform registry; active profile routing lives in each platform entry's `current_profile` pointer.
- `platforms/<platform>/profiles.db`: **primary live SQLite profile catalog** for Claude, Codex, and Grok.
- **Automatic `profiles.toml` migration and TOML exchange boundary**:
  - Legacy `platforms/<platform>/profiles.toml` files are automatically migrated into `profiles.db` on first access and backed up under `backups/<platform>/`.
  - Do not manually copy example or backup `profiles.toml` files onto the live path — CCR 7.x reads `profiles.db` as the authoritative catalog and ignores a `profiles.toml` copied back beside an initialized database.
  - TOML is used for interactive editor projections via `ccr <claude|codex|grok> profile open` and for `ExchangeV1` export/import via `ccr <claude|codex|grok> profile export` / `import`.
- `history/` and `backups/`: masked audit history and automatic rollback snapshots.
- `ccr-ui/`: frontend and runtime assets managed by `ccr ui`.

## Runtime/profile source of truth

Auth and profile routing use explicit platform-scoped command families:

- `ccr <claude|codex|grok> auth ...`: official-auth account snapshots and session state
- `ccr <claude|codex|grok> profile ...`: `profiles.db` catalog management and host runtime file writes
- `ccr current` (aliases: `ccr status`, `ccr show`): side-by-side runtime overview

Inside `~/.ccr/config.toml`, each platform entry's `current_profile` field is the routing pointer of record. Older registries may still contain `default_platform` / `current_platform` for backward compatibility, but those fields no longer drive routing.

## Platform status and host targets

| Platform | CLI capability tier | Authoritative profile store | Host settings target |
|----------|---------------------|-----------------------------|----------------------|
| Claude Code (`claude`) | Full `profile` + `auth` | `~/.ccr/platforms/claude/profiles.db` | `~/.claude/settings.json`, `~/.claude.json` |
| Codex (`codex`) | Full `profile` + `auth` | `~/.ccr/platforms/codex/profiles.db` | `~/.codex/config.toml`, `~/.codex/auth.json` |
| Grok (`grok`) | Full `profile` + `auth` | `~/.ccr/platforms/grok/profiles.db` | `~/.grok/config.toml`, `~/.grok/user-settings.json` |
| Antigravity CLI (`gemini`) | Retained compatibility adapter (`legacy_adapter`) | No CLI `profile`/`auth` surface | `~/.gemini/antigravity-cli/settings.json`; MCP: `~/.gemini/antigravity-cli/mcp_config.json` |
| Factory Droid (`droid`) | Retained compatibility adapter (`legacy_adapter`) | No CLI `profile`/`auth` surface | `~/.factory/settings.json` |
| Qwen CLI (`qwen`) | Reserved / Stub | No pre-created catalog | Reserved platform key with partial data-domain support |

## Common operations

```bash
# Initialize platform catalogs
ccr claude profile init
ccr codex profile init
ccr grok profile init

# Open in $EDITOR or export/import ExchangeV1 TOML
ccr claude profile open
ccr claude profile export --no-secrets -o claude-profiles.toml
ccr claude profile import claude-profiles.toml --dry-run

# Switch and diagnose
ccr claude profile switch <name>
ccr codex profile switch <name>
ccr grok profile switch <name>
ccr current
ccr validate
ccr doctor
```

## Relationship to CCR UI and VS Code

- CLI, TUI, `ccr-ui`, and `ccr-vscode` share the same `~/.ccr/config.toml` registry and per-platform `profiles.db` catalogs.
- The explicit runtime state reported by `ccr current` is the same state model reflected by CCR UI and VS Code.
- `ccr platform list` remains a compatibility registry view rather than a global active-platform switch.

## Related docs

- [CLI Workflows](/en/guide/cli-workflows)
- [Quick Start](/en/guide/quick-start)
- [Platform Support](/en/reference/platforms/)
