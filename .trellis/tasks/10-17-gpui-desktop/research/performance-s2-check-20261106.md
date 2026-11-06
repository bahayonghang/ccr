# S2 cross-layer independent check — 2026-11-06

Reviewed shared core, DB ACK integration, native/Tauri adapters, IPC/DTO and both
control surfaces. The ongoing task/worktree was reused; no worker changes were
reverted and no commit was made by this reviewer. Compilation leases stayed with
the implementers/coordinator. Bottom ACK details and the independently fixed
COMMIT-BUSY defect remain in `performance-s2-ack-check-20261106.md`.

## Review findings and corrections

- **Native confirmation initially violated the existing early-Enter contract.**
  One focused Start button changed its label/boolean and a second activation could
  launch the persistent one-use trial. This was reported to the coordinator and
  fixed by the native owner: separate review/Cancel/Confirm controls, post-mount
  arming, safe Cancel first focus, focus trap/Escape, generation-bound handlers
  and suspend invalidation. The current source hides the original Start during
  review, refuses early/stale Confirm and restores the trigger on dismissal.
  Suspend clears review/subscription and invalidates pending focus callbacks;
  route focus remains Shell-owned. Reentry explicitly refreshes retained state
  and subscription, without starting or cancelling the process-owned stream.
- **React same-revision recovery could strand Start disabled.** After a failed
  Start RPC with cached NotStarted revision 0, successful reconnect returned the
  same revision, `accept` rejected it and the transport-error flag never cleared.
  The frontend owner reproduced red and corrected successful recovery separately
  from revision adoption. The response must still belong to the current action
  generation/run; stale replies cannot replace newer data or clear newer errors.
  That owner also reproduced/fixed an older recovery failure overwriting a newer
  cancellation result. The final source retains `<= revision`/foreign-run data
  rejection and protects start/cancel errors/results with action generations.
- Shared `complete_joined` was strengthened during implementation: only Cleaning
  with no tracked producers/heartbeat and no pending DB tickets can transition;
  an existing terminal snapshot returns unchanged. This prevents premature
  completion from Queued, without weakening true-join requirements.

No further Root/Tauri source blocker was found after these corrections. This
review did not modify the producer/adapter/UI source in parallel with its owners.

## Source and ownership conclusions

**Shared core.** Production is fixed at 1,000 scheduled slots/s, 60,000 slots and
60 seconds, queue capacity 128. Pacing uses real Instant, skips elapsed slots
instead of catch-up bursts, and uses nonblocking bounded admission. Report UTC
only timestamps deterministic synthetic data. Requested, scheduled, attempted,
skipped, dropped, abandoned, pending and actual rates remain distinct. Source
rate uses producer duration; delivered/committed rates include drain, final I/O
and join. Cleanup over 500ms remains visible and fails the trial rather than
pretending work ended at the budget.

Admission is S2/id-specific and the static process tombstone is never reset.
Cancellation requires the retained run ID; source/adapters refuse Local, old
Fixture, smaller scenarios and remote starts before stream owners/effects.
Updates and snapshots allocate ordered revisions under one mutex; terminal
state freezes counters, rates, elapsed values and revision. Run/worker recovery
requests cancellation and actually awaits tracked tasks before finalization;
dropped/unstarted work does not invent a successful source run.

**Native adapter.** The bounded app executor retains the job and its finalizer
after the view detaches. Actual sink ring admission and synchronous captured
SQLite results become ACKs; no returned unit is treated as DB persistence.
Eviction deltas are harvested/detached under the monitoring lock from original
victim IDs. The finalizer executes after work is joined, recovers an unstarted
or failed driver, detaches the tracker, then completes. Runtime shutdown retains
the real owner outside the GPUI event-loop quit budget. No new live CLI/network
capability is inferred from Benchmark identity.

**Tauri adapter.** AppState injects the captured existing main pool into the same
LogPersistenceService. The sink retains at most 128 per-entry tickets; ordinary
threshold/final flushers may finish mixed batches, but only those tickets are
harvested once. Buffered/Attempting are pending, not committed. Final flush waits
for every retained terminal rather than using buffer length as a join proof.
The actual shared EventLog tracker catches unit callers' S2 victim evictions and
its frozen final delta is not double-counted with per-push receipts.

Blocking drive and final I/O run on joined blocking workers. A driver panic
enters recovery; a cleanup failure stays nonterminal rather than authorizing DB
shutdown. ExitRequested is prevented while S2 is retained: the 500ms timeout is
an observation, after which the same join is still awaited. Only terminal joined
cleanup permits the normal exit/database shutdown path. This is cooperative
application shutdown, not a claim to survive OS process termination.

**IPC/permissions.** Three registered/generated commands really correspond to
BenchmarkStream Start/Query/Control, hence command count +3 and lifecycle count
+3 are legitimate test-golden changes, not policy exceptions. Start transfers
the registry permit into the retained outer owner through final flush/detach/
emit. Query is ReadOnly/Parallel; Control is Parallel and reaches cancellation
while Start's ModuleExclusive permit is held. Business-owned execution is not
dropped by an outer request timeout. Both registry/runtime admission and command
body reject a non-S2 source. The read-only frontend capability flag is display
context, not authority. The cancel DTO accepts only a run ID; no rate, duration,
root or SQL controls are exposed. Wire snapshots contain typed synthetic IDs,
counters, finite optional rates/durations and static failures, not DB handles,
tickets, paths or credentials.

**Frontend/controls.** React listens before its recovery read, ignores stale
revisions/foreign runs, detaches late subscriptions and leaves the process owner
running on unmount. Explicit cancel requests do not claim completion. Confirmation
uses safe Cancel and guarded focus restoration without stealing focus from a new
dialog/route. Native confirmation now follows the same required safety boundary.
UI representations distinguish requested profile from actual stages and avoid
claiming a successful comparison from lifecycle completion. The DB backend's new
seven-part checked-persistence/COMMIT-error spec accurately describes the source,
including best-effort rollback and catastrophic-I/O qualification.

## Verification attribution

This review statically followed the current source and reused these implementer
receipts as explicitly permitted. It did not run competing Cargo/link jobs:

- Shared core **5/5** short cfg(test) tests, including real pacing/drop/skip,
  cancellation/final-I/O barrier and revision/terminal freeze. Its cfg(test)
  active workload is 300 slots; this is **not** a completed production 60s run.
- Tauri headless adapter **2/2** and native adapter initial **2/2** exercised
  normal dependency production-profile sources with explicit early cancellation,
  real captured SQLite, mixed ordinary logs, DB ticket/row agreement and joined
  cleanup. These early-cancel executions are not 60-second throughput samples.
- Latest Tauri owner: registry **21/21**, events **7/7** including strengthened
  equal-prefix foreign tracker, adapter **2/2**, four-mode early gate **1 + 4
  actual children**, transferred-permit regression **1/1** passed.
- Root app/db strict all-target/all-feature Clippy **exit 0**. Tauri strict
  all-target/all-feature **exit 0**, followed by cfg(test)-only old-count updates
  and fresh registry **21/21**. Coordinator owns final full gates after freeze.
- Native owner supplied actual parent-Monitoring confirmation test **1/1**:
  early Enter/early handler, safe Cancel, cancellation and suspend/stale Confirm
  never admit the run. Final monitoring **12/12**, stream adapter **3/3**, actual
  new parent-Monitoring control at `RUST_MIN_STACK=1048576` **1/1** (0.07s),
  native fmt-check and strict `--lib --tests` **exit 0** (4.26s). These overlap
  earlier focused adapter/control receipts and are not summed as unique tests.
  The exact new control is registered in the central stack guard.
- Frontend owner supplied final **25/25** (21 S2 plus 4 existing) after actual
  red/green same-revision recovery, old failure and old success clearing hazards,
  and historical-over-budget wording corrections. Final full tsc and scoped
  ESLint **exit 0**; source freeze was confirmed. Its earlier build/mock-browser
  focus observation is not fresh final-source production-build or real
  Tauri/GPUI native evidence. Final aggregate build remains coordinator-owned.
- Bottom DB ACK **8/8**, strict and real COMMIT-BUSY/Attempting-until-join evidence
  were independently established in the preceding ACK check; not rerun here.

## Remaining acceptance

No additional static source defect is left open in the reviewed pipeline. The
coordinator's final full CI/central stack rerun and frontend IPC count goldens
are not counted as passed here; the latter updates must reflect the actual new
three-command registry, not loosen checks. This establishes implementation/controlled headless behavior, not a full
60s S2 performance profile, UI delivery ACK, OS first-present, native pixels,
other-OS acceptance or performance comparability. No GUI, real account/provider,
SSH/WSL or user database action was performed by this reviewer. `on_next_frame`
in the native confirmation is focus scheduling only, never a present timestamp.
