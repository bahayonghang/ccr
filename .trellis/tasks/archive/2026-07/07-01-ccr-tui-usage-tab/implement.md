# Implement — ccr-tui usage/statistics tab

## Checklist

1. Add shared read-only usage projection crate.
   - Create `crates/ccr-usage` with `AppPaths`, `SourceKind`, `FeatureKey`,
     `QueryFilter`, `ProviderBreakdownDto`, and `Dashboard::provider_breakdown`.
   - Move or mirror only the provider-breakdown capability/query tests needed to
     prove schema 14, unattributed rows, and source/provider filters.
   - Add it to the root workspace.
2. Delegate Tauri provider breakdown to the shared crate.
   - Replace the duplicate `llmusage_adapter::db::provider_breakdown()` SQL with
     a call to `ccr_usage`.
   - Convert DTO/filter/source values at the adapter boundary without changing
     the Tauri command JSON contract.
3. Register the TUI tab.
   - Add `TuiTabId::Usage`, default order entry, string mapping, and config tests.
   - Add `TabVariant::Usage` and a synthetic `Usage` tab in `App::with_task_executor`.
   - Route key handling, activation, ticks, and rendering.
4. Build the Usage app and UI.
   - Async load and refresh via existing `AsyncTaskExecutor`.
   - Render platform/provider token and cost rows with responsive truncation.
   - Show loading, empty, unsupported, and error states.
5. Verify.
   - `cargo test -p ccr-usage`
   - `cargo test -p ccr-config tui_config -- --test-threads=1`
   - `cargo test -p ccr-tui -- --test-threads=1`
   - `cd ccr-ui/src-tauri && cargo test llmusage -- --test-threads=1` if adapter
     delegation changes compile surface significantly.

## Risk Points

- Do not copy the provider SQL into both Tauri and TUI long term; `ccr-usage` is
  the shared implementation.
- Keep `ccr-usage` read-only and crate-free from upstream llmusage.
- Do not let the synthetic Usage tab receive profile apply/select behavior.
- Existing custom `tui.toml` orders will fall back until users add `usage`.

## Manual Smoke

Run the TUI against a schema-14 llmusage DB with provider rows, switch to Usage,
press `r`, and verify Claude/Codex rows plus `unattributed` render without layout
breakage at roughly 80-column width.
