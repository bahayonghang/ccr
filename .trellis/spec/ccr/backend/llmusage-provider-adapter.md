# llmusage Provider Adapter Contract

> Executable contract for the CCR desktop llmusage provider dimension.

## Scenario: provider-scoped usage analytics from llmusage schema 14

### 1. Scope / Trigger

- Trigger: changing provider usage attribution, sync import wiring, usage dashboard filters, or the llmusage read-only adapter.
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
      Gemini,
      Opencode,
  }

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

  // Dashboard owns the full read-only query surface:
  // overview / trends_daily / model_breakdown / provider_breakdown /
  // project_breakdown / source_breakdown / heatmap / logs / diagnostics /
  // home_overview, all gated through ensure_feature_for_filter.
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
- `crates/ccr-usage` owns `Dashboard::provider_breakdown` and **all** usage SQL (overview, trends, model/provider/project/source breakdowns, heatmap, logs, diagnostics, home overview) for all CCR surfaces. Tauri and TUI code must delegate to this crate instead of duplicating any aggregation query; the Tauri adapter `Dashboard` is a thin wrapper that only maps `UsageError` to `LlmusageAdapterError`.
- `Dashboard::provider_breakdown` opens `llmusage.db` read-only, groups `usage_bucket_30m` by `provider_label`, returns token splits and both cache-aware/cache-free costs, and maps empty provider labels to `provider = null`.
- `AppPaths::discover()` honors `LLMUSAGE_HOME`, otherwise uses `<home>/.llmusage`; Tauri may use `AppPaths::from_root(existing_root)` to preserve its existing path contract.
- A provider filter must apply to overview, daily trends, model breakdown, source breakdown, project breakdown, heatmap, and logs through the shared `QueryFilter`.
- `get_usage_dashboard_v2` includes `provider_stats` and its cache key must include the provider filter.
- When no provider filter is supplied and provider capability is unavailable, dashboard payloads degrade to `provider_stats: []`; an explicit provider filter must surface the unsupported error.
- TUI usage views should load the shared projection on a background task through an injectable loader seam (`UsageLoader`), consume `TaggedProviderBreakdown` directly (no per-surface shadow row structs), request `SourceKind::Claude` and `SourceKind::Codex` separately when rendering those platform sections (via `provider_breakdown_by_source`), and display `provider = null` as `unattributed` (`ProviderBreakdownDto::display_provider`).
- Only pass `--provider-map <path>` when `$CCR_ROOT/analytics/provider_activation.jsonl` exists. The installed llmusage CLI treats an explicit missing provider-map path as a hard sync error.
- New frontend business wrappers belong in `src/api/domains/*`; `src/api/tauri.ts` remains a compatibility facade.

### 4. Validation & Error Matrix

- DB schema `< 14` + provider breakdown/filter -> `SchemaUnsupported { expected: 14, ... }`.
- Schema 14 without `provider_label` on either required table -> `FeatureUnavailable { feature: "provider_breakdown", ... }`.
- Dashboard without provider filter on an old DB -> success with `provider_stats: []`.
- Dashboard with explicit provider filter on an old DB -> error; do not silently return unfiltered data.
- `ccr-usage::open_dashboard` missing DB -> `UsageError::DbMissing`.
- `ccr-usage::open_dashboard` unreadable DB/home resolution failure -> `UsageError::DbUnreadable`.
- `ccr-usage::provider_breakdown` missing provider table/column -> `UsageError::FeatureUnavailable`.
- Missing activation log file -> omit `--provider-map`; sync should still run.
- Existing activation log file -> pass `--provider-map <path>` after other sync options.

### 5. Good / Base / Bad Cases

- Good: schema 14 fixture with `openai`, `anthropic`, and empty labels; provider totals sum to source totals and empty labels serialize as `null`.
- Good: `provider = "openai"` narrows overview/model/source totals to only OpenAI-attributed rows.
- Good: Tauri provider dashboard and TUI Usage tab both call `ccr-usage` and differ only in DTO/error mapping and presentation.
- Base: old llmusage DB still renders the dashboard, but provider stats are empty until the user upgrades/syncs.
- Bad: adding provider filtering only to `provider_breakdown`; dashboard cards would show mixed-provider totals.
- Bad: copying any usage SQL (provider breakdown or otherwise) into `ccr-ui` or `ccr-tui`; future schema/capability fixes would diverge.
- Bad: re-introducing a per-surface shadow row struct (field-by-field copy of a ccr-usage DTO plus a tag) instead of consuming `TaggedProviderBreakdown`.
- Bad: always passing `--provider-map` even when the activation log file is absent.
- Bad: adding a new direct `invoke()` wrapper in `src/api/tauri.ts`.

### 6. Tests Required

- `cargo test -p ccr-usage`
- `cargo test -p ccr-tui -- --test-threads=1` when adding or changing the TUI Usage tab
- `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml llmusage_adapter -- --nocapture`
- `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::handler_registry -- --nocapture`
- `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml --test llmusage_no_crate_guard -- --nocapture`
- `cd ccr-ui && bun run test:smoke -- tests/usage-dashboard-payload.smoke.test.ts`
- `cd ccr-ui && bun run test:smoke -- tests/api-facade-boundary.smoke.test.ts`
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
