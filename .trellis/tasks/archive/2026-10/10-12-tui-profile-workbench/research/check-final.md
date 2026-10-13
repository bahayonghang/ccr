# Final Profile workbench review

Date: 2026-10-13. Role: dispatched `trellis-check`; user-approved revision-2 candidate A. This review covers the complete implementation, following the earlier scoped data review in `check-history.md`. Product edits were made only after the coordinator froze the implementers' files. No unrelated build-cache/journal changes were reverted, no personal data was opened, and no commits or task-status changes were made.

## Reviewed scope

- All changed/new product code in `ccr-types::profile_activity`, `ccr-store::storage::{database,profile_activity}`, the `ccr-cli::application::profile_activity` facade and their exports/tests.
- `ccr-tui/src/tui/{app,profile_tasks,runtime,event,mod,ui,theme}.rs`, `ui/workbench.rs`, and `usage/app.rs`, with the actual existing profile lifecycle adapter and its contract tests.
- Current parent PRD/design/implementation plan and curated 15-entry check manifest; full TUI backend rules, new Profile workbench/history contracts and relevant repository/lifecycle/serialization/usage constraints. Reviewed existing implementation evidence without treating it as an independent test run.
- Actual Ratatui cell artifacts: `rendered-tui/codex-en-mocha-200x50.png` and the corrected six-tab `rendered-tui/grok-en-mocha-80x20.png`. The coordinator owns the complete two-theme image export/inspection matrix and final aggregate CI.

## Findings (fixed)

### 1. New tests did not pass the repository's strict unwrap gate

Files: `crates/ccr-types/src/profile_activity.rs`, `crates/ccr-store/tests/profile_activity.rs`, `crates/ccr-cli/src/application/profile_activity.rs`, `crates/ccr-tui/src/tui/ui/workbench.rs`.

The implementation checks used `-D warnings`, while `just lint-strict` also enables `-D clippy::unwrap_used`. The extra flag exposed new fixture/serialization/export unwraps. Converted fallible tests and helpers to `Result` with `?`; fixture invariants have explicit assertion/expect messages where appropriate. Did not add lint suppressions or weaken assertions, default test parallelism, or export separation. This supersedes any interpretation of the earlier narrower Clippy command as the complete repository gate.

### 2. A failed whole Profile read retained stale effective-runtime claims

File: `crates/ccr-tui/src/tui/app.rs` (`profile_read_failed`).

The existing nested-error snapshot path cleared current markers, but a top-level loader error/disconnected reader only set `profile_load_error`. A previously loaded profile/runtime could therefore still be presented as confirmed current after the refresh failed. The failure path now clears current markers and typed runtime summaries, exposes the runtime error, and invalidates the matching initialized Codex/Claude Auth summary while retaining list/configuration/selection for browsing. The detail renderer avoids printing the identical load/status error twice.

Regression: `failed_profile_worker_invalidates_runtime_but_preserves_browsing_state` exercises the actual App failure handler with an initialized isolated Codex Auth fixture, preserving list/selection and checking both runtime caches and current markers.

### 3. Compact navigation lost platform identity and clipped the sixth tab

Files: `crates/ccr-tui/src/tui/{app,ui}.rs`, `ui/workbench.rs` tests.

After removing emoji, compact Auth labels became three indistinguishable `Auth` labels. Full labels also exceeded some widths above the old fixed 100-column breakpoint. Compact labels now retain the platform (`Codex Auth`, `Claude Auth`, `Grok Auth`, and localized equivalents). The header chooses compact spacing from the measured complete label budget, uses reduced padding/dividers, and shares those exact choices with hit testing.

Regression: `six_tab_header_keeps_platform_names_and_mouse_targets_at_narrow_widths` renders all six tabs at 60/80/100 columns in EN/ZH, with each of the six active-tab states; it asserts each complete title and its first/last clickable cells. The corrected actual 80x20 Grok artifact was also visually inspected.

### 4. Invalid category counts could wrap out of their row

File: `crates/ccr-tui/src/tui/ui/workbench.rs` (`usage_ledger`).

The count column was budgeted using the formatted number, but a negative category displays the longer localized `Invalid` marker. Its final characters could wrap to another line. The width calculation now budgets the displayed marker itself. Normal values and fractional bar geometry are unchanged.

Regression: `invalid_category_counts_stay_on_their_aligned_row` checks the marker remains beside its input category at 58/80 content columns.

### 5. Existing adapter source guard still expected synchronous App execution

File: `crates/ccr-cli/src/application/profile_contract.rs` (`profile_adapters_keep_application_boundary`).

The coordinator's aggregate CI exposed an existing source assertion that still required `profile_backend::apply` inside `App::apply_selected`. The approved scheduling change moved that call into `profile_tasks::mutate`, so the assertion failed despite retaining the shared lifecycle owner. Updated the source guard to follow App request submission, the default mutation writer, captured request execution inside `spawn_blocking`, separate apply/off worker arms, and the existing backend's shared lifecycle entry. The apply path still forbids off/direct platform application; worker/handoff/backend boundaries reject direct `.apply_profile` and command bypass, and the synchronous mutation adapter rejects `.await`. Existing lifecycle and Tauri guards remain unchanged. No runtime code changed in this follow-up.

Regression: the previously failing source-boundary test passed independently after the update. Strict CLI/TUI Clippy/typecheck also passed; aggregate CI remains the coordinator's separate gate.

## Cross-layer review conclusions

- History remains one read-only, platform/exact-name batch through ccr-store -> ccr-cli -> background snapshot -> cached App -> view. Strict success/operation filtering, RFC3339 UTC ordering, bounded top-three storage, typed missing/schema/error/partial/future states and credential exclusion remain intact. The writer still uses the original path precedence; the reader never calls initialization/migration or an immutable URI that ignores WAL.
- ProfileTasks bounds readers per platform, coalesces reloads and invalidates pre-mutation generations. Mutation requests capture platform/name/operation identity, and mutation outcomes are consumed independently of stale-read filtering. The complete synchronous lifecycle/off owner remains inside one blocking closure; transaction/backup/CAS/replay internals are unchanged.
- Duplicate writes are rejected; Profile browsing stays available and entry to Auth is gated while writing. q/Esc/Ctrl+C defer exit through a distinct readiness accessor. Every processed event reaches completion pumping, including Ctrl+T/resize/ignored motion. Pending polls cap at 50ms; elapsed-clock ticks retain the 250ms Auth cadence. No unconditional idle redraw or per-selection I/O was introduced.
- Snapshot application preserves per-tab/name selection and independently updates current-runtime data. Initialized Codex/Claude Auth caches receive fresh typed summaries without account-selection reset. Grok's richer activation state distinguishes current/drifted/unsafe rather than treating a saved pointer as effective activation.
- Table width budgets, header/rule subtraction and mouse body geometry agree. New details wrap styled spans before scroll calculation, preserve final-line reachability and fit their height. Both locales, CJK/long values, current-vs-selected identity, empty/error/partial data, sanitized URLs and the Grok no-token contract have source/fixture coverage.
- Provider usage reuses the shared source-tagged projection, including Grok; no SQL was added outside its owner. Common-scale category magnitudes do not claim percentages, quotas or summed composition. Explicit all-time provider scope and approximate official-equivalent USD remain visible/reachable on narrow layouts. No provider label falls back to the unattributed aggregate.
- Tests include injected blocking readers/writers, stale generations, actual App transitions, actual rendered cells/colors and the existing lifecycle preflight/replay/ancillary/compensation/external-version adapter contracts. They are not solely snapshots of helper implementation. Legacy detail-wrapper tests were retained, while new workbench tests exercise the real rendering path.

## Findings (not fixed)

No unresolved product defect found in the approved scope after these fixes.

The planned optional usage load-time field is absent from the actual UsageDataset/UsageApp contracts. The coordinator accepted and documented its omission; no fabricated timestamp or new read model was introduced during review. Native terminal glyph/font/flush behavior and true startup-frame timing remain explicitly unverified; TestBackend artifacts and injected scheduling benchmarks do not establish those claims. Existing measured render results are a modest cost increase below 1ms in the documented debug fixture, not a render speedup. Final aggregate `just ci` remains the coordinator's gate, not a pass claimed by this review.

## Verification

Independent reruns after reviewer fixes:

- Lint / Rust TypeCheck: **pass**, `cargo clippy -p ccr-tui -p ccr-cli -p ccr-store -p ccr-types --all-targets --all-features -- -D warnings -D clippy::unwrap_used`.
- Workbench tests: **11 passed**, 1 intentional export ignore, `cargo test -p ccr-tui tui::ui::workbench::tests -- --skip export_bindings`.
- Whole-worker failure regression: **1 passed**, `cargo test -p ccr-tui failed_profile_worker -- --skip export_bindings`.
- History projection: **7 passed**, 1 intentional benchmark ignore, `cargo test -p ccr-store --test profile_activity -- --skip export_bindings`.
- CLI/type activity tests: **4 passed**, `cargo test -p ccr-cli -p ccr-types profile_activity -- --skip export_bindings`.
- Scoped `rustfmt --check` for reviewer-edited Rust files and `git diff --check`: **pass**.

Aggregate-CI follow-up, after the source-guard relocation:

- `cargo test -p ccr-cli application::profile_contract::tests::profile_adapters_keep_application_boundary -- --skip export_bindings`: **1 passed**.
- `cargo clippy -p ccr-cli -p ccr-tui --all-targets --all-features -- -D warnings -D clippy::unwrap_used`: **pass** (lint and Rust typecheck).
- `rustfmt --edition 2024 --check crates/ccr-cli/src/application/profile_contract.rs` and scoped `git diff --check`: **pass**.

The full TUI/workspace and unchanged adapter lifecycle suites are delegated to the coordinator's final `just ci` rather than duplicated during parallel review. Cargo used the existing external target cache via approved escalation; no cache configuration or unrelated filesystem data changed.

## Read-only gate-recovery follow-up: validate fixture environment

The coordinator's next aggregate attempt preserved in `ci-attempt-4.log` failed `validate::validate_accepts_registry_without_legacy_current_platform` at `validate.rs:101`: `ConfigManager::save` returned `FileLockError` because the lock-file directory was missing (Windows error 3). The commands binary reported 150 passed / 1 failed. The coordinator reported earlier and isolated passes; those passes alone do not prove a particular historical interleaving.

Reviewed the coordinator's two-line change to `crates/ccr/tests/commands/validate.rs` against `.trellis/spec/ccr/backend/test-fixtures.md`, `tests/commands.rs`, `tests/support/env.rs`, and the actual guarded-write/lock code. The spec explicitly requires a named `CcrIntegrationTestEnv` for the full manager-save scope even with an explicit target path: guarded writes resolve `CCR_LOCK_DIR`. The added `let _env = crate::setup_ccr_test_env();` spans the complete helper/save operation, holds the existing commands-binary mutex, keeps an isolated lock directory alive, and restores captured environment variables on drop. The explicit profile file target and existing subprocess environment assignments remain unchanged.

Review verdict: approved local fixture repair; no additional findings or product edits. No global test serialization, retries, changed assertions, or production behavior were introduced. The coordinator owns the default-parallel complete commands rerun and subsequent aggregate gate; this reviewer did not duplicate those checks or expand into other fixtures.

## Read-only gate-recovery follow-up: Tailwind at-rule lint configuration

The subsequent aggregate log `ci-final.log` records successful strict Rust checks/tests, release build, audit, governance and Tauri CI, then 69 `at-rule-prelude-no-invalid` failures on existing `@apply` declarations in `ccr-ui/src/features/configs/styles/app-settings.css`. The stylesheet has no diff from HEAD. Reviewed the coordinator's configuration-only repair against `ccr-ui/AGENTS.md`, the frontend spec index, package lint scripts, and the installed Stylelint implementation.

The sole addition to `ccr-ui/.stylelintrc.json` keeps `at-rule-prelude-no-invalid` enabled and adds `ignoreAtRules: ["apply"]`. The installed rule validates this option, checks the at-rule name before invoking its CSS lexer, and uses `optionsMatches` / `matchesStringOrRegExp`; plain `"apply"` uses strict string equality, not a wildcard or a broad regex. All other preludes still reach the normal rule. Existing `at-rule-no-unknown`, its Tailwind allowlist, standard-config inheritance, and component-specific overrides are unchanged. The inspected stylesheet uses actual Tailwind utilities via `@reference`, consistent with the installed Tailwind dependency.

Review verdict: approved narrow compatibility repair, with no additional findings or reviewer product edits. The coordinator reports passing direct API controls for valid Tailwind `@apply`, malformed `@supports` still rejected by `at-rule-prelude-no-invalid`, and unknown `@made-up` still rejected by `at-rule-no-unknown`; `bun run lint:style` also passed. These are coordinator-run results, not independently rerun gates. This reviewer verified the source behavior and diff; the coordinator owns the resumed complete frontend/aggregate validation.
