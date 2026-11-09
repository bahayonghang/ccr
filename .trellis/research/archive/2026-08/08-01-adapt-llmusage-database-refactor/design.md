# Design: 适配 llmusage 数据库重构并优化查询

## 1. Boundary And Data Flow

```text
installed llmusage CLI 1.1.1
  -> NDJSON sync events
  -> ccr-ui llmusage_adapter (parse + progress mapping)

~/.llmusage/llmusage.db
  -> ccr-usage Dashboard (one read-only connection + capability snapshot)
  -> typed projection DTOs
  -> thin Tauri adapter / existing commands
  -> generated TypeScript bindings + Usage views
```

`llmusage` remains the only writer and schema owner. Every SQL statement stays in `crates/ccr-usage`; `ccr-ui/src-tauri/src/llmusage_adapter` must not regain an independent projection implementation.

## 2. Compatibility Matrix

| Schema | Upstream contract relevant to CCR | CCR behavior |
| --- | --- | --- |
| 10-12 | usage/pricing columns exist; historical source key is `gemini` | Keep base usage features; map Antigravity filters to legacy `gemini`; normalize returned key to `antigravity` |
| 13 | migration rewrites `gemini` to `antigravity` | Query only current key; no duplicated Antigravity rows |
| 14 | `provider_label` added to event/bucket and bucket PK | Enable provider breakdown/filter only when both required columns exist |
| 15-17 | behavior, lock and parse-diagnostic evolution | Ignore unrelated additions; preserve read-only required-column detection |
| 18 | range/behavior query indexes; current representative DB | Fully supported; use `idx_usage_event_event_at` where applicable |
| 19 | Activity event-cost covering index only | Fully supported but not required for existing CCR features |
| future | unknown additive schema | Permit read-only projections when required tables/columns remain present; do not migrate |

The existing minimum schema remains 10. Feature support is decided by minimum version plus required columns, not by requiring the source checkout's latest schema.

## 3. Source Contract

`SourceKind` becomes the single seven-value persisted-source model:

| Rust variant | SQLite/wire id | Accepted CCR input aliases | UI label |
| --- | --- | --- | --- |
| Claude | `claude` | `claude-code`, `claude code` | Claude |
| Codex | `codex` | `openai-codex`, `openai codex` | Codex |
| Opencode | `opencode` | `open-code`, `open code` | OpenCode |
| Antigravity | `antigravity` | `gemini`, `gemini-cli`, existing Google Gemini aliases | Antigravity CLI |
| KimiCode | `kimi_code` | `kimi-code`, `kimi code` | Kimi Code |
| Pi | `pi` | `oh-my-pi`, `oh my pi`, `omp` | Pi / Oh My Pi |
| Grok | `grok` | `grok-build`, `grok build` | Grok Build |

For schema 10-12, SQL generation resolves `Antigravity` to the stored `gemini` key. For schema 13+, it resolves to `antigravity`. `source_breakdown` canonicalizes old rows before merging/percentage calculation, so the frontend never has to understand schema history.

The main Usage dashboard exposes all seven sources. The home projection follows the upstream shape: summary and `by_platform` include every detected source, while the fixed daily series remains Claude/Codex/Antigravity/OpenCode. This avoids silently dropping totals without inventing upstream-undefined series fields.

## 4. Timezone And SQL Filter Design

### Typed parameters

`SqlFilter.params` changes from `Vec<String>` to `Vec<rusqlite::types::Value>`. Source/model/provider/project values remain text; computed UTC bounds are RFC 3339 text values matching persisted timestamp ordering.

### Sargable range predicates

User dates remain inclusive at the API boundary. SQL uses:

```sql
hour_start >= :start_utc
AND hour_start < :day_after_end_utc
```

The same shape applies to `usage_event.event_at`. Grouping can use a local-date expression, but range filtering never wraps the indexed timestamp column.

### DST correctness

Reuse the upstream mechanism in a CCR-owned minimal module:

- add direct `chrono-tz` and `iana-time-zone` dependencies;
- enable rusqlite's `functions` feature;
- resolve the machine IANA zone once for a query/dashboard context;
- register deterministic SQLite local-date scalar functions on the read-only connection;
- use actual historical offsets for nonexistent/ambiguous DST boundaries, matching upstream resolution tests.

These are new production dependency declarations. The implementation approval for this plan explicitly approves them; no other production dependency is authorized.

## 5. Capability Snapshot

`Dashboard` owns:

```rust
pub struct Dashboard {
    paths: AppPaths,
    conn: Connection,
    capabilities: DbCapabilitySnapshot,
}
```

`Dashboard::open` opens one read-only connection, configures timeout/timezone functions, reads schema version and required table/column sets once, then stores an immutable snapshot. `ensure_feature_for_filter` evaluates this snapshot and the optional provider filter without opening another connection.

`DbCapabilities::detect(paths)` for external readiness reporting can still open its own short-lived connection. The performance constraint applies to queries performed through an already-open `Dashboard`.

## 6. Query Refactor

| Surface | Current shape | Target shape |
| --- | --- | --- |
| overview | 7 bucket aggregates + 2 run-log scalar queries | 1 bucket conditional aggregate + 1 run-log conditional aggregate |
| daily/model/provider/source/heatmap | range predicates wrap timestamps in `date()` | shared typed UTC half-open predicate; grouping remains local-date aware |
| project | probes nonexistent bucket `project_path` | select project hash/label/ref only; Rust fallback uses ref then label |
| logs | date-wrapped event predicate; existing keyset pagination | sargable event range; preserve cursor, raw join and optional count |
| home | daily trends query + second date/source query | one date/source aggregate from which summary/by-platform/series are derived |
| capabilities | each section reopens DB and repeats PRAGMA checks | immutable snapshot on the Dashboard connection |

The overview query uses `SUM(CASE WHEN hour_start >= ? THEN ... ELSE 0 END)` for last-24h fields. The filter's explicit date bounds continue to constrain both total and last-24h values exactly as today. Run-log query failures retain the existing warn-and-`None` presentation behavior.

No new indexes are added by CCR because it is not the schema owner. Performance must come from using upstream indexes and reducing scans.

## 7. NDJSON Protocol

Add explicit `JobEvent` variants matching upstream fields:

- `pricing_upgrade_started`
- `pricing_upgrade_progress`
- `pricing_bucket_reconcile_started`
- `pricing_upgrade_finished`
- `token_accounting_repair_started`
- `token_accounting_repair_finished`

All current source ids deserialize through the expanded `SourceKind`. Commands map pricing/migration/repair lifecycle events to the existing bootstrap/syncing stage without changing terminal semantics. Non-JSON stdout noise remains skippable; malformed or unknown brace-prefixed JSON remains an explicit protocol error so upstream breaking changes stay visible.

## 8. Frontend And Wire Changes

- Generate `HomeOverviewSeriesItem.antigravity` in place of `.gemini` and update `DashboardUsageMovement` plus fixtures.
- Expand `UsagePlatform` and the toolbar list to seven canonical ids.
- Centralize usage-source labels so toolbar, source summary, cost tab and operational cockpit do not maintain divergent maps.
- Add en-US/zh-CN names and update llmusage dependency/source descriptions that still claim only four old platforms.
- Preserve API argument names and command registration. Incoming `gemini` remains a backend alias, so existing callers do not break.

## 9. Test And Performance Evidence

Fixture matrix:

- schema 10 legacy `gemini`;
- schema 13 post-cutover Antigravity;
- schema 14 provider dimension;
- schema 18 representative indexed shape;
- schema 19 current source shape;
- future schema with unchanged required columns;
- missing/empty/malformed optional and error cases.

Hard deterministic checks:

- query result equivalence;
- query count/connection-open instrumentation;
- exact generated SQL predicates;
- `EXPLAIN QUERY PLAN` contains the expected time index for bucket/event ranges;
- no SQL outside `crates/ccr-usage`;
- current NDJSON event fixtures parse and progress reducers remain exhaustive.

Representative performance checks use a copied or read-only live DB, identical filters, warm-up, multiple iterations and median comparison. Time thresholds are evidence gates, not CI assertions; query plan and query count remain the non-flaky automated gate.

## 10. Rollback

Implementation is ordered so each stage is independently revertible:

1. source/protocol/fixture compatibility;
2. timezone and shared filter machinery;
3. query/capability refactor;
4. wire/frontend adaptation;
5. spec and verification evidence.

If DST registration fails on an unsupported host, return the existing typed database/query error; do not silently fall back to a current fixed offset. If a query optimization changes results, revert that query to the previous aggregation while retaining the independently verified source and half-open-filter fixes.
