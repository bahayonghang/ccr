# Native runtime metrics implementation — 2026-11-06

ReadyForReview. Reused task10-17-gpui-desktop and preserved the shared worktree. Baseline `fc093e78cf0db18a4b86cc048a97f9939387d94e` is the separately committed/accepted S2 batch. No commit by this implementer. Current batch does not modify S2 producer/persistence/IPC, Tauri metrics/wire semantics, SQL, loader/report clock, cache/password policy, theme or destinations.

## Exact delivered product scope

1. Shared `crates/ccr-app/src/monitoring.rs`: pure bounded `LatencyWindow` with fixed2048 capacity, `record(Duration)` and `snapshot() -> LatencySnapshot {sample_count,p95_ms}`. It only stores observations supplied by its captured caller; it starts no work/timer. Evict oldest, nearest rank ceil(n*.95)-1. Retained count is not a lifetime success count. EmptyNone/count0 and actualSome(0)/count1 are distinct.
2. Shared `crates/ccr-app/src/environments/ssh.rs`: `Workspace::retained_session_count() -> usize` reads only existing optional Session under its mutex. Logical retained authority0/1, not active TCP/reachability, host-map size or password cache. It does no filesystem/DB lookup/transport/process/identity access.
3. Native `runtime/mod.rs`: one captured Arc<Mutex<LatencyWindow>> per Services. Services clones share it. `with_insights_paths` compares effective paths before/after override, resetting only if those captured values really change; a same-path builder or ignored Benchmark override cannot reset history. No dynamic source registry/global metrics singleton.
4. Native `runtime/insights.rs`: only the `InsightsQuery::Usage` branch records successful completed `ccr_app::usage::read::read_dashboard_at` calls. `open_dashboard` and captured as_of retrieval finish before Instant starts; elapsed is frozen immediately after the report returns and before acquiring the stats mutex. Failures, missing/unopenable DB, other queries and before-work cancellation never produce a sample. Queue/admission, DB opening, worker join, UI/render and stats-lock waiting are excluded. A successfully completed service result still counts if a later UI generation discards it: the service work actually completed.
5. Native `runtime/monitoring.rs`: existing explicit refresh snapshots captured latency and `Services.ssh.get()` without constructing a Workspace. No owner means actual0 held sessions. Snapshots release the stats mutex before the feed's existing database IO. Report Metrics receives both observations explicitly, keeping source identity and optional p95 intact.
6. Native `features/monitoring.rs`: existing metrics block shows **Usage page read p95 / Usage 页面读取 p95（成功样本）**, current retained samples/2048 and None/real0 correctly. Its hint limits the cohort to the most recent successful Usage page reports with DB already open, excludes Home/Workbench, opening, queueing, rendering and failed reads, and says samples do not establish benchmark performance. SSH is labelled a session retained by this app, whose availability is not verified. No new screen/framework or synthetic Moka/password rows.

This is intentionally not Tauri's command_p95_ms/db_query_p95_ms. The existing Tauri windows cover explicit Usage-handler timings and broader service work, with cache fast-path distinctions; their meaning/API were left unchanged. Source assessment and exact applicability are in [the read-only gap report](performance-runtime-metrics-gap-20261106.md). The common sampler is reusable code, not a claim that both apps currently share an identical timing cohort.

## Tests and actual evidence

Root/Tauri leases were returned immediately after shared owner verification. Native lease was returned after final strict/narrow/actual1MiB checks. No running Cargo/exe/preview remained at handoff. Root/Tauri reuse their existing external target links; native uses its strict workspace-local target. No HOME/USERPROFILE or user config/environment mutation.

| Command/evidence | Actual result |
| --- | --- |
| `cargo test -p ccr-app --lib monitoring::tests --locked --offline -- --skip export_bindings` |2/2PASS; empty vs realzero, known nearest rank, oldest eviction/capped retained count2048 and independent empty window |
| `cargo test -p ccr-app --lib retained_session_metrics --locked --offline -- --skip export_bindings` |1/1PASS,0.50s; synthetic real owner connect0→1, then lookup disabled/panic-on-transport guard still reads1; disconnect→0 |
| `cargo clippy -p ccr-app --all-targets --all-features --locked --offline -- -D warnings` |exit0,4.33s; these shared production sources froze afterward |
| Native `cargo test --lib latency --locked --offline -- --skip export_bindings` |final2/2PASS,0.59s; actual owned Usage report creates sample, missing/invalid/Logs do not; clones/same roots share, changed captured roots isolate; real native Monitoring snapshot shows None/zero and injected synthetic SSH0/1 |
| Native `cargo test --lib monitoring --locked --offline -- --skip export_bindings` |13/13PASS,3.87s including enhanced real Monitoring rendering/hidden behavior, captured storage and prior system/benchmark refusals; later final native edits only moved as_of outside the timed body and used a struct initializer in the direct test, then final2narrow reran |
| Native `cargo fmt -- --check` and `cargo clippy --lib --tests --locked --offline -- -D warnings` |latest exit0; strict11.71s |
| Final native artifact with child-only `RUST_MIN_STACK=1048576`, exact `features::monitoring::tests::monitoring_filters_real_main_events_and_suspension_detaches_without_polling` |1/1PASS,1.15s. This same existing central Windows1MiB path renders the actual modified Monitoring metric block; no new matrix/stack-budget increase |
| `git diff --check` |exit0 |

Narrow-test details:

- `usage_latency_samples_only_successful_opened_report_body_and_keeps_captured_owners_isolated` uses two domain-created projection DBs, actual report invocation, a rejected inverted date range, Logs and missing storage. It asserts no samples leak into another root, same-path clones keep the same owner, changed effective roots receive a new empty window and the old window stays intact. The date fixture uses NaiveDate as the real QueryFilter type.
- `monitoring_projects_latency_and_retained_ssh_without_initializing_or_probing` first refreshes a completely empty captured root and proves no Workspace/directory/file creation. Setup uses the existing bounded RuntimeHandle for real shared domain fixture save/probe/confirm/connect through the in-memory transport, then injects that owner. Metric reads preserve its held authority and transport CLI/SFTP observation lists; Root's panic-on-transport regression independently protects the getter itself. This is controlled fixture behavior, not a real SSH probe/connection.
- Enhanced existing view test supplies a genuinely readable usage fixture but opening/refreshed Monitoring keeps the sample window empty. Only an explicit test-only Duration::ZERO insertion exercises accessible real-zero rendering. It verifiesUnavailable/0 samples, then0.000ms/1sample, suspend detachment, stable read count and no extra sample during hidden waits. Actual service timing evidence comes from the separate real Usage-branch test, not that controlled display value.

Initial native test compilation found private test access and a DateTime/NaiveDate fixture mismatch, both corrected without weakening assertions. Initial synthetic SSH setup called an async domain confirm directly on GPUI's test executor, which lacks Tokio's reactor; moving only fixture setup onto the existing admitted RuntimeHandle fixed the real cause. Final tests above passed; no extra runtime/framework or production bypass was added. Existing Windows incremental hard-link fallback warning may appear during compilation; latest strict source gate exits0.

## Contracts and ownership

`shared-desktop-services.md` now has the complete seven-part source/API/error/cohort/test contract and replaces the older blanket unmeasured-latency sentence with precise Usage-page/SSH facts. `execution-environment.md` documents the lock-only count seam and its zero-IO regression. The coordinator owns migration-inventory, platform-parity and aggregate implementation/integration ledgers; this worker did not edit them.

Independent check is coordinator-dispatched; the reviewer was given coherent source and final results without competing compilation. Coordinator owns the complete Root/Tauri CI, GPUI aggregate/package and final commit. They are not labelled passed by this receipt before their actual results.

## Remaining limits

No GUI/native acceptance, live SSH/WSL/identity/account/network operation or performance trial occurred. This adds real source instrumentation and a usable native metric display; sparse/current rolling samples are not a qualified performance p95 or budget pass. It does not measure all Usage consumers, Home/Workbench, every command/SQL statement, render/OS-present or user input latency. Tauri result-cache/remembered SSH/password-map metrics remain inapplicable to native's different owners. Performance comparability/native visual acceptance remain separately deferred; source implementation is no longer blocked by that deferral.
