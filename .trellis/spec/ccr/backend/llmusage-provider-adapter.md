# llmusage Provider Adapter Contract

> Executable contract for the CCR desktop llmusage provider dimension.

## Scenario: provider-scoped usage analytics from llmusage schema 14

### 1. Scope / Trigger
- Trigger: changing provider usage attribution, sync import wiring, usage dashboard filters, or the llmusage read-only adapter.
- Applies to `ccr-ui/src-tauri/src/llmusage_adapter/**`, `ccr-ui/src-tauri/src/commands/usage.rs`, and the matching frontend usage API/types.
- CCR must keep the no-crate boundary: invoke the installed `llmusage` CLI for sync and read the SQLite DB read-only; do not link the upstream `llmusage` Rust crate.

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
  pub struct QueryFilter {
      pub provider: Option<String>,
      // existing fields omitted
  }

  pub fn provider_breakdown(
      &self,
      filter: &QueryFilter,
  ) -> Result<Vec<ProviderBreakdownDto>, LlmusageAdapterError>;
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
- `Dashboard::provider_breakdown` groups `usage_bucket_30m` by `provider_label`, returns token splits and both cache-aware/cache-free costs, and maps empty provider labels to `provider = null`.
- A provider filter must apply to overview, daily trends, model breakdown, source breakdown, project breakdown, heatmap, and logs through the shared `QueryFilter`.
- `get_usage_dashboard_v2` includes `provider_stats` and its cache key must include the provider filter.
- When no provider filter is supplied and provider capability is unavailable, dashboard payloads degrade to `provider_stats: []`; an explicit provider filter must surface the unsupported error.
- Only pass `--provider-map <path>` when `$CCR_ROOT/analytics/provider_activation.jsonl` exists. The installed llmusage CLI treats an explicit missing provider-map path as a hard sync error.
- New frontend business wrappers belong in `src/api/domains/*`; `src/api/tauri.ts` remains a compatibility facade.

### 4. Validation & Error Matrix
- DB schema `< 14` + provider breakdown/filter -> `SchemaUnsupported { expected: 14, ... }`.
- Schema 14 without `provider_label` on either required table -> `FeatureUnavailable { feature: "provider_breakdown", ... }`.
- Dashboard without provider filter on an old DB -> success with `provider_stats: []`.
- Dashboard with explicit provider filter on an old DB -> error; do not silently return unfiltered data.
- Missing activation log file -> omit `--provider-map`; sync should still run.
- Existing activation log file -> pass `--provider-map <path>` after other sync options.

### 5. Good / Base / Bad Cases
- Good: schema 14 fixture with `openai`, `anthropic`, and empty labels; provider totals sum to source totals and empty labels serialize as `null`.
- Good: `provider = "openai"` narrows overview/model/source totals to only OpenAI-attributed rows.
- Base: old llmusage DB still renders the dashboard, but provider stats are empty until the user upgrades/syncs.
- Bad: adding provider filtering only to `provider_breakdown`; dashboard cards would show mixed-provider totals.
- Bad: always passing `--provider-map` even when the activation log file is absent.
- Bad: adding a new direct `invoke()` wrapper in `src/api/tauri.ts`.

### 6. Tests Required
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
