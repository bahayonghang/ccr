# Independent check: retained terminal / Recent / source import

Reviewed against the current parent PRD/design/implement, next-session implementation prompt, ccr-app shared desktop/configuration specs, desktop command policy, usage lifecycle and installed-CLI adapter contracts. Scope excludes the concurrently implemented shell, workbench, MCP and tools surfaces.

## Findings (fixed)

- `ccr-desktop/src/runtime/jobs.rs`: the finalizer looked up a command terminal by ID after waiting for a blocking permit. The 64-entry registry can evict that terminal in the meantime, silently losing Recent history. With parent approval for the shared seam, the runner now retains its owned terminal before giving up the worker permit; the finalizer consumes that value. Queued/unavailable work captures `CommandJobs::abandon_snapshot` directly. There is no finalizer registry lookup.
- `crates/ccr-app/src/jobs/commands.rs`: added `abandon_snapshot(ticket) -> Option<Snapshot>` using the existing immutable `complete` transition and its snapshot clone under the same lock. The original `abandon(ticket) -> ()` remains compatible. No caller synthesizes an unavailable result when the owner supplies none.
- Added shared capacity-eviction regression and desktop actual-storage regression: evict the original terminal with `MAX_JOBS` admissions, prove lookup fails, then persist the held terminal with a detached, shutdown-joined finalizer and reopen the database. The test explicitly constructs the captured-terminal handoff; existing `CommandRequests` detach/replay tests cover the real adapter path. It does not depend on Tokio task poll order to force semaphore waiter order.
- Updated `shared-desktop-services.md` to require owned terminal persistence across cache eviction/TTL.

## Findings (not fixed)

None remaining in the reviewed batch. Native test compilation exposed `unused_mut` in `features/insights/observer/presentation.rs:26`; this is outside reviewer ownership in the concurrently implemented UI batch. Reported to that owner and the parent for removal before strict desktop lint. This is a source/lifecycle and focused fixture check, not native visual acceptance or the full migration verdict.

## Source conclusions

- Finalizers use the same bounded blocking semaphore and tracked task; cancellation while queued does not skip finalization, and shutdown waits after view detachment. Finalizer panic/write failure does not replace the runner completion.
- Replayed admission returns no new ticket, so it cannot schedule another finalizer; observation does not persist history. History storage omits all arbitrary arguments, including `full_command`; raw-domain-row assertions cover this boundary.
- Scoped import uses the existing installed CLI `--source` and `--recent-days` options, captures them in retained state, and skips Claude observer ingestion for other sources. No new CLI switch, usage SQL, upstream crate link, ambient-home mutation or unowned executable was introduced.
- Fixture launches and custom provider homes are rejected before admission. The selected-source fixture asserts no observer database is created; the immutable finished state survives a later cancel request. Source controls are disabled during an active run and locked in a platform context.

## Verification

- Shared command owner: `cargo test -p ccr-app --locked jobs::commands::tests -- --skip export_bindings` — PASS, 10 tests, including eviction regression.
- Shared lint/type-check: `cargo clippy -p ccr-app --locked --all-targets --all-features -- -D warnings` — PASS.
- Scoped rustfmt and `git diff --check` — PASS.
- Parent-provided earlier receipts, source/assertions independently inspected: root runtime 11/11; native import 5/5; native jobs/UI-state 3/3 each before the eviction fix. No unchanged test was rerun solely to duplicate those receipts.
- Native changed jobs: `cargo test --locked --lib runtime::jobs::tests -- --skip export_bindings` — PASS, 4 tests including the new eviction/persistence regression; 1 intentionally ignored closed subprocess invoked by the real-process test.
- Native UI-state: `cargo test --locked --lib runtime::ui_state::tests -- --skip export_bindings` — PASS, 3 tests covering raw persisted argument omission, retry/isolation and committed cancellation.
- Native type compilation for these tests — PASS. Full desktop strict lint remains the parent's integration gate; the independent shared strict lint above passed. No desktop lint pass is inferred from test compilation.
- Native visual/all-platform evidence and final complete CI: outside this narrow review; no pass claim.

Initial shared Cargo invocations hit the existing external mbx target-cache access boundary. The successful runs used the actual Rust 1.98 Cargo with approved linked-cache access. This was an environment permission issue, not a failing assertion.

No commit made by the reviewer. Local atomic Recent verdict: PASS after the eviction fix and focused regressions. Full desktop integration still owns the separate concurrent UI warning, native checks and final gates.
