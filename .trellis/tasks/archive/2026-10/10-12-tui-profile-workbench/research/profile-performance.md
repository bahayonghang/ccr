# Research: Profile workbench performance

- Query: Identify startup/query/render/selection/apply/reload bottlenecks for Codex, Claude, and Grok profiles, and propose narrowly scoped improvements with transaction-safe ordering and fixture-based acceptance.
- Scope: mixed; repository source is the evidence of current behavior; official Tokio and SQLite references clarify implementation constraints.
- Date: 2026-10-12 (user timezone)
- Status: Phase 1 research only. No production files edited, real profile operations executed, credentials read, private databases opened, or performance measurements collected.

## Findings

### Conclusions and confidence

1. **Proven synchronous work:** all three platform snapshots load before the first frame; apply/off/manual reload run on the terminal event thread; successful apply/off reload every profile platform. Disk/lock/DB latency can therefore stall input. Exact milliseconds are unmeasured.
2. **Proven repeated work:** profile/current/runtime-summary APIs reread profile and runtime state; provider activation scans the complete JSONL file; each lifecycle history append constructs a new store pool and checks migrations. Removing redundant outer reload work is lower risk than changing transaction internals.
3. **Already optimized:** selection and profile rendering use in-memory data; profile list construction is limited to the current page; usage queries run on `spawn_blocking`, are loaded once per session, and are refreshed explicitly. There is no evidence of SQL per profile selection or a continuous full-frame idle redraw loop.
4. **Unmeasured hypotheses:** allocator/string-formatting cost, provider lookup cost, two-source aggregate scan cost, terminal flush cost, native credential-store reads, and antivirus/filesystem effects need fixtures and timing before prioritization.

### Files found

| Path | Responsibility |
| --- | --- |
| `crates/ccr-tui/src/tui/mod.rs` | Startup ordering, theme setup, event interval, terminal lifecycle |
| `crates/ccr-tui/src/tui/app.rs` | Platform snapshots, selection, apply/off/reload, tick dispatch |
| `crates/ccr-tui/src/tui/runtime.rs` | Shared Tokio executor, terminal guard, synchronous event/render loop |
| `crates/ccr-tui/src/tui/event.rs` | Crossterm blocking poll and key-event filtering |
| `crates/ccr-tui/src/tui/ui.rs` | Current runtime banner, visible profile rows, detail formatting |
| `crates/ccr-tui/src/tui/usage/app.rs` | Injectable, asynchronous, session-scoped provider dataset |
| `crates/ccr-tui/src/tui/profile_backend.rs` | Thin shared profile lifecycle adapter and cross-client contract tests |
| `crates/ccr-tui/src/tui/grok_auth/app.rs` | Existing single-worker operation/read channel, busy gate, disconnected-worker handling |
| `crates/ccr-tui/src/tui/codex_auth/app.rs` | Existing generation/account guarded reads and latest-pending coalescing |
| `crates/ccr-cli/src/application/profile_lifecycle.rs` | Shared synchronous apply/enable/update, locks, write journal, replay, ancillary work |
| `crates/ccr-cli/src/application/profile_switch.rs` | Existing CLI `spawn_blocking` wrapper around the synchronous lifecycle |
| `crates/ccr-cli/src/application/profile_contract.rs` | Shared isolated lifecycle fixtures and failure/replay matrix |
| `crates/ccr-cli/src/platforms/claude.rs` | Claude profile read/current-marker resolution |
| `crates/ccr-cli/src/platforms/grok.rs` | Grok profile read/current-runtime matching and operation locks |
| `crates/ccr-codex/src/platforms/codex.rs` | Codex current-runtime matching and profile secret overlay |
| `crates/ccr-cli/src/services/claude_auth_service.rs` | Claude runtime/auth diagnosis |
| `crates/ccr-codex/src/services/codex_auth_service.rs` | Codex auth snapshot and runtime summary |
| `crates/ccr-config/src/managers/provider_activation.rs` | Provider activation JSONL record, deduplication, append |
| `crates/ccr-store/src/history.rs` | CLI history persistence and reads |
| `crates/ccr-store/src/storage/database.rs` | CLI database path, pool construction, migrations |
| `crates/ccr-core/src/core/write_journal.rs` | Thread-local journal, verification, compensation |
| `crates/ccr-usage/src/db.rs` | Read-only llmusage connection and provider SQL |
| `crates/ccr-usage/src/fixtures.rs` | Synthetic upstream SQLite fixture schema/data |

### Startup and reads

- `tui/mod.rs:202-213` loads preferences, initializes theme, constructs the complete `App`, enters terminal mode, then starts the loop. The first frame is immediate within `runtime.rs:289`, but **after** app construction. `app.rs:512-522` iterates the supported Claude/Codex/Grok platforms and calls `build_profile_tab_data` for each, even if the configured first tab is an auth tab.
- `app.rs:267-294` serially obtains current profile, Codex/Claude runtime summary, then the profile list. Each platform's failure becomes an in-UI issue; runtime-summary failures currently disappear through `.ok()` (`app.rs:279-289`). A future asynchronous loader should retain recoverable error distinctions rather than presenting failed summaries as a normal empty state.
- Claude `get_current_profile` delegates through `stable_current_profile` to `current_profile_resolution` (`platforms/claude.rs:116-125,344-345`), which loads profiles, separately reads the current marker, and reads the registry. Its runtime summary creates another platform/current query and loads profiles again when bound (`claude_auth_service.rs:1001-1013`); the outer builder finally loads profiles again. This establishes repeat parsing, with exact counts dependent on state.
- Codex `stable_current_profile` loads registry, profiles, runtime config, and auth to compare the desired route (`ccr-codex/src/platforms/codex.rs:2107-2166`); `load_profiles` overlays profile secrets (`2179-2183`). `CodexAuthService::get_runtime_summary` rereads auth/current/profile data (`codex_auth_service.rs:583-598`). Caches must preserve this runtime drift check and must never use a stale display snapshot as mutation authority.
- Grok current resolution loads profiles and registry/fallback marker and compares runtime config plus entry state (`platforms/grok.rs:869-900`); the outer builder loads profiles again. Preserve raw activation intent and the runtime-match semantics when creating a combined read projection.
- Theme auto detection already has a 100 ms timeout and TTY guard (`tui/theme.rs:272-277`); pinned themes bypass detection (`231-242`). Treat this as an explicit startup allowance, not an unexplained query regression.

**Smallest useful change:** construct tab metadata and empty/loading snapshots before terminal entry, then launch bounded background reads after the first frame. Prioritize the configured visible platform; load inactive platform data lazily or with a small bounded prefetch. Preserve tab order, initial current selection, and recoverable load errors. Reuse `AsyncTaskExecutor`; do not add a second runtime or per-row workers. Existing startup specs will need an explicit amendment for lightweight App construction while preserving App-before-terminal and first-frame behavior.

**Second stage, only if measured:** introduce a domain-owned read snapshot that shares profile/current/runtime input inside one platform refresh. Snapshot parsing stays in services/repositories, never in TUI. Existing public APIs can wrap shared internal resolvers. Preserve Claude file-before-registry precedence and Codex/Grok runtime drift detection; avoid a global persistent credential cache.

### Event loop, navigation, and rendering

- `run_loop` calls synchronous `handle_key` then draws (`runtime.rs:306-309`). It draws for actionable mouse input, filters move/drag, and redraws on ticks only when state changed (`311-335`). Do not claim that it renders continuously every 250 ms.
- `EventHandler::poll_event` polls for 250 ms (`mod.rs:211`, `event.rs:44-62`). Read completion is pumped by `on_tick`, not by an executor wakeup. Without input, an already finished worker may wait roughly one poll interval; with an uninterrupted stream of Press/mouse events, ticks can be delayed. Windows Release/Repeat events currently map to Tick, whereas other terminal streams may differ. This is a control-flow observation, not measured latency.
- Profile usage starts on the first profile tick (`app.rs:1482-1488`, `usage/app.rs:74-95`) and completes on a later channel drain. Thus initial usage includes scheduling/poll delay independent of SQL time. Usage draining currently occurs only on profile tabs; new profile job completion must be drained at App level even if the active tab changes.
- Next/previous/select-at only update indexes/name/scroll (`app.rs:812-838`); selected config uses the resident `IndexMap` (`365-372`). Profile tab activation returns without rereading (`1206-1214`). Keep zero-I/O navigation.
- List formatting iterates only `current_page_profiles` (`ui.rs:525,585-596`). `sync_profile_page_size` is called from render but exits immediately unless page size changed (`app.rs:704-708`); an O(N) selected-name search occurs only after a viewport/page-size change (`710-721`). Adding headers/separators must subtract their rows from page size and update hit tests, not render every profile.
- Codex banner searches the resident list for the current profile per draw (`ui.rs:72-77`); usage detail filters/finds a resident provider row (`ui.rs:1535-1536`). These are O(profile/provider count) in-memory operations. A current-name reference or `(source, provider)` map can remove scans if fixture sizes justify it.
- Details allocate formatted lines every draw (`ui.rs:711-746`) and clone all lines into `Paragraph` only to retain their count (`757-764`). Capture count and move lines is a surgical allocation reduction. A broader formatted-detail cache needs keys for profile snapshot version, selection, usage version, viewport/width, expanded state, language, theme, and time bucket; do not introduce this complexity without measured need.
- Scroll extent is currently based on logical line count despite wrapping (`ui.rs:753-760`). The richer right pane should test long wrapped values and width changes; improved performance must not keep an incorrect scroll limit.

**Minimal loop improvement:** pump completed jobs and due timers after each event as well as timeouts, using a monotonic deadline. While a task is pending, cap wait to a proposed 50 ms; return to the existing idle interval when no timer needs faster service. A pending flag alone is insufficient if ticks occur only on timeout. Avoid an always-redraw loop or an unbounded key-event drain that starves worker results. Unit-test the scheduler/pump using an injected clock/event source.

### Usage and the new last-used column

- `UsageApp` has an injectable `UsageLoader`, one `task_active` guard, one dataset, and `spawn_blocking` (`usage/app.rs:36-53,61-72,99-132`). Default load asks for Claude and Codex all-time provider breakdown (`159-167`). Grok is not requested by this current loader even though `SourceKind::Grok` exists; extending the pane should add it intentionally with empty/unsupported states.
- `Dashboard::open` opens SQLite read-only, sets a 5 s busy timeout, and probes schema/capabilities (`ccr-usage/src/db.rs:202-225`). Provider aggregation reads `usage_bucket_30m`, groups and sorts (`447-489`); the source wrapper calls it once per source (`494-514`). Therefore initial current scope is **two aggregate queries per dataset**, independent of profile count. Three-platform scope can remain three bounded aggregates. A combined-source optimization needs equivalence evidence and a spec update, not merely an assumption that one SQL statement is faster.
- There is no actual query plan or production database evidence here. Use `EXPLAIN QUERY PLAN` against owned fixtures and record scanned rows/bytes and group/sort behavior before changing SQL. Never create indexes or migrations in the upstream llmusage database.
- A refresh overwrites the previous usage state with `Loading`, and repeated refreshes while active are dropped (`usage/app.rs:99-108`). Keeping last successful data with a separate pending/error status avoids blanking the new dense detail panel. If refreshes must represent changes after the current read began, coalesce one latest-pending refresh instead of silently dropping it.
- Last profile activation is separate from provider token usage. Shared lifecycle records `Switch`, successful result, profile in `to_config`, platform in `extra`, and operation id in history (`profile_lifecycle.rs:333-349`). Query this as a **single bulk projection keyed by platform/profile** at refresh, not one query per row, and label the meaning as last applied/used via CCR. Other data research should decide the exact fallback for old rows with ambiguous platform metadata.
- `HistoryManager::with_default` calls `Database::init_default` (`ccr-store/src/history.rs:180-182`), which creates a directory, a pool, and checks migrations (`storage/database.rs:25-41,52-71,92-125`). It is inappropriate to construct this manager inside drawing or each row, or to assume this constructor is a pure read. Provide a store-owned read-only/open-existing projection for display; missing DB is missing history, not a reason to create one.
- Avoid `HistoryManager::load()` plus per-row filtering: it selects all columns/all rows (`history.rs:274-284`). Only required aggregate metadata should enter the UI snapshot. Existing history indexes are timestamp and operation (`storage/database.rs:236-237`); a composite covering index is a **measurement-dependent proposal** and belongs to an additive CCR-store migration, not TUI or llmusage.
- History timestamps are RFC3339 local-offset strings (`history.rs:248`). Compare parsed instants or a consistent database time expression; raw lexical `MAX(timestamp)` is not guaranteed chronological across offsets. The generic full-row decoder substitutes `Local::now()` on invalid timestamps (`193-195`), so a last-used projection must not accidentally display corrupt data as just used. Unknown/invalid should stay unknown, and relative time should update only at a suitable minute/time bucket.

### Applying, off, and redundant refresh

- **Current Enter/Space semantics are stay-open:** both map to `ApplySelected` (`app.rs:757`), `dispatch` calls `apply_selected()` but discards its boolean (`858-859`), and then returns `Ok(false)` (`877`). That boolean describes activation success, not permission to exit. Preserve stay-open on success and failure after asynchronous conversion. `q`/`Esc` map to Quit (`750`); `handle_key` currently exits immediately for Ctrl+C before any busy guard (`1318-1322`). A new profile-mutation busy guard must account for this earlier path.
- `app.rs:960-975` calls `profile_backend::apply` synchronously and then `reload_profiles`; `off_selected` synchronously calls `profile_off_for_platform` and reloads (`1003-1016`). `r` reloads before scheduling usage (`867-874`). `reload_profiles` loops all profile tabs (`1028-1043`). These are the strongest directly evidenced responsiveness targets.
- The lifecycle application lock can wait 10 s (`profile_lifecycle.rs:105-109`), and the lock implementation sleeps while contended (`ccr-core/src/core/lock.rs:147-156`). The 10 s is a configured upper wait for that lock, not a measured normal operation duration; other work may add more time.
- Lifecycle validates before locking, replays and rereads under lock, obtains the source version, then executes (`profile_lifecycle.rs:201-218`). Its repetition guards changes between preparation and application; removing the second validation to save I/O would break the contract.
- The journal verifies before/after mutation, stores a committed outcome, rolls back or reports recovery on failure, then commits (`258-313`). Usage count, history, provider attribution, Codex auth-registry sync, and final outcome update follow commit (`317-379`). `WriteJournal` state is **thread-local** (`ccr-core/src/core/write_journal.rs:16`); the complete operation must remain in a single synchronous worker, without awaits or thread hops inside the journal.
- CLI already wraps its synchronous adapter in `tokio::task::spawn_blocking` (`profile_switch.rs:22-33`). TUI should move its complete adapter call into the existing executor, not replace the lifecycle or split runtime/pointer/history operations into independent jobs.
- Every apply calls `HistoryManager::with_default` (`profile_lifecycle.rs:333`), thus creating a new pool/checking migrations as above. This is proven repeated setup, but its latency is unmeasured. If material after UI unblocking, use an explicitly injected/reused store handle tied to the resolved CCR root/session. Do not add an unkeyed process-global pool that breaks test/home overrides or bypasses migration ownership.
- Provider analytics resolves the whole profile map again (`provider_activation.rs:150-153`) and reads the entire activation log before reverse scanning (`173-179`). File I/O/allocation scale with total log bytes even when the last line matches. If measurements warrant, use chunked reverse reading that stops at the latest matching platform while preserving malformed-line skipping and UTF-8 boundaries; do not truncate analytics. Its current dedupe read precedes the append lock (`157-164,191-195`), so a changed implementation must also assess concurrent read-plus-append ordering rather than introducing a stale cached last line.

### Required ordering and failure behavior for asynchronous profile jobs

Reuse patterns, not a new general task framework:

- Read pattern: Codex Auth increments a generation, retains one latest-pending request, and accepts a result only for current generation/account (`codex_auth/app.rs:1435-1459,1623-1645`). Profile results should carry `(platform, generation)`; do not use whichever tab happens to be selected on completion. Return a complete snapshot for atomic replacement on the TUI thread.
- Mutation pattern: Grok Auth freezes a request, permits one worker, runs mutation plus subsequent read, then sends both outcomes (`grok_auth/app.rs:170-191`); its disabled-executor and disconnected-channel paths are explicit (`174-176,400-409`). Keep operation outcome independent of refresh success. A committed profile with failed reload must say applied + stale/read failure, never imply rollback or auto-retry the mutation.
- Use one in-flight profile mutation across this workbench as the simplest safe UI policy. **Recommended concrete policy:** during profile Apply/Off, ignore duplicate Enter/Space/Off and gate both keyboard tab changes and header mouse tab changes before Auth delegation. This prevents reaching Auth mutation handlers while the profile worker runs, without redesigning all Auth initialization. Do not queue one new operation id per repeated Enter. Lock ownership inside the domain remains authoritative against other processes.
- Freeze platform/name/operation id at submission. Resize, language/theme display, and feedback remain functional; selecting/scrolling resident rows can remain available only if the operation result stays associated with the frozen request. The simplest first implementation may freeze row selection as well. Do not apply a result to a subsequently selected row. Once the worker result is consumed, tab/operation gates release.
- Invalidate reads that began before mutation submission; after completion refresh only the affected platform's profiles/runtime/history projection. Refresh any initialized matching Auth view or mark it dirty for activation. An older read must not replace the post-apply current marker. Explicit global reload can still refresh all platforms, with bounded/coalesced work.
- Handle `q`/`Esc` and TUI-delivered Ctrl+C during a profile mutation by recording deferred exit and showing that the operation is finishing; do not return `true` before its outcome is collected. Install this guard before the existing global Ctrl+C early return. After result collection, terminal teardown and existing safe post-exit summary may proceed. Without an explicit quit request, Enter/Space completion stays in TUI. Do not report cancellation, silently drop the outcome, or start compensation outside the domain owner. Read-only jobs may be abandoned without replay. A disconnected mutation worker is an unknown outcome requiring safe reread/recovery guidance, not automatic resubmission. Forced process termination cannot be made safe by a UI guard; retain the journal/interrupted-marker contract.
- Worker messages carry typed domain outcomes and safe raw error categories; localize only on the TUI thread, because language is thread-local. Cached display DTOs must not duplicate raw credential payloads or log profile maps.

### Fixture-based measurement plan

No timings were executed during this research. Implementation should first capture a baseline, then compare the exact same release build/fixtures/terminal sizes after each targeted change.

1. **Fixtures:** 0/1/25/100/1,000 profiles per platform, CJK/long names and descriptions, enabled/disabled/current/drifted/missing/corrupt state, and unknown last-used values. Isolate `CCR_ROOT`, `CCR_DATA_DIR`, `CCR_LOCK_DIR`, Codex/Claude/Grok homes using the existing process environment fixture; no personal config/DB and no real account/network activity. The shared `profile_contract::isolated_fixture` is at `profile_contract.rs:67`; all-platform apply/replay/ancillary/failure fixtures begin at `82,108,164,200,251,272,299`.
2. **Loads:** inject counted/delayed read loaders to prove zero UI-thread reads. Measure first loading frame separately from first ready profile/runtime frame and from ready usage/history. Test delayed inactive platform loading does not delay the visible tab. Include repeated reload and an intentionally stale read finishing after mutation.
3. **Render:** Ratatui `TestBackend`, 80x24 / 100x30 / 140x40 / 220x60 plus compact 40x12; EN/ZH and both palettes. Run warm-up then at least 1,000 selection/render samples. Count loader invocations during 1,000 navigation/scroll events (must remain unchanged). Measure CPU draw separately from actual PTY output flush; TestBackend timing alone is not visible terminal latency.
4. **Queries:** build llmusage fixtures with 0 / 100k / 1m buckets distributed across sources/providers, plus 0 / 10k / 100k history rows with offset timestamps, ambiguous metadata, failures, and invalid dates. Reuse `ccr-usage` test-fixtures (`fixtures.rs:1-26`, schema includes supported range indexes). Time DB open/capability discovery separately from aggregate SQL; record query count and query plans. Confirm missing/old/locked/corrupt DB has a recoverable state. Run each operation cold/new-connection and warm/repeated; report hardware, filesystem, build profile, fixture sizes, samples, median/p95/max.
5. **Mutation:** a fake backend/barrier delays apply 1 s so the test can prove immediate pending state, responsive resize/language, no duplicate submit, safe quit deferral, and eventual snapshot application. Actual lifecycle fixtures then verify successful apply, replay with same id, failure at every write, ancillary warnings, and external CAS conflict. Measure lock wait, activation transaction, ancillary work, and post-apply reload separately. Vary activation JSONL at 0 / 1k / 100k lines.
6. **Idle/scheduler:** injected clock/event sequence tests for continuous key events, worker completion, resize bursts, and empty ticks. Once snapshots settle and no toast/time bucket changes, ticks should cause no redraw or reload. No real terminal session is needed for correctness tests; final visual/PTY smoke verifies real behavior separately.

### Suggested acceptance budgets (proposals, not measured achievements)

| Concern | Proposed acceptance | Rationale / limits |
| --- | --- | --- |
| Selection/scroll | Zero file/DB reads and zero worker submissions after snapshots load | Deterministic guarantee; prevents N+1 work regardless of hardware |
| Render CPU | Release fixture p95 <= 16 ms at 140x40 with 1,000 loaded profiles | Keeps common interactive redraw within a frame-sized CPU budget; terminal flush measured separately |
| Action acknowledgement | Pending/apply state visible within 100 ms; fake backend blocked for 1 s must not block resize/language | Measures responsiveness rather than weakening durable operation semantics |
| Completion propagation | Worker-finished to visible-state p95 <= 100 ms on reference machine | A pending 50 ms poll cap plus rendering gives practical headroom; continuous input must not starve completion |
| Initial loading frame | <= 250 ms from TUI entry with pinned theme on reference fixture | Should no longer grow with profile count or inactive service delays; report theme auto probe separately (bounded 100 ms) |
| Usage/history refresh | One bounded bulk history projection and one aggregate per requested source per refresh; no profile-count multiplication | Strong structural target; retain schema/source/provider correctness |
| Query time | Provisional p95 <= 250 ms for 100k rows and <= 1 s for 1m rows on declared reference hardware | Planning target only; collect baseline before making it a release gate, since upstream indexes and hardware control it |
| Apply transaction | No additional writes/activation replay; total p95 must not regress >20% in same fixture/hardware, excluding induced lock waits | No absolute promise for filesystem durability or external locks; primary gain is UI responsiveness and narrower refresh |
| Idle | Zero draw/load calls for unchanged-state ticks | Avoid trading lower completion latency for continuous CPU/output work |

Keep deterministic call-count/state-order tests in routine CI. Timing thresholds need a documented release-build benchmark environment and enough samples; do not put fragile wall-clock assertions into parallel unit tests. File-name and manifest searches found no representative benchmark target in the inspected TUI/CLI/usage/store crates and no Criterion/Divan/IAI/`[[bench]]` declaration in those manifests. This is explicitly unmeasured; no measurement code is being added during planning. If later implementation adds one, a small focused harness is sufficient, with no new framework unless needed.

### Suggested implementation order

1. Add counted, delayed loader/backend seams and capture startup/apply/query/render baseline with synthetic fixtures.
2. Move startup/read/reload and complete lifecycle apply/off calls to bounded existing executor jobs; implement generations, mutation serialization, stale/read-failure state, and quit behavior together.
3. Change successful mutation invalidation from all-platform reload to the affected platform plus dependent initialized Auth/history snapshots. Keep explicit refresh behavior reviewable.
4. Add bulk last-used and additional usage/history read projections in their owners; all table/detail cells consume snapshots only.
5. Remove obvious detail-line clone and measure visible-row rendering. Add indexing/caching/read dedup only for demonstrated residual costs.
6. Consider activation JSONL tail-read and store-pool reuse as separately verified, scoped optimizations if they materially affect measured total apply time.

### Related specs and verification

- `.trellis/spec/ccr-tui/backend/backend-guidelines.md`: state/render separation, zero-I/O embedded usage, display-width handling, per-tab selection, localized worker results, startup theme, existing busy/unknown-outcome pattern.
- `.trellis/spec/ccr-cli/backend/profile-application-lifecycle.md`: one synchronous lifecycle owner, operation lock, journal, replay, compensation, ancillary warning semantics. Current text says TUI calls the synchronous adapter; any new scheduling should clarify that the adapter still executes synchronously inside one TUI worker.
- `.trellis/spec/ccr-cli/backend/grok-profile-runtime.md`: runtime CAS, operation serialization, drift/entry state, no credential disclosure.
- `.trellis/spec/ccr-cli/backend/test-fixtures.md`: `TestHome`/`TestHostEnv` and process-environment lock ownership.
- `.trellis/spec/ccr-config/backend/profile-repository.md`: pure queries; operation -> resource -> guarded leaf lock order; versions, partial edits, runtime-aware markers.
- `.trellis/spec/ccr/backend/llmusage-provider-adapter.md`: all usage SQL belongs to `ccr-usage`; upstream DB remains read-only; provider-level attribution is not per-profile usage.
- `.trellis/spec/ccr-store/backend/backend-guidelines.md`: CLI history/storage owner, batch/query ownership, tempfile fixtures.
- `.trellis/spec/ccr-db/backend/backend-guidelines.md`: storage split; do not introduce `ccr-db`/desktop pool dependencies into CLI just for last-used.
- Relevant future checks: `cargo test -p ccr-tui --all-features -- --skip export_bindings`, affected `ccr-cli`/`ccr-codex` lifecycle tests, `cargo test -p ccr-usage --all-features -- --skip export_bindings`, `cargo test -p ccr-store --all-features -- --skip export_bindings` when touched, `just fmt-check`, `just lint-strict`, and cross-crate final `just ci`. Preserve root AGENTS default test parallelism; older spec examples asking for whole-suite serial tests do not override that contract.

### External references and versions

- Local lockfile: Tokio **1.53.1** (`Cargo.lock:3874-3875`), Ratatui **0.30.2** (`2881-2882`), rusqlite **0.40.2** (`3156-3157`); TUI manifest selects Crossterm **0.29.0**, unicode-width **0.2.2**. No dependency upgrade is required by this research.
- [Tokio 1.53.1 spawn_blocking documentation](https://docs.rs/tokio/1.53.1/tokio/task/fn.spawn_blocking.html), checked 2026-10-12: started blocking closures cannot be aborted; dropping/timing out a wait does not cancel their work. This supports explicit in-flight mutation/quit handling and one bounded job, rather than cancellation claims.
- [SQLite EXPLAIN QUERY PLAN](https://www.sqlite.org/eqp.html), checked 2026-10-12: useful for scan/index/sort investigation; output text can change by SQLite release. Capture plans against the bundled fixture engine and avoid brittle whole-plan string comparisons.

## Caveats / Not Found

- No measured bottleneck ranking, real-home timing, database inspection, or benchmark result is claimed. Static synchronous/repeated work is proven; latency impact remains to be measured.
- This research did not run test/build commands because it changes only this research document. Future commands above are proposed validation, not passed checks.
- Runtime-summary service calls can consult platform credential stores, but only source was inspected here. Worker/display DTO design must preserve existing masking and Grok secret exclusions.
- The richer right pane and last-used semantics need coordination with the parallel data/UI research. Existing provider totals cannot establish per-profile token consumption or exact last prompt time.
- CLI history missing/invalid platform or date metadata needs an explicit unknown/ambiguous policy. Avoid guessing from same-named profiles, current status, file mtime, or provider totals.
- Store pool reuse and provider JSONL tail scanning extend beyond TUI scheduling; stage them only with measured evidence and dedicated safety tests. No speculative indexing of the upstream DB.
