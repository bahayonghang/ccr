# Common report clock and paired-seed admission — 2026-11-06

Status: implemented and independently reviewed; targeted shared/native/Tauri/frontend gates passed. No commit, GUI/WebView/native probe, real account, CLI import, WSL, SSH, external network or native performance sample was performed. Existing task and all concurrent ownership were preserved.

## Delivered boundary

`ccr-app::benchmark::{Endpoint,Seed::load}` is the shared read-only file admission owner. It contains the native loader's existing algorithm, selected explicitly by endpoint; it does not construct a platform, open SQL, migrate, log, change environment variables or initialize a window. Both clients' `--benchmark-fixture-root` call it, and reject combination with their old fixture mode. Native probe/insights/delay remain mutually exclusive. Old Fixture markers never infer Benchmark mode.

The selected endpoint must be its recorded canonical gpui/tauri sibling. Admission checks absolute/traversal-free/unlinked roots and descendants, required directories, bounded marker/paired manifest JSON, versions/synthetic/scenario/fixed counts, both canonical logical hashes, UTC report clock and all four DB artifact hashes. The same selected-end tree walk and full DB byte hashing occur in both clients, including S0's explicitly absent catalog rule. No startup validation time is subtracted from process-create→present; no budget success is claimed.

Independent review found real committed-WAL recovery could change visible data while leaving the main DB hash unchanged. The reviewer reproduced this with UiStateManager, then added sidecar admission: each role's WAL/journal must be absent or empty; SHM may be nonempty, but every sidecar must be regular, unlinked and readable. This includes a missing S0 catalog's sidecars. Admission refuses rather than recovering/migrating input. The reviewer also reproduced fixed tool-window truncation of +1ms future timestamps and repaired the existing SQL owner to use Julian comparisons, preserving its live branch. Chrono accepts leap-second input that JS Date rejects; shared admission now rejects that before a window. Native/JS parsers already reject unsupported expanded years; no duplicate year framework was added.

`ccr-app::usage::report_clock::ReportClock` carries report as_of+calendar zone only. Local/default wrappers remain live Local; explicit Benchmark uses manifest UTC. Instant, elapsed timers, timeouts, progress publication, ingestion timestamps, process creation and presentation clocks are unchanged.

## Actual consumers

- Native runtime: workbench default window, session count window/generated timestamp, default Home query and Claude Observer receive the captured clock. Insights starts its timezone control at UTC only for Benchmark; explicit Usage/Logs filters and caller Home dates remain unchanged. Overview's report date labels UTC. Legacy Fixture uses its old live Local clock and old source subroots.
- Shared usage owner: `Dashboard::{overview_at,heatmap_at}` parameterize only their existing queries; overview's fixed last-24h interval excludes future rows, while live overview keeps the original predicate and real generated timestamp. `read_dashboard_at` passes the capture; existing read_dashboard remains a live wrapper. Heatmap retains its documented independent rolling-days window rather than replacing the caller's explicit overview filter.
- Shared Observer adds compatible *_with_clock wrappers. Usage date filters stay in ccr-usage; tool query time windows stay in ccr-db::claude_tool_calls_repo with compatible *_at/live functions. Live SQL behavior is retained. No adapter gained SQL.
- Tauri Rust: existing Usage services, Home Insights, Home Usage Overview, freshness/report timestamps, summary/heatmap adapter and Observer services consume captured report time/UTC only in explicit Benchmark. The old Home Overview's session query used literal end-date strings and excluded the ending day. Existing usage_repo bodies now have separate UTC date-window wrappers (normalized UTC bounds, exclusive next-day end and UTC date buckets); Benchmark uses these, old Local functions retain their exact string contract.
- Tauri frontend: the shared fixture WebView builder serializes one read-only Object.freeze report context, with no root/credential data. No Local/old-Fixture injection occurs and no new IPC stack was added. reportClock.ts validates supported UTC RFC3339 calendar data and fails closed. Actual Usage store defaults/reset/preset changes, platform usage default range, Home cost window and report clock label consume it. Explicit dates/manual ranges remain unchanged. Date/Date.now and real intervals are not monkeypatched.
- Read-only Observer commands were admitted after their actual captured llmusage/main-pool call graphs were inspected. Scanner/import/mutation capabilities were not enabled by a risk-label shortcut.

Native production SHA ownership moved into the shared module; the pre-existing pinned native sha2 edge now lives in dev-dependencies for fixture/projection hash helpers only. ccr-app enables serde_json float_roundtrip at the existing pinned version; canonical object-key ordering preserves all values, arrays, opaque IDs and root-bound source identities. No new package or production SQLite dependency was added.

## Verification

Owner-run:

- Root `cargo check --offline -p ccr-app` passed.
- Root `cargo test --offline -p ccr-app --lib clock -- --skip export_bindings`: 4/4 passed (report calendar, paired admission, Observer, recent/heatmap and explicit filter preservation). Repeated after UTC source changes.
- Root `cargo test --offline -p ccr-db --lib claude_tool_calls_repo -- --skip export_bindings`: 5/5 passed; independent review later added +1ms red→green in the same test.
- Root UTC report-session boundary test: 1/1 passed; covers start/end day, next-day exclusion, offset timestamps, invalid dates and old literal-date compatibility. An initial test setup reused a unique source path and was corrected; it was not a production red.
- Root `cargo test --offline -p ccr-app --lib usage::read -- --skip export_bindings`: 3/3 passed.
- Explicit generated S0/S1/S2 gpui+tauri six-root shared admission: 1/1 passed in 20.10s; existing data/hash identities unchanged. Reviewer repeated with final sidecar guards: 1/1, 9.89s. These durations are check execution diagnostics, not benchmark samples.
- Tauri with pinned toolchain + `--config .cargo/tauri-ci.toml --locked --offline`: fixture tests 4/4 plus isolated core child passed; actual explicit Benchmark flag/default Home/Observer readback 1/1 with 3 isolated S0/S1/S2 children passed, reporting UTC date 2026-11-06 and generated_at exactly 2026-11-06T12:00:00+00:00; sessions 0/10000/10000. Final rerun with manifest-derived seven-day request-count assertions and review's final sidecar/leap/+1ms guards also passed: 1 parent + 3 children, 6.34s; requests 0/65469/65469 and sessions 0/10000/10000.
- Tauri existing `services::usage` suite: 30/30 passed.
- Frontend targeted clock tests: 4/4 passed. Entire `tests/usage`: 19 files, 90/90 passed. TypeScript noEmit and scoped ESLint passed.
- Scoped rustfmt and git diff --check passed.

Independent checker reported final Root ccr-app/ccr-db/ccr-usage alltargets/allfeatures strict Clippy exit0; native lib/tests strict exit0; latest native Benchmark 7/7+poison child1 and actual S0/S1/S2 default workbench/Observer readback1 passed. Native requests 0/65469/65469, sessions None/10000/10000, using the exact manifest UTC seven-day window. The native child also preserves an explicit Home date and old Fixture's live Local behavior. Checker corrected the test's WorkbenchDay label field use; no production behavior was hidden by the setup correction. Final reviewer evidence is recorded separately.

Final owner-run Tauri `--config .cargo/tauri-ci.toml clippy --locked --offline --manifest-path ccr-ui/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` passed exit0 (37.79s), including all final shared guards. The existing proc-macro MSVC linker stdout note remained; rustc explicitly reports that linker_messages ignores -D warnings. No source warning/failure was suppressed. Tauri target lease was released after completion. No whole-repository just ci or native run is claimed by this receipt.

## Limits and next boundary

Clock/default query wiring and validation policy equality are implemented and headlessly exercised; native presentation/WebView loading and true process/create→first-present/action-ready evidence are still not_run. Equal validation policy does not establish startup, input, memory, energy or responsiveness budgets. No native sampling was started, thresholds changed, validation overhead removed or first-present surrogate added. The current GPUI CPU attempt profiler/on-next-frame path is not an OS-present completion signal.

S2 stream remains not_emitted until a separately reviewed explicit Benchmark/S2-only retained runtime source emits real scheduled events. It must not be replaced by prefilled rows or claimed 1000/s from a requested rate. All performance comparison remains **not_comparable**. The static seed mutation/reset rule remains: restore original inputs, with all owners stopped, before a new trial. No changed stream data is silently admitted as the original seed.
