# Usage Job Lifecycle

> Admission, cancellation, bounded execution, and terminal ownership for usage imports.

## Scenario: usage import admission and bounded process completion

### 1. Scope / Trigger

- Applies to usage import jobs, cancellation/status commands, streaming execution, and React job consumers.
- The lifecycle owner is usage_jobs::UsageImportJobs; AppState holds one lock around that registry.

### 2. Signatures

- UsageImportJobs::admit(snapshot) returns the snapshot and an optional CancellationToken. Only the newly admitted runner receives a token; concurrent start receives the active snapshot.
- UsageImportJobs::{request_cancel,progress,complete} own transitions. Only the runner calls complete with UsageImportCompletion after execution returns.
- run_sync_stream returns Result<SyncSummaryEvent, LlmusageAdapterError> after process exit and reader completion. Progress callbacks do not receive wire terminal events.
- LlmusageAdapterError::{Cancelled,TimedOut,CleanupFailed} keep outcomes typed across the command boundary.

### 3. Contracts

- Admission publishes snapshot and cancellation capability under the same lock. Cancellation before runner start prevents process creation.
- cancel_requested is nonterminal. Admission remains occupied during termination, reap, and reader cleanup.
- Finished, failed, cancelled, timed_out, and cleanup_failed snapshots are immutable. Repeated cancel and late progress/completion cannot replace a terminal result.
- An NDJSON finished event is provisional. A later process failure, deadline, or cleanup failure prevents successful completion.
- The production sync deadline remains one hour. One deadline covers stdout, event callbacks, child wait, and stderr completion.
- Termination, reap, and stderr join share a five-second cleanup budget. A reader that exceeds the deadline is aborted; missing cleanup proof produces cleanup_failed.
- Cleanup failure takes precedence over cancellation, timeout, or parse failure. Drop is an OS cleanup fallback and does not prove successful cleanup.
- ManagedProcess confirms owned-tree exit separately from direct-child reap. Unix descendants that ignore graceful termination must be force-terminated even after the direct child exits; Windows completion requires an empty Job Object.
- A cleanup_failed completion releases usage admission only after bounded cleanup attempts and reader joins return. The diagnostic remains visible; releasing admission does not prove that an external process whose cleanup failed has exited.
- Stdout stops at 1 MiB plus one byte without waiting for newline/EOF. Stderr retains at most 64 lines of 64 KiB each and drains concurrently.
- Cancellation tokens and process ownership remain backend-private. Snapshot DTOs gain no executable, environment, or secret fields.
- React keeps cancel_requested loading, displays timed_out/cleanup_failed as failures, rejects terminal regressions, and queries status after subscription to recover immediate completion.
- Control-command admission remains a runtime-policy responsibility. Cancel/get must reach this owner during a separate foreground import. This contract does not authorize global concurrency relaxation.

### 4. Validation & Error Matrix

- Runner paused at a barrier, cancel before release -> zero spawns; admission remains until runner completion.
- Repeated start while active/cancelling -> existing job ID, no second runner.
- Wire finished but child or pipe still live -> deadline applies; no Finished snapshot.
- Cancellation plus tree cleanup error -> cleanup_failed with a diagnostic.
- Oversized stdout without EOF -> llmusage_stdout_line_too_long and bounded cleanup.
- Held stderr after tree cleanup -> bounded join and cleanup_failed.

### 5. Good / Base / Bad Cases

- Good: start receives the token from admission and moves the token into the runner.
- Base: successful exit and completed readers return the deferred summary.
- Bad: create a cancellation token inside a delayed runner, or mark Cancelled from the cancel command.
- Bad: commit Finished from an NDJSON callback before waiting for the OS process.

### 6. Tests Required

- Tauri usage tests: lifecycle barrier, terminal matrix, typed outcomes, bounded fake processes, and adapter/service regressions. Skip export_bindings during parallel behavior tests.
- just tauri-process-smoke: gateway limits and native process-tree cleanup.
- Regenerate changed DTOs with transactional `just tauri-bindings`, then run `just tauri-bindings-check`. The generator serializes export tests and restores the previous generated tree on failure; do not use a direct export filter plus normalizer as a replacement for this failure-safe path.
- Frontend usage-import-lifecycle and usage-import-normalization smoke tests, type-check, and formal lint.
- Run native fixtures on Windows, Linux, and macOS before claiming the platform matrix complete.

### 7. Wrong vs Correct

- Wrong: cancel -> mark_cancelled -> remove active -> await process.
- Correct: admit snapshot + token -> request cancel -> bounded terminate/reap/readers -> complete -> release active.
