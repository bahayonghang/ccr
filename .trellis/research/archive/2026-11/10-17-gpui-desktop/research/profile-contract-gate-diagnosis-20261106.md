# Profile commit replay gate diagnosis — 2026-11-06

## Scope and original failure

Reviewed at HEAD `998dd9f2`, retaining the existing task and concurrent worktree.
The original `outputs/desktop/integration-20261106-ci-s2.log:3663` records a
Tauri all-feature failure in
`commands::profile_lifecycle::application_contract::profile_commit_failure_replays_operation`:
the isolated child ran for 11.89s and returned `AppliedWithWarning` instead of
`Applied`. Its assertion did not print `ProfileOutcome.warnings`. The parent
summary was 333 passed / 1 failed / 5 ignored. This evidence does **not** identify
the failing ancillary operation or prove a timeout.

## Diagnostic change

`crates/ccr-cli/src/application/profile_contract.rs` now includes
`"{replayed:?}"` in the existing equality assertion. The expected `Applied`,
activation/count checks, test budgets, default parallelism and subprocess
isolation are unchanged. `ProfileOutcome` deliberately contains identifiers and
status, not profile values or raw IO errors; the existing synthetic fixture is
retained. No production behavior was changed.

## Source findings

- The real Tauri payload delegates to the shared lifecycle owner. The injected
  catalog-commit failure is thread-local and rolls back the first application;
  the second request executes the real replay path.
- The child owns a fresh marked temporary HOME/USERPROFILE. `TestHome` binds
  CCR_ROOT, CCR_LOCK_DIR, Claude runtime/settings/backup and Codex paths; the
  shared contract additionally binds CCR_DATA_DIR and GROK_HOME. The captured
  lifecycle context scopes nested synchronous locks to its resolved lock path.
  The failed test itself does not share its catalog/data/operation paths with
  another contract child.
- For this Claude activation, warning exits are operation-record persistence
  (including the provisional `AncillaryPending` record), usage-count update,
  history initialization/insert, and activation analytics. Auth-registry sync
  is optional Codex work and is not active here. The code intentionally reports
  ancillary failure after the activation commit rather than pretending the
  activation rolled back.
- Some file locks have 10s budgets, analytics has 5s, and SQLite uses its existing
  busy/connection budgets. An 11.89s child duration alone cannot select one of
  those paths or show who held a resource. No deterministic production defect
  was established by source review.

## Actual checks

Commands used the pinned 1.98.0 MSVC toolchain, locked/offline dependencies and
`.cargo/tauri-ci.toml`:

```powershell
cargo --config .cargo/tauri-ci.toml test --locked --offline --manifest-path ccr-ui/src-tauri/Cargo.toml --all-features --bin ccr-desktop commands::profile_lifecycle::application_contract::profile_commit_failure_replays_operation -- --exact --skip export_bindings --nocapture
cargo --config .cargo/tauri-ci.toml test --locked --offline --manifest-path ccr-ui/src-tauri/Cargo.toml --all-features --bin ccr-desktop -- --skip export_bindings --nocapture
```

- Before the diagnostic-only change, the exact all-feature test passed **1/1**
  (0.45s).
- After the diagnostic-only change, the full same-feature/default-parallel
  binary passed **334/334**, **5 ignored**, **200 binding tests filtered**, in
  **32.40s**. The target test passed. Output is retained in
  `outputs/desktop/profile-contract-diagnostic-20261106.log`.
- Compilation/type checking passed as part of the actual test build. No fresh
  Clippy claim is made for this diagnostic-only assertion change; the original
  full gate had already passed preceding Root/strict checks. The existing MSVC
  proc-macro linker stdout lint advisory is not a source failure.

## Unresolved finding and handoff

The original ancillary warning remains **unidentified**, not classified as
flaky, fixed, or a proved resource timeout. The two executions above are useful
reproduction attempts, not a root-cause fix. A future failure now exposes the
actual warning category without weakening acceptance. No retry loop, timeout
increase, serialization change or relaxed status expectation was introduced.
The coordinator owns the final full-CI decision and any subsequent diagnostic
failure. No native release/package target, GUI, user home, account/provider,
SSH/WSL or external service was touched.
