# Profile workbench integration validation

Date: 2026-10-13. Scope: the approved A design for Codex, Claude and Grok Profile. Implementation is present in the shared workspace; no commit, push or personal-profile activation has been performed.

## Acceptance evidence

| Contract | Evidence |
| --- | --- |
| Runtime independent of selected row | Typed platform runtime snapshots; actual three-platform EN/ZH renders; current-vs-selected regression |
| Aligned table and last used | One measured geometry for headers/rule/body, pagination and mouse hits; exhaustive column budgets and responsive TestBackend matrix |
| Honest activity data | Store/facade/DTO tests for exact platform/name, successful Switch, strict RFC3339 UTC ordering, missing/schema/error/future/invalid data, bounded three-record projection and read-only storage |
| A right panel | Identity, compact connections, provider totals/category magnitudes and vertical activations; normal unset suppression; wrapped scroll reachability and semantic color tests |
| Platform identity | Claude Profile English title, three product-name fallbacks, platform-specific colors; six distinguishable tabs in measured compact headers |
| Responsive safe mutations | Delayed injected loaders/writers, frozen target, one writer, stale-read generations, all quit keys, initialized Auth cache preservation and existing actual lifecycle adapter tests |
| Rendering and selection do no I/O | Counted injected loaders and panic-on-render usage fixture; settled idle ticks do not load or redraw |
| Source/secret boundaries | Existing usage projection with Grok source, provider-level all-time estimate labels, URL sanitization and credential sentinels; Grok never displays an inline token, even masked |

Independent review is recorded in `check-history.md` and `check-final.md`. The main session also inspected actual renderer images and found the wrapped-span color loss and narrow-header ambiguity, both now covered by regressions.

## Visual proof

`render-cells/*.json` contains 34 exports from the actual Ratatui TestBackend. `render-actual-cells.py` converts their symbols, positions, foreground/background colors and modifiers into `rendered-tui/*.png`; it does not reconstruct the layout. The fixture exporter presents all six tabs without initializing Auth I/O.

Final visual inspection covered Codex/Claude/Grok, EN/ZH, Mocha/Latte, 200×50 and 80×20, plus long content, errors, empty datasets, zero values and scroll end. Cell assertions cover additional 60×16, 89×22, 90×22, 100×30, 119×30, 120×22 and 140×30 sizes. The header regression separately checks all six labels and hit targets at widths 60, 80 and 100 in both locales.

Useful previews:

- [Codex wide](rendered-tui/codex-en-mocha-200x50.png)
- [Claude Chinese / light](rendered-tui/claude-zh-latte-200x50.png)
- [Grok compact](rendered-tui/grok-en-mocha-80x20.png)
- [Wrapped content](rendered-tui/codex-en-mocha-long.png)
- [Final scroll position](rendered-tui/codex-en-mocha-scroll-end.png)

These are actual buffer renders with isolated data, not native-terminal captures. Native font/glyph appearance and personal runtime correctness remain unverified. Official graphic logos were not introduced: the approved portable product-name fallback avoids requiring a special font or image protocol.

## Performance evidence and limits

See `async-implementation.md`, `history-implementation.md` and `visual-implementation.md` for commands, sample counts and environments.

- Same 10 ms delayed platform fixture, 30 reload samples: main-thread acknowledgement p95 31.524 ms before versus 0.029 ms after; reads decreased from 90 to 30 because only the affected platform reloads. Data-ready p95 was 12.089 ms after.
- A 1 s delayed writer with 1,000 profiles, 30 samples: Apply submission p95 0.078 ms, completion propagation p95 7.771 ms, whole synthetic operation p95 1008.276 ms. Exactly 30 writes; browsing continued during each write. This proves responsiveness, not faster durable writes.
- Read-only 100k-row history fixture, 1,000 requested names, 30 warm-cache fresh connections: p95 61.017 ms. No equivalent old batched API existed, so no query speedup is claimed.
- 1,000-profile debug TestBackend render: p95 523 µs at 140×40 and 864 µs at 200×50. The richer layout costs more than the old layout (390/654 µs respectively); the evidence did not justify another display cache.
- No cold-disk, native first-frame/terminal-flush, or real durable-transaction before/after measurement was taken. Those planning targets are not presented as achieved results. Rendering/selection/idle I/O counts and transaction safety are deterministic test contracts.

The planned optional usage load-time field is omitted because the actual UsageDataset and UsageApp expose no such timestamp. No timestamp or additional state was invented to fill the panel.

## Gate status

The integrated `just ci` run passed its first 12 gates, including strict Clippy, all workspace tests, Release, security audit, governance and complete Tauri CI, then exited non-zero at frontend Stylelint; see `ci-final.log` (17:44.513). Workspace tests included the default-parallel commands binary (151 passed) and TUI unit tests (274 passed, four intentional ignored export/benchmark cases). Binding exports ran in their separate gate.

After the narrow configuration repair below, the unchanged standard CI suffix passed (exit 0) as `just frontend-check frontend-audit frontend-coverage vscode-ci`; see `ci-frontend-recovery.log`. No Rust/product source changed after its successful gates. The original aggregate failure is retained as evidence; it is not relabeled as a successful command exit.

Aggregate attempts exposed four issues, all repaired before their respective reruns:

1. New test-only `unwrap()` calls violated the stricter repository lint flags. Tests now use error propagation/contextual fixture assertions; exact strict Clippy passed without lint suppression.
2. An existing source-boundary assertion expected synchronous App execution. It now follows the captured request through the blocking worker to the same lifecycle owner, retaining no-direct-write/no-off-before-apply guards. The focused regression and strict CLI/TUI gate passed.
3. An existing validate test used `ConfigManager::save` without the required named environment guard, allowing concurrent `CCR_LOCK_DIR` changes to invalidate its lock directory. A two-line fixture repair reuses the existing commands-test environment guard. The full 151-test commands suite passed independently and again in this aggregate run. No production behavior, retries, assertions, global parallelism or coverage thresholds changed. Independent review is appended to `check-final.md`.
4. Frontend Stylelint interpreted the existing Tailwind `@apply` arguments using standard CSS grammar, producing 69 errors in the unchanged settings stylesheet. Both stylesheet and lint configuration matched HEAD before the repair. `ccr-ui/.stylelintrc.json` now declares only `apply` in `at-rule-prelude-no-invalid.ignoreAtRules`, following the installed rule's supported option. A direct lint-API control accepts `.fixture { @apply flex min-w-0; }`, still rejects malformed `@supports` with that same rule, and still rejects an unknown at-rule with `at-rule-no-unknown`. `bun run lint:style` passed, and independent source review approved the exact exception. No frontend appearance or dependency changed.

All 16 standard CI recipes have now passed across the aggregate run and the resumed suffix. The final source differs from the first 12 successful gates only in the reviewed frontend Stylelint configuration; those unchanged Rust gates were not repeated. Frontend smoke and coverage each passed 170 files / 906 tests; VS Code passed 51 tests and its VSIX package audit. Coverage thresholds, audit policy and default parallelism are unchanged.

| Gate group | Final evidence |
| --- | --- |
| Version, formatting, OMP, harness, Copilot | Passed in `ci-final.log` |
| Strict Clippy, workspace check, workspace tests | Passed in `ci-final.log` |
| Release build, security audit, CI governance, Tauri CI | Passed in `ci-final.log` |
| Frontend checks/build/docs, dependency audit, coverage, VS Code CI/package | Passed in `ci-frontend-recovery.log`, suffix exit 0 |
| Independent full-scope review and narrow recovery follow-ups | `check-final.md`, no unresolved findings |

Trellis parent/child implementation and acceptance records are reconciled. Their workflow status remains `in_progress` pending a user-directed commit/archive; implementation and verification metadata are complete. No commit, push, installation or personal-profile activation was performed. The release binary is available at `target/release/ccr.exe`. Unrelated build-cache specification and journal changes present at session start are preserved.
