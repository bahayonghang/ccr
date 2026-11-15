# CLI Workflows

This page organizes CCR into task-oriented CLI workflows. The canonical `auth` and `profile` surface centers on explicit Claude, Codex, and Grok platform subcommands rather than an implicit global “current platform” setting.

## Workflow 1: Initialize catalogs and inspect runtime state

```bash
ccr claude profile init
ccr codex profile init
ccr grok profile init
ccr current
ccr platform list
```

Use this for:
- first-run initialization of `~/.ccr/platforms/<platform>/profiles.db`
- inspecting active runtime state via `ccr current` (aliases: `ccr status`, `ccr show`)
- checking platform registry entries and per-platform `current_profile` pointers via `ccr platform list [--json]`
- setting up repository-level agent workflows with `ccr project init`

> `ccr platform list` is a registry overview and compatibility view. Use `ccr current` for live runtime state.

## Workflow 2: Claude runtime / profile

```bash
printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile create work \
  --base-url https://api.example.com \
  --auth-token-stdin

ccr claude profile open
ccr claude profile list
ccr claude profile switch work
ccr claude profile current
ccr claude profile off
```

Use this for:
- creating profiles safely via `--auth-token-stdin` or editing a projection in `$VISUAL` / `$EDITOR` via `profile open`
- applying a saved profile into `~/.claude/settings.json`
- leaving profile mode with `ccr claude profile off` to return to Claude official login

## Workflow 3: Codex runtime / profile and multi-account auth

```bash
# Official-auth account snapshots
ccr codex auth save work
ccr codex auth list
ccr codex auth switch work
ccr codex auth off

# Runtime profile routing
printf '%s' "$OPENAI_API_KEY" | ccr codex profile create proxy \
  --base-url https://api.example.com/v1 \
  --auth-token-stdin
ccr codex profile list
ccr codex profile switch proxy
ccr codex profile current
ccr codex profile off
```

Use this for:
- keeping `codex auth` (official account snapshots) separate from `codex profile` (`~/.codex/config.toml` + runtime `auth.json` routing)
- switching third-party relays or model providers and cleaning up CCR-managed routes on `profile off` before `codex login`

## Workflow 4: Grok runtime / profile and official sessions

```bash
# Bind an environment variable name or pipe a direct API key via stdin
ccr grok profile create xai --env-key XAI_API_KEY
printf '%s' "$GROK_RELAY_KEY" | ccr grok profile create relay \
  --base-url https://api.example.com/v1 \
  --api-key-stdin

ccr grok profile list
ccr grok profile switch xai
ccr grok profile off
ccr grok auth current
```

Use this for:
- managing profile routing across `~/.grok/config.toml` and `~/.grok/user-settings.json`
- referencing environment variables via `--env-key <ENV_VAR>` or piping direct credentials via `--api-key-stdin`

## Workflow 5: Profile backup, sharing, and migration (`ExchangeV1`)

```bash
# Export sanitized profiles for sharing
ccr claude profile export --no-secrets -o claude-team.toml

# Preview and merge incrementally
ccr claude profile import claude-team.toml --dry-run
ccr claude profile import claude-team.toml --on-conflict skip
```

Use this for:
- exporting `ExchangeV1` TOML bundles from `profiles.db` for backups or cross-machine transfers
- previewing changes with `--dry-run` and controlling conflict resolution with `--on-conflict skip|overwrite|rename` (or full catalog replacement with `--replace -y`)

## Workflow 6: Validate and diagnose

```bash
ccr current --verbose
ccr validate --json
ccr doctor --online
```

Current behavior:
- `ccr current --verbose`: shows the runtime overview plus registry target details
- `ccr validate [--json]`: performs static validation across the registry and Claude / Codex / Grok profiles and host settings
- `ccr doctor [--online] [--all-platforms]`: runs unified read-only health checks (including local compatibility checks for `gemini` / `droid` and optional online provider probing)

## Workflow 7: Sync, history, cleanup, and interactive surfaces

```bash
ccr history --limit 50
ccr sync config
ccr sync all status
ccr clean backups --days 30 --dry-run

# Launch the graphical UI or terminal TUI
ccr ui -p 15173 --backend-port 38081
ccr
```

- `ccr ui`: recommended graphical entrypoint
- `ccr`: default interactive terminal entrypoint (TUI)

## Upgrading Legacy Scripts

If existing scripts still invoke retired implicit switching commands or deprecated top-level single-file commands, refer to the [Migration Guide](/en/reference/migration) and [Platform Command Migration](/en/reference/platforms/migration) for the full 7.x replacement map.

## Related docs

- [Quick Start](/en/guide/quick-start)
- [Configuration Model](/en/guide/configuration)
- [Entrypoints](/en/guide/entrypoints)
- [Command Reference](/en/reference/commands/)
