# Design — C2 llmusage provider ingest + adapter provider dimension

This child uses the parent design as the source of truth:

- Parent `07-01-provider-usage-stats/design.md` section 2 defines the upstream
  llmusage contract: schema version 14, `provider_label` on `usage_event` and
  `usage_bucket_30m`, bucket key includes `provider_label`, and `sync
  --provider-map <path>` stamps labels from CCR activation intervals.
- Parent `07-01-provider-usage-stats/design.md` section 4 defines the CCR side:
  read-only adapter projection, provider filter, provider breakdown capability
  gate, sync wiring, and Tauri commands.

## Confirmed Precondition

The local llmusage implementation is available through the installed CLI:

- `llmusage --version` reports `0.8.2`.
- `llmusage sync --help` exposes `--provider-map <PATH>`.
- Local `~/.llmusage/llmusage.db` is schema 14 with `provider_label` on both
  `usage_event` and `usage_bucket_30m`; the bucket primary key includes
  `(source, provider_label, model, hour_start, project_hash)`.

Existing rows may still have empty provider labels until the user runs a sync or
rebuild with a provider map. Tests must therefore rely on fixtures for non-empty
provider attribution and use the real DB only as runtime capability evidence.

## CCR Boundary

Keep the existing no-crate contract intact:

- CCR invokes the installed `llmusage` CLI for sync.
- CCR reads the llmusage SQLite DB read-only.
- CCR does not link the upstream llmusage Rust crate, does not migrate the DB,
  and does not parse raw Claude/Codex usage logs.

## Adapter Shape

- Extend `QueryFilter` with an optional `provider` predicate.
- Add `ProviderBreakdownDto` to `queries.rs`.
- Add `Dashboard::provider_breakdown(&QueryFilter)` using `usage_bucket_30m`,
  grouping by `provider_label`; empty string maps to `provider = null`.
- Add `FeatureKey::ProviderBreakdown`, gated by schema `>= 14` and required
  `provider_label` columns.
- Extend `SyncCommandOptions` with `provider_map: Option<PathBuf>` and append
  `--provider-map <path>` when present.
- Resolve the CCR activation map path as
  `$CCR_ROOT/analytics/provider_activation.jsonl` through the existing
  ccr-config root convention when usage import commands run.

## Command Shape

- Add `get_usage_by_provider_v2(platform?, start?, end?)`.
- Add `provider` to `get_usage_dashboard_v2` so drill-down can reuse existing
  dashboard reads.
- Register the new command in `commands/handler_registry.rs`.

## Compatibility

Older llmusage DBs must continue to support existing usage commands. Provider
breakdown should return the existing typed unsupported error path when schema is
below 14 or required provider columns are absent.
