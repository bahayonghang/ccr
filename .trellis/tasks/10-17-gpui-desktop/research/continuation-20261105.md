# GPUI continuation — 2026-11-05

The user requested continuation of the existing approved GPUI task tree. Restored active task 08 on clean `dev@5d069e63`; no task was recreated. Commands, native installation and their consumed-owner prerequisites are this continuation's implementation boundary. The parent and task 08 remain in progress. Size/DPI remains user-deferred; other OS, real glass, performance and full migration acceptance remain open.

## Implementation

- Commands atomically restore subscription plus retained state on return. Lag recovers authoritative bounded snapshots; UI updates stay at most 10Hz. Suspend detaches, stale generation/job results are ignored and terminal state cannot regress.
- Foreground and help are exposed in the current native workbench. Help has a closed allowlisted target and does not consume the argument draft. Foreground uses the shared retained streaming runner; Tauri's one-shot capped-output/result contract is not claimed equivalent.
- A separate native llmusage installation page consumes the existing CLI InstallService. Its canonical PlanId, TTL, single-use, host fingerprint and singleton stay domain-owned. An app-tracked worker drains events and waits for actual slot release; view return restores a safe watch projection. Explicit confirmation binds to the reviewed PlanId; manual commands can be copied only. Fixture/remote adapters reject real host work.
- Independent review identified three consumed-owner files with four defects: unbounded/version-probe cleanup and UTF-8 excerpt panic, install redactor stale offsets, and unbounded reader joins. The original Unicode and multi-credential panic tests failed before repair (`detect-red.log`). Repairs remain in the CLI owner rather than masking adapter errors: version output is capped at 4096 bytes per stream with 3-second execution/5-second cleanup budgets; install readers share a cleanup deadline and timeout/IO/panic cannot become success.
- Services, module exports, native shell navigation and paper-surface regression include installation. The direct Tokio dependency reuses the desktop-locked 1.53.3; dev-only CLI test-support supplies synthetic installation fixtures. No third-party package version/add/remove changed in any lock (only the native Tokio dependency edge); see `lock-comparison.json`.

## Verification

Raw logs/command receipts: `outputs/desktop/continuation-20261105/`. Matching Rust/Cargo 1.98; desktop uses its own target, root/Tauri gates use the existing process-local alternate target because the default root build lock is inaccessible. Default Rust parallelism and `--skip export_bindings` are retained.

| Gate | Result |
| --- | --- |
| `app-full` | Exit 0: 58 library + 15 integration tests; 3 pre-existing explicit fixture probes ignored |
| `gpui08-app-jobs`, `gpui08-app-lint` | Exit 0: 28 focused tests; all-target/all-feature strict Clippy |
| `cli-install-owner` | Exit 0: 12 isolated canonical-plan owner tests; real-host detection test excluded |
| `tauri-command-retry` | Exit 0: 5 command consumer regressions before CLI owner repairs; final consumer rerun pending |
| `install-focused-final` | Exit 0: 11 native adapter/control tests, including queued zero-spawn cancel, provisional terminal, detach/revisit, shutdown cleanup, confirmation and rendered cancel |
| `detect-fixture-final`, `detect-lint` | Exit 0: 65 isolated install tests, 1 explicit child-fixture helper ignored; CLI all-target/all-feature strict Clippy |
| `review-desktop-check`, `review-desktop-lint` | Exit 0: independent locked all-target check and strict Clippy |
| `preflight-final`, `secret-write` | Exit 0: version/dependency governance and sensitive persistence |
| Final integrated check/lint/test/build/native | Pending; not accepted by the entries above |

Intermediate failures are retained: root target build lock, native module declarations while installation was still being integrated, an invalid Tauri `--lib` invocation, the two original panic regressions, and installation focus-test setup failures (frame callbacks/initial trigger focus and one test context compile error). The first descendant-query fixture used `tasklist`, which failed with an environment credential error; the test now checks the owned fixture PID directly with Windows process APIs and distinguishes access denial from a nonexistent PID. No failed command is counted as a passing gate. An attempted offline metadata resolution would have downgraded unrelated packages; it was discarded, leaving all original package identities intact. An early owner suite included the existing real-host read-only detection probe (PATH/version); the final 65-test isolated run explicitly excludes it. This historical execution is not counted as synthetic evidence.

## Native evidence

Pending an immutable source-matched candidate. Headless controls and domain synthetic-process tests are not native pixels, real installation, trusted sidecar or platform acceptance.

## Remaining work

Task 08 still needs shared sync/checkin/balance migration and their native lifecycle/security acceptance. [Owner research](../../10-17-gpui-08-jobs/research/remaining-owner-map-20261105.md) identifies the next boundaries. In particular, only global checkin concurrency 5 has been found; the PRD's per-origin queue/throttle remains an unproven requirement, not a verified existing capability. The complete 06 resource, 05 account/provider and 07 monitoring/import migrations remain open; 09/10 are still planning. Do not archive tasks or infer completion from page/command counts.

The narrow sibling scan found `ui_service.rs::check_just_installed` still using an uncapped synchronous output probe. It is outside this newly consumed call chain and was recorded without an unrelated repair.

Independent review: [task 08 check](../../10-17-gpui-08-jobs/research/check-20261105.md). No real user configuration/account writes, package installation, external sync/checkin/quota, commit, push, archive, signing or publication was performed.
