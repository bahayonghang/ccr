# Migration Guide

This page summarizes three core migrations in CCR 7.x: the move to explicit platform-scoped commands, the `profiles.toml -> profiles.db` SQLite catalog upgrade, and the `ccr-ui` usage analytics transition to the external `llmusage` runtime.

> There is no `ccr migrate` command; catalog migration runs automatically when you invoke canonical commands.

## Command migration quick map (7.x)

| Legacy command (Removed / Retired / Hidden in 7.x) | Canonical 7.x path | Notes |
|---|---|---|
| `ccr switch <name>` | `ccr <claude\|codex\|grok> profile switch <name>` | Retired; target platform is explicit in the command path |
| `ccr <name>` | `ccr <claude\|codex\|grok> profile switch <name>` | Shortcut retired |
| `ccr platform switch <platform>` | `ccr <claude\|codex\|grok> profile switch <name>` / `auth ...` | Removed from Clap; no implicit global platform |
| `ccr platform current` | `ccr current` (aliases: `ccr status`, `ccr show`) | Inspect side-by-side runtime state |
| `ccr platform info <platform>` | `ccr platform list [--json]` / `ccr doctor --platform <name>` | Removed; inspect registry or diagnostics instead |
| `ccr platform init <platform>` | `ccr <claude\|codex\|grok> profile init` | Removed; initializes `profiles.db` from embedded template |
| `ccr platform profile ...` | `ccr <claude\|codex\|grok> profile <action>` | Removed; consolidated into platform `profile` commands |
| `ccr init` | `ccr <claude\|codex\|grok> profile init` (`ccr project init` for repo workflows) | Hidden and deprecated in 7.x (removed in 8.0) |
| `ccr list` | `ccr <claude\|codex\|grok> profile list` | Hidden and deprecated in 7.x |
| `ccr add` | `ccr <claude\|codex\|grok> profile create <name> ...` or `profile open` | Pipe secrets via `--auth-token-stdin` / `--api-key-stdin` / `--env-key` |
| `ccr delete` / `enable` / `disable` | `ccr <claude\|codex\|grok> profile delete\|enable\|disable <name>` | Hidden and deprecated in 7.x |
| `ccr clear` / `ccr optimize` | `ccr <claude\|codex\|grok> profile off` / `profile open` | Leave profile mode or edit catalog projection in `$EDITOR` |
| `ccr export` | `ccr <claude\|codex\|grok> profile export [--no-secrets] [-o file.toml]` | Exports `ExchangeV1` TOML |
| `ccr import` | `ccr <claude\|codex\|grok> profile import <file.toml> [--dry-run] [--on-conflict skip\|overwrite\|rename] [--replace -y]` | Merges incrementally by default (legacy `ccr import` replaced the whole catalog by default) |

## Profile storage and registry migration

### `profiles.toml` -> SQLite `profiles.db`

- In CCR 7.x, the authoritative live profile catalog for Claude, Codex, and Grok is `~/.ccr/platforms/<platform>/profiles.db` (SQLite).
- Existing `~/.ccr/platforms/<platform>/profiles.toml` files are automatically migrated into `profiles.db` on first access, with the original TOML backed up under `backups/<platform>/`.
- TOML is now used strictly for temporary `$EDITOR` projections via `ccr <claude|codex|grok> profile open` and for `ExchangeV1` bundles via `profile export` / `profile import`; copying a backup TOML back onto the live path will not be read by the 7.x binary.

### Registry compatibility

- Older `~/.ccr/config.toml` files may still contain `default_platform` / `current_platform`, which remain readable for backward compatibility.
- The active routing pointer is now each platform entry's independent `current_profile` field.

## Mental model migration

Legacy mental model:

- pick a global “current platform” first
- run generic top-level commands against a `profiles.toml` file

Current mental model:

- run `ccr current` for a cross-platform runtime overview
- invoke `ccr <claude|codex|grok> profile ...` or `ccr <claude|codex|grok> auth ...` directly
- keep official login state (`auth`) separate from runtime profile routing (`profile`)

## ccr-ui usage analytics migration to llmusage

ccr-ui Usage Dashboard now treats `llmusage` as an external runtime instead of using the legacy `ccr-db` usage importer: sync/import calls the installed `llmusage` CLI, rendering reads the `llmusage` SQLite database read-only, and ccr-ui no longer links the upstream `llmusage` Rust crate. This only changes the desktop usage analytics path. It does not change Claude / Codex / Grok profiles, the ccr-ui `SessionIndexer`, or the legacy Stats / budget pages. The old `ccr-db` usage schema remains for compatibility, but it is no longer the new data source for Usage Dashboard.

### Data location

By default, ccr-ui uses a root resolution order aligned with the llmusage CLI:

1. `LLMUSAGE_HOME`
2. `~/.llmusage`

That means ccr-ui and llmusage Web/CLI read the same local SQLite root by default:

```text
~/.llmusage/llmusage.db
```

The previous CCR-isolated root (`~/.ccr/llmusage`) is not automatically read, merged, or migrated. The Usage diagnostics panel shows the active Archive Root so you can confirm which store is being queried. To use a different store intentionally, set `LLMUSAGE_HOME` before launching ccr-ui or llmusage:

```powershell
$env:LLMUSAGE_HOME = "D:\data\llmusage"
llmusage status
llmusage sync --rebuild --recent-days 30
```

### First run and resync

- Opening the ccr-ui Usage page and starting an import now runs `llmusage --home <root> sync --json-events` and bridges CLI events into the existing `usage:job-progress`, `usage:job-recent-ready`, `usage:job-finished`, and `usage:job-failed` events.
- ccr-ui keeps its previous 30-day recent import window by default. Use the CLI command above, or a future maintenance entry point, when you need a full rebuild.
- `cache_savings`, dual-cost fields, and log `recorded_at` now come from the ccr-ui read-only SQL adapter over llmusage tables/views, so the frontend no longer derives them from `total_cost` or cost deltas.

### Rollback and compatibility boundary

- Legacy `ccr-db` usage tables are not deleted; historical rows remain in place.
- This migration does not include a `ccr-db -> llmusage` historical importer. To rebuild llmusage data, reparse local Claude / Codex / Gemini / OpenCode source logs.
- `ccr-store::CostTracker` still serves the legacy Stats / budget flow and is outside the Usage Dashboard retirement scope.
