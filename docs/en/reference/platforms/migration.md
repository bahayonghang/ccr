# Platform Command Migration Guide

This page explains how to migrate from legacy global platform switches and top-level single-file commands to CCR 7.x explicit platform commands and SQLite `profiles.db` catalogs.

## Core Rules

- There is **no** standalone `ccr migrate` subcommand; legacy catalogs upgrade automatically on first canonical command access.
- Removed `ccr platform switch|current|info|init|profile` subcommands are no longer parsed by Clap; use the canonical commands in the table below (see also the [`platform` command reference](../commands/platform)).
- Top-level `ccr init|list|add|delete|enable|disable|clear|optimize|export|import` commands are hidden and deprecated in 7.x (printing a deprecation warning on `stderr` and scheduled for removal in 8.0).
- `Platform::auth_profile_supported()` includes only `claude`, `codex`, and `grok`. `gemini` (Antigravity CLI) and `droid` (Factory Droid) are retained compatibility adapters (`legacy_adapter`) with no CLI `profile` or `auth` subcommands.

## Legacy-to-7.x Command Replacement Map

| Legacy Command (Removed / Retired / Hidden in 7.x) | Canonical 7.x Replacement | Notes |
|---|---|---|
| `ccr platform init <platform>` | `ccr <claude\|codex\|grok> profile init` | Initializes `~/.ccr/platforms/<platform>/profiles.db` from embedded template |
| `ccr platform switch <platform>` | `ccr <claude\|codex\|grok> profile switch <name>` | Target platform is explicit in the command prefix |
| `ccr platform current` | `ccr current` (aliases: `status`, `show`) | Side-by-side runtime overview |
| `ccr platform info <platform>` | `ccr platform list [--json]` / `ccr doctor --platform <name>` | Inspect registry entries or deep platform health |
| `ccr platform profile ...` | `ccr <claude\|codex\|grok> profile <action>` | Consolidated under per-platform `profile` commands |
| `ccr switch <name>` / `ccr <name>` | `ccr <claude\|codex\|grok> profile switch <name>` | Retired; returns explicit migration guidance |
| `ccr init` | `ccr <claude\|codex\|grok> profile init` (`ccr project init` for repo workflows) | Hidden 7.x compatibility alias |
| `ccr list` | `ccr <claude\|codex\|grok> profile list` | Hidden 7.x compatibility alias |
| `ccr add` | `ccr <claude\|codex\|grok> profile create <name> ...` or `profile open` | Pipe secrets via `--auth-token-stdin` / `--api-key-stdin` / `--env-key` |
| `ccr delete` / `enable` / `disable` | `ccr <claude\|codex\|grok> profile delete\|enable\|disable <name>` | Hidden 7.x compatibility aliases |
| `ccr clear` / `ccr optimize` | `ccr <claude\|codex\|grok> profile off` / `profile open` | Exit profile mode or edit catalog projection in `$EDITOR` |
| `ccr export` | `ccr <claude\|codex\|grok> profile export [--no-secrets] [-o file.toml]` | Exports `ExchangeV1` TOML |
| `ccr import` | `ccr <claude\|codex\|grok> profile import <file.toml> [--dry-run] [--on-conflict skip\|overwrite\|rename] [--replace -y]` | Merges incrementally by default (unlike legacy `ccr import`, which replaced by default) |

## Pattern 1: Initialize and manage platforms side by side

```bash
# Initialize SQLite profile catalogs (profiles.db) for supported platforms
ccr claude profile init
ccr codex profile init
ccr grok profile init

# Switch and inspect each platform directly with no global platform switch
ccr claude profile switch <claude-profile>
ccr codex profile switch <codex-profile>
ccr grok profile switch <grok-profile>
ccr current
```

> For `gemini` (Antigravity CLI) and `droid` (Factory Droid), manage their host settings files directly (or use the `/antigravity` module in `ccr ui`) and run read-only checks with `ccr doctor --platform gemini|droid`.

## Pattern 2: Export, share, and migrate via `ExchangeV1`

Export sanitized or full bundles with `profile export`, then preview and import them with `profile import`:

```bash
# Export sanitized ExchangeV1 TOML without plaintext secrets
ccr claude profile export --no-secrets -o claude-profiles.toml

# Preview the import, then merge incrementally into the target profiles.db
ccr claude profile import claude-profiles.toml --dry-run
ccr claude profile import claude-profiles.toml --on-conflict skip

# Populate real credentials safely via stdin and validate
printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile edit work --auth-token-stdin
ccr validate
```

Notes:
- `ccr <platform> profile import` performs an **incremental merge** by default, controlled by `--on-conflict skip|overwrite|rename`; pass `--replace -y` only when replacing the entire catalog.
- Review model identifiers, endpoints, and credential fields whenever adapting profiles across platforms.

## Pattern 3: Upgrade from legacy `profiles.toml` or `~/.ccs_config.toml`

- **Unified `~/.ccr/platforms/<platform>/profiles.toml`**: automatically migrated into `profiles.db` on first 7.x access, with the original TOML backed up under `backups/<platform>/`.
- **Legacy single-file `~/.ccs_config.toml`**: treat it as reference input, initialize the target catalog, and recreate profiles via `profile create` or `profile open`:

```bash
ccr claude profile init
printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile create work \
  --base-url https://api.example.com \
  --auth-token-stdin
ccr claude profile list
```

## Post-Migration Verification Checklist

```bash
ccr current --verbose
ccr claude profile list
ccr codex profile list
ccr grok profile list
ccr validate
ccr doctor --all-platforms
```

Verify that:
- `profiles.db` is initialized and lists expected profiles;
- switching writes the expected host runtime files;
- `ccr validate` and `ccr doctor` report no configuration or credential errors.

## Rollback Primitives

- Switch to another profile on the same platform: `ccr <claude|codex|grok> profile switch <previous-name>`
- Leave CCR profile mode and return to official login: `ccr <claude|codex|grok> profile off`
- Restore a known-good `ExchangeV1` snapshot across the whole catalog: `ccr <claude|codex|grok> profile import snapshot.toml --replace -y`

## See Also

- [Platform Support](./index)
- [Claude Platform](./claude)
- [Codex Platform](./codex)
- [Antigravity CLI Platform](./gemini)
- [Architecture](/en/reference/architecture)
