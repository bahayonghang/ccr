# GPUI07 implementation log

Date: 2026-10-27. This worker owns07 usage/session/observer/import policy and native feature modules. Other agents own05 settings/auth and common integration files, and04 profile focus repair. Their changes are preserved. No real model, quota API, installed-CLI sync, account or user configuration is accessed.

## Change boundary and order

The current native shell has no usage or archive surface. Existing projection SQL is already owned by `ccr-usage`; the missing behavior is shared orchestration and a native bounded-worker consumer. The first slice moves the existing State-free Home Insights merge/session-window policy into `ccr-app/src/usage/`, while keeping Tauri wire DTOs and generated export paths in their existing owner. It adds typed usage reads and native exact-value/virtualized presentation. Session registry-aware policy then moves intact into `ccr-app/src/sessions/`, preserving opaque IDs, provider path validation and truthful availability/fidelity.

Subsequent slices preserve the current import admission/NDJSON/cleanup owner, Claude observer, monitoring bounds, budgets/pricing and distinct Codex rolling/quota operations. Quota requires an actual explicit-path/fake-network seam before native fixture exposure. No usage SQL, parser or security policy is copied into a new surface. No placeholder success, fabricated zero, default-home fixture fallback or raw diagnostic display is acceptable.

Shared manifests/locks, `ccr-app/src/lib.rs`, native module exports and shell navigation remain single-owner05 files until an explicit coordinator handoff. New modules may be prepared without those files; they are not delivered or tested merely because they exist. Existing package versions/checksums are retained. Meaningful fixture parity, stale response, pagination/visible rows and failure/cancellation tests precede full integration checks. Default Rust parallelism and `--locked --skip export_bindings` remain.

## Verification

2026-10-27 first slice: `ccr-app::usage::home::read_home_insights` reads the existing owner projection and joins archive counts using the exact reporting windows. Tauri retains its wire DTOs/TS exports and maps the shared snapshot. Its existing cache round-trip test now compares the full shared and wire JSON payloads.

| Check | Actual result | Evidence |
| --- | --- | --- |
| `cargo test -p ccr-app --locked usage::home:: -- --skip export_bindings` | PASS exit0,3 tests | `outputs/desktop/insights/checks/home-shared-test.log`, tool completion05b231 |
| Canonical/unknown source merge and report boundaries | PASS in real projection/archive fixtures; UTC start/end boundaries, unmapped source joins once, upstream primary DB bytes unchanged | `src/usage/home_tests.rs` |
| Unindexed session state and exact distributions | PASS; flag false, values preserved directly from ccr-usage | Same fixture tests |
| Missing upstream DB | PASS; error rather than fabricated zero, absent root remains absent | Same fixture tests |
| Published dependency metadata | Exact chrono0.4.45, chrono-tz0.10.4, phf/phf_shared0.12.1 official crates.io API returned200; checksums match desktop lock, MSRVs1.62/1.65/1.61/1.61 below1.95 | `outputs/desktop/insights/dependencies/receipt.json` and raw responses |

The web fetch tool failed for those metadata URLs; a direct read of the official API succeeded and raw responses were retained. Existing versions were preserved, not upgraded. Common manifest/lock owner05 integrated required edges and exports.

Tauri Home tests/parity and native consumer build/control checks are still pending. The first-slice shared sources and Tauri mapping are currently frozen for04's short repair-candidate build window; new native modules can be prepared unreferenced. No native, performance, complete49-command or parent acceptance is claimed. Actual later commands/results and remaining gaps are appended as each slice is verified.

## Continued source-ready slice, 2026-10-27

The old short source freeze is released for final integration. This section supersedes the temporary freeze statement above, not its historical test results. No build, formatter, lint, test, installed CLI sync, network call, GUI window, account mutation or real HOME/config access was performed by this continuation worker. All newly added tests below are **NOT_RUN**, not PASS. Coordinator owns the serialized final gates and runtime evidence.

### Concrete delivered source boundary

- `crates/ccr-app/src/usage/read.rs` assembles overview, daily trends, model/provider/project/source breakdowns and optional heatmap from one `ccr_usage::Dashboard` connection with the complete typed filter. Native uses canonical DTOs without currency rounding or recalculating pricing. Unsupported provider capability degrades only when no explicit provider filter exists and is retained as `providers_supported=false`, not a measured empty dimension. Reversed report bounds fail.
- Tauri `services/usage.rs::compute_dashboard` now consumes that exact report through `llmusage_adapter/db.rs::Dashboard::shared`. Existing wire conversion functions, DTO derives/TS export destinations, archive diagnostics, readiness, cache ownership and command signatures remain in Tauri. No generated binding or command inventory is changed.
- `crates/ccr-app/src/sessions/mod.rs` owns the existing list validation/keyset policy and explicit-registry detail restoration. All eight archive families remain distinct, including Pi and OMP. List pages default80 and clamp1..200; detail defaults100 and clamps1..200. Current and archived fidelity combine conservatively. Typed static errors carry no paths, transcript or parser diagnostic payload.
- Tauri `services/agent_sessions.rs` delegates list and explicit-registry detail to the shared policy and retains its wire structs/mappings, default production registry construction, refresh orchestration and counters. No refresh/job/cancellation behavior was redefined.
- `ccr-desktop/src/runtime/insights.rs` accepts **only** the three supplied `InsightsPaths` roots. Reads run through the existing bounded `Services.executor.read` owner; non-local `RequestKey` environments fail before filesystem or provider queries. Usage/log reads do not open or initialize the archive. Home/archive reads consume an existing explicitly supplied CCR archive using the existing domain pool; no migrations or legacy/default-home discovery are invoked. Existing pool PRAGMAs/SQLite sidecars are not claimed to be a new read-only SQLite connection seam.
- `ccr-desktop/src/features/insights/{mod,rows}.rs` provides Home/Usage/Sessions tabs; explicit source/model/provider/project/date filters, local/UTC reporting, canonical exact-value tables and selected-row token/cache/reasoning/cost/pricing details. A bounded first-four source comparison uses native primitives with exact labels and all source rows remain in the table. Home explicitly stays all-history/all-source and reports seven-day request/session windows plus honest unindexed state. Usage shows stored last-sync evidence, not an invented freshness/job state.
- Usage records use100-row keyset pages with raw JSON disabled. Archive rows use100-row keyset pages; explicit row activation opens100-message provider pages with older-page continuation. Fixed36px summary/message rows are virtualized with stable domain IDs and opaque archive IDs; only the selected message instantiates the full scrollable plaintext detail, which is cleared on page switch/suspend. This is a virtualized message selector plus one transcript reader, not a claim of measured multi-height transcript-list virtualization or S1/S2 performance.
- Shared integration was requested from `PlatformsContinue`: `pub mod sessions` in ccr-app; `pub mod insights` in native runtime/features; existing ccr-usage fixture feature as a desktop dev edge; launch flags `--insights-usage-root PATH --insights-archive-db PATH --insights-provider-home PATH` required as a group; Services optional paths/accessor; one Insights navigation entity with internal tabs. Worker did not edit common manifests, locks, lib/runtime/features exports or shell files.

### Exact new fixture coverage and coordinator filters

Keep default parallelism; skip `export_bindings`. No actual result is recorded for the following new source.

| Coordinator filter | Added coverage | State |
| --- | --- | --- |
| root `cargo test -p ccr-app --locked usage:: -- --skip export_bindings` | Existing three Home join/window fixtures plus new complete source/model/provider/project/UTC-date filtering, owner-projection equality for every section, exact stored pricing/cost/reasoning/cache splits, unattributed provider, primary upstream bytes unchanged, measured empty zero versus missing DB, reversed dates | NOT_RUN for changed tree |
| root `cargo test -p ccr-app --locked sessions:: -- --skip export_bindings` |205 archive rows, clamp200, tied-date keyset order/nonoverlap, Pi103 versus OMP102, query/date/cursor rejection,250 real Codex messages with two nonoverlapping100-message pages, archived partial not upgraded, provider bytes unchanged, missing/escaped source categories | NOT_RUN |
| desktop test filter `runtime::insights::tests` | Real bounded-runtime usage read and101-event log fixture pagination100+1, exact cost/reasoning, raw JSON absent, no archive/provider-root creation, remote/unconfigured/missing fail closed, static safe messages | NOT_RUN |
| desktop test filter `features::insights::tests` | Rendered Usage/Models/Apply controls against real projection fixture, exact cost/pricing detail and provider/date filters;10k archive candidate virtualization/stable opaque selection; rendered archive row -> real250-message Codex detail -> older page, partial fidelity, no duplicate keys, scoped plaintext cleanup and provider bytes unchanged | NOT_RUN |
| Tauri `services::usage::service_tests` | Existing dashboard service regressions plus `compute_dashboard_matches_shared_report_through_existing_wire_mappings`: same source/provider/date fixture, all shared sections converted through unchanged Tauri mappings, exact tokens and heatmap | NOT_RUN |
| Tauri `services::agent_sessions::tests` | Existing refresh/source guards now consume shared reads; new `tauri_archive_reads_match_shared_explicit_registry_policy` compares full list/detail JSON after explicit wire mapping, including cursor/fidelity/messages | NOT_RUN |
| Tauri `services::home_insights` | Existing full shared/wire Home payload parity fixture retained from first extraction | NOT_RUN on integrated tree |

Coordinator should format the owned files once, then serialize root ccr-app tests/lint, affected Tauri usage/Home/archive tests and bindings/inventory/no-crate checks, desktop check/test/lint/build and actual native explicit-root smoke. Existing ccr-usage provider-capability/old-schema and historical-DST fixtures remain the authoritative owner tests; this continuation adds no SQL outside that owner. Native keyboard/assistive technology, actual light/dark/en/zh rendering, newly built executable identity, timing and memory are still unverified. Size/DPI matrix remains user-deferred.

### Preserved49-command readiness inventory

`SOURCE_READY` below means a concrete native read path and authored fixtures, **not compiled/tested/native accepted**. `UNIMPLEMENTED_NATIVE` preserves the broader approved backlog. No command is promoted to completed.

| Command | Disposition |
| --- | --- |
| `get_usage_summary_v2` | SOURCE_READY: Usage summary |
| `get_usage_capabilities_v2` | UNIMPLEMENTED_NATIVE: no complete capability surface; selected report/provider errors only |
| `get_usage_trends_v2` | SOURCE_READY: Daily exact-value table |
| `get_usage_by_model_v2` | SOURCE_READY: Models table/details |
| `get_usage_by_provider_v2` | SOURCE_READY: Providers table/details |
| `get_usage_by_project_v2` | SOURCE_READY: Projects table/details |
| `get_usage_heatmap_v2` | SOURCE_READY: exact date/request table, not heatmap visualization acceptance |
| `get_usage_logs_v2` | SOURCE_READY:100-record cursor pages, no raw payload display |
| `get_usage_dashboard_v2` | SOURCE_READY for read sections; native archive/readiness/bootstrap parity remains open |
| `get_home_usage_overview_v2` | UNIMPLEMENTED_NATIVE: distinct Home overview/bootstrap contract |
| `get_home_insights` | SOURCE_READY: shared Home join and source comparison/table |
| `ensure_session_index_v2` | UNIMPLEMENTED_NATIVE |
| `get_session_index_job_status_v2` | UNIMPLEMENTED_NATIVE |
| `start_usage_import_job_v2` | UNIMPLEMENTED_NATIVE |
| `get_usage_import_job_status_v2` | UNIMPLEMENTED_NATIVE |
| `cancel_usage_import_job_v2` | UNIMPLEMENTED_NATIVE |
| `import_usage_v2` | UNIMPLEMENTED_NATIVE |
| `import_all_usage_v2` | UNIMPLEMENTED_NATIVE |
| `agent_sessions_list` | SOURCE_READY: bounded archive pages |
| `agent_sessions_get_detail` | SOURCE_READY: explicit validated provider/message pages |
| `agent_sessions_get_provider_status` | SOURCE_READY: eight availability/fidelity statuses |
| `agent_sessions_start_refresh` | UNIMPLEMENTED_NATIVE: no automatic or manual refresh exposed |
| `agent_sessions_get_refresh_status` | UNIMPLEMENTED_NATIVE |
| `claude_observer_get_insight` | UNIMPLEMENTED_NATIVE |
| `claude_observer_daily_trend` | UNIMPLEMENTED_NATIVE |
| `claude_observer_cost_breakdown` | UNIMPLEMENTED_NATIVE |
| `claude_observer_cache_stats` | UNIMPLEMENTED_NATIVE |
| `claude_observer_top_sessions` | UNIMPLEMENTED_NATIVE |
| `claude_observer_tool_heatmap` | UNIMPLEMENTED_NATIVE |
| `claude_observer_top_tools` | UNIMPLEMENTED_NATIVE |
| `claude_observer_subscription_get` | UNIMPLEMENTED_NATIVE |
| `claude_observer_subscription_set` | UNIMPLEMENTED_NATIVE |
| `claude_get_budgets` | UNIMPLEMENTED_NATIVE |
| `claude_update_budgets` | UNIMPLEMENTED_NATIVE |
| `codex_list_sessions` | UNIMPLEMENTED_NATIVE: distinct mutable Codex-owned surface |
| `codex_get_session_detail` | UNIMPLEMENTED_NATIVE: not substituted by archive browsing |
| `codex_export_session` | UNIMPLEMENTED_NATIVE |
| `codex_clone_session` | UNIMPLEMENTED_NATIVE |
| `codex_delete_session` | UNIMPLEMENTED_NATIVE |
| `codex_get_usage` | UNIMPLEMENTED_NATIVE: dedicated rolling owner remains separate |
| `codex_get_dashboard_overview` | UNIMPLEMENTED_NATIVE |
| `codex_get_dashboard_usage_summary` | UNIMPLEMENTED_NATIVE |
| `codex_get_all_quotas` | UNIMPLEMENTED_NATIVE: owner explicit-path/fake-network prerequisite |
| `codex_get_quota` | UNIMPLEMENTED_NATIVE: owner explicit-path/fake-network prerequisite |
| `grok_get_dashboard_overview` | UNIMPLEMENTED_NATIVE |
| `set_pricing` | UNIMPLEMENTED_NATIVE |
| `get_pricing_list` | UNIMPLEMENTED_NATIVE |
| `remove_pricing` | UNIMPLEMENTED_NATIVE |
| `reset_pricing` | UNIMPLEMENTED_NATIVE |

The15 preserved route records are not all migrated by these internal tabs: `/` Insights subset, `/usage` read subset and `/agent-sessions` archive-read subset have new consumers; Claude/Codex/Grok/Antigravity/OpenCode homes, budget/pricing/monitoring, distinct Codex sessions and the three redirects retain their original pending acceptance.02 retains monitoring-feed command ownership. No quota, default auth, API, sync, import, refresh mutation or fake progress runner is exposed.

Common integration handoff: `PlatformsContinue` reported the exports, Services paths, strict grouped **absolute** launch flags, boot branch, Usage/Sessions shell navigation and desktop dev-only fixture feature integrated. No package version/checksum/lock update was needed for that empty fixture feature. This is source-owner coordination, not verification evidence.

Coordinator gate-recovery handoff: the first integrated native check reported an escaping render `self` capture and a missing `TestAppContextExt` import; strict app all-target Clippy reported63 test-fixture unwrap/expect diagnostics (`outputs/desktop/continuation-20261027/app-clippy.log`). Fixed selection access through the processor's `this` entity, imported the existing wait-for extension, removed the reported unused traits, and applied the repository's existing module-scoped test-only fixture allowance to the shared/native fixture files. No production lint relaxation or behavior suppression was added. Coordinator formatting changes were preserved. No check was rerun by this worker; corrected tree remains pending coordinator verification.

Second coordinator recovery: removed the unused Tauri `LlmusageAdapterError` import and obsolete adapter `source_breakdown`/`insights` wrappers after fallback reference search found only their own delegations. Canonical ccr-usage APIs and shared report consumers remain unchanged; no warning suppression was added. Fixed the two native dynamic test locators to use owned `SharedString`, following the existing command-state test convention, instead of borrowing temporary formatted strings into static element IDs. No checks rerun; source frozen for coordinator.

### Coordinator verification after handoff

Shared app36 tests passed (two existing ignored), strict all-target/all-feature Clippy passed; Tauri services52 passed. Native Rust36 passed, including exact usage/filter/provider/pagination/virtualized candidate fixtures; desktop strict Clippy, fmt-check and debug build passed. One collapsible conditional was corrected without behavior change. Tooling12 Python tests and3 AccessKit harness regressions also passed.

Actual native Home/Usage entry with explicit missing fixture storage displayed the safe missing-database error without fabricated zero values; final process exited0. Screenshot/receipt: `outputs/desktop/profiles/native/repaired-20261027-231848-frame-fixed/`. Native populated-data workflows, full bindings, broader49-command/15-route inventory and performance remain open. Parent `research/resume-20261027.md` records exact integration scope and evidence.
