# Native runtime metrics independent check — 2026-11-06

## Scope and verdict

Reviewed the accepted small metrics batch against
`performance-runtime-metrics-gap-20261106.md`, at starting HEAD `fc093e78`.
Existing worktree/concurrent implementation was retained. No source defect
requiring reviewer self-fix was found. This review used current source, not
literal Tauri DTO parity as its acceptance criterion.

The owner was asked to narrow product wording to the **Usage page report**
cohort: the observation is native `InsightsQuery::Usage`, not every usage
operation, Home, Workbench, Tauri command, individual SQL statement or runtime
job. This is a wording correction within the accepted scope, not an expansion
of instrumentation. Final wording now names the Usage page and explicitly
excludes Home/Workbench; the current spec/inventory records the same scope.

## Source checks

- `ccr-app::monitoring::LatencyWindow` holds at most 2048 FIFO durations, sorts
  a snapshot, and uses nearest rank `ceil(n * 95 / 100) - 1`. Empty is
  `{sample_count: 0, p95_ms: None}`; a real zero duration remains `Some(0)`.
  Count describes retained observations, not process-lifetime successes. The
  owner accepts `Duration`, so NaN/negative numeric samples cannot enter it.
  It has no global state, timer, IO, dependency or SQL instrumentation.
- `Services::new` creates a fresh Arc/Mutex window. Clones share that captured
  owner; independently constructed Services do not. `with_insights_paths`
  compares effective paths and resets the window only when the captured source
  actually changes. Old request owners retain the old Arc, so late reads cannot
  enter the rebound source's sample window. Equal effective paths preserve it.
  Benchmark's existing layout takes precedence over ignored overrides, so a
  meaningless override cannot relabel the benchmark source or reset its data.
- Native `InsightsRequests` captures paths and Services together. The timer
  starts after `open_dashboard` returns and immediately surrounds the real
  shared `read_dashboard_at`; elapsed is frozen before acquiring the sample
  mutex. Only `Ok` is recorded. Missing/unconfigured DB, failed reports,
  admission/queue waiting, DB opening, runtime join, serialization/rendering,
  and other query variants are outside this cohort. Clock freezing does not
  change the real monotonic Instant used for elapsed time.
- A successful report body can be observed even if its eventual UI subscriber
  has detached. The metric measures completed successful service reads, not
  delivered UI responses, cancellation latency or paint/present ACKs. It does
  not start a read to obtain a sample.
- `Workspace::retained_session_count` only locks and tests its existing
  `Option<Session>`. It does not resolve paths, query a DB, inspect identities,
  start a process/transport, connect or test socket liveness. Monitoring uses
  `Services.ssh.get()` and returns actual zero when no owner exists, without
  initializing one. An existing retained logical capability is one; disconnect
  clears it. No host/session identifier or credential enters the metrics DTO.
- Monitoring still refreshes explicitly. Construction/render/snapshot do not
  poll usage or SSH; suspend drops the subscription, clears facts and advances
  generation to refuse late UI results. The strengthened existing control test
  observes an allowed in-flight completion followed by no further reads, and
  verifies Monitoring refresh does not create a Usage sample.
- RSS remains optional and comes from its existing shared owner. Tauri's Moka
  result count, disconnected-state map and password cache are neither invented
  in native nor conflated with retained SSH authority. No Tauri wire/policy,
  provider, S2 stream, SQL owner or source admission behavior is changed here.

## Verification evidence

The frozen implementation owner's final focused receipts were received and
checked against the relevant final source. This reviewer did not start
competing Cargo/link jobs or operate a native window:

- Root latency-window tests **2/2 PASS**; SSH pure-memory/NoTransport regression
  **1/1 PASS** (0.50s); ccr-app strict all-target/all-feature Clippy **exit 0**
  (4.33s).
- Native latest latency/source and Monitoring projection tests **2/2 PASS**
  (0.59s). Earlier affected Monitoring suite **13/13 PASS** (3.87s); this is
  identified as the earlier suite, not repeated final-source aggregate proof.
- Latest native fmt-check **exit 0**; strict `--lib --tests` Clippy **exit 0**
  (11.71s).
- Final actual `RUST_MIN_STACK=1048576` exact Monitoring control child
  **1/1 PASS** (1.15s), covering unavailable/zero/count display, readable Usage
  fixture construction without sampling, and suspension/no hidden polling.
- Reviewer-owned scoped `git diff --check`: **exit 0**.

The native fixture-session test initially tried asynchronous domain setup on
the GPUI executor without a Tokio reactor. The owner corrected setup to run
through the existing blocking RuntimeHandle read boundary using its current
Tokio handle; final code still uses the real synthetic Workspace transition,
not a production metrics fallback or an injected success result.

The new pure tests cover empty/zero/nearest-rank and FIFO capacity. Native
source tests cover successful/failed/missing report reads, independent captured
roots, clone sharing and rebinding. SSH tests remove DB lookup authority and
install a panic transport after a controlled fixture connection, proving the
counter remains a pure memory observation. The strengthened native control
reuses the exact Monitoring test already listed by the central 1 MiB guard.

## Limits

No performance threshold, paired performance result, 60-second throughput,
native visual acceptance or OS presentation measurement is established by this
batch. User native validation remains deferred. Final aggregate gates belong
to the coordinator for the new candidate; earlier HEAD CI results do not cover
these changes. No real account, SSH/WSL host, provider, user database or GUI was
used by this reviewer.

## Additional inventory reconciliation (read-only)

The coordinator's four `system_prompts_list/get/save/create` metadata updates
were checked narrowly against current owning source. Tauri list adds
`list_claude_rules` metadata; get/save/create all resolve finite `prompt_spec`
IDs, not arbitrary rule paths. Native `runtime/resources.rs` retains fixed-file
targets with original/absence-token writes, while the separate captured
`runtime/resources/claude_files.rs` maps `Kind::Rules` to the real shared rule
file owner. The discoverable Resources “Rules and Claude resources” entry and
existing fixture control test cover rule creation, Cancel before mutation,
confirmed Save, external-change conflict with retained draft, and deletion.

Therefore the current `implemented_pending_native_acceptance` mapping is
supported by source. Keeping the old `execution_20261029.remaining` as a
historical checkpoint and explicitly recording closure of its old list-rules
omission is accurate. No new resource implementation or fresh test/GUI claim
was made in this check; its cited historical integrated gates do not substitute
for the new metrics candidate's final gates.
