# llmusage Install Lifecycle

## 1. Scope / Trigger

Applies to `services/install_{service,detect,exec,ring_buffer}.rs` and their Tauri/GPUI consumers. Preserve one domain owner for canonical plans, host policy, package-manager actions, attempt admission, redaction and cleanup.

## 2. Signatures

- `InstallService::{detect, probe_capabilities, plan}` prepares a safe `PlanOutcome`; only `InstallPlanView` crosses the UI boundary.
- `InstallService::execute(PlanId)` consumes the backend plan and returns `InstallAttempt { attempt_id, events }`.
- `InstallService::{cancel, current_attempt_id, recent_events, manual_catalog}` exposes cancellation requests and recovery without renderer-supplied executable actions.
- `InstallService::test_with_runner` is restricted to test/test-support and creates a synthetic canonical plan without host detection or package-manager execution.
- Native `InstallRequests::{query, execute, cancel, observe}` uses a Services-owned `InstallOwner` projection and the existing bounded application runtime.

## 3. Contracts

Plans expire after 120 seconds, are single-use, and bind to backend-probed host capabilities. An acknowledgment authorizes only the reviewed PlanId; edited text cannot define an executable or its arguments. Manual commands are display/copy actions only.

Version probes consume capped stdout/stderr, use managed process-tree ownership, and await cleanup on timeout or overflow. A successful version exit with truncated output is not proof of an available compatible binary. Redact complete stdout/stderr before extracting a version token or excerpt: extracting first loses credential/header labels. Error excerpts and redaction must preserve UTF-8 boundaries. Legacy generic `key=` masking must cover quoted and escaped values as well as ordinary values; authorization schemes stay intact until the core owner masks the complete header credential.

The attempt slot remains occupied until its executor finishes. Domain terminal publication follows process-tree cleanup and bounded reader cleanup. A failed cleanup or reader join must not become success/cancellation success. Install logs use the existing core redactor before delivery/storage; multiple credentials or Unicode prefixes cannot corrupt replacement offsets.

The native adapter runs host/process work on the existing bounded worker. Its tracked job continuously drains events and retains a redacted bounded projection. Page leave only drops observation; no hidden-page polling. Application shutdown requests domain cancellation and waits for the attempt slot to release. A terminal received before actual slot release remains provisional in the projection.

Fixture mode rejects host detection and execution before any domain call. Tests inject the domain synthetic runner; no global HOME/PATH mutation or real package-manager action is needed. Query results must pass cancellation/revision checks before publishing; stale query completions cannot replace a newer plan or an active attempt.

## 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Missing acknowledgment | Reject before plan consumption or spawn |
| Unknown, expired, reused or host-mismatched PlanId | Domain typed error; no execution |
| Duplicate execution or active cleanup | Keep original attempt; reject new execution |
| View detaches | Runner drains and retains state; revisit restores latest projection |
| Cancel before queued runner starts | Do not spawn; clear queued state with retained feedback |
| Cancel after start / application shutdown | Request cancellation and await cleanup; slot stays busy until then |
| Provisional terminal with occupied slot | Keep running/cleanup status; do not report completed |
| Probe timeout/flood or reader cleanup failure | Bounded failure; await owned cleanup and report failure |
| Multibyte stderr, quoted/repeated credential assignments or spaced/unspaced authorization headers | Safe bounded/redacted text; no panic or leaked value, including in parsed version metadata |
| Fixture/remote install request | Unsupported; never fall back to real local host |

## 5. Good / Base / Bad Cases

- Good: review a backend plan, explicitly confirm it, detach, and restore the same attempt's bounded status on return.
- Base: copy manual instructions without executing them in the application.
- Bad: run renderer command text, treat cancel request as terminal, or trust `Child::kill_on_drop` as awaited tree cleanup.

## 6. Tests Required

Keep default Rust test parallelism and skip export bindings. Cover TTL, single-use/concurrent consumption, forged capabilities, unknown plan, occupied attempt, cancellation before start, detach/revisit, provisional terminal, shutdown waiting for cleanup, and retained failure feedback. Use synthetic process flood/timeout and held-reader fixtures; assert bounded output and cleanup failure vocabulary. Include Unicode excerpt and multiple credential regressions with a positive non-secret text assertion. Headless control tests cover acknowledgment binding, Cancel entry, Tab/Escape, focus return and clearing decisions on navigation. Native fixture checks are separate from real installation and cross-platform acceptance.

## 7. Wrong vs Correct

```text
Wrong: timeout(Child.output()) -> drop child -> report unavailable/completed
Correct: capped concurrent readers + ManagedProcess -> await wait/terminate_tree -> bounded reader join -> result

Wrong: replace one credential, then reuse offsets from the original string
Correct: delegate the complete line to the core redaction owner

Wrong: view drop -> lose event receiver -> assume the attempt finished
Correct: tracked app job drains events -> wait for domain slot release -> retain terminal -> detachable view observes
```
