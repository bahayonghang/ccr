# Profile Workbench

## 1. Scope / Trigger

Changes to Codex, Claude or Grok Profile loading, mutations, table geometry, runtime strips, last-use data or the shared A detail panel must preserve this contract. Auth account workflows keep their existing owners. `profile_tasks.rs` schedules work; `app.rs` consumes typed outcomes; `ui/workbench.rs` renders cached snapshots.

## 2. Signatures

- `ProfileTasks::{request_read, start_mutation, drain, ready_to_exit}`.
- `MutationRequest::{Apply(ApplyProfileRequest), Off(Platform), Edit(EditRequest)}`.
- `ProfileEditor` collects a draft. `EditorEvent::{Save, Cancel, None}` is the only signal `app.rs` routes. `e` opens it; `Ctrl+S` then `y` schedules `Edit`.
- `TuiApp::{poll_background, has_pending_work, ready_to_exit}` supplement `on_tick`.
- `EventHandler::{poll_event, take_tick_due}` separates completion polling from periodic timers.
- `TableGeometry` owns header, rule, body and columns; `App::list_area` stores the body Rect, not the outer border.
- Activity DTOs come through the [read-only activity facade](../../ccr-store/backend/profile-activity.md); usage comes through the existing `UsageApp`.

## 3. Contracts

- Construct the configured tab shell without synchronously loading every platform. Start the active Profile read first; initialize other Profile tabs on first visit. Selection/rendering use cached data only. Explicit reload and completed mutations refresh the affected platform.
- Permit one mutation at a time, with the original platform/name/operation ID captured before scheduling. Execute the entire existing synchronous apply/off lifecycle in a blocking worker. Preserve committed warnings, recovery outcomes, backup, CAS and the thread-local journal.
- Profile Apply (`Enter` / `Space`) and Profile Off (`o`) open a default-cancel confirmation before any writer starts. Schedule `MutationRequest` only after `y`. Freeze the Apply platform and profile name when the dialog opens; `y` must not read the cursor again. While that confirmation is open, repeated keys must not start another writer. Account switch and official logout stay with the Auth apps.
- Profile edit (`e`) opens only on a Profile tab with a selection, a known catalog revision, no in-flight mutation, and no open confirmation. Auth tabs ignore `e`. The editor records the revision from the snapshot (`ConfigManager` on the read worker, stored on the tab). `Ctrl+S` validates locally, then opens a default-cancel save confirmation that lists changed field names. Secrets say replaced or will-clear and never include the value. `y` schedules `MutationRequest::Edit` with `ApplyPolicy::IfCurrent` and the revision captured when the editor opened; it does not read the cursor again. Model fields cycle with `←→` through the platform catalog. Codex starts the list at `gpt-6.1-sol` (shown as 6.1 Sol · default) and includes `gpt-6-astra`; Claude starts at `claude-opus-5-5`. The recommended id is not written until the user selects it. Typing, pasting, or Backspace switches that field to a custom id and does not rewrite an untouched current value. While the editor is open, `q`, `h/j/k/l`, `o`, `r`, and `x` are ordinary characters, Tab and header clicks do not change platform tabs, and `Ctrl+C` cancels like Esc instead of quitting. A dirty Esc arms a second Esc to discard. The draft survives resize and background reads. A version conflict keeps the editor and draft open. Grok `RenameApplyFailed` and `RenameCleanupFailed` are warnings, not success, and still refresh. After a successful edit the platform reloads and the selection follows `new_name`. Wide keeps the dimmed list and puts the form plus change preview in the detail rail. Standard and Compact replace the area between header and footer. Bracketed paste is enabled with raw mode and disabled on every restore path. Paste inserts only while the editor is open, with CR/LF stripped.
- Read generations may discard obsolete snapshots; mutation outcomes are always consumed. Repeated action keys must not create another write. Profile browsing remains available while writing; entering Auth tabs is gated during a Profile mutation.
- q, Esc and Ctrl+C defer exit until the mutation completes. `ready_to_exit` is separate from the redraw return value of `on_tick`. Drain completion channels after every event, including Ctrl+T, resize and ignored mouse motion. Pending work caps event polling at 50 ms; normal timer cadence remains 250 ms even under continuous input. Idle ticks do not force redraws.
- Refresh initialized Auth runtime summaries from the new snapshot without resetting the selected account. A failed runtime refresh cannot leave a cached marker claiming a known effective activation. Grok uses its typed activation inspection, preserving drift and unsafe states.
- The table spends one row on headers and one on a horizontal rule. Pagination and mouse hit-testing consume exactly its body height/origin. Drop optional description/model columns before status/name/last-used; measure CJK and long strings in terminal cells.
- Header compactness follows the measured total tab width, including padding/dividers. Compact Auth labels retain their platform names; render and hit-testing share the same spacing so all six tabs remain identifiable and clickable at supported narrow sizes.
- Wide layout retains the 46/54 split. The detail rail fills the rect assigned to it instead of shrinking to its content. In the wide side-by-side layout its bottom border shares the list's bottom row. Fixed sections are identity, connection, provider usage, then the latest three recorded successful activations. Within one platform and viewport, identity, connection, and usage title rows stay put when the selection changes: connection height is reserved from the platform field set, and an unset field leaves a blank slot instead of pulling the next section up. Empty usage and activation sections are a single ruler line, title on the left and status on the right. Their scope note is rendered once, and only when that section has records. Populated usage still grows, so the activation title that follows it can move. Connection fields use two columns at an inner width of at least 64 and one column below that. Endpoint, key, and values that do not fit a column take a full row. Wrapping and semantic colors stay on the value. The remaining inner height is a stored-field inspector. Its title includes the `profiles.toml` path captured with the snapshot; rendering does not resolve paths. Core fields come first and `platform_data` follows insertion order. `auth_token` and `api_key` render as `‹已设置›` or are omitted; Grok renders no token field, masked or otherwise. `env_key` shows only the variable name. The inspector scrolls independently while the fixed sections fit, using the existing detail scroll offset. If the fixed sections plus one inspector row do not fit, the whole pane scrolls and keeps the scrollbar and `x–y / n` indicator. Expanded details still show unset fields and may enter that whole-pane scroll. The profile runtime strip is 3 rows in every viewport. Compact keeps the unlabeled value line. Standard and Wide put mode, current profile/model, and auth on one labeled line. A table cell with no activation record shows a muted `—`; loading, unavailable, invalid, and future times keep their existing text and color. The footer labels the expand key `x all fields` and shows `e edit` on profile tabs. On a footer narrower than 90 columns, `e` is dropped before `x` and `o`.
- Last used is a recorded successful CCR activation, not the last API request. Unknown, missing, malformed and future records retain distinct presentation. Relative-time labels schedule redraw at their next boundary instead of invalidating every frame. The activation section still says "no record" in words; only the table cell uses `—`.
- Usage totals remain provider/source scoped and all-time, with approximate official-equivalent cost clearly labeled. Bars compare independent category counts with a common maximum and partial-cell resolution; zero and unknown remain different. Narrow panels retain numeric values and scope. No per-selection SQL or new analytics engine.
- Use TUI-local Codex/Claude/Grok names in portable terminal text; the English Claude tab is `Claude Profile`. Do not change global `Platform::icon` consumers or use generic emoji as product logos. Existing masking and safe URL helpers remain mandatory; Grok renders no inline token, even masked.

## 4. Validation & Error Matrix

| Event/state | Required behavior |
| --- | --- |
| Slow initial read | Shell stays responsive; active tab shows loading |
| Refresh races a mutation | Discard old read; consume mutation; refresh affected platform |
| Reload fails with existing list | Keep browsing data and selection; show failure without claiming fresh current state |
| Repeated Apply/Off | One writer and one operation identity |
| Apply/Off key | Confirmation opens first; no writer until `y`; Apply keeps the name captured at open |
| Quit while writing | Wait; consume final outcome; exit without another key |
| Input continues without idle ticks | Completion and elapsed-time timers still advance |
| Missing history / usage | Explicit unavailable or empty state, never fabricated zero timestamps |
| Short/narrow view | Reach final wrapped line; retain primary footer actions |
| Edit key | Open only with a selection, revision, and no writer or confirmation; otherwise toast or ignore |
| Version conflict | Keep the editor and draft; say the file changed and to refresh with `r` |
| Secret edit | Dots while typing; confirm, toast, and `Debug` omit the sentinel. Grok still renders no token |
| Paste | Strip CR/LF into the focused field only while the editor is open; disable bracketed paste on restore |

## 5. Good / Base / Bad Cases

- Good: selecting a cached Grok profile only changes in-memory details; a delayed Apply preserves that original target even after switching Profile tabs.
- Good: at 160×45 the detail border ends on the list's bottom row for both an empty-usage profile and a profile with usage, and the connection title stays on the same row.
- Base: missing local history shows no record and leaves storage untouched. The table cell is a muted dash, and the empty usage and activation sections are each one ruler line.
- Bad: calling synchronous profile loading in `render`, moving only half the journal to a worker, treating a redraw boolean as exit, or subtracting a border offset twice during mouse hit-testing.
- Bad: cropping the detail panel to its content height, resolving `profiles.toml` during render, or painting a Grok token as text, a mask, or `‹已设置›`.

## 6. Tests Required

- Inject Profile loaders/writers and usage loaders; fixtures must not read or mutate personal home data.
- Cover active-first reads, no navigation I/O, stale results, duplicate actions, frozen mutation targets, all quit keys, completion under continuous input, timer cadence and initialized Auth selection/runtime preservation.
- Retain actual adapter lifecycle/replay/compensation and secret-sentinel tests.
- Assert TestBackend cells, semantic colors, header/body geometry, CJK alignment and wrapped scroll end across platforms, EN/ZH, compact/standard/wide and missing/error/long/zero states.
- Assert the full-height rail, stable connection and usage titles, one-line empty sections, two-column connection layout, the 3-row runtime strip, the muted no-record dash, stored-field redaction with independent scroll, and the wide `e edit` hint that narrow footers drop before `x` and `o`.
- Assert editor routing, frozen edit requests, secret redaction, version-conflict drafts, and Wide/Standard/Compact editor layout.
- Run tests with default parallelism and `--skip export_bindings`. The palette is process-global: export Mocha/Latte fixtures in separate processes instead of toggling themes inside parallel tests.
- Benchmarks report fixture size, build mode, sample count and p50/p95 separately for UI acknowledgement and work completion. Do not turn observational timing into flaky CI thresholds. TestBackend images prove buffer composition, not native-terminal font rendering or private runtime correctness.

## 7. Wrong vs Correct

Wrong: synchronously load every platform on startup, invoke the lifecycle from the Apply or Off key before confirmation, and check its channel only on idle ticks.

Correct: start a bounded read for the active tab, schedule a complete captured mutation, pump results after every event, and render typed cached state with an independent deferred-exit check.
