# llmusage 1.1.1 Database Refactor Impact On CCR

## Baseline

- Source checkout: `D:/Documents/Code/CLI/llmusage`
- Source HEAD: `d99762bfdd9f920b0d1859fa0a2f7357a9a48d68`
- Installed executable: `C:/Users/lyh/.cargo/bin/llmusage.exe`, version `1.1.1`
- Representative database: `C:/Users/lyh/.llmusage/llmusage.db`, 1,160,073,216 bytes, schema 18 at inspection time
- CCR baseline: `cargo test -p ccr-usage -- --test-threads=1` passed 33 tests before product edits

No upstream file is in the CCR change scope.

## Schema And Migration Impact

Upstream `src/store/migrations.rs:46-93` defines schema 1-19. CCR-relevant transitions are:

- v10: CCR's current base compatibility line; the usage and pricing columns required by existing projections are present by this point.
- v13 (`src/store/migrations.rs:603-638`): rewrites `source='gemini'` to `source='antigravity'` across usage, cursor, sync and behavior tables, then deletes the old meta key.
- v14 (`src/store/migrations.rs:641-698`): adds `usage_event.provider_label`, rebuilds `usage_bucket_30m` with `provider_label` in its primary key, and recreates `idx_usage_bucket_30m_hour_start`.
- v18 (`src/store/migrations.rs:839-874`): adds behavior/event range indexes including `idx_usage_event_event_at`.
- v19 (`src/store/migrations.rs:876-887`): adds `idx_usage_event_activity_cost`; existing CCR usage projections do not require it.

The base bucket definition at `src/store/migrations.rs:279-293` contains project hash/label/ref but no `project_path`. `project_path` is added only to `usage_event` at `src/store/migrations.rs:432`.

Conclusion: keep CCR's base minimum at 10 and provider minimum at 14. Accept 18, 19 and compatible future schemas by required columns; never require latest=19 globally.

## Source Contract Impact

Upstream `src/domain/models.rs:11-42` defines seven stable ids:

```text
claude, codex, opencode, antigravity, kimi_code, pi, grok
```

Display/capability metadata is in `src/domain/source_descriptor.rs:43-127`. Antigravity is historical-only; Kimi Code, Pi/Oh My Pi and Grok Build are passive sources. The representative database contains all seven ids.

CCR conflicts:

- `crates/ccr-usage/src/source.rs:7-33` exposes only Claude/Codex/Gemini/OpenCode and emits `gemini`.
- `crates/ccr-usage/src/db.rs:562-570` filters source breakdown to those four ids.
- `ccr-ui/src/components/usage/UsageDashboardToolbar.vue:168-173` and `ccr-ui/src/types/usage.ts:56` expose only old four filters.

Observed consequence on the representative DB: Antigravity/Kimi/Pi/Grok omission hides about 4,226 events and 444.5M tokens from CCR source breakdown totals.

## Sync Protocol Impact

Upstream `src/parsers/mod.rs:44-124` emits these events beyond CCR's current enum:

- pricing upgrade started/progress;
- pricing bucket reconcile started;
- pricing upgrade finished;
- token-accounting repair started/finished.

Kimi/Pi support was introduced by `4d6b04e`; Grok by `a5cb8cf`; automatic legacy token repair by `8591059`. A legal current `llmusage sync --json` stream can therefore fail CCR deserialization before any query runs.

CCR must explicitly model these current events and preserve hard failure for malformed or genuinely unknown brace-prefixed JSON.

## Query Contract Impact

Upstream `src/query/filter.rs:20-39,86-143` treats dates as local calendar dates and emits typed UTC half-open bounds. `src/query/timezone.rs` plus `src/store/connection.rs:47-49` registers DST-aware SQLite functions backed by `chrono-tz` and the machine IANA timezone.

CCR conflicts:

- `crates/ccr-usage/src/db.rs:38-73` stores all SQL parameters as strings.
- `crates/ccr-usage/src/db.rs:76-150` filters with `date(hour_start/event_at, 'localtime')`, so the indexed column is wrapped in a function.
- `crates/ccr-usage/src/capabilities.rs:157-190` reopens the database for each feature check even after `Dashboard::open` already owns a connection.
- `crates/ccr-usage/src/db.rs:250-307` performs seven separate bucket aggregate queries for overview plus two run-log reads.
- `crates/ccr-usage/src/db.rs:478-484` probes `usage_bucket_30m.project_path`, an upstream-nonexistent column.
- `crates/ccr-usage/src/db.rs:758-830` runs daily trends and a second date/source aggregate for home overview.

## Representative Query Evidence

Measurements used the same read-only representative database, matching filters, warm-up and repeated median timings.

| Probe | Current CCR shape | Candidate shape | Result |
| --- | ---: | ---: | ---: |
| logs date range | 11.24 ms | 0.127 ms | about 98.9% lower |
| overview bucket work | 7.888 ms / 7 queries | 1.038 ms / 1 query | about 86.8% lower |
| home bucket work | 3.123 ms / 2 queries | 1.294 ms / 1 query | about 58.6% lower |
| principal dashboard bucket sections | 6.144 ms | 4.246 ms | about 30.9% lower |

`EXPLAIN QUERY PLAN` evidence:

- old bucket predicate `date(hour_start, 'localtime') >= ?`: `SCAN usage_bucket_30m`;
- UTC half-open predicate `hour_start >= ? AND hour_start < ?`: uses `idx_usage_bucket_30m_hour_start`.

The timing values are planning evidence, not CI thresholds. Automated enforcement should assert result equivalence, query count and query plans; the same representative DB benchmark must be rerun after implementation.

## Dependency Impact

Upstream uses:

- `rusqlite` with `functions`;
- `chrono-tz 0.10.4`;
- `iana-time-zone 0.1.65`.

CCR already uses `rusqlite 0.39` with `bundled` and has `iana-time-zone` transitively through chrono, but does not directly declare `chrono-tz` or the rusqlite `functions` feature. Adding these as direct production dependencies requires explicit plan approval.

## Final Impact List

1. Source enum/parser, source breakdown and frontend filter list.
2. Home series `gemini` to `antigravity` binding migration while keeping the upstream four-field series.
3. NDJSON event enum and two progress reducers.
4. Typed/DST-aware half-open query filter shared by every bucket/event path.
5. Dashboard-local capability snapshot.
6. Overview and home aggregation consolidation.
7. Project breakdown removal of impossible bucket-column probing.
8. Schema matrix fixtures, current event fixtures, query-plan guards, performance rerun and provider-adapter spec update.
