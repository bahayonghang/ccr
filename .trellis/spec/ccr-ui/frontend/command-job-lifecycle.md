# Command Job Lifecycle

## 1. Scope / Trigger

Read this contract when changing command workbench state, command events, job
recovery, or history submission. Owners: `src/features/commands/commandJobStore.ts`,
`commandJobState.ts`, and `src/shell/eventBridge.ts`. The page selects state from
`useCommandsStreamStore`. Query owns the catalog. Job state is memory-only for
the app session; the existing view preference persistence schema is unchanged.

## 2. Signatures

Existing typed IPC remains unchanged:

```typescript
startCcrCommandJob({ command, args, confirmationToken }): Promise<StartCommandJobResponse>
getCcrCommandJobStatus(jobId: string): Promise<CommandJobSnapshot>
cancelCcrCommandJob(jobId: string): Promise<CommandJobSnapshot>
```

The store owns start, cancel, reconcile, receiveDeltas, receiveSnapshot,
reportListener, and history actions. CommandJobDelta has job_id, seq, channel,
lines, dropped count, and optional status. CommandJobSnapshot has no sequence
watermark. Never assume a snapshot acknowledges a particular sequence number.

## 3. Contracts

- The shell installs one set of command listeners. Routes never install their
  own listeners or keep a second snapshot. Route entry reconciles the retained
  job through the status API.
- Start sets submitting synchronously before IPC. A second click cannot start
  another job while submission is pending or the retained job is active. Only
  the returned job ID selects early buffered events. Old or unknown events
  cannot choose the active job.
- A queued start supports contiguous increasing delta sequences. Duplicate and
  older sequences are ignored. A gap, status lookup, or cancel response switches
  the job to authoritative snapshots. Later deltas schedule reconciliation
  instead of appending output whose coverage cannot be determined. Identical
  output lines with distinct sequence positions remain distinct.
- The shell batches progress for 250 ms from the first event. The owner
  coalesces recovery reads with a fixed 250 ms window and one in-flight read.
  Continuous output cannot reset that deadline. Terminal events flush the
  command batch immediately and then install final output and status.
- Terminal states (success, failed, cancelled, cleanup_failed, unavailable)
  never regress. Generation and request counters isolate stale responses.
  Cancel requests cancellation; an active response stays active until backend
  cleanup reports a terminal outcome. Active/submitting jobs cannot be cleared.
- Retained output is capped at 500 lines and 512 KiB of UTF-8 per channel.
  Progress batches and early-event buffers cap at 500 events; early unknown
  identities cap at 16 jobs. Loss of sequence coverage triggers reconciliation.
- Command listener rejection is visible. Active jobs use status recovery and
  then poll every 1 second after successful reads. A failed read stops automatic
  recovery, including an already scheduled timer, until a new event or explicit
  retry. Terminal and expired jobs stop
  recovery. A successful registration clears only its own failure. Usage
  listener behavior is unchanged.
- Listen may resolve after cleanup. Its late unlisten runs immediately.
  Command callbacks and registration results after cleanup cannot change the
  store. Full shell cleanup suspends recovery scheduling. Pending start/cancel
  responses retain the job outcome but cannot restart recovery while the shell
  is absent. Shell remount resumes recovery and reconciles the retained job.
- History writes have pending/saved/failed states and retain a memory-only
  command/result summary. Submit the existing addRecentItem API at most once
  per job. Saved requires successful resolution; a claim only means pending.
  Failure remains visible across routes and later jobs. History refresh only
  reads; it never retries the write.
- The existing history API generates a new UUID per call and accepts no job
  idempotency key. Rejection may be an ambiguous transport result. Do not replay
  the write or claim durable exactly-once delivery. The failure summary lasts
  only for the app session.

## 4. Validation & Error Matrix

| Condition | Required result |
| --- | --- |
| Terminal before queued start response | Select matching buffered terminal; one history submission |
| Old job event/status/cancel response after new job | Keep new job and controls |
| Duplicate/out-of-order delta | No duplicated output or terminal regression |
| Gap or snapshot without watermark | Read authoritative status; do not guess delta coverage |
| Exact Command job '<id>' not found for current ID | Mark retention expired; keep last output; disable cancel; no invented completion/history |
| Other status failure | Keep job; show error and retry |
| Listener rejects | Visible degraded state and status recovery |
| History rejects | Visible failed summary; no saved claim or resubmission |
| Late listen resolution after cleanup | Unlisten once; ignore stale callbacks |

## 5. Good / Base / Bad Cases

- Good: leave the route while running, receive completion in the shell, return
  to the retained output and one submitted history entry.
- Base: queued start, seq 0 running, contiguous output, terminal snapshot.
- Bad: route cleanup clears the job; cancel acknowledgement invents cancelled;
  a stale queued response replaces terminal state.
- Bad: append every delayed delta after a snapshot; mark a submitted but
  rejected history write as saved.

## 6. Tests Required

Run `bun run test:smoke -- tests/commands tests/shell` from ccr-ui.
command-workbench-lifecycle.smoke.test.tsx mounts actual CommandsView in
MemoryRouter with the real shell bridge. It covers route removal/return, missed
completion, duplicate clicks, terminal-before-start, sequence/gap ordering,
fixed scheduling during continuous output, stale responses, cancellation,
expiration/retry, confirmation, history read ordering, listener rejection, and
failed history persistence. command-job-reducer.smoke.test.ts checks terminal
states and UTF-8 bounds. event-bridge-leak.smoke.test.tsx covers immediate,
delayed, StrictMode and post-cleanup callback behavior.

Also run type-check, formal lint, architecture boundary/cycle checks, and the
production build. Record protected baseline failures separately. Scoped lint
does not replace formal lint.

## 7. Wrong vs Correct

Wrong: page-local state accepts the start response unconditionally.

```typescript
const response = await startCcrCommandJob(request)
setCurrentSnapshot(response.snapshot)
```

Correct: invoke the owner that claims submission before IPC and reduces early
terminal events with the returned identity.

```typescript
await useCommandsStreamStore.getState().start(request)
const snapshot = useCommandsStreamStore((state) => state.job.snapshot)
```
