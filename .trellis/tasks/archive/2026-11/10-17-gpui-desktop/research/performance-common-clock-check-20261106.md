# Common Seed / ReportClock independent check — 2026-11-06

Scope: shared Seed admission and both explicit Benchmark startup adapters;
ReportClock defaults through GPUI/Tauri, shared usage/Observer/SQL owners, and
frontend context/preset consumers. Review began at HEAD `92b3e9e9`; pending work
from other owners was preserved. No commit, native window/CUA, real account,
SSH/PID/port probe or performance sample was used.

## Findings fixed

1. **Committed WAL escaped main-file integrity.** Seed originally hashed only
   the four main DB files. A real `ccr_db::UiStateManager` write in an owned
   fixture committed a favorite to WAL, left the main digest unchanged and was
   accepted by the old Seed. The regression actually failed on that acceptance.
   Shared Seed now checks `-wal/-journal/-shm` before admitting every DB role,
   including S0's absent catalog. WAL/journal must be empty; SHM may contain bytes
   but all sidecars must be regular, unlinked and metadata-readable. Missing is
   allowed, other errors fail closed. Admission never opens SQLite, recovers or
   checkpoints input. The green test verifies unchanged main/WAL bytes and
   exercises empty/nonempty/nonfile cases for all four roles.
2. **Fixed tool reports included future fractional timestamps.** `datetime(ts)`
   discarded fractions, so an event 1ms after as_of was counted. Actual red was
   3 rows versus expected 2. Fixed predicates now use the existing numeric
   `julianday` comparison (SQLite-supported millisecond boundary), retaining
   exact-as_of and rejecting +1ms. All three tool projections and the unbounded
   fixed upper-bound case are covered; legacy live predicates remain unchanged.
3. **Rust/frontend clock schema disagreed on leap seconds.** Chrono accepted
   `2016-12-31T23:59:60Z`, which the frontend Date utility rejects. A pure Seed
   test reproduced startup admission of that unsupported clock. Shared Seed
   now rejects leap-second nanoseconds before effect owners. The same test
   verifies both endpoint variants and non-UTC refusal. Chrono's strict RFC3339
   parser already enforces a four-digit year; no redundant year rule was added.
4. The new native default-report test used nonexistent `WorkbenchDay.date`.
   It now checks the real `label` (`11-06`) alongside the full report as_of.

Existing shared-desktop/fixture-isolation specs were updated with these contracts
and the preserved heatmap/window qualification. These are local guard/SQL/test
fixes; no public signatures, IPC shape or report/filter models were expanded.

## Call-path audit

- Native Layout and Tauri `load_benchmark` each call the same
  `ccr_app::benchmark::Seed::load` exactly once. Endpoint selection changes only
  member names/identity. The shared policy checks absolute/traversal-free and
  canonical roots, ancestor/descendant reparse safety, required directories,
  bounded marker/manifest JSON, scenario/count/synthetic/version, both member
  identities/logical digests, UTC clock, sidecars and selected-member DB hashes.
  It streams each main file once. Native's remaining hash helpers are cfg(test)
  only; Tauri does not call the legacy fixture validator after Seed. No duplicate
  production hash pass or implicit legacy Fixture-to-Benchmark selection found.
- Native performs validation on its existing bounded startup worker; Tauri does
  it before logger/context/automatic WebView creation. Errors do not fall back to
  Local. Captured home/main/archive/usage/log/preferences/crypto/lock roots remain
  explicit. Neither validator rewrites original IDs, raw paths or launch strings.
- Services returns a fresh live Local clock outside Benchmark and immutable
  manifest UTC in Benchmark. GPUI default workbench/Home/Observer and frontend
  Home cost, platform usage, presets, store reset and status clock consume it.
  Explicit Home as_of/generated_at, ordinary query dates/timezone and manual
  store ranges are retained; the global Date/timers and elapsed clocks are real.
- Heatmap retains its **pre-existing independent rolling-days** contract. Clock
  replaces its default today anchor; timezone and other dimensions remain caller
  owned. It is not claimed to obey another panel's manual since/until. The
  coordinator explicitly chose compatibility rather than a product redesign.
- `ccr-usage` owns overview/heatmap SQL; `ccr-db` owns tools and archive SQL.
  Legacy wrappers retain live behavior. Explicit UTC archive queries use numeric
  instant bounds, inclusive start/exclusive next-day end, and UTC `date()` buckets;
  offset and end-day regressions cover this. Observer parameters are supplied at
  its shared owner instead of copying SQL into either UI adapter.
- Tauri installs only serialized `{asOfUtc, timezone}` via `Object.defineProperty`
  with nonwritable/nonconfigurable and frozen value, through the shared main/tray/
  rebuild WebView builder. There is no source path/secret/script payload in the
  clock JSON. Utility validates UTC/canonical calendar and throws on invalid
  context; it does not silently use host time. Real default consumers are wired,
  not merely a standalone utility test. Legacy fixture does not inject context.

## Verification actually run

Pinned Rust1.98, locked/offline Cargo, default parallelism, skip export_bindings.
All writes were to new owned test directories; the existing paired output was
never regenerated, repaired or checkpointed.

- Actual red → green: committed-WAL admission, +1ms tool upper bound, and
  unsupported leap-second admission, as detailed above.
- `ccr-app clock`: **4/4 passed** after the final guards (shared Seed, ReportClock,
  explicit-filter report assembly, Observer defaults).
- `ccr-db claude_tool_calls_repo`: **5/5 passed**; explicit UTC archive day-bound
  regression **1/1 passed** on the fresh Root artifact.
- Explicit ignored shared admission over existing S0/S1/S2 six roots: **1/1**,
  9.89s, all original logical digests matched. This is filesystem admission
  evidence; its duration is not a startup/present sample.
- Native fresh benchmark module: **7 passed / 0 failed / 3 ignored**, plus the
  actually spawned poison child **1 passed**. Includes default workbench UTC,
  explicit Home dates and old Fixture live Local clock. This run includes the
  WAL/SQL fixes; final leap rejection is separately covered through both shared
  endpoint variants and the final native strict compilation.
- Explicit native actual-pair default workbench/Observer: **1/1**, 6.60s. All use
  2026-10-31..2026-11-06 UTC; requests **0 / 65469 / 65469**, indexed sessions
  **None / 10000 / 10000** for S0/S1/S2. Golden requests come from each generated
  logical trend manifest. No 50-page full-session traversal or old 161s workload
  was repeated. None for missing S0 archive state is not a fake measured count.
- Frontend report-clock smoke: **4/4**; `tsc --noEmit` and scoped ESLint **exit 0**.
- Root strict Clippy `-p ccr-app -p ccr-db -p ccr-usage --all-targets
  --all-features -- -D warnings`: **exit 0**. Native `--lib --tests -- -D warnings`:
  **exit 0** (10.03s), including the sha2 production-to-dev dependency move.
- Scoped rustfmt and diff-check passed. Transient peer crypto-guard missing-file/
  visibility errors initially blocked native compilation and were fixed by that
  owner; they were not hidden, suppressed or counted as successful tests.
- Tauri owner supplied usage services **30/30**, frontend usage **90/90**, tsc and
  scoped lint receipts. Its final actual-pair clock/Home/Observer after these
  shared fixes passed **1/1 + 3/3 actual children** (6.34s), including 7-day golden
  requests and sessions 0/10000/10000, manifest UTC as_of/generated_at. Its ensuing
  Tauri `all-targets/all-features -D warnings` Clippy passed **exit 0** (37.79s).
  These are explicitly the owner's executions, not this reviewer's. The existing
  MSVC proc-macro linker lint stdout is not a suppressed source warning.

## Findings not fixed / acceptance boundary

No further source defect found in the reviewed default-clock/admission scope.
Matching the validation algorithm does not establish equal process costs or full
feature parity. Current tests prove domain loading/default calendar behavior;
they do not prove live WebView storage, native UI/loading or other-OS behavior.
S2 dynamic stream remains not_emitted. No CPU duration, JS paint, main return or
on_next_frame callback is claimed as OS first-present. Performance status remains
`not_run/not_comparable`, with complete benchmark/native acceptance owned by the
coordinator.
