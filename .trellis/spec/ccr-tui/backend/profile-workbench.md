# Profile Workbench

## 1. Scope / Trigger

Changes to Codex, Claude or Grok Profile loading, mutations, table geometry, runtime strips, last-use data or the shared A detail panel must preserve this contract. Auth account workflows keep their existing owners. `profile_tasks.rs` schedules work; `app.rs` consumes typed outcomes; `ui/workbench.rs` renders cached snapshots.

## 2. Signatures

- `ProfileTasks::{request_read, start_mutation, drain, ready_to_exit}`.
- `MutationRequest::{Apply(ApplyProfileRequest), Off(Platform)}`.
- `TuiApp::{poll_background, has_pending_work, ready_to_exit}` supplement `on_tick`.
- `EventHandler::{poll_event, take_tick_due}` separates completion polling from periodic timers.
- `TableGeometry` owns header, rule, body and columns; `App::list_area` stores the body Rect, not the outer border.
- Activity DTOs come through the [read-only activity facade](../../ccr-store/backend/profile-activity.md); usage comes through the existing `UsageApp`.

## 3. Contracts

- Construct the configured tab shell without synchronously loading every platform. Start the active Profile read first; initialize other Profile tabs on first visit. Selection/rendering use cached data only. Explicit reload and completed mutations refresh the affected platform.
- Permit one mutation at a time, with the original platform/name/operation ID captured before scheduling. Execute the entire existing synchronous apply/off lifecycle in a blocking worker. Preserve committed warnings, recovery outcomes, backup, CAS and the thread-local journal.
- Read generations may discard obsolete snapshots; mutation outcomes are always consumed. Repeated action keys must not create another write. Profile browsing remains available while writing; entering Auth tabs is gated during a Profile mutation.
- q, Esc and Ctrl+C defer exit until the mutation completes. `ready_to_exit` is separate from the redraw return value of `on_tick`. Drain completion channels after every event, including Ctrl+T, resize and ignored mouse motion. Pending work caps event polling at 50 ms; normal timer cadence remains 250 ms even under continuous input. Idle ticks do not force redraws.
- Refresh initialized Auth runtime summaries from the new snapshot without resetting the selected account. A failed runtime refresh cannot leave a cached marker claiming a known effective activation. Grok uses its typed activation inspection, preserving drift and unsafe states.
- The table spends one row on headers and one on a horizontal rule. Pagination and mouse hit-testing consume exactly its body height/origin. Drop optional description/model columns before status/name/last-used; measure CJK and long strings in terminal cells.
- Header compactness follows the measured total tab width, including padding/dividers. Compact Auth labels retain their platform names; render and hit-testing share the same spacing so all six tabs remain identifiable and clickable at supported narrow sizes.
- Wide layout retains the 46/54 split. The A detail rail follows identity/status, compact connection fields, provider totals/category magnitudes, then the latest three recorded successful activations. It fits its content and scrolls using wrapped height. Normal healthy views omit unset optional rows; expanded details retain them.
- Last used is a recorded successful CCR activation, not the last API request. Unknown, missing, malformed and future records retain distinct presentation. Relative-time labels schedule redraw at their next boundary instead of invalidating every frame.
- Usage totals remain provider/source scoped and all-time, with approximate official-equivalent cost clearly labeled. Bars compare independent category counts with a common maximum and partial-cell resolution; zero and unknown remain different. Narrow panels retain numeric values and scope. No per-selection SQL or new analytics engine.
- Use TUI-local Codex/Claude/Grok names in portable terminal text; the English Claude tab is `Claude Profile`. Do not change global `Platform::icon` consumers or use generic emoji as product logos. Existing masking and safe URL helpers remain mandatory; Grok renders no inline token, even masked.

## 4. Validation & Error Matrix

| Event/state | Required behavior |
| --- | --- |
| Slow initial read | Shell stays responsive; active tab shows loading |
| Refresh races a mutation | Discard old read; consume mutation; refresh affected platform |
| Reload fails with existing list | Keep browsing data and selection; show failure without claiming fresh current state |
| Repeated Apply/Off | One writer and one operation identity |
| Quit while writing | Wait; consume final outcome; exit without another key |
| Input continues without idle ticks | Completion and elapsed-time timers still advance |
| Missing history / usage | Explicit unavailable or empty state, never fabricated zero timestamps |
| Short/narrow view | Reach final wrapped line; retain primary footer actions |

## 5. Good / Base / Bad Cases

- Good: selecting a cached Grok profile only changes in-memory details; a delayed Apply preserves that original target even after switching Profile tabs.
- Base: missing local history shows no record and leaves storage untouched.
- Bad: calling synchronous profile loading in `render`, moving only half the journal to a worker, treating a redraw boolean as exit, or subtracting a border offset twice during mouse hit-testing.

## 6. Tests Required

- Inject Profile loaders/writers and usage loaders; fixtures must not read or mutate personal home data.
- Cover active-first reads, no navigation I/O, stale results, duplicate actions, frozen mutation targets, all quit keys, completion under continuous input, timer cadence and initialized Auth selection/runtime preservation.
- Retain actual adapter lifecycle/replay/compensation and secret-sentinel tests.
- Assert TestBackend cells, semantic colors, header/body geometry, CJK alignment and wrapped scroll end across platforms, EN/ZH, compact/standard/wide and missing/error/long/zero states.
- Run tests with default parallelism and `--skip export_bindings`. The palette is process-global: export Mocha/Latte fixtures in separate processes instead of toggling themes inside parallel tests.
- Benchmarks report fixture size, build mode, sample count and p50/p95 separately for UI acknowledgement and work completion. Do not turn observational timing into flaky CI thresholds. TestBackend images prove buffer composition, not native-terminal font rendering or private runtime correctness.

## 7. Wrong vs Correct

Wrong: synchronously load every platform on startup, invoke the lifecycle from the key handler, and check its channel only on idle ticks.

Correct: start a bounded read for the active tab, schedule a complete captured mutation, pump results after every event, and render typed cached state with an independent deferred-exit check.
