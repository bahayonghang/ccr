# Fixed S2 runtime stream implementation — 2026-11-06

Status: ReadyForReview; current source independently checked, final aggregate CI/package owned by the coordinator. No commit by this implementer. Reused task10-17-gpui-desktop. Baseline and final pre-handoff HEAD `998dd9f2204abe99a3f468a857c362fb9e6b8120`; shared worktree changes were preserved. Native GUI verification remains explicitly deferred by the user.

## Delivered boundary

The real source is now implemented behind explicit Benchmark/S2 controls. This does not change manifest admission/report-clock algorithms from57abc91, add a task/framework/window/destination/theme, expose arbitrary rate/duration/root/SQL input, start a producer on startup, or produce a performance pass.

- `crates/ccr-app/src/benchmark/s2.rs` plus direct tests: fixed1000 intended slots/s,60seconds/60000slots, bounded queue128, process-wide once/tombstone, retained snapshots/cancel, real Instant scheduler and cleanup. `benchmark.rs` only declares this module.
- `ccr-desktop/src/runtime/benchmark/stream.rs` plus tests: exact captured Benchmark/S2 admission, retained app-owned runtime job/finalizer, cancellation handle, actual captured monitoring ring/DB receipts, actual driver join and finalizer-owned cleanup before terminal. `runtime/mod.rs` adds one lazy adapter cell; no startup producer.
- `ccr-desktop/src/runtime/monitoring.rs` plus tests: compatible unit writer delegates to checked instrumentation, retains existing sanitization, ring500/512KiB and real single-entry SQL path. One lazy eviction scope is mutated and finally extracted/detached under the ring state lock, including ordinary pushes removing S2 IDs.
- `ccr-desktop/src/features/monitoring/benchmark.rs` plus tests: one child entity in existing Monitoring, fixed requested profile and actual ACK/rate display, independent review/Cancel/Confirm, post-mount arming and generation invalidation. `monitoring.rs` adds optional S2 entity/refresh/suspend/render only;13 destinations remain. Central `features/environments/tests.rs` stack guard adds this actual parent-Monitoring test.
- `ccr-ui/src-tauri/src/benchmark_s2.rs` plus tests: retained S2 adapter, blocking-driver IO, bounded own ticket harvesting, real final flush, scoped ring detach, actual join and immutable terminal. `state.rs` adds lazy adapter plus explicit captured persistence pool for fixture paths; normal construction keeps the Global wrapper.
- `monitoring.rs` adds a checked path through actual EventLog/persistence/app:monitoring without changing ordinary unit callers' policy. Both checked and headless storage regression share the actual append helper. Native has no equivalent per-input Tauri emit: its emit attempts remain0 rather than claiming a synthetic successful event.
- `main.rs` adds only the S2 module/active-S2 exit interception. Normal exit unchanged. Active S2 prevents ExitRequested until producer stop, drain/abandon, checked final flush, real worker join; DB shutdown cannot run ahead of its IO.
- `fixture.rs` adds early S2-only capability refusal and a separate readonly boolean bootstrap; report-clock shape/algorithm unchanged. OldFixture/S0/S1 and Local refuse all three commands before handler/owner effects. S2 commands are not in generic CAPTURED_COMMANDS.
- New `commands/benchmark.rs`, `services/benchmark.rs`, registry/runtime-policy declarations and direct tests provide exactly3 typed IPC commands. Registry-generated client, DTOs, command manifest/capabilities, inventory/docs and main ACL/schema outputs move together. `api/domains/benchmark.ts` only reexports generated client; frozen `api/tauri.ts` and tray permission subset unchanged.

Bottom persistence/ring ACK work was delegated by the coordinator to the existing worker, then independently reviewed. See [ACK implementation](performance-s2-ack-20261106.md) and [ACK check](performance-s2-ack-check-20261106.md). They own log_persistence.rs/events.rs and the reviewed minimal COMMIT-error rollback correction in log_repo. React source/tests/locales/localhost UI evidence belong to [frontend receipt](benchmark-s2-ui-20261106.md), not this worker's browser evidence. Whole-pipeline independent review: [S2 check](performance-s2-check-20261106.md).

## Scheduling, accounting and lifetime

Actual attempted entries are constructed individually. A late wake skips to the current slot; no catch-up burst or60000-entry prefill. Intended stream IDs/content/logical timestamps are shared across endpoints; actual accepted subsets may differ and are reported. Captured report as_of only timestamps data, never controls timers or elapsed.

Separate facts: scheduled/attempted/skipped; queue accepted/dropped/abandoned/pending; ring accepted/rejected/scoped evicted; DB attempted/committed/failed/pending; per-input emit attempted/failed; unacknowledged. Persistence tickets are retained at most128 and harvested exactly once by own identity. Another ordinary caller may flush the same batch; its rows are not attributed to S2, and its dropped receipt does not lose ticket outcomes. Buffer-empty alone is not a joined/inflight-complete signal. Counts acknowledge observed real stages; no unit call is inferred successful storage.

Attempted rate uses attempted/real producer-active seconds. Ring-delivered and committed rates use actual counts/total elapsed through drain, final flush and driver join. Producer elapsed freezes on source stop; Cleaning elapsed continues. Valid zero remains0; absent positive denominator yieldsNone. Input-log emit excludes benchmark:s2 status notifications.

Every active projection receives a fresh monotonically increasing revision while holding the state lock; this avoids equal-millisecond response ambiguity. Existing100ms progress cadence coalesces updates, including Cleaning heartbeat. Terminal is stored only after real driver join and final adapter cleanup; subsequent DTO/revision/elapsed/counter reads are frozen. A queued or still-pending owner cannot complete.

Stop order: request producer stop, drain/count abandoned within the existing500ms policy while concurrently consuming and awaiting producer completion, join already-started IO, checked final flush/settle all tickets, join adapter cleanup and finalize scope under pop lock, then terminal. Overbudget remains Cleaning and records failure/overrun rather than falsely detaching work. Existing pool30s, SQLite5s, native2s and exit500ms constants unchanged. No OS force-kill can be treated as a joined graceful result.

One admitted run consumes this process forever, including admission failure after the source claim. Cancel/complete cannot reset. A new trial needs stopped owners, restored original seed and a new process; modified DB/WAL cannot silently relaunch as the original seed.

## IPC and controls

- start/status/cancel IDs: `benchmark_s2_start`, `benchmark_s2_get_status`, `benchmark_s2_cancel`.
- Cancel outer payload `request`, nested `run_id`; deny unknown fields. No dynamic stream configuration.
- Start/cancel:LocalMutation+UserGesture+LocalUser+MetadataOnly+BusinessOwned. Start transfers the original ModuleExclusive registry permit until retained cleanup; status/cancel remain Parallel. Status:ReadOnly, no confirmation token.
- DTOs: BenchmarkS2SnapshotDto, BenchmarkS2PhaseDto, BenchmarkS2CancelRequest. Explicit numeric TS projection foru64 counters/revision, nullable finite actual rates/durations, static failure codes, no pool/ticket/path/credential on wire. Event benchmark:s2 carries the same snapshot.
- `window.__CCR_BENCHMARK_S2__` is true/readonly/nonconfigurable only in validated S2 WebView builder. React additionally requires actual Tauri runtime. This is presentation context; Rust independently authorizes. Local/oldFixture/S0/S1 render/request nothing.
- Exact current inventory:343 base,351 Windows,39 base modules,281 typed/exact,26 lifecycle commands. Keep strict exact assertions. Frozen facade untouched.

The initial native Start reused its focused button for confirmation. Independent review correctly rejected this: a second Enter/double activation could consume a persistent once-only trial. Current UI has a separate AlertDialog with initial safe Cancel, a post-mount armed gate, Escape/focus trap, generation-bound Confirm and suspend invalidation. `on_next_frame` only schedules mounted focus; it is never a render/present measurement.

## Verification actually run

All controlled headless tests below avoid GUI/probes/real identities/provider/network/WSL/SSH. Rust behavior tests use default parallelism with export_bindings skipped; ts-rs export is separately serial. Root/Tauri cached targets are existing external links; native uses its strict workspace-local target (packaging rejects a junction). Routine authorized execution reused the respective caches without changing HOME or ACLs.

| Owner command / evidence | Result |
| --- | --- |
| Root cargo test -p ccr-app --lib benchmark::s2 --locked --offline -- --skip export_bindings |5/5; real cfg(test)300ms short source, slow900ms sink/abandon/overbudget, deterministic once/identity, final-flush barrier, revision/terminal freeze |
| Root cargo clippy -p ccr-app -p ccr-db --all-targets --all-features --locked --offline -- -D warnings |exit0; final guarded source compiled |
| Tauri cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml benchmark_s2 --locked --offline -- --skip export_bindings |2/2; production profile actively cancelled; actual SQLite/ring/mixed unit flush and frozen scope readback |
| Native cargo test --lib runtime::benchmark::stream --locked --offline -- --skip export_bindings; final fresh artifact rerun |3/3 latest; Local/oldFixture/S0/S1 refusal before allocation and actual captured DB cancellation/join |
| Native cargo test --lib monitoring --locked --offline -- --skip export_bindings |12/12; actual parent-Monitoring confirmation, ordinary S2 victim tracking, previous monitoring/system refusals/retention |
| Fresh native artifact, child RUST_MIN_STACK=1048576, exact new parent-Monitoring confirmation test |1/1,0.07s; early Enter/early handler, safe Cancel, suspend/stale Confirm never start. Registered in central stack guard |
| Native cargo fmt -- --check; cargo clippy --lib --tests --locked --offline -- -D warnings |latest exit0; strict4.26s after confirmation correction |
| Tauri all-targets/all-features strict Clippy |exit0; existing MSVC linker_messages note is not a source warning. Subsequent production unchanged; cfg(test) inventory counts corrected |
| Fresh Tauri artifact events::tests |7/7, including final same-prefix wrong-log tracker regression |
| Fresh Tauri stream_gate_is_early_for_each_captured_mode_in_isolated_processes |parent1+4actual children passed,0.27s; oldFixture/S0/S1 reject body sentinel, S2 admits |
| Fresh Tauri s2_transferred_admission_holds_until_cleanup_but_controls_remain_available |1/1,0.03s; transferred permit retained, cancel/status do not block |
| Tauri registry filtered full suite after exact count correction |21/21,0.03s; metadata/confirmation/lifecycle/client/inventory/ACL |
| Rust ts-rs services::benchmark export tests, serial; registry inventory update; normalization |3DTO exports+inventory test passed; generated files used by real frontend |
| bun x vitest run --config vitest.smoke.config.ts tests/api/api-facade-coverage.smoke.test.ts tests/api/command-runtime-policy.smoke.test.ts |10/10; exact343/351/base343, wrapper coverage, start/cancel proof and typed request, token-free status, frozen facade |
| Scoped ESLint for changed IPC test; git diff --check |exit0 |

The pre-handoff full Tauri behavior run yielded331passed/3failed/5ignored. All3 failures were missed old exact registry/lifecycle count goldens (348→351,23→26), corrected without loosening assertions; fresh registry21 then passed. Do not relabel that first full run as green. Final complete Tauri/Root CI is coordinator-owned. Root CI log `outputs/desktop/integration-20261106-ci-s2.log` already confirms all5 latest S2 tests after the Queued complete guard, within197passed/4ignored ccr-app tests; final aggregate gate/package state belongs to the coordinator's delivery receipt.

Independent ACK owner/reviewer supplied final8/8 persistence/real SQLite Attempting-until-join/COMMIT-BUSY rollback and events7 checks. Frontend worker supplied final25/25 S2+existing confirmation/sanitize, tsc/scoped lint/i18n4564 and localhost mock UI/keyboard evidence; see linked receipts for exact attribution/limitations. Overlapping suites are not added as unique tests.

## Source freeze / remaining acceptance

Root/Tauri production inputs froze before coordinator full CI; later authorized cfg(test) exact counts were updated before its Tauri gate. Native froze after the latest safeCancel/1MiB/fmt/strict results; all target leases were returned. Last subsequent frontend IPC edits were test-only exact count/payload assertions,10/10green before its gate. Seven-part executable specs cover source, IPC, retained delivery/exit and DTOs; UI and DB specs remain their owners' contributions.

No60second throughput trial was run. Short cfg(test) and actively cancelled production-profile adapter tests are functional evidence, not a valid full S2 sample. No GUI/native pixels, first-present, memory/energy/input latency threshold or OS diversity claim. GPUI CPU attempt/on-next-frame and JS paint/main are not OS presentation completion. Performance remains not_run/not_comparable; user native acceptance is still deferred. Cross-platform WebView/storage restrictions and common seeded source/report-clock limits remain in the earlier source receipts.


Coordinator aggregate checkpoint after handoff (not owner rerun): GPUI full fmt/lint/test gates passed with233lib/4ignored, shell9, UI2, shutdown2 and central1MiB children including the new S2 review; package build is ongoing. First complete Root/Tauri CI exited1 at an existing profile replay contract expecting Applied but receiving AppliedWithWarning (`profile_commit_failure_replays_operation`, contract.rs401). Its Tauri allfeatures result333passed/1failed/5ignored has new S2 tests green. Independent checker owns warning/root-cause diagnosis; no assertion relaxation or serialized-test workaround is claimed. Frontend aggregate and any corrected CI rerun remain coordinator-owned. This checkpoint is not final full-CI/package acceptance.
