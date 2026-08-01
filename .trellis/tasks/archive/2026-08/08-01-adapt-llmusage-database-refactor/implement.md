# Implement: 适配 llmusage 数据库重构并优化查询

Implementation starts only after the user approves the final planning summary and `task.py start` changes the task to `in_progress`.

## Phase A: Source, Protocol, And Fixtures

- [x] Expand `crates/ccr-usage/src/source.rs` to seven current source kinds; preserve legacy aliases and add schema-aware Antigravity storage-key resolution.
- [x] Extend adapter `JobEvent` with all current pricing and token-accounting repair events; update progress reducers in `commands/usage.rs` and `services/usage.rs`.
- [x] Upgrade the shared fixture to schema 19 table/index shape without requiring schema 19 at runtime.
- [x] Add focused schema 10/13/14/18/19 source and event protocol tests before query rewrites.
- [x] Verify: `cargo test -p ccr-usage source -- --test-threads=1` and Tauri `llmusage_adapter::events`/usage progress tests.

Rollback point: source/event changes are independent of SQL performance work.

## Phase B: Typed DST-Aware Filters

- [x] Add workspace/direct dependencies `chrono-tz`, `iana-time-zone`; enable rusqlite `functions` for `ccr-usage`.
- [x] Add the minimal CCR-owned timezone resolver/SQLite scalar-function module based on the current upstream contract.
- [x] Change `SqlFilter` to typed `rusqlite::types::Value` parameters.
- [x] Convert bucket/event `since` and `until` to UTC half-open bounds; retain inclusive user-date semantics.
- [x] Replace filter-time `date(column, ...)` predicates across trends/model/provider/project/source/heatmap/logs/home paths.
- [x] Add UTC, normal local day, spring-forward and fall-back tests plus bucket/event `EXPLAIN QUERY PLAN` assertions.
- [x] Verify: focused filter/timezone tests, then `cargo test -p ccr-usage -- --test-threads=1`.

Rollback point: dependency/filter commit can be reverted without changing DTOs.

## Phase C: Capability And Query Refactor

- [x] Build a `DbCapabilitySnapshot` from the existing Dashboard connection; make section gates consume it.
- [x] Instrument tests to prove opening a dashboard plus multiple sections does not reopen capability connections.
- [x] Merge overview's seven bucket scans into one conditional aggregate and its two run-log reads into one conditional aggregate.
- [x] Remove bucket `project_path` probing and preserve the project ref/label fallback chain.
- [x] Remove the four-source whitelist from source breakdown; canonicalize legacy `gemini` before share calculations.
- [x] Merge home overview into one date/source bucket query; include all sources in summary/by-platform and keep the four-field daily series.
- [x] Preserve logs keyset pagination, optional total and raw-event behavior while applying the shared event range.
- [x] Add result-equivalence tests and query-count assertions for each rewritten path.
- [x] Verify: `cargo test -p ccr-usage -- --test-threads=1` and `rg 'usage_bucket_30m' --type rust` shows SQL only under `crates/ccr-usage`.

Rollback point: each query rewrite remains separable; retain verified filter/capability changes if one aggregation needs rollback.

## Phase D: Wire And Frontend Adaptation

- [x] Rename home series `gemini` to `antigravity` in Rust DTOs and regenerate committed TypeScript bindings.
- [x] Expand `UsagePlatform`, toolbar options and usage labels for seven sources; keep backend alias compatibility for incoming `gemini`.
- [x] Update `DashboardUsageMovement`, source/cost/ops presentation helpers and home/usage fixtures.
- [x] Add/update en-US and zh-CN usage source labels and dependency descriptions.
- [x] Add smoke coverage for all seven toolbar options, Antigravity filtering, source share rows and home series consumption.
- [x] Verify: `cd ccr-ui && bun run test:smoke -- tests/usage-dashboard-state.smoke.test.ts tests/usage-source-summary-card.smoke.test.ts tests/usage-cost-tab.smoke.test.ts tests/home-usage-overview.store.smoke.test.ts`.
- [x] Verify: `cd ccr-ui && bun run type-check` and `bun run lint`.

Rollback point: frontend/wire change can be reverted together while backend canonical ids remain independently tested.

## Phase E: Representative Performance And Contract Closeout

- [x] Re-run the same read-only representative database benchmark with warm-up and multi-iteration medians.
- [x] Record before/after latency, query count and query plans in `research/performance-after.md`; evaluate PRD R7 thresholds honestly.
- [x] Run Tauri-focused tests: `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml llmusage_adapter -- --nocapture` and focused usage service/command tests.
- [x] Run `just version-check`, `just fmt-check`, `just lint-strict`, `just frontend-check-quick`, then final `just ci` because the change crosses Rust, Tauri wire and frontend layers.
- [x] Run `git diff --check`, inspect tracked/untracked/deleted files, and confirm no SQL escaped `crates/ccr-usage` and no upstream file changed.
- [x] Use `trellis-update-spec` to update `.trellis/spec/ccr/backend/llmusage-provider-adapter.md` with seven sources, schema compatibility, DST range predicates, capability snapshot and current event protocol.
- [x] Run `trellis-check`, resolve all correctness/spec/testing findings, and map final evidence to AC1-AC9.

## Expected Change Surface

- `Cargo.toml`, `Cargo.lock`, `crates/ccr-usage/Cargo.toml`
- `crates/ccr-usage/src/{source,db,capabilities,queries,fixtures,lib}.rs` plus a focused timezone module
- `ccr-ui/src-tauri/src/llmusage_adapter/events.rs`
- `ccr-ui/src-tauri/src/{commands,services}/usage.rs`
- generated usage TypeScript bindings and focused Usage/home view, type, i18n and smoke-test files
- `.trellis/spec/ccr/backend/llmusage-provider-adapter.md`

Do not modify `D:/Documents/Code/CLI/llmusage`, unrelated UI surfaces, release metadata, remote refs or GitHub state.
