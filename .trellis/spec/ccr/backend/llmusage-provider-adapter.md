# llmusage Provider Adapter Contract

> Executable contract for the CCR desktop llmusage provider dimension.

## Scenario: current llmusage read-only analytics across schema 10+

### 1. Scope / Trigger

- Trigger: changing llmusage sources, schema compatibility, sync NDJSON events, usage queries, provider attribution, dashboard filters, or the read-only adapter.
- Applies to `crates/ccr-usage/**`, `ccr-ui/src-tauri/src/llmusage_adapter/**`, `ccr-ui/src-tauri/src/commands/usage.rs`, TUI usage readers, and the matching frontend usage API/types.
- CCR must keep the upstream no-crate boundary: invoke the installed `llmusage` CLI for sync and read the SQLite DB read-only; do not link the upstream `llmusage` Rust crate. The local `ccr-usage` crate is allowed and is the shared read-only projection owner: **every** usage SQL statement (overview, trends, model/provider/project/source breakdowns, heatmap, logs, diagnostics, home overview) lives in `crates/ccr-usage`. Review checklist: `rg 'usage_bucket_30m' --type rust` must only hit `crates/ccr-usage` for SQL (doc/comment hits elsewhere are fine).

### 2. Signatures

- Sync options:
  ```rust
  pub struct SyncCommandOptions {
      pub provider_map: Option<PathBuf>,
      // existing fields omitted
  }
  ```
- Adapter filter and projection:
  ```rust
  // crates/ccr-usage
  pub struct AppPaths {
      pub root_dir: PathBuf,
      pub db_path: PathBuf,
  }

  pub enum SourceKind {
      Claude,
      Codex,
      Opencode,
      Antigravity,
      KimiCode,
      Pi,
      Grok,
      Zcode,
      DeepseekHarness,
  }

  // Canonical stored/wire ids:
  // claude, codex, opencode, antigravity, kimi_code, pi, grok, zcode, deepseek_harness.
  // gemini, existing Gemini spellings, and antigravity_ide are input aliases for Antigravity;
  // omp is an alias for Pi; kimi is an alias for KimiCode (SourceKind::parse_id and serde).

  pub struct QueryFilter {
      pub source: Option<SourceKind>,
      pub model: Option<String>,
      pub provider: Option<String>,
      pub since: Option<NaiveDate>,
      pub until: Option<NaiveDate>,
      pub project_hash: Option<String>,
      pub timezone: ReportTimezone,
  }

  pub fn open_dashboard(paths: AppPaths) -> Result<Dashboard, UsageError>;

  pub struct Dashboard {
      paths: AppPaths,
      conn: rusqlite::Connection,
      capabilities: DbCapabilitySnapshot,
      local_zone: ResolvedZone,
  }

  // Dashboard owns the full read-only query surface:
  // overview / trends_daily / model_breakdown / provider_breakdown /
  // project_breakdown / source_breakdown / heatmap / logs / diagnostics /
  // home_overview / insights, all gated through ensure_feature_for_filter.
  pub fn provider_breakdown(
      &self,
      filter: &QueryFilter,
  ) -> Result<Vec<ProviderBreakdownDto>, UsageError>;

  // Source-tagged variant for surfaces mixing several sources in one list
  // (queries each source separately, then tags the rows).
  pub struct TaggedProviderBreakdown {
      pub source: SourceKind,
      pub breakdown: ProviderBreakdownDto,
  }

  pub fn provider_breakdown_by_source(
      &self,
      sources: &[SourceKind],
      filter: &QueryFilter,
  ) -> Result<Vec<TaggedProviderBreakdown>, UsageError>;

  // DTO/error ownership: projection DTOs (OverviewPayload, DailyTrendDto,
  // ModelBreakdown, ProviderBreakdownDto, …) are defined once in ccr_usage;
  // ccr-ui/src-tauri/src/llmusage_adapter re-exports them (keeping the right
  // to fork later) and keeps its own error type (LlmusageAdapterError, with
  // CLI-only variants) plus presentation-mapping DTOs (UsageSummaryDto,
  // ModelStatDto, …). Errors are mapped at the adapter boundary.
  ```
- Tauri commands:
  ```rust
  get_usage_by_provider_v2(platform?: string, start_date?: string, end_date?: string)
  get_usage_dashboard_v2(platform?, provider?, start_date?, end_date?, heatmap_days?, include_heatmap?)
  ```
- Frontend wrapper shape:
  ```typescript
  getUsageByProviderV2(platform?: string, startDate?: string, endDate?: string)
  getUsageDashboardV2(platform?, startDate?, endDate?, heatmapDays?, includeHeatmap?, provider?)
  ```

### 3. Contracts

- Provider data comes from llmusage schema 14 columns: `usage_event.provider_label` and `usage_bucket_30m.provider_label`.
- `FeatureKey::ProviderBreakdown` requires schema `>= 14` and both provider columns. Existing non-provider features keep their existing minimum schema unless a provider filter is supplied.
- Base projections support schema 10 and later by minimum version plus required table/column checks. Schema 13 changes persisted `gemini` to `antigravity`; schema 14 adds provider columns; schema 18 adds the event range index; schema 19 adds only an Activity covering index. Compatible future additive schemas remain readable and must never be migrated by CCR.
- `SourceKind::Antigravity::storage_key(schema_version)` resolves to `gemini` for schema 10-12 and `antigravity` for schema 13+. SQL projections for source breakdown, logs, diagnostics, and home overview canonicalize returned legacy `gemini` values to `antigravity` before they reach Tauri or the frontend.
- The current source set is exactly `claude`, `codex`, `opencode`, `antigravity`, `kimi_code`, `pi`, `grok`, `zcode`, and `deepseek_harness`. Usage filtering and source-share denominators include all nine; unknown database sources may remain visible as raw labels but are not promoted into typed frontend filters.
- `crates/ccr-usage` owns `Dashboard::provider_breakdown` and **all** usage SQL (overview, trends, model/provider/project/source breakdowns, heatmap, logs, diagnostics, home overview) for all CCR surfaces. Tauri and TUI code must delegate to this crate instead of duplicating any aggregation query; the Tauri adapter `Dashboard` is a thin wrapper that only maps `UsageError` to `LlmusageAdapterError`.
- `Dashboard::open` opens one `SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_URI` connection, registers the DST-aware SQLite function, reads schema/required columns once into an immutable capability snapshot, and reuses both for every section. Section feature gates must not reopen the database.
- Date API bounds are inclusive local calendar dates. Query SQL converts them to typed RFC 3339 UTC half-open bounds (`timestamp >= ? AND timestamp < ?`), preserving `hour_start` and `event_at` index use. Local grouping uses the machine IANA zone with historical DST rules; an unavailable or unknown IANA zone is an explicit `UsageError::Query`, never a current-offset fallback.
- `overview` performs one bucket conditional aggregate and one run-log conditional aggregate. `home_overview` performs one date/source bucket aggregate. Project breakdown uses bucket `project_hash`/`project_label`/`project_ref`; `project_path` exists only on `usage_event` and must not be probed on buckets.
- `Dashboard::provider_breakdown` groups `usage_bucket_30m` by `provider_label`, returns token splits and both cache-aware/cache-free costs, and maps empty provider labels to `provider = null`.
- `AppPaths::discover()` honors `LLMUSAGE_HOME`, otherwise uses `<home>/.llmusage`; Tauri may use `AppPaths::from_root(existing_root)` to preserve its existing path contract.
- A provider filter must apply to overview, daily trends, model breakdown, source breakdown, project breakdown, heatmap, and logs through the shared `QueryFilter`.
- `get_usage_dashboard_v2` includes `provider_stats` and its cache key must include the provider filter.
- When no provider filter is supplied and provider capability is unavailable, dashboard payloads degrade to `provider_stats: []`; an explicit provider filter must surface the unsupported error.
- Sync NDJSON accepts the current pricing upgrade, bucket reconcile, and token-accounting repair lifecycle events in addition to migration/lock/source/terminal events. These additive lifecycle events keep the existing running/importing stage; malformed or unknown brace-prefixed JSON remains a protocol error.
- Home `by_platform` and summary include all detected sources. The fixed daily wire series remains `claude`, `codex`, `antigravity`, and `opencode`; do not invent Kimi/Pi/Grok/ZCode/DeepSeek Harness fixed series fields. Frontend typed filters and the Usage toolbar expose the nine canonical ids while retaining backend `gemini` alias input compatibility.
- TUI usage surfaces (the profile-detail Usage section; the standalone Usage tab was retired 2026-07) should load the shared projection on a background task through an injectable loader seam (`UsageLoader`), consume `TaggedProviderBreakdown` directly (no per-surface shadow row structs), request `SourceKind::Claude` and `SourceKind::Codex` separately when rendering those platform sections (via `provider_breakdown_by_source`), and display `provider = null` as `unattributed` (`ProviderBreakdownDto::display_provider`).
- Only pass `--provider-map <path>` when `$CCR_ROOT/analytics/provider_activation.jsonl` exists. The installed llmusage CLI treats an explicit missing provider-map path as a hard sync error.
- New frontend business wrappers belong in `src/api/domains/*`; `src/api/tauri.ts` remains a compatibility facade.
- The legacy `ccr_store::CostTracker` stats command family (`get_cost_overview`, `get_provider_usage`, `get_daily_stats`, 10 commands total, plus `stats_snapshot.rs`) was removed from ccr-ui in 2026-07 (usage-family-absorb). All statistics surfaces consume V2 usage commands; do not reintroduce JSONL-scan stats commands. The only remaining `CostTracker` consumers in ccr-ui are the claude budgets path (`claude_get_budgets`) and the startup storage-dir check in `main.rs`.

### 4. Validation & Error Matrix

- DB schema `< 14` + provider breakdown/filter -> `SchemaUnsupported { expected: 14, ... }`.
- Schema 14 without `provider_label` on either required table -> `FeatureUnavailable { feature: "provider_breakdown", ... }`.
- Dashboard without provider filter on an old DB -> success with `provider_stats: []`.
- Dashboard with explicit provider filter on an old DB -> error; do not silently return unfiltered data.
- `ccr-usage::open_dashboard` missing DB -> `UsageError::DbMissing`.
- `ccr-usage::open_dashboard` cannot open the DB -> `UsageError::DbUnreadable`.
- Capability detection can open the DB but cannot read schema metadata -> every DB-backed feature is unsupported with `UnsupportedReason::DbUnreadable`; never return an empty feature map.
- Schema `< 10` or absent/malformed textual schema version -> `UsageError::SchemaUnsupported` for dashboard open.
- Machine IANA timezone lookup/parsing or SQLite function registration failure -> `UsageError::Query`; do not silently use `Local::now()` as a fixed offset.
- `ccr-usage::provider_breakdown` missing provider table/column -> `UsageError::FeatureUnavailable`.
- Missing activation log file -> omit `--provider-map`; sync should still run.
- Existing activation log file -> pass `--provider-map <path>` after other sync options.

### 5. Good / Base / Bad Cases

- Good: schema 14 fixture with `openai`, `anthropic`, and empty labels; provider totals sum to source totals and empty labels serialize as `null`.
- Good: schema 10 `gemini` rows filter and return as canonical `antigravity`; schema 13+ uses only the current stored key.
- Good: schemas 18, 19, and a compatible future schema return equivalent overview results; bucket/event range plans name the upstream time indexes.
- Good: `provider = "openai"` narrows overview/model/source totals to only OpenAI-attributed rows.
- Good: Tauri provider dashboard and the TUI profile-detail Usage section both call `ccr-usage` and differ only in DTO/error mapping and presentation.
- Base: old llmusage DB still renders the dashboard, but provider stats are empty until the user upgrades/syncs.
- Base: an additive unknown database source contributes to home/source totals and displays its raw label, but cannot be emitted as a typed filter until the source contract is updated.
- Bad: wrapping `hour_start` or `event_at` in `date(..., 'localtime')` inside `WHERE`; this disables the range-index path and uses a current offset instead of historical DST.
- Bad: resolving an unavailable IANA zone to the current fixed offset; results become seasonally wrong without surfacing an error.
- Bad: querying `usage_bucket_30m.project_path`; upstream has never defined that bucket column.
- Bad: adding provider filtering only to `provider_breakdown`; dashboard cards would show mixed-provider totals.
- Bad: copying any usage SQL (provider breakdown or otherwise) into `ccr-ui` or `ccr-tui`; future schema/capability fixes would diverge.
- Bad: re-introducing a per-surface shadow row struct (field-by-field copy of a ccr-usage DTO plus a tag) instead of consuming `TaggedProviderBreakdown`.
- Bad: always passing `--provider-map` even when the activation log file is absent.
- Bad: adding a new direct `invoke()` wrapper in `src/api/tauri.ts`.

### 6. Tests Required

- `cargo test -p ccr-usage -- --test-threads=1` with assertions for schema 10/13 source cutover, typed UTC/DST bounds, query plans, one-connection capability reuse, aggregation equivalence, logs/diagnostics canonicalization, and unreadable capability snapshots.
- `cargo test -p ccr-usage --features test-fixtures -- --test-threads=1` with schema 18/19/future overview compatibility and the full projection fixture.
- `cargo test -p ccr-tui -- --test-threads=1` when adding or changing TUI usage surfaces
- `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml llmusage_adapter -- --nocapture`
- `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml services::usage::service_tests -- --nocapture --test-threads=1`
- `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::handler_registry -- --nocapture`
- `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml --test llmusage_no_crate_guard -- --nocapture`
- `cd ccr-ui && bun run test:smoke -- tests/usage/usage-dashboard-payload.smoke.test.ts tests/usage-dashboard-toolbar.smoke.test.ts tests/usage-source-summary-card.smoke.test.ts tests/home-usage-overview.store.smoke.test.ts`
- `cd ccr-ui && bun run test:smoke -- tests/api/api-facade-boundary.smoke.test.ts`
- `cd ccr-ui && bun run type-check`
- `cd ccr-ui && bun run lint`

### 7. Wrong vs Correct

#### Wrong

```rust
let options = SyncCommandOptions {
    provider_map: Some(activation_log_path),
    ..Default::default()
};
```

This turns a missing activation log into a strict llmusage sync failure.

#### Correct

```rust
let provider_map = activation_log_path.is_file().then_some(activation_log_path);
let options = SyncCommandOptions {
    provider_map,
    ..Default::default()
};
```

Gate provider analytics by schema capability, preserve old-dashboard behavior, and keep sync tolerant when CCR has not yet written an activation log.

#### Wrong

```rust
// In a Tauri or TUI surface:
let mut stmt = conn.prepare("SELECT provider_label, SUM(total_tokens) FROM usage_bucket_30m GROUP BY provider_label")?;
```

This creates a second provider projection with separate schema gates, filtering semantics, and unattributed-row handling.

#### Correct

```rust
let dashboard = ccr_usage::open_dashboard(ccr_usage::AppPaths::from_root(root_dir))?;
let rows = dashboard.provider_breakdown(&ccr_usage::QueryFilter {
    source: Some(ccr_usage::SourceKind::Codex),
    ..ccr_usage::QueryFilter::default()
})?;
```

Keep provider attribution in `crates/ccr-usage`; presentation layers only map DTOs, errors, and UI labels.

#### Wrong

```sql
WHERE date(hour_start, 'localtime') >= ?
  AND date(hour_start, 'localtime') <= ?
```

This wraps the indexed column, prevents a sargable range scan, and uses SQLite's process-local offset behavior instead of the resolved IANA zone.

#### Correct

```sql
WHERE hour_start >= ? AND hour_start < ?
```

Bind typed RFC 3339 UTC instants computed from the requested local dates and the dashboard's resolved IANA zone. Keep the local-date function for grouping only.

## Scenario: Adopt upstream llmusage static model pricing in CCR

### 1. Scope / Trigger

- Trigger: local/upstream `llmusage` adds or changes static model pricing that CCR legacy archives, default pricing rows, or desktop model breakdowns must recognize.
- Applies to `crates/ccr-types/src/model_rate_catalog.rs`, legacy `ccr-store` pricing defaults, legacy `ccr-db` import/migration/model-stat readers, `crates/ccr-usage` read-only projections, and `ccr-ui/src-tauri/src/services/usage.rs`.
- CCR must keep the dependency boundary: do not add the upstream `llmusage` Rust crate. Legacy CCR-owned pricing is embedded in `ccr-types`; desktop dashboard rows from `llmusage.db` are passed through from `crates/ccr-usage`.

### 2. Signatures

- Embedded catalog:
  ```rust
  pub fn official_model_rate_overrides() -> Vec<ModelRateOverride>;
  pub fn normalize_model_id(model: &str) -> String;
  pub struct ModelRateCatalog;
  impl ModelRateCatalog {
      pub fn official() -> Self;
      pub fn calculate(&self, model: &str, input: i64, output: i64, cache_read: i64, cache_creation: i64) -> PricingComputation;
      pub fn rate_summary(&self, model: &str) -> Option<String>;
  }
  ```
- Read-only projection fields owned by upstream `llmusage.db`:
  ```rust
  ModelBreakdown {
      model,
      cost_with_cache_usd,
      cost_without_cache_usd,
      pricing_status,
      pricing_source,
      pricing_rate,
      ..
  }
  ```

### 3. Contracts

- Add canonical CCR default rows only for model ids CCR should expose in built-in defaults. Aliases may resolve for calculation without creating duplicate default rows.
- Alias matching must be exact after known provider-prefix normalization. For example, `anthropic/claude-fable-5`, `anthropic.claude-fable-5`, and `anthropic-claude-fable-5` may resolve, but `not-fable-5` and preview names must not.
- Legacy CCR import/migration/model-stat paths may calculate prices through `ModelRateCatalog::official()`.
- Desktop Usage Dashboard must not recalculate `llmusage.db` pricing. It should preserve stored `pricing_status`, `pricing_source`, `pricing_rate`, and stored cache-aware/cache-free costs.
- Installed-CLI detection remains tolerant. Do not add a hard minimum installed `llmusage` version gate unless product policy explicitly asks for it.

### 4. Validation & Error Matrix

- Missing catalog row in `ccr-types` -> legacy import/migration model stats mark the model `unpriced`.
- Overbroad alias matching -> unrelated models can be priced incorrectly; add negative tests whenever adding aliases.
- `llmusage.db` row has `pricing_status = static`, `pricing_source = static-v1`, `pricing_rate = 10/1/50` -> desktop DTO must return those exact values.
- Older installed `llmusage` writes `unpriced` rows -> CCR displays the stored state instead of fabricating static pricing.
- New `llmusage = ...` manifest dependency or `llmusage::` import -> architecture violation; the no-crate guard must fail.

### 5. Good / Base / Bad Cases

- Good: add `claude-fable-5` and `claude-mythos-5` canonical rows to `official_model_rate_overrides()`; exact aliases resolve in `ModelRateCatalog::calculate()`.
- Good: Tauri service tests seed `ccr_usage::fixtures::SeedBucket` with upstream `static-v1` fields and assert pass-through to `ModelStatDto`.
- Base: a user has an older `llmusage.db`; the dashboard still loads and shows whatever stored pricing state exists.
- Bad: duplicate pricing SQL in `ccr-ui` or `ccr-tui`.
- Bad: adding aliases with substring matching such as `model.contains("fable")`.
- Bad: hard-blocking dashboard reads because `llmusage --version` is older than the latest catalog addition.

### 6. Tests Required

- `cargo test -p ccr-types -- --test-threads=1` with positive canonical/alias cases, negative non-matches, and sample cache-aware/cache-free costs.
- `cargo test -p ccr-store -- --test-threads=1` asserting built-in default rows expose canonical models.
- `cargo test -p ccr-db -- --test-threads=1` covering legacy import, migration repricing, and model stats.
- `cargo test -p ccr-usage` when projection SQL changes; otherwise keep `ccr-usage` as the read-only SQL owner.
- `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml llmusage_adapter -- --nocapture` and a focused Tauri service test for `ModelStatDto` pass-through.
- `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml --test llmusage_no_crate_guard -- --nocapture`.
- If frontend fixture rows change, run the focused smoke test plus `just frontend-check-quick`.

### 7. Wrong vs Correct

#### Wrong

```rust
if model.contains("fable") {
    return Some(anthropic_rate(10.0, 50.0, 1.0));
}
```

This prices unrelated models such as `not-fable-5`.

#### Correct

```rust
let normalized = normalize_model_id(model);
if matches!(normalized.as_str(), "claude-fable-5" | "fable-5") {
    return Some((anthropic_rate(10.0, 50.0, 1.0), "official:anthropic", "priced"));
}
```

Exact aliases are priced, and unrelated ids stay `unpriced`.

## Scenario: Home Insights snapshot (`get_home_insights`)

### 1. Scope / Trigger

- Trigger: changing `Dashboard::insights`, `FeatureKey::Insights`, the Insights DTOs, `services::home_insights`, the `get_home_insights` command, the session-archive platform counts, or a `SourceKind::parse_id` alias.
- Applies to `crates/ccr-usage/src/{insights.rs,db.rs,source.rs,capabilities.rs,timezone.rs}`, `crates/ccr-db/src/database/repositories/usage_repo.rs`, `ccr-ui/src-tauri/src/services/home_insights.rs`, `ccr-ui/src-tauri/src/commands/usage.rs`, and the generated types under `ccr-ui/src/types/generated/usage/`.
- The command is a cross-layer contract: one IPC call returns the full home Insights block. The frontend must not send a second statistics request for the block.

### 2. Signatures

```rust
// crates/ccr-usage
pub const INSIGHTS_WEEKS: usize = 53;

impl Dashboard {
    pub fn insights(&self, filter: &QueryFilter, as_of: NaiveDate) -> Result<InsightsPayload, UsageError>;
}

// crates/ccr-db/src/database/repositories/usage_repo.rs (read-only)
pub fn has_any_session_archive(conn: &Connection) -> Result<bool, rusqlite::Error>;
pub fn count_session_archive_by_platform(conn: &Connection) -> Result<Vec<SessionArchivePlatformSummary>, rusqlite::Error>;
pub fn count_session_archive_by_platform_between(
    conn: &Connection,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<SessionArchivePlatformSummary>, rusqlite::Error>;

// ccr-ui/src-tauri/src/services/home_insights.rs (no State, no clock)
pub fn compute_home_insights(
    llmusage: &LlmusageRuntime,
    pool: &ccr_db::database::DbPool,
    as_of: NaiveDate,
    generated_at: DateTime<Utc>,
) -> Result<HomeInsightsResponse, String>;

// ccr-ui/src-tauri/src/commands/usage.rs, registry module usage_v2
#[ccr_tauri_command_macros::command]
pub async fn get_home_insights(state: State<'_, AppState>) -> Result<HomeInsightsResponse, String>;
```

```typescript
// ccr-ui/src/api/generated/usageV2.ts (generated from the registry row)
export const getHomeInsights = (): Promise<HomeInsightsResponse> => invoke('get_home_insights')
```

### 3. Contracts

- Wire DTO `HomeInsightsResponse`. Every `i64` field carries `#[ts(as = "f64")]` or `#[ts(as = "Vec<f64>")]`, so no generated file contains `bigint`.
  - `as_of`, `first_date?`, `trend_start`: `YYYY-MM-DD` local dates. `generated_at`: RFC 3339 UTC.
  - `totals { requests, tokens, sessions, agents, projects, active_days }`.
  - `sessions_indexed: boolean`; `last_7_days` and `previous_7_days { requests, sessions, active_days }`.
  - `daily: InsightsDay[]`: only days with requests inside the 53-week window, ascending. `current_streak`, `longest_streak` (`u32`), `busiest_day?`.
  - `hourly` (24 items), `weekday` (7 items, Monday first), `monthly` (12 items, January first).
  - `trend: InsightsTrendSeries[]`: `weekly` has 53 items; the last item is the week of `as_of`.
  - `agents: InsightsAgentTally[] { source, requests, tokens, sessions, unmapped }`; `projects` and `models: InsightsTally[] { key, label, requests, tokens }`.
- Clock ownership: only the command reads the clock (`chrono::Local::now().date_naive()` for `as_of`). The service and the projection receive `as_of` and `generated_at` as arguments, so tests are deterministic.
- Time rules. All dates use the report zone of `QueryFilter::timezone` with historical IANA offsets.
  - Weeks start on Monday. `trend_start` is the Monday of the week of `as_of`, minus 52 weeks.
  - `last_7_days` is `[as_of-6, as_of]`. `previous_7_days` is `[as_of-13, as_of-7]`.
  - `current_streak` counts back from `as_of`. When `as_of` has no requests, it counts back from `as_of-1`. `longest_streak` and `busiest_day` scan the full history. A `busiest_day` tie resolves to the earlier day.
  - Rows dated after `as_of`, and rows whose `hour_start` does not parse, count only in `totals.requests`, `totals.tokens`, and the source totals. `first_date` ignores rows after `as_of`.
- Source keys: the time query groups by the raw stored `source`. Rust maps each value through `SourceKind::parse_id` (`canonical_source_key`). Stored `gemini*` and `antigravity_ide` go to `antigravity`, `omp` goes to `pi`, and `kimi` goes to `kimi_code`. A value that `parse_id` rejects keeps its raw label. The alias table is shared, so the aliases also apply to usage filters and to the import and sync commands.
- `trend` contains only sources with requests in the window. It is ordered by window total descending, then by source key ascending. Presentation rules such as "top 4 plus other" belong to the frontend.
- Agent rows: one row per key. `requests` and `tokens` come from usage. `sessions` come from the session archive; the archive `platform` goes through the same `parse_id`. `unmapped = SourceKind::parse_id(&key).is_none()` on both sides. Rows are ordered by requests descending, then sessions descending, then key ascending.
- `totals.agents` counts the usage sources with requests or tokens. `agents` is the union of usage keys and session keys, so `agents.length` can be larger than `totals.agents`. `totals.sessions` is the sum of all archive platform counts, including unmapped platforms.
- `sessions_indexed` comes from `has_any_session_archive`. When it is `false`, every session count on the wire is `0`, and the frontend shows `—`.
- Session windows reuse `InsightsUsageWindow.start_utc` and `end_utc` from the projection. Request counts and session counts therefore use the same zone and the same bounds. `created_at` is compared as a half-open `[start, end)` range of `DateTime<Utc>::to_rfc3339()` strings, the same format that `upsert_session_archive_entry` writes.
- `projects` skips empty `project_hash` values. A project with an empty label uses its hash as the label. `totals.projects == projects.len()`. `projects` and `models` are ordered by requests descending, then key ascending.
- `FeatureKey::Insights` (`"insights"`) requires `usage_bucket_30m.{source, model, hour_start, project_hash, project_label, event_count, total_tokens}` and is part of `DB_BACKED_FEATURES`.
- Cache: the key is `usage:snapshot:home_insights:<as_of>` under `USAGE_SNAPSHOT_CACHE_PREFIX`, with TTL `USAGE_SNAPSHOT_CACHE_TTL_SECS` (30 s) and a single-flight fill. Every path calls `finish_cache_fill`, including a serialization error. The command skips the cache while a usage import job or a session index job is active. `invalidate_usage_snapshot_cache` clears the key through the prefix.
- The computation runs inside `tokio::task::spawn_blocking`. It records the command duration and `db_ms`. On a 1.4 GB llmusage database the first call took 39.9–139.1 ms and warm calls took 30.5–34.7 ms.
- Query plans with the bundled SQLite: the full-history count scans the covering index `idx_usage_session_archive_platform_state`. The range count scans the covering index `idx_usage_session_archive_platform_created_at`. The range count cannot seek by time because it has no `platform =` predicate. Neither plan uses `TEMP B-TREE`.

### 4. Validation & Error Matrix

- llmusage DB missing or unreadable -> `Err("Dashboard open error: …")`. Never return an empty snapshot.
- Required Insights column missing -> `UsageError::FeatureUnavailable { feature: "insights", … }` -> `Err("Insights query error: …")`.
- Unknown or unavailable IANA zone -> `UsageError::Query`. No current-offset fallback.
- Session archive empty -> success with `sessions_indexed = false` and every session count `0`.
- ccr-db pool checkout fails -> `Err("DB error: …")`. A session count query fails -> `Err("Session archive … query error: …")`.
- The blocking task panics or is cancelled -> `Err("Task join error: …")`. The cache fill still finishes.

### 5. Good / Base / Bad Cases

- Good: stored `omp`, `kimi`, `gemini`, and `antigravity_ide` rows appear under `pi`, `kimi_code`, and `antigravity` in `trend` and `agents`, with no duplicate row.
- Good: a session created at 23:30 local time on `as_of` counts in `last_7_days` when that instant is the next day in UTC.
- Good: an unregistered key with both usage rows and session rows is one `agents` row with `unmapped = true`, its requests, and its sessions.
- Base: sessions are not indexed -> `sessions_indexed = false`. The UI shows `—`, not `0`.
- Base: a platform with sessions and no usage -> an `agents` row with `requests = 0`. `totals.agents` does not count it.
- Bad: a SQL `CASE` that folds only `gemini`. Stored `omp` then shows next to `pi`.
- Bad: reading the clock inside the service or the projection. Tests then depend on the current date.
- Bad: comparing `created_at` with an inclusive end date string such as `created_at <= 'YYYY-MM-DD'`. Sessions on the end day are dropped.
- Bad: keying agent rows by `(unmapped, key)`. One unregistered key then shows as two rows.

### 6. Tests Required

- `cargo test -p ccr-usage --all-features -- --test-threads=1`:
  - `insights::tests`: zero snapshot; fixed-offset and IANA zones; window bounds; Monday weeks and 53 buckets; 53-week trimming; streak from the previous day; longest streak over the full history; busiest-day tie; future rows; 7-day split; trend order.
  - `insights::tests::dashboard`: a trend series for every source; the `gemini` merge; alias canonicalization with `antigravity_ide` and an unknown label; the project total equals the ranking length.
  - `capabilities::tests::insights_capability_reports_missing_bucket_column`; the `source` alias test asserts `kimi`, `omp`, and `antigravity_ide`.
- `cargo test -p ccr-db --all-features -- --test-threads=1`: `session_archive_counts_group_full_history_by_platform`, `session_archive_range_counts_use_half_open_utc_bounds`, and `session_archive_platform_counts_use_covering_platform_indexes` (asserts the two plans above and no `TEMP B-TREE`).
- `cd ccr-ui && bun run tauri:test`: `services::home_insights::tests` (platform mapping and unknown rows; one row per unmapped key across usage and sessions; the as-of-day session in the last 7 days; unindexed sessions; cache round trip; missing DB error).
- `just tauri-bindings-check` and `cd ccr-ui && bun run test:smoke -- tests/api/api-facade-coverage.smoke.test.ts`: the registry counts and the generated client stay in sync.

### 7. Wrong vs Correct

#### Wrong

```sql
SELECT hour_start,
       CASE WHEN source = 'gemini' THEN 'antigravity' ELSE source END AS canonical_source,
       SUM(event_count)
FROM usage_bucket_30m
GROUP BY hour_start, canonical_source
```

The SQL knows only one alias. Stored `omp` and `antigravity_ide` rows are not folded, and the SQL alias list drifts away from `SourceKind::parse_id`.

#### Correct

```rust
// SQL groups by the raw `source`; Rust owns the one alias table.
let key = canonical_source_key(&raw_source); // SourceKind::parse_id, raw label on a miss
```

#### Wrong

```rust
let key = (SourceKind::parse_id(&platform).is_none(), platform.clone());
rows.entry(key).or_default().sessions += count;
```

The usage side and the session side produce different keys for one unregistered source, so the leaderboard shows two rows.

#### Correct

```rust
let key = SourceKind::parse_id(&platform.platform)
    .map(|kind| kind.as_str().to_string())
    .unwrap_or_else(|| platform.platform.clone());
row_for(&mut rows, key).sessions += platform.session_count.max(0);
```
