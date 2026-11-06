# Monitoring Log Contracts

> Desktop bridge, persist deadline, and frontend IPC limits.

## Scenario: Runtime channel

### 1. Scope / Trigger

- Changing `ccr-ui/src-tauri/src/bridge.rs` or `monitoring.rs` persist rules.

### 2. Contracts

- Bridged `ccr_*` warn/error become `channel=runtime`.
- `runtime.error` persists. `runtime.warn` is live-only.
- `frontend.warn` / `frontend.error` still persist.
- `record_monitoring_entry` does not `force_flush` per event.
- Emit/flush failures increment an internal counter. They must not emit bridgeable `ccr_*` tracing.

### 3. Tests Required

- `runtime_entry_maps_error_and_warn`
- `should_persist_warn_and_whitelisted_events`
- `reentrant_consumer_does_not_enqueue`

## Scenario: Flush deadline

- Threshold 20 or 2s ticker, whichever first.
- Exit: `close_bridged_log_sender`, then `force_flush` with 500ms timeout, then `database::shutdown`.

## Scenario: Frontend IPC

- `sanitize_frontend_log` is the server boundary. Batch 32. See parent design IPC table.
- Tests: `ccr-ui/src-tauri/src/log_sanitize.rs`, `ccr-ui/tests/shell/logger.smoke.test.ts`.

## Scenario: Startup window handlers

### 1. Scope / Trigger

- Changing `ccr-ui/src/utils/startupRecovery.ts` or `ccr-ui/src/main.tsx` mount wiring.

### 2. Contracts

- `installStartupErrorHandlers()` returns a disposer. `main.tsx` must keep it and call it from the first React shell mount `useEffect`.
- Before uninstall: `window` `error` / `unhandledrejection` may call `reportStartupFailure` (`[startup] …` + `renderFatalStartup`).
- After uninstall: those events must not replace `#app` and must not log `[startup] Unhandled window error`.
- `frontend.error` still persists into Event stream. Do not delete user `log_entries` to hide a historical startup error.

### 3. Tests Required

- `ccr-ui/tests/shell/startup-recovery.smoke.test.ts`: pre-uninstall fatal path; post-uninstall does not replace `#app`; `main.tsx` keeps and invokes the disposer.

## Scenario: S2 frontend consumer

### 1. Scope / Trigger

- Applies to `features/monitoring/BenchmarkS2Panel.tsx` and its minimal `MonitoringView` mount. Generated benchmark clients/DTOs remain registry-owned; import the three actions from `api/domains/benchmark`, never add a direct facade invoke.

### 2. Signatures

- Generated `startBenchmarkS2()`, `getBenchmarkS2Status()` and
  `cancelBenchmarkS2({ run_id })` return `BenchmarkS2SnapshotDto`.
- `benchmark:s2` carries the same DTO; active `revision` orders projections,
  while `run_id` identifies the retained process-owned run.

### 3. Contracts

- Render and subscribe only when `isTauriRuntime()` and `window.__CCR_BENCHMARK_S2__ === true`. Web, Local, old Fixture, S0 and S1 expose no S2 controls and issue no S2 requests. The frontend marker is not authorization; Rust independently checks the captured source and S2 scenario.
- On mount/reconnect, await registration of `benchmark:s2` before querying status. Detach listeners on leave, including registration that resolves after leave. Leaving the view does not cancel the producer or reset its process-owned state.
- Accept only a newer revision and the same established `run_id`. Active projections own new monotonic revisions; terminal snapshots are immutable. Successful same-revision recovery (including `not_started` revision0) clears a connection/refusal error separately from payload adoption. Recovery and action errors/results use their captured action generation so an old request cannot clear or overwrite a newer action error.
- Start is available only for the authoritative `not_started` state. Reuse `surfaceNotify.confirm` / the global ConfirmModal with warning semantics and an explicit fixture-write/one-process-once explanation. No invoke before confirmation. The fixed profile is requested1000 events/s for60s, queue128; there are no editable rate/duration controls or terminal Reset.
- Initial confirmation focus is the safe Cancel button; early Enter must not start. The imperative global dialog has no Radix `DialogTrigger`. This consumer restores its own Start focus after cancellation only on the next animation frame, while mounted, with the same opening/action generation and connected original trigger, and only if focus is still on body and no other dialog exists. A later dialog, user focus move or leave must not be overridden.
- Cancel sends the current `run_id` and requests stopping only. Do not invent a cancelled/completed phase while the owner is still Cleaning. `cleanup_over_budget` is a historical fact that may survive terminal state; its warning must not claim completion remains pending after the owner publishes terminal. Another trial requires closing the process and restoring seed data.
- Show requested configuration separately from actual attempted/ring-delivered/database-committed rates and scheduler/queue/ring/database/log-emit failure counters. Display DTO rates without frontend recomputation: attempted uses active source time; delivered/committed use total time including drain, flush and cleanup. Log emit counters exclude S2 status notifications. Null is unavailable; valid measured zero remains zero. Stable failure codes map to bounded friendly copy; raw errors/paths are not rendered.
- A localhost component harness with synthetic DTOs and mocked IPC is UI/keyboard evidence only. Label it visibly; it is neither native loading nor a real S2 performance result.

### 4. Validation & Error Matrix

- Runtime/marker absent -> no control, listener or request.
- Duplicate/older revision or foreign run -> no snapshot adoption.
- Successful same-revision recovery -> clear only the current action's error;
  older request success/failure cannot clear/replace a newer refusal.
- Cancel accepted -> show request state; retain the owner's phase until join.

### 5. Good / Base / Bad Cases

- Good: reconnect resolves revision0 and re-enables a refused NotStarted trial
  without replacing identical counters.
- Base: leave detaches UI subscriptions while the retained owner continues.
- Bad: infer Completed from an empty queue or claim pending solely from a
  historical `cleanup_over_budget` flag on a terminal snapshot.

### 6. Tests Required

- `tests/monitoring/benchmark-s2.smoke.test.tsx`: source refusal, subscribe-before-query, stale revision/run rejection, same-revision successful recovery, old successful/failed recovery versus a newer action, actual/missing/zero metrics, safe confirmation/cancel/focus, leave/remount, Cleaning/terminal facts and locale switching.
- `tests/ui/confirm-interaction.smoke.test.tsx` and `tests/quality/monitoring-log-sanitize.smoke.test.tsx` retain existing confirmation and log-safety coverage.
- Run frontend type-check, scoped no-fix ESLint, focused smoke tests and strict i18n leaf parity/count checks. Browser DOM/keyboard evidence and screenshot capture failures are reported separately; neither substitutes for native/performance acceptance.

### 7. Wrong vs Correct

```ts
// Wrong: duplicate revision also suppresses successful transport recovery.
if (incoming.revision <= previous.revision) return
setRequestFailed(false)
```

```ts
// Correct: data adoption and current-request recovery have separate gates.
accept(incoming)
if (currentGeneration && sameRun) setRequestFailed(false)
```


## Scenario: S2 retained backend delivery and exit

### 1. Scope / Trigger

Tauri `benchmark_s2.rs`, checked monitoring bridge, S2-only registry capabilities and shutdown exception. Normal Local batching/exit stays unchanged.

### 2. Signatures

- `benchmark_s2_start()`, `benchmark_s2_get_status()`, `benchmark_s2_cancel(request:{run_id:string})` return generated `BenchmarkS2SnapshotDto`.
- `record_monitoring_entry_checked(app, entry, persist) -> CheckedMonitoringReceipt {ring,ticket,emitted}` follows actual EventLog, captured pool and app:monitoring.
- `EventLog::{register_eviction_tracker, finish_eviction_tracker}` bracket the observed run under the real pop lock; final consumes the handle. `Adapter::joined()` awaits its retained worker.

### 3. Contracts

- Rust rejects all three IPCs before handler/allocation for Local, oldFixture,S0/S1; never add them to generic CAPTURED_COMMANDS. Readonly S2 bootstrap marker is UI context, not authorization.
- Start/cancel require UserGesture, status ReadOnly. Start transfers the original ModuleExclusive permit through driver join/final checked flush/detach; cancel/status are Parallel. Registry generates exact DTO client, inventory and main ACL. Frozen facade/tray stay unchanged.
- Ticket terminal state survives any ordinary/checked mixed flush and is harvested once for S2 only. Retain at most128 pending tickets; remaining_buffered=0 excludes another flusher's inflight batch and cannot establish completion. Input-log emit ACKs exclude benchmark:s2 status events.
- Native ring remains500/512KiB; Tauri500/5MiB and10KiB per-event. Scope counts removed S2 IDs even on ordinary push, never ordinary victims. Finish/detach freezes this observation window atomically with pop.
- Active S2 intercepts ExitRequested, stops producer and joins before DB shutdown. Over500ms staysCleaning, records overrun and awaits real IO without raising timeout constants. Forced OS process termination is not a joined shutdown result.

### 4. Validation & Error Matrix

| Boundary | Required outcome |
| --- | --- |
| Non-S2/missing marker source | Rust early refusal; no handler effect |
| Missing/forged gesture token | Registry audit rejection before dispatch |
| Wrong run_id | Refusal, current owner unchanged |
| Pending mixed flush | Cleaning until each owned ticket terminal and driver joined |
| DB failure/emit failure | Separate actual failed counts; no unit-call success inference |
| Cleanup overrun or panic | No premature terminal/DB shutdown |

### 5. Good / Base / Bad Cases

Good: cancel reaches retained owner while start permit is still held, mixed ordinary flush settles only owned tickets, final detach precedes terminal. Base: ordinary record_monitoring_entry retains unit wrapper and old persistence policy. Bad: reset after cancel, count whole receipt batch, or shutdown after merely timing out the join future.

### 6. Tests Required

- `stream_gate_is_early_for_each_captured_mode_in_isolated_processes`: oldFixture,S0,S1 refuse; S2 admits; body sentinel proves early boundary.
- `s2_transferred_admission_holds_until_cleanup_but_controls_remain_available`: second start cannot take module permit while status/cancel reach the live owner.
- `production_profile_cancel_joins_real_batched_storage_and_scoped_evictions`: actual SQLite own-ID readback, ordinary mixed flush, no pending stage and post-terminal eviction freeze.
- events seven direct regressions cover real capacity/size policies, lazy scope, unit victims, same-prefix wrong-log rejection, Drop/cancel tombstone and atomic final detach.
- Native `s2_review_starts_with_safe_cancel_rejects_early_enter_and_dies_on_suspend` runs in actual Windows1MiB test child and inside parent Monitoring render. GUI/performance sampling remains separate.

### 7. Wrong vs Correct

Wrong: emit benchmark:s2 and increment input emit_attempted, or sum all rows from a mixed flush receipt. Correct: count only actual app:monitoring input outcomes and own terminal tickets; status notifications never recursively count as workload.
