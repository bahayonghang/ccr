# Migration Guide

This page explains the move from the old global platform/profile routing model to the explicit Claude Runtime / Codex Runtime model.

## Command migration quick map

| Legacy command | Current path | Notes |
|---|---|---|
| `ccr switch <name>` | `ccr claude profile switch <name>` / `ccr codex profile switch <name>` | no more implicit platform inference |
| `ccr <name>` | same mapping | shortcut retired |
| `ccr platform switch <platform>` | no longer the main auth/profile path | use explicit profile/auth commands |
| `ccr platform current` | `ccr current` | inspect dual runtime state |
| `ccr platform profile ...` | `ccr claude profile ...` / `ccr codex profile ...` | explicit platform-scoped profile commands |

## Registry migration

- older files may still contain `default_platform` / `current_platform`
- CCR still reads them for backward compatibility
- the routing truth is now each platform entry's `current_profile`

## ccr-ui usage analytics migration to llmusage

ccr-ui Usage Dashboard now reads from the `llmusage` 0.5.1 runtime instead of the legacy `ccr-db` usage importer. This only changes the desktop usage analytics path. It does not change Claude / Codex profiles, the ccr-ui SessionIndexer, or the legacy Stats / budget pages. The old `ccr-db` usage schema remains for compatibility, but it is no longer the new data source for Usage Dashboard.

### Data location

By default, ccr-ui follows llmusage's standard `AppPaths::discover()` resolution:

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

- Opening the ccr-ui Usage page and starting an import now uses llmusage `JobRegistry` while preserving the existing `usage:job-progress`, `usage:job-recent-ready`, `usage:job-finished`, and `usage:job-failed` events.
- ccr-ui keeps its previous 30-day recent import window by default. Use the CLI command above, or a future maintenance entry point, when you need a full rebuild.
- `cache_savings`, dual-cost fields, and log `recorded_at` now come directly from the llmusage adapter, so the frontend no longer derives them from `total_cost` or cost deltas.

### Rollback and compatibility boundary

- Legacy `ccr-db` usage tables are not deleted; historical rows remain in place.
- This migration does not include a `ccr-db -> llmusage` historical importer. To rebuild llmusage data, reparse local Claude / Codex / Gemini / OpenCode source logs.
- `ccr-store::CostTracker` still serves the legacy Stats / budget flow and is outside the Usage Dashboard retirement scope.
