# S2 persistence and ring ACK — 2026-11-06

Stage: **ReadyForReview** (bottom-layer ACK only). Baseline `998dd9f2204abe99a3f468a857c362fb9e6b8120`, initially clean. Shared S2 producer, native/Tauri adapters, AppState integration and IPC/UI belong to the performance owner; this receipt does not claim their implementation or an actual S2 profile run.

## Delivered API and ownership

`crates/ccr-db/src/services/log_persistence.rs`:

- `LogPersistenceService::with_db(config, DbAccess)` injects the existing captured pool. `new(config)` preserves `Global`; all instance reads/writes use that same handle. No second pool, new SQL/schema or alternative storage owner.
- `append_entry_checked(&MonitoringEntry).await -> LogPersistenceTicket`. The cloneable Send/Sync ticket identifies one monitoring ID through `entry_id()`; `state()` is `Buffered`, `Attempting`, `Committed`, or `Failed { attempted }`.
- A ticket is held only by its buffered entry and the checked caller. Every append/flush path shares the same batch owner; a normal unit append that crosses threshold or a normal periodic/final flush still publishes the checked entries' terminal states. There is no receipt journal or lossy broadcast. The sink must retain bounded pending tickets and retire each only after accounting its terminal state.
- `Buffered` is neither attempted nor committed. `Attempting` begins after connection acquisition immediately before the existing transactional batch call. Only successful return from that call publishes `Committed`. Connection acquisition failure reports `Failed { attempted: false }`; transactional failure reports `Failed { attempted: true }` for each batch member. Terminal state is published before diagnostic tracing callbacks. Dropping an unflushed owner or unwinding an unfinished batch finalizes nonterminal tickets as failed; finished tickets remain unchanged.
- `force_flush_checked().await -> LogFlushReceipt { entry_ids, attempted, committed, failed, remaining_buffered }` identifies the entire drained batch. Counts cover all batch rows, including ordinary logs; S2 must use its own ticket IDs rather than attribute mixed totals to itself. `remaining_buffered` is an observation of queued entries, not proof that another concurrent flush has no in-flight I/O. Pending tickets provide that proof.
- The batch owner runs synchronously after draining, with no await between beginning I/O and publishing terminal results. The checked API neither introduces an outer I/O timeout nor detaches work. An adapter using a worker must still join it. Pool checkout/busy budgets and application exit budgets are unchanged.

The existing threshold remains 20; unit `append_entry`, `append_entries`, and `force_flush` return unit, delegate to the same body, retain the existing loss-on-flush-error behavior and original diagnostics. No retries or global resets were added.

`ccr-ui/src-tauri/src/events.rs`:

- `EventLog::push_checked(AppEvent).await -> EventLogPushReceipt` reports `accepted`, `oversized`, `evicted`, `evicted_monitoring_ids`, and optional internal ring `entry_id`.
- Monitoring eviction IDs are the original monitoring IDs, allowing a sink to separate S2 from ordinary entries. Internal ring IDs are a different identity; the existing assignment/reuse behavior is unchanged.
- Original `push` delegates and discards the receipt. Serialization sizing, oversize drop counter/warning, count/byte eviction and entry order are preserved. Evictions do not increment the oversize drop counter. Existing zero-capacity/single-item-over-total-limit acceptance is deliberately retained and tested; the ACK describes actual behavior rather than silently tightening policy.

## Focused validation

- Root `cargo test --locked --offline -p ccr-db log_persistence -- --skip export_bindings`: **7 passed**, default parallelism.
- Tauri `cargo test --locked --offline events::tests -- --skip export_bindings`: **3 passed**, default parallelism.
- Root `cargo clippy --locked --offline -p ccr-db --all-targets --all-features -- -D warnings`: passed.
- Tauri `cargo clippy --locked --offline --all-targets -- -D warnings`: passed. The toolchain printed the existing procedural-macro linker stdout warning, which explicitly ignores `-D warnings`; no source lint failure.
- Scoped rustfmt and `git diff --check`: passed.

Tests cover mixed unit/checked 20-entry threshold batches; unit final flush updating only caller-held checked tickets; checked final receipt IDs/counts and empty flush; full transactional rollback after a controlled failing trigger with no false commits and the original no-retry policy; isolated-child uninitialized Global failure with zero database attempts; dropped pending owner terminalization; a real owned temporary SQLite write lock with `Attempting` visible until the blocking I/O returns and the worker is joined; oversize drop without eviction; count/byte eviction with monitoring-vs-ordinary ID attribution; and original unit ring behavior.

New tests use explicit memory pools, an owned temporary database, or an isolated child that never initializes the global pool. The existing global-unit test remains unchanged. No user database, provider, native GUI, SSH/WSL or account action is involved. Final integration review, complete CI and real S2 profile qualification remain separate work.

## Independent check follow-up: COMMIT failure

The original 7-test receipt covered INSERT-trigger rollback, not a failure of
COMMIT itself. Independent review reproduced a separate old repository defect
with an owned DELETE-journal DB and a reader-held shared lock: COMMIT returned
BUSY and the checked ticket correctly failed, but the pooled writer was still
inside the transaction. The first test setup used an invalid min_idle/max_size
combination; that setup panic is not counted as the regression red. The actual
red failed `is_autocommit()` after the real BUSY commit.

With coordinator approval, `log_repo::insert_logs_batch` now attempts ROLLBACK
also on COMMIT error and returns the original error. No transaction framework,
production timeout, SQL schema or retry policy changed. The direct regression
checks original DatabaseBusy, restored autocommit, no failed rows and a later
normal batch containing only its new ID. Root `log_persistence` is now **8/8**
and ccr-db all-targets/all-features strict Clippy **exit 0**. This proves the
controlled COMMIT-BUSY path; it is not an assertion that rollback can succeed
despite every possible catastrophic I/O failure. Full independent review is in
`performance-s2-ack-check-20261106.md`.

## Follow-up: ordinary pushes evicting S2 rows

Per-push receipts alone were insufficient: a normal unit `EventLog::push` can
evict an S2 entry while discarding its receipt. The EventLog owner now offers
one lazy `register_eviction_tracker(id_prefix).await` registration. It returns
a non-Clone `EventLogEvictionTracker`; `evicted()` is a live observation.
Every actual ring pop, including unit and checked pushes, increments this one
counter only when the victim is a Monitoring entry whose ID has that prefix.
There is no additional ring scan, per-ID history, receipt journal or broadcast.
The counter is created only on explicit registration; another active run and an
empty prefix are rejected.

`finish_eviction_tracker(handle).await` verifies the actual Arc identity of the
current registration, then detaches and reads the final counter under the same
queue write lock used for all pops. It must not match by prefix alone or detach
another EventLog's tracker. Pops before that boundary count; pops afterwards do
not alter the frozen run. The adapter must use that final total (or the delta
from its previous live observation), not add it to per-push eviction receipts.
This boundary does not keep tracking entries that remain in the ring after the
run is finalized.

Dropping a handle, including cancelling its pending finish future, atomically
deactivates it. At most one lightweight inactive tombstone can remain in the
queue; the next valid push or registration removes/replaces it. It contains
only the run prefix and atomic counters, never an app/pool/observer callback.
Drop does not promise a final observed count: an already-started pop can race
with that deactivation. Only explicit successful finish provides the frozen
count at a queue-lock boundary. Capacity, byte limits, oversize/drop counters,
zero-capacity behavior and entry ordering remain unchanged.

Tracker validation: the first fresh `events::tests` run passed **7/7** with
default parallelism, including the original three ring tests and four tracker
tests for unit victim attribution/final freeze, lazy single registration/Drop,
cancellation while waiting for the queue lock, and foreign-owner rejection.
The foreign-owner regression was subsequently strengthened to register the
same prefix on both EventLogs and verify the surviving owner still counts an
actual pop; final source is rustfmt/diff-check clean. The performance owner has
the coherent final source and Tauri compile lease for the fresh seven tests
and integrated strict Clippy. This follow-up does not claim that pending
integrated check from the earlier three-test Clippy result above. No new lint
allowance was added while the new API awaited its real adapter consumer.
