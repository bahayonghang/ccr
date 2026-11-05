# Research: GPUI 07 Insights, usage, sessions and monitoring source map

- Query: Identify the exact preserved07 inventory, existing framework-free owners, minimal Tauri extraction seams, fixture injection, safety/state contracts and independent validation slices.
- Scope: internal source inspection only; prerequisite research while05 implementation proceeds. Active task is05, not activation/acceptance of07. This file is the only write owner of this researcher.
- Date: 2026-10-27
- No build, test, GUI, installed-CLI sync, network request, real account/configuration access or inventory regeneration was performed. Line anchors below describe the inspected working tree.

## Findings

### 1. Inventory: 49 commands, 15 route records

Read the existing parent `.trellis/tasks/10-17-gpui-desktop/research/migration-inventory.json` without recapturing it.07 owns **49 commands**, **12 surfaces** and **3 redirects**. Parent348-command/76-route capture identity remains untouched. Per-command native disposition must be recorded individually; one combined dashboard or one compiled view is not acceptance of49 operations.

| Group | Count | Commands |
| --- | ---: | --- |
| `usage_v2` |18| `get_usage_summary_v2`, `get_usage_capabilities_v2`, `get_usage_trends_v2`, `get_usage_by_model_v2`, `get_usage_by_provider_v2`, `get_usage_by_project_v2`, `get_usage_heatmap_v2`, `get_usage_logs_v2`, `get_usage_dashboard_v2`, `get_home_usage_overview_v2`, `get_home_insights`, `ensure_session_index_v2`, `get_session_index_job_status_v2`, `start_usage_import_job_v2`, `get_usage_import_job_status_v2`, `cancel_usage_import_job_v2`, `import_usage_v2`, `import_all_usage_v2` |
| `agent_sessions` |5| `agent_sessions_list`, `agent_sessions_get_detail`, `agent_sessions_get_provider_status`, `agent_sessions_start_refresh`, `agent_sessions_get_refresh_status` |
| `claude_observer` |9| `claude_observer_get_insight`, `claude_observer_daily_trend`, `claude_observer_cost_breakdown`, `claude_observer_cache_stats`, `claude_observer_top_sessions`, `claude_observer_tool_heatmap`, `claude_observer_top_tools`, `claude_observer_subscription_get`, `claude_observer_subscription_set` |
| `claude` |2| `claude_get_budgets`, `claude_update_budgets` |
| `codex` |10| `codex_list_sessions`, `codex_get_session_detail`, `codex_export_session`, `codex_clone_session`, `codex_delete_session`, `codex_get_usage`, `codex_get_dashboard_overview`, `codex_get_dashboard_usage_summary`, `codex_get_all_quotas`, `codex_get_quota` |
| `grok` |1| `grok_get_dashboard_overview` |
| `pricing` |4| `set_pricing`, `get_pricing_list`, `remove_pricing`, `reset_pricing` |

| Route | Capture index | Kind / consumption |
| --- | ---: | --- |
| `/` |2| Home overview + Insights; several domain summaries, not one fictitious homogeneous dataset |
| `/claude-code` |4| Claude home/observer and existing profile/auth summaries |
| `/codex` |9| Codex overview, rolling usage, real quota outcomes |
| `/grok` |10| Actual Grok activation/auth/config summary |
| `/antigravity` |14| Platform home, canonical usage source `antigravity` |
| `/gemini-cli` |15| Redirect to `/antigravity`; preserve discoverability, not a second surface |
| `/stats` |21| Redirect to `/usage` |
| `/budget` |22| Existing Claude budget policy/storage |
| `/pricing` |23| Existing custom-pricing management |
| `/usage` |24| Shared read-only usage projections and owned import jobs |
| `/monitoring` |25| Existing event feed + usage capability/summary |
| `/sessions` |26| Redirect to `/monitoring`, not `/agent-sessions` |
| `/agent-sessions` |29| Eight-family local archive browser |
| `/codex/sessions` |50| Codex-owned detail/export/clone/delete, distinct from archive browser |
| `/opencode` |62| Platform home plus shared usage; platform/resource summaries consume their respective04–06 owners |

Cross-child detail: `get_monitoring_feed` and `get_recent_events` are preserved inventory owner **gpui-02-services**, module `events`.07's monitoring surface consumes those capabilities; do not silently reassign or duplicate their owners. Actual Tauri handlers are `commands/system.rs:473,523`.

### 2. Framework-free owners versus Tauri orchestration

Paths in this table are repository-relative; line numbers are one-based.

| Area / file | Existing owner and useful APIs | Smallest appropriate integration |
| --- | --- | --- |
| `crates/ccr-usage/src/lib.rs:1-37`, `db.rs:197-231` | Framework-free read-only `AppPaths`, `Dashboard`, typed `UsageError`, capability snapshot and source vocabulary. `open_dashboard(paths)` opens the supplied database; no upstream llmusage crate, sync, migration or raw-provider parsing. | Native blocking reader can consume `ccr_usage` directly. Shared orchestration may compose typed snapshots; never move/copy SQL into ccr-app/native/Tauri. |
| `crates/ccr-usage/src/db.rs:254,353,394,447,517,560,620,671,761,793,879` | `overview`, `trends_daily`, model/provider/project/source breakdowns, `heatmap`, cursor `logs`, diagnostics, `home_overview`, `insights`. All SQL resides here. | Reuse exactly these queries and filters. A new native graph or table consumes their values; it must not calculate a subtly different denominator/cost/time grouping. |
| `ccr-ui/src-tauri/src/services/usage.rs:377-572,898-1124` | State-free freshness/readiness/import-result projection and usage-dashboard assembly; currently references Tauri adapter runtime/error/conversion DTOs, with `ts_rs` wire export derives. `compute_dashboard` takes explicit active-job booleans and raw-session-presence evidence. | Extract only shared policy/assembly into ccr-app, accepting `ccr_usage` projections/paths and explicit context. Keep Tauri wire structs/export paths and small mapping adapters in Tauri. Do not link the Tauri binary crate from GPUI. |
| `services/usage.rs:709-896,1126` | Home usage/session snapshots and bootstrap/readiness assembly. Existing `has_any_raw_sessions` at691 discovers real default provider directories; `local_usage_date_window` at633 reads the clock. | Inject an explicit registry/root and time/window evidence into the extracted orchestration. Fixtures must never call the default-directory probe. Preserve `cold`, `needs_import`, indexed/empty/stale distinctions. |
| `ccr-ui/src-tauri/src/services/home_insights.rs:116-254` | State-free `compute_home_insights(llmusage,pool,as_of,generated_at)`, session-repository join, canonical-source merge. Already accepts date/time explicitly, adds no SQL. | Strong first extraction candidate: consume `ccr_usage::Dashboard::insights` + injected ccr-db pool. Preserve both source identities and `sessions_indexed`; keep wire response in its existing Tauri owner. |
| `crates/ccr-store/src/sessions/providers.rs:268,281,337,393,401,560,592,618,675,758` | `AgentSessionProviderRegistry::new(home).without_environment_overrides()`, discovery/status/fingerprints, source restoration/validation, summary and bounded message pages. Domain owns filesystem/SQLite/bundle parsing and root containment. | Use the existing explicit home registry for fixtures, with overrides disabled. Native UI sends opaque archive IDs/cursors, never arbitrary source paths. Do not copy source validation/parser code. |
| `ccr-ui/src-tauri/src/services/agent_sessions.rs:253,313,325,363,385,391,625-822` | State-free list/detail/refresh DTO projection and archive orchestration. Current public detail/status/refresh wrappers construct the default-home registry. `_with_registry` detail and refresh helpers are private. | Extract these policies intact to shared orchestration with an explicit registry + pool, retaining Tauri DTO mappings/export paths. Exposing an injectable owner API is needed before native fixtures use detail/refresh; invoking default-home wrappers would breach isolation. |
| `services/claude_observer.rs:147-179,181,256,315,405,444,501,528` | State-free Claude-only llmusage summary/trends/cost/cache plus `ccr_db::claude_tool_calls_repo` top-session/tool queries. Current time-window helpers read `chrono::Local`. | Compose existing usage + tool repository owners, injecting reporting time for deterministic fixtures; preserve cache-aware cost and Claude filter. Do not copy tool SQL or invent data when one source is absent. |
| `ccr-ui/src-tauri/src/claude_observer/subscription.rs:43,76`, `pricing.rs:80` | Subscription policy over existing user-settings repository and embedded pricing helper; no Tauri `State`, but DTO TS export derives remain. | Extract framework-free policy only where native and Tauri both need it. Retain three setting keys and loose forward-compatible mode/plan validation/defaulting. No duplicate settings store. |
| `crates/ccr-codex/src/services/codex_session_service.rs:138,162,238,284,304,327,376` | Path-injected Codex session inventory/list/detail/Markdown export/clone/delete. Current Tauri wrappers at `commands/codex_sessions.rs:4-134` add default path lookup, JSON shape and cache/archive invalidation. | Consume service directly with an explicit Codex home, share only mutation follow-up orchestration (cache invalidation/archive deletion marks). Preserve path validation in the domain; don't turn native UI text into an arbitrary filesystem target. |
| `crates/ccr-codex/src/services/codex_usage_service.rs:233,240,904,912` | Existing dedicated Codex raw-session scan/rolling-window owner; constructor takes Codex home, `scan(as_of)` and `compute_rolling_usage_at` support deterministic time. | Keep this distinct current Codex quota/rolling-usage surface when migrating its commands; do not quietly substitute global llmusage aggregates or duplicate parsing in native views. |
| `crates/ccr-codex/src/services/codex_quota_service.rs:128-160` | Actual quota/account/token lifecycle owner. Public `new()` resolves real paths; query paths may refresh/repair credentials and write snapshots (`:465,752,837,892`). | Must add/reuse an explicit owner path/transport seam before isolated native tests. A quota read is not a pure read-only projection. Preserve05 auth identity invalidation and test fake responses; no real network/account during implementation. |
| `commands/grok.rs:1310-1357` | Grok home uses existing `GrokPlatform::load_profiles` / `inspect_activation_state` / `profile_auth_mode`; non-local returns typed UnsupportedEnvironment. | Reuse injected Grok platform/lifecycle context from04/05, map safe summary in a worker; no duplicate activation detector. Consider keeping05's platform overview owner and07's native home consumer coordinated. |
| `crates/ccr-store/src/budget_manager.rs:24,139`, `cost_tracker.rs:29`; `commands/claude_settings.rs:142,184` | Budget manager accepts explicit config path and computes status through the legacy `CostTracker`; setters validate/persist through existing owner. Tauri updates supplied fields individually. | Inject both budget path and tracker storage root. Preserve existing budget source and update semantics; do not claim this is the same llmusage cost pipeline. Constructor/storage setup may create directories, so don't label the whole workflow a pure read. |
| `crates/ccr-store/src/pricing_manager.rs:21,83,105,111,138,167`; `commands/pricing.rs:7-94` | Path-injected custom pricing owner, validation and atomic file writes. `new(path)` reads/defaults without saving; `with_claude_defaults` explicitly saves. | Call owner directly from bounded native jobs. Preserve list/set/remove/reset policies and confirmations. Existing custom pricing file is separate from the installed llmusage pricing-upgrade protocol; don't imply a local edit reprices its database. |

### 3. Usage import extraction is a lifecycle move, not a new runner

`ccr-ui/src-tauri/src/llmusage_adapter/mod.rs:1-75` already separates shared SQL from adapter responsibilities. `LlmusageRuntime::from_paths` is a lightweight injected handle, but lives inside the Tauri package. Native must not import that package. Reuse `ccr_usage::AppPaths::from_root` (`crates/ccr-usage/src/paths.rs:22`) for reads; extract only the CLI/NDJSON/lifecycle portion genuinely needed by both frontends into an appropriate framework-free application module.

- `llmusage_adapter/cli.rs:29-59,112-156` owns `LlmusageCli`, explicit `--home`, structured sync options, and `run_sync_stream`. `SyncCommandOptions` includes rebuild/recent-days/source/provider-map (`:60-93`). Do not construct shell command strings or move process creation into GPUI render.
- `llmusage_adapter/events.rs:7-122` owns typed JobEvent/summary parsing. Plain non-JSON logging noise may be skipped; brace-prefixed malformed/unknown JSON is a protocol failure. Preserve additive pricing/bucket/token repair lifecycle events and all nine source IDs.
- `process/gateway.rs:76,184,197,266-291,583-643` owns capability descriptors/trusted executable resolution and managed wrappers, delegating tree ownership to `ccr_core::core::process_gateway::ManagedProcess` (`crates/ccr-core/src/core/process_gateway.rs:74-90`). The framework-neutral part can be shared; retain capability/output/deadline policy and Tauri-owned integration, not a second ungoverned spawn path.
- `usage_jobs.rs:236-333` is the existing state machine: admit snapshot+token under one lock; a concurrent start returns the active snapshot without a second token; request_cancel is nonterminal; progress is refused after cancellation/terminal; runner alone calls complete after cleanup, then releases admission. These types currently reference Tauri-facing result DTOs/adapter errors and require an extraction/mapping seam.
- `commands/usage.rs:431-712` is app-handle orchestration of actual execution, progress, final summary, monitoring emission, cache invalidation and observer ingest tail. `run_admitted_usage_sync` at712 also routes foreground imports through the same admission owner; keep this shared with background imports. Preserve post-import tails and cancellation/get reachability rather than broadening all command concurrency.
- Deadline remains one hour. Direct child wait, stdout callbacks and stderr reader are covered by that same deadline; termination/tree exit/reap/stderr join share five seconds. Stdout is bounded to1MiB+1 detection; stderr retains64 capped64KiB lines. A wire Finished event is provisional until child/readers complete. CleanupFailed outranks cancellation/timeout/protocol failure. Native view drop may detach a job, never assert a cancelled terminal.
- Tauri app owns `State`/event emission/cache registration. GPUI owns its UI subscription and `RequestKey`/`ResponseGate`; ccr-app owns framework-free job orchestration. Preserve one owner for admission and immutable terminals across native command variants.
- `commands/usage.rs:120` only passes provider-map when the activation map exists. Explicitly passing a missing path makes the installed CLI fail. Fixtures should supply a closed synthetic map or omit it; never discover actual home analytics accidentally.

Session-index/agent-refresh jobs are distinct from usage-sync jobs. `commands/agent_sessions.rs:49-138` currently checks active refresh, inserts a snapshot, spawns the refresh and maps counters into `session_index_jobs`. Do not pretend archive refresh has usage-sync's cancellation/terminal guarantees merely because both have job IDs. Preserve existing behavior and inspect races when providing one native app-owned owner.

### 4. Typed scope, fidelity and data semantics that must survive

1. **Reporting time:** `ccr_usage::QueryFilter` includes a typed timezone; `db.rs:158-184` validates date bounds; `db.rs:202-231` resolves the reporting zone once with capability snapshot. Date bounds are inclusive local dates turned into RFC3339 UTC half-open ranges, with historical DST. Home Insights joins session counts using the projection's window boundaries (`services/home_insights.rs:150-158`). Never use the machine's current UTC offset for historical grouping.
2. **Source versus platform identity:** usage has nine canonical source IDs, and legacy aliases are normalized by `ccr_usage`. Agent-session registry has eight independent families including Pi **and** OMP; this is intentionally not `ccr_config::Platform`. Don't collapse the registry's identities or treat native request platform as the full usage source filter. Request generation/filter/account changes must invalidate earlier responses through the complete existing key.
3. **Zero versus missing:** `HomeInsightsResponse.sessions_indexed` (`home_insights.rs:80`) is false for an empty session archive: native displays a dash/unindexed state rather than a fake measured zero. Missing llmusage DB, unsupported schema/capability, no raw sessions, stale empty window, active sync and index-required are distinct states. `build_usage_readiness` (`usage.rs:420`) and `detect_home_empty_reason` (`:651`) already encode the policy.
4. **Provider filtering:** old DB without provider capability may return an unfiltered dashboard with `provider_stats=[]` only when no provider filter was requested (`usage.rs:1082-1089`). An explicit provider filter must fail supported-feature checks, never silently return unfiltered totals. Unattributed provider is `None` (`ccr-usage/src/queries.rs:142` display helper), not a fabricated provider.
5. **Cost channels:** preserve cache-aware/cache-free costs, reasoning/output splits and pricing status/source metadata. Dedicated Codex rolling usage, llmusage projections, Claude tool-call ROI and legacy budget CostTracker are separate existing channels. A migration must identify them rather than combine their totals as if interchangeable.
6. **Archive safety:** Agent list uses `(updated_at DESC,archive_id DESC)` keyset cursor; detail restores validated provider source from opaque archive ID. Defaults80/100 and clamp1..200 (`agent_sessions.rs:253-362`); messages have stable keys/ordinals and256KiB clipping in the domain. Availability and fidelity are independent; encrypted Antigravity remains locked/partial, never invented plaintext/full fidelity.
7. **Local-only:** native checks execution environment before scheduling list/detail/provider discovery/refresh. Current Tauri Agent-session command wrappers extract a pool but do not themselves accept/capture an environment token (`commands/agent_sessions.rs:14-47`); frontend spec owns local-only gating. Do not rely on that frontend-only check when exposing native actions, and never fall back from WSL/SSH to local data.
8. **Sensitive content:** archive/list metadata must not leak physical paths; transcript/export plaintext stays scoped and unlogged, with explicit warning/cleanup. Typed static failure categories should be mapped to bilingual native explanations. Existing `Result<_,String>` services sometimes include raw DB/worker text, so do not directly display every legacy error string in native UI.
9. **Virtualization/backpressure:** page/query in bounded workers; stable list and transcript keys; native only instantiates visible rows. Existing Web monitoring batches at250ms (`features/monitoring/eventBatcher.ts:1`) but its temporary `buffer.push` is unbounded; don't copy that code as the native S2 backpressure implementation. Native contract remains its128-capacity/10Hz/2000-row/2MiB owner with truthful dropped/overflow counters and separately retained terminals.

### 5. Fixture and compatibility seams

- Usage fixture API is already public behind `ccr-usage/test-fixtures`: `create_projection_db(root)`, `SeedBucket`, `seed_bucket`, `SeedEvent`, `seed_event`, `seed_run_log`, `seed_source_file` (`fixtures.rs:23,102,153,194,245,291,317`). Production must not enable this to initialize real databases. Seed through these fixture APIs; no new native SQL.
- Use `AppPaths::from_root(temp)`; open with shared read-only owner and compare database identity/bytes around reads. Missing/corrupt/older/future-compatible schemas deserve distinct fixtures; WAL-sidecar observations are not business writes.
- Agent source fixture: `AgentSessionProviderRegistry::new(temp_home).without_environment_overrides()`; existing provider tests cover this at `providers.rs:2540`. Inject the archive pool separately. Never mutate process HOME/USERPROFILE or read default providers as a test convenience.
- Shared home/observer orchestration should accept explicit report time or supplied projection windows; retain default production clock resolution in the adapter. This makes parity tests deterministic without changing timezone policy.
- Codex session and rolling usage constructors already take explicit home paths. Quota service does **not** expose a public injected constructor/transport in the inspected API; owner-level seam is a real prerequisite, not a UI fallback. Coordinate with05 account/credential ownership before touching these files.
- Pricing/budget managers accept explicit config paths and tracker root. Test read/list separately from mutating setters/reset. Tauri wires retain camelCase fields and existing success shapes; extract behavior, not wire derives/export ownership.
- AppState caches, emit functions, generated command registry/client, TS export paths and old command names stay in Tauri. Reusing neutral projection DTOs already in `ccr_usage` is appropriate; moving a Tauri `#[ts(export_to=...)]` DTO into ccr-app would change binding-generation ownership and needs review, not a mechanical move.

### 6. Ordered independently verifiable slices

1. **Read-only usage + Home Insights.** Add only the needed shared orchestration/dependencies/typed projection, then native filter + exact-value table/trend with capability/empty/error states. Prove parity against the same `ccr_usage` fixture and injected archive pool. This unlocks real S1 fixture data without waiting for auth/quota.
2. **Agent archive read/status/detail and refresh.** Extract intact registry-aware projection/refresh helpers; use the existing domain registry and repository APIs. Implement stable page/list/transcript virtualization and local-only checks. Prove no paths in native projections and no overstatement of fidelity. Own refresh lifetime before wiring automatic refresh.
3. **Usage import lifecycle.** Move the existing neutral NDJSON/runner/registry policy once, with Tauri adapters consuming it; keep all fault-injection/process tests. Native start/status/cancel exposes actual snapshots and runner cleanup. Do not mark this complete from a fake progress stream or ordinary executor cancellation.
4. **Claude Observer and monitoring.** Extract dual-source projections/subscription policy, keep tool SQL in ccr-db and usage SQL in ccr-usage. Consume02's event service; implement native bounded monitoring retention/overflow independent of the Web timer implementation.
5. **Budgets/pricing and dedicated Codex/Grok home/session actions.** Directly consume path-injected domain APIs; preserve existing distinct channels, mutation invalidation and confirmation. Quota fake-network/path seam and05 auth generation must be ready before tests/native wiring.
6. **Native evidence + inventory and S1/S2.** Map all49 commands/12 surfaces/3 redirects to exact native actions or reviewed consolidations. Collect new solid-fallback screenshots with current build identity; true glass remains unsupported until the03 capability contract changes. S1/S2 tests require actual seeded surfaces and real timing hooks, not a debug screenshot or an empty two-profile window.

Each slice should finish its own narrow fixtures/consumer checks before a larger shared integration pass. Main must serialize shared manifests/lock generation, `ccr-app/src/lib.rs` exports, native runtime/module/shell wiring and overlapping05 Codex/auth files. No single worker should independently rewrite those shared owners.

### 7. Existing test anchors and commands to reuse (not run here)

| Owner | Specific assertion anchors |
| --- | --- |
| `crates/ccr-usage/src/db.rs:1116,1138,1261,1358,1471,1526,1555,1811` | DST half-open bounds, upstream index use, provider totals/unattributed, explicit filter narrowing, old/missing provider capability and cursor log pagination. |
| `crates/ccr-usage/src/insights.rs:454,502,659,682,726,761,793,840` | True zero snapshot, report-zone windows, ordering, historical IANA offsets, all-source trends, Gemini cutover/aliases/unknown labels and project totals. |
| `services/usage.rs:1749,1773,1804,1886,1939,1976,2033,2090,2116,2169,2218` | Shared fixture summary/trends/model/provider/heatmap/log pagination/dashboard and cold/warm/stale bootstrap. Preserve equivalent assertions through extraction and Tauri adapter parity. |
| `services/home_insights.rs:358,402,437,478,515` | Merged canonical/unknown source rows, seven-day inclusion, honest unindexed sessions, missing DB error. |
| `services/agent_sessions.rs:862,889,907,931,994,1011,1058,1105` | Cursor/limit validation, fidelity degradation, unchanged refresh skips parsing, eight-family filtering, missing/wrong-shape/escaped-source categories, deleted-source reconciliation. |
| `crates/ccr-store/src/sessions/providers.rs:2570,2664,2678,2792,2859,2880,2976,3049,3101` | Source shape/root containment, readonly SQLite members, unchanged-container enumeration gate, encrypted fidelity, bounded stable long-session pages, UTF8 clipping. |
| `usage_jobs.rs:378,420,443` | Cancel-before-spawn barrier, all immutable terminals, running cancel retains admission until cleanup-failure completion. |
| `llmusage_adapter/cli_lifecycle_tests.rs:8,111,123,135,160,172,228,240,257,278,296,328` | One-hour production deadline, silent/live-child/readers/callback deadlines, cleanup precedence/bounds,1MiB stdout flood, capped stderr, real Windows owned-child cancellation/reap. |
| `services/claude_observer.rs:625,649,682,697,723,748` | Claude filter, sort/limit/dimension validation, cache splits, real tool-call repo heatmap/top tools/session sorting and ROI branches. |
| `process/gateway.rs:738,913,957,980` | Closed descriptor catalog, structured stdin/streams, output cap, timeout and reap; `just tauri-process-smoke` owns complete process-tree matrix. |

Narrow validation begins with `cargo test -p ccr-usage --locked -- --skip export_bindings`, affected ccr-store/ccr-db/ccr-codex and new ccr-app tests, corresponding Tauri usage/home/observer/agent-session tests, native UI tests and strict package Clippy. All behavior suites retain **default parallelism** and existing export filters. One historical Agent-session spec command includes `--test-threads=1`; current repository harness instructions explicitly override that, so do not copy the serial flag. Binding ownership changes require transactional `just tauri-bindings`, followed by binding/command inventory checks. Do not silently rebaseline intermittent generated-binding drift.

## Related specs and internal references

- `.trellis/tasks/10-17-gpui-07-insights/{prd.md,design.md,implement.md}`.
- `.trellis/tasks/10-17-gpui-desktop/{design.md,implement.md,research/architecture.md,research/performance.md,research/airy-glass-plan.md}`.
- `.trellis/spec/ccr/backend/llmusage-provider-adapter.md`: SQL ownership, source vocabulary, schema/timezone/provider gates and no upstream crate.
- `.trellis/spec/ccr/backend/usage-job-lifecycle.md`: admission/token/cleanup/immutable terminal contract.
- `.trellis/spec/ccr-ui/frontend/agent-session-observability-contracts.md`: local-only/archive/provider/path/fidelity contracts.
- `.trellis/spec/ccr-app/backend/shared-desktop-services.md`: bounded worker/generation/response/lifetime contract.
- External references: none; no new dependency/version recommendation is made. Exact locked dependencies must remain unchanged unless the eventual implementer proves a needed addition and follows dependency governance.

## Caveats / Not Found

- This is bounded research, not code or runtime validation.07 is still unimplemented in the inspected native feature tree; absent pages cannot satisfy native or performance ACs.
- Wire-like/state-free Tauri functions are promising extraction candidates, not proof that the existing Tauri package is framework-free or safely linkable.
- Public fixture seams for Codex quota transport/paths, neutral import policy ownership, and registry-aware archive orchestration are real gaps in the inspected interfaces. They need owner-level work before safe native exposure; no fallback to real default home/network is authorized.
- Monitoring feed ownership crosses02/07; the current native usable core does not by itself prove that every02 event capability is integrated. Check the preserved per-command status before consuming it.
- Budget multi-field update currently invokes separate persisting setters. Do not quietly claim atomic/CAS semantics it does not have; improving that behavior is a distinct reviewed policy change if required.
- Parent current screenshots and the original performance plan do not prove 10k rows,1000 events/s, S1/S2 frame/input/CPU/memory budgets or cross-platform output. Preserve honest NOT_RUN/UNSUPPORTED states and user-deferred native size/DPI matrices.
