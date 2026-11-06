# S2 bottom-layer ACK independent check — 2026-11-06

Scope: `ccr-db/services/log_persistence.rs`, Tauri `events.rs`, their direct
regressions and receipt. Began at HEAD `998dd9f2`. The coordinator explicitly
extended ownership to the COMMIT-error branch of `log_repo::insert_logs_batch`
after the concrete finding below. Other workers' S2 producer, adapters and UI
were read only where needed and were not modified. No commit was made.

## Finding fixed

The existing log repository rolled back statement errors but returned COMMIT
errors through `?` without rollback. A real owned DELETE-journal SQLite DB with
a reader holding a shared lock caused COMMIT BUSY. The ticket was Failed with
attempted=true, but the pooled connection remained in a transaction and retained
uncommitted failed rows. Thus the original INSERT-trigger test did not establish
the full failure-discard claim.

The new actual red failed the writer's `is_autocommit` assertion after COMMIT
BUSY. The minimal fix attempts the same existing ROLLBACK on COMMIT error and
returns the original error. No new framework, schema, retry or production budget
was added. The green regression verifies receipt (1 attempted/0 committed/1
failed), ticket Failed, restored autocommit, no failed rows, unchanged
DatabaseBusy error from the direct repository call, and successful later flush
with only its new ID. Test-only zero busy timeout and a single captured pooled
connection keep it deterministic. The initial test fixture min_idle/max_size
setup panic was corrected before the genuine behavior red and is not counted as
evidence of the product bug.

## Other source conclusions

- Checked tickets have independent private atomic state and original monitoring
  ID; they are retained only by the buffered row and caller. Unit append/threshold
  flush and unit final flush execute the same body, so an ordinary caller cannot
  lose a checked entry's terminal result. Default threshold remains 20.
- Connection checkout happens before Attempting. No checkout means Failed with
  attempted=false; calling the actual transactional batch marks Attempting and
  transaction failure means attempted=true. Committed is published only after
  the repository returns a successful COMMIT. Terminal states precede tracing
  callbacks. Dropped nonterminal buffered rows use CAS to fail Buffered/Attempting
  without overwriting a committed/failed terminal state.
- There is no await between beginning transactional I/O and publishing tickets.
  `force_flush_checked` observes queued rows after the drained batch; its
  remaining_buffered is not a proof that another flush has no in-flight work.
  Per-ticket terminal accounting is the integration proof. Mixed batch receipt
  counts and IDs include ordinary rows: S2 must filter its own identities instead
  of attributing the whole batch to S2. INSERT OR REPLACE semantics remain the
  existing owner policy; receipt counts do not invent unique-row persistence.
- All instance reads/writes use the same `DbAccess`; injected Pool never falls
  back to Global. No second pool or receipt journal was introduced. Dropped
  pending owner and uninitialized Global failure are explicitly covered.
- EventLog checked push preserves the old sizing, count/byte eviction, ordering,
  single-entry limit exception and unit API. Oversized returns no acceptance,
  ring ID or evictions and increments only the old oversize counter. Evictions
  report all row counts but only original monitoring IDs in the monitoring list.
  Internal ring IDs remain a distinct, potentially reused identity. No payload
  was added to diagnostics/receipts and no legacy zero-capacity policy was fixed
  incidentally. Redaction remains the pre-existing monitoring producer boundary;
  this API does not claim to sanitize arbitrary event contents.
- The shared S2 actor integration is separately owned. Its implementer confirmed
  a bounded pending-ticket set and final flush followed by terminal harvesting,
  not treating remaining_buffered as join. This check does not approve unfinished
  producer/adapter/IPC/UI wiring or actual profile results.

## Verification

Pinned Rust1.98/MSVC, locked/offline, default parallelism and skip export_bindings.
Root lease was coordinated; Tauri link lease was not taken.

- Fresh Root `cargo test --locked --offline -p ccr-db log_persistence -- --skip
  export_bindings`: **8/8 passed**, including the real COMMIT-BUSY regression,
  mixed unit/checked 20-row threshold, unit final flush, trigger rollback,
  drop-finalization and actual isolated uninitialized-Global child.
- Actual SQLite locked-writer test was independently run on the prior artifact
  **1/1**, then included again in the fresh 8. It observes Attempting while the
  writer is blocked, confirms the worker has not exited, releases the lock and
  actually joins the thread before checking Committed and the row. No outer
  timeout was called cleanup completion.
- Prior-artifact trigger rollback exact regression **1/1** and Tauri
  `events::tests` **3/3** independently passed. Tauri event source was unchanged
  after that artifact. Full fresh Tauri ACK/adapter gates remain the integration
  owner's work after this additional Root repository input.
- Fresh Root ccr-db all-targets/all-features strict Clippy/typecheck:
  **exit 0** (2.00s). Scoped rustfmt/diff-check passed. The owner's earlier
  Tauri strict receipt was checked; no source lint rule was lowered here.

## Findings not fixed / boundaries

No remaining new defect found in the bottom ACK scope. The rollback call retains
the old best-effort error policy; controlled BUSY recovery is proven, not a
guarantee against catastrophic rollback I/O failure. Adapters must still retain
bounded pending tickets, account each terminal exactly once and join real workers
even when a cleanup budget is exceeded. Event/DB ACKs alone are not proof of UI
delivery or an S2 throughput/profile pass. No actual S2/full benchmark/native GUI
sample, user DB/account/provider/SSH/WSL or global environment mutation was used.

## Follow-up: S2 ring eviction tracker

Independently reviewed only the appended `events.rs` tracker source/tests; no
producer/adapter/IPC/UI code was changed. **No additional defect found.**

- Unit `push` and checked push share the actual `pop_front` loop. Attribution uses
  the removed Monitoring entry's original ID prefix, not the incoming entry,
  internal ring ID or total eviction count. Ordinary and foreign-prefix victims
  do not increment this run; unit callers can now evict a tracked S2 row without
  losing its attribution. Existing per-push receipts remain unchanged.
- Registration is lazy and permits at most one active scope. The queue holds one
  lightweight Arc containing only prefix/count/active fields, with no app, pool,
  observer or callback reference. Empty prefix and a second active registration
  are rejected; an inactive registration is replaceable.
- Finalization validates `Arc::ptr_eq`, not equal prefix text. A foreign handle
  cannot detach the other EventLog's owner even when both prefixes are identical.
  WrongLog consumes/drops the supplied handle, disabling that handle's own scope;
  it supplies no frozen result and must not be treated as successful finalization.
- Finish, detach and final count read occur under the same queue write lock as
  real pops. Successful finish therefore freezes the count at that lock boundary;
  later pops cannot update it. Drop/cancel while waiting for that lock merely
  deactivates a scope and leaves at most one replaceable inactive tombstone. It
  deliberately provides no atomic final-count guarantee for a pop already in
  progress. A subsequent valid push or registration releases/replaces the Arc;
  read-only/oversize operations need not eagerly remove the tombstone.
- Ring capacities, byte limits, ordering, oversize counter and legacy single-entry
  acceptance remain unchanged. Integrators must use the frozen tracker total (or
  its last-observed delta), not add it again to per-push S2 eviction counts.

With the Tauri owner explicitly confirming link had finished, existing artifact
`ccr_desktop-809f58144a032826.exe events::tests -- --skip export_bindings` was run:
**7/7 passed**. No Cargo or native UI was started. The final foreign test source
was inspected: it registers the same prefix on both logs, rejects a foreign
finish, then proves the surviving owner's actual pop still increments and can
finish. The implementer could not certify whether that exact strengthened test
body was read before the existing artifact's compilation, so these seven passes
are **not** claimed as a fresh build of that final assertion. Its fresh tests and
strict lint remain explicitly assigned to the performance owner's next coherent
Tauri build; the earlier three-test strict receipt is not promoted to cover this
new tracker. That pending integration gate does not change this source finding.
