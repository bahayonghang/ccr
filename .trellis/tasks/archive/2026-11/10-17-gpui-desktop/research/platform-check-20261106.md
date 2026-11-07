# Platform source and headless review — 2026-11-06

Independent Trellis check of the existing uncommitted platform batch on `dev`
(dispatch HEAD `9637696`). This review did not launch a native app or operate the
computer. The user's later native-check deferral takes precedence over the
earlier handoff's screenshot instructions. No size/DPI matrix was resumed.

## Findings (fixed)

- **P1 — Claude collection confirmation did not enter a decision button.**
  `ccr-desktop/src/features/resources/claude_files.rs` focused its trap container
  during render and never advanced to the safe decision. The real-control
  regression failed at the initial Cancel-focus assertion (`Some(false)` versus
  `Some(true)`). The same editor now schedules entry after mounting, enters
  **Keep editing**, uses a non-tab-stop grouped trap, and restores the captured
  prior input on Escape/cancel. A separate confirmation generation invalidates
  callbacks on dismiss, reopen, confirm and leave. The confirmation has an
  explicit labelled AlertDialog role. No global modal implementation changed.
- `claude_files/tests.rs` extends the existing source-write test, retaining its
  original file backup, CAS conflict/draft and private/unknown-field assertions.
  Added assertions cover early Enter without a write, initial safe focus,
  Tab/Shift+Tab containment, Escape/input restoration, and a raw-mouse
  dismiss/reopen with callback-before-draw ordering. The stale callback cannot
  steal focus before the newly mounted decision schedules its own entry.
- **P2 — Observer's ordinary table clipped long identifiers without a visible
  full-value channel.** `features/insights/observer/presentation.rs` now uses the
  installed Kit `Tooltip` API on cells, retains full accessibility labels, and
  right-aligns numeric headers/cells with the existing mono font token. Text
  columns retain the normal body family. No table framework was introduced.
  This final presentation change awaits the coordinator's stable aggregate
  compilation/tests; its native tooltip appearance was not exercised.

## Findings (not fixed)

- **Acceptance evidence remains open.** Current native rail/workbench appearance,
  CJK fallback rendering, hover/focus pixels, actual accessibility announcements,
  and default-window composition have no new native receipt in this review.
  This is not a visual ship PASS. Windows headless tests do not establish macOS
  or Linux loading/interaction, glass support, or performance budgets.

## Prior monitoring findings closed

Both earlier findings are closed by the actual storage/read path:

1. `crates/ccr-db/src/database/repositories/log_repo.rs:223`
   `query_logs_strict` collects row Results instead of discarding decode errors.
   It shares `filtered_query` with legacy `query_logs`; existing permissive
   Tauri callers keep their prior behavior.
2. `log_repo.rs:28` strictly parses RFC3339 time and propagates conversion failure.
   The native connection-bound seam in `services/log_persistence.rs:36` calls
   this strict function; `runtime/monitoring.rs` exposes `Error::Storage` while
   retaining real memory events. It cannot turn corrupt persisted timestamps
   into current-time events.

The coordinator's supplied root regression receipt is **1 passed** for
`strict_query_rejects_corrupt_rows_and_time_without_changing_legacy_reads`.
The native test `bounded_feed_redacts_before_storage_and_reports_decode_errors`
asserts Storage for both corrupt message and invalid timestamp. Its existing
5-test monitoring receipt predates the final labelled ListBox wrapper; that
wrapper needs the coordinator's final aggregate gate, not an inherited pass.

## Source review conclusions

- `shell/platform_navigation.rs` retains all 13 destinations, uses one Lucide
  icon set, and exposes actual platform capabilities. `shell/mod.rs` parks
  hidden views; remote selection returns the remote boundary instead of opening
  local platform actions. Workbench contexts route to existing feature entities.
- `features/workbench.rs` resets state and generation on platform changes.
  Profiles, credential state, usage and Claude Observer use captured domain
  owners. Credential detection explicitly does not assert remote authentication.
  The 30-day usage window is local-calendar based, identifies recorded USD cost,
  and keeps read errors distinct from a successful empty report. No quota or
  session count is inferred from request totals.
- `runtime/platforms/rolling.rs` refuses remote/non-Codex requests before root
  resolution. It reads the captured Codex root on the bounded worker, rejects
  linked/reparse ancestors and nested entries, and scopes derived-cache locks.
  `workbench/rolling.rs` makes the cache refresh an explicit action, labels
  5-hour/7-day/all-history usage records separately from account quota, states
  the UTC observation time, and exposes missing/partial/cache diagnostics.
- Claude common forms submit field patches through the existing source-bound
  owner; blank/omitted values retain existing values, secrets are not preloaded,
  and advanced patching remains explicit. Ordinary Save/confirmation controls
  precede variable-height common fields. Leaving clears field/editor buffers.
- Observer tables use the real shared DTO keys. Unknown values render as a dash,
  usage and main-DB tool errors remain separate, and the native adapter omits
  legacy unsupported token/model placeholders. Its observed-session ranking is
  correctly labelled as the owner's fixed 30-day window. Read/DB work is outside
  render; opening this presentation does not launch an import.
- Theme remains the single Rust token owner with an opaque sheet. Shadow blur
  is not claimed as background glass. System font selection is source evidence
  only; actual fallback glyph selection needs native inspection.

## Design-atlas judged review

Read the user-specified `C:\Users\lyh\.skillsmanage\skills\design-atlas\SKILL.md`
and `references/{design-system-core,implementation,anti-slop,review-prompt}.md`,
the current native DESIGN, task PRD/design/implement and latest handoff. Existing
`style:airy-glass` remains the design source; no theme replacement or waiver was
introduced. This is independent **source/headless** review, not the screenshot
review described by the reference prompt.

**Verdict: fix — source-only; visual verdict not_run.**

- Source composition is a platform current-state section, bounded usage reading
  and real management actions, rather than generic equal feature tiles.
- The confirmed keyboard defect and Observer identifier/value treatment were
  fixed locally. Actual composition and glyph/focus/tooltip rendering remain
  unjudged without native screenshots.
- Judged gates 8/9/3/6b/29/38iii/47c/65/66/68/80/83/85: source checked; no generic
  marketing composition, fabricated material, image slots or ornamental tiles
  were found. The version line identifies the installed package.
- Gates 46a/69/70: source checked; metrics and forms trace to actual domain
  projections/mutations. The synthetic fixture controls remain fixture-only.
- Gates 11/41/F4: source checked; Kit action states use the shared paired tokens.
  Gate 40 has existing exact palette contrast assertions in `theme.rs`, whose
  final run belongs to the aggregate gate; source inspection is not a pixel test.
- Gates 54/58/63: native operational labels and reports carry source/window
  facts; marketing word ceilings are not a meaningful native data-table budget.
  These source observations do not waive access, correctness or honesty floors.
- Gates 44b/49a/73/F26 and visual portions of 40/41/F21: **not_run** because they
  require current native layout/focus/font screenshots. The 320px/mobile clauses
  do not override this desktop's explicit minimum or the deferred matrix.
- Gates 76/84/F23: image/page-metadata checks are not applicable to these native
  Rust surfaces. No browser/HTML mock is offered as delivery evidence.

Mechanical scan actually run:

```text
node C:\Users\lyh\.skillsmanage\skills\design-atlas\scripts\slop_check.mjs ccr-desktop/src --design ccr-desktop/DESIGN.md --scope app --json
files=0, fail=0, reflex=0, warn=0, note=0, exit=0
```

**Coverage gap:** the scanner does not scan Rust/GPUI. Zero scanned files is no
mechanical or visual pass. No waiver was used to hide this limitation.

## Verification

- Actual new regression: pinned Rust 1.98.0, workspace-local desktop target,
  `python scripts/desktop.py cargo test --locked --lib shell_claude_resources_write_confirm_conflict_and_keep_private_fields -- --skip export_bindings`
  **exit 0, 1 passed**, after the intentional initial red assertion. Default
  test parallelism and the binding-generation boundary were retained.
- Scoped rustfmt on the three edited files: **pass**. First attempt encountered a
  transient Windows mapped-file write error; the subsequent attempt succeeded.
- Scoped `git diff --check`: **pass**.
- Lint/type-check/full desktop tests/updated 1 MiB guard: **pending coordinator's
  single final stable aggregate run**, deliberately not duplicated here.
- Supplied receipts independently inspected by assertion, not added as disjoint
  counts: native_shell 9/9 predates later changes; rolling 3/3 means two runtime
  tests plus one actual-control test; Claude common controls 1/1 overlaps the
  updated regression above; Observer 2/2 includes a synthetic DTO table test and
  a separate real subscription write test. The prior 12-child 1 MiB guard
  overlaps these controls and predates this focus change.
- An early compile attempt hit concurrent session-helper visibility and
  monitoring stateful-role issues; their owners corrected them before the
  passing regression. The passing build still reported an in-flight MCP form
  unused-import warning, which was sent to its owner. No exception or warning
  suppression was added here.
- Native screenshots/IME/UIA, external network/SSH/WSL, package/release, full
  `just ci`, performance sampling and OS matrices: **not_run by this reviewer**.
  No real user data or HOME/USERPROFILE was changed.

No new spec contract was needed: the existing shared-desktop confirmation
contract already requires mounted, generation-guarded safe-button focus. This
fix brings Claude collection controls into that existing rule. Session delivery,
the new ordinary platform/MCP forms, tools/layers/batch implementation and their
delivery notes are owned by other workers and excluded from this review.
