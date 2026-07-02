# C2 — llmusage provider ingest + adapter provider dimension

Parent: `07-01-provider-usage-stats` · Design: parent `design.md` §2 + §4 · Order: after C1

## Goal

Make the provider dimension real end-to-end: llmusage stamps `provider_label`
onto usage events from CCR's activation timeline, and the CCR `llmusage_adapter`
reads it, exposing a per-provider breakdown + `provider` filter through a new
Tauri command — schema-gated so old DBs degrade gracefully.

## Requirements

### Upstream llmusage (user implements per parent design §2)

- `provider_label TEXT NOT NULL DEFAULT ''` on `usage_event` **and**
  `usage_bucket_30m` (`''` = unattributed); bump `schema_version` to `14`
  (real llmusage is at 13).
- `usage_bucket_30m` aggregation key **includes** `provider_label` (else
  per-provider bucket sums are wrong).
- `sync --provider-map <path>` builds per-platform half-open intervals from the
  timeline and stamps `provider_label` by `(source == platform, event_at ∈
window)`; unmatched/`clear`/pre-first-window → `''`. Rebuildable via `--rebuild`.
  UTC RFC3339 on both sides.

### CCR adapter + wiring (this repo)

- `llmusage_adapter/db.rs`: `provider_breakdown(&QueryFilter)` (GROUP BY
  `provider_label`, `''`→unattributed); add `provider` predicate to the filter;
  add `provider_label` to the new feature's required columns.
- `queries.rs`: `ProviderBreakdownDto` (provider + token splits + requests +
  cost_with/without_cache_usd).
- `capabilities.rs`: `FeatureKey::ProviderBreakdown`, gated on `schema_version ≥ 14`
  - column presence; reuse `ensure_feature` degrade path.
- `cli.rs`: `SyncCommandOptions.provider_map` → `--provider-map`; import commands
  in `commands/usage.rs` pass `$CCR_ROOT/analytics/provider_activation.jsonl`.
- New command `get_usage_by_provider_v2(platform?, start?, end?)` +
  registration; `provider` param on `get_usage_dashboard_v2`.
- Preserve the read-only, no-crate-link adapter contract (NFR1).

## Acceptance Criteria

- [ ] Against a synced DB at schema `N`, `provider_breakdown` per-provider sums +
      unattributed == the source-level totals for the same window.
- [ ] `provider` filter narrows results correctly; `clear` windows land in
      unattributed.
- [ ] On a schema `< N` DB (no `provider_label`), the feature reports unsupported
      and existing `/usage` commands still work (degrade, no error).
- [ ] `tests/llmusage_no_crate_guard.rs` still passes; new gate/degrade test added.
- [ ] `cd ccr-ui/src-tauri && cargo check`, `just lint-strict`, `just test` pass.
- [ ] Reviews: tauri-ipc-reviewer, sqlite-migration-reviewer, rust-security-reviewer.

## Notes / dependencies

- Depends on C1's provider-map format and on the user's upstream llmusage build
  (or a schema-`N` fixture DB for local dev/tests).
- Blocks C3 (commands) and is the preferred data path for C4.
