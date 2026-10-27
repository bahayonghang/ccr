# AG04 integration review

## Findings fixed

- `src/ui/{checkbox,select,combobox}.tsx` and `src/features/configs/components/ConfigCard.tsx`: existing low-alpha focus rings overrode the newly strengthened base focus treatment. Changed only the ring color to the solid accent; existing widths, offsets, component behavior and geometry remain unchanged. Parent approved this local correction. Runtime focus-state screenshots are separate from screenshots taken before this correction.
- `src/styles/utilities/utilities.css`: removed a contradictory reduced-transparency comment that still instructed retaining blur. The existing reset implementation was not changed by this review.
- The review identified strict text-tier gaps on actual base/overlay reading roles. Main-session calibration strengthened light primary/secondary inks and slightly darkened dark overlays, mirrored RGB/preview/docs, and extended the existing text guard across all four solid reading roles. This preserves the 12/7/4.5 thresholds rather than narrowing the promise to card surfaces. Main's final theme receipt covers 13 files / 114 tests.

## Findings not fixed

- **Pre-existing global confirmation focus return defect.** AG02 reproduced the actual `ConfigCard` Switch → global warning confirmation → Escape path with an isolated memory-only QueryClient fixture. After two seconds focus remains on `BODY`, in all four theme/flavor combinations. Evidence: `../../10-26-ccr-ag02-workspaces/evidence/populated/2026-10-26T16-46-23.032Z/results.json`. `base-modal.tsx` is unchanged; the only `dialog.tsx` behavior-adjacent difference is removal of scrim blur. The main session chose to preserve the frozen modal behavior in this visual task. Focus trap/Escape/cancel passing must not be reported as focus-restoration passing.
- **Pre-existing independent i18n count drift.** `bun run check:i18n` exits 1 because `scripts/check-i18n.mjs` expects 4501 leaves while both locales contain 4510. `tests/i18n.test.cjs` already expects 4510 and the canonical `bun run test` i18n lane passes. The checker and locale files have no diff against HEAD. Main instructed leaving this unrelated governance mismatch unchanged. Raw failure: `final-checks/check-i18n.log`.
- **Canonical architecture self-check launcher failure.** `bun run check:arch-boundaries` cannot launch the installed Scoop `bunx` shim. The task-only copy substitutes only the subprocess launcher with installed Node + local ESLint, retaining the same generated config, fixtures, policies and expected errors. All four boundary fixtures pass that equivalent no-install check. The original command remains failed; do not label its canonical invocation PASS. See `final-checks/arch-launcher-adaptation.json`, `arch-local-launcher.log` and both command receipts.
- Native WebView, production CSP enforcement, actual native DPI/tray/macOS, unsupported-backdrop engine behavior and populated IPC/chart/virtual-list performance require separate runtime evidence. Existing browser/unavailable states, source guards or mocks do not establish these results.

## Verification

Checks use existing Bun 1.4.4 through process-local PATH only; no installation or package/lockfile changes. Test worker configuration and coverage thresholds are unchanged.

| Check | Result | Evidence |
|---|---|---|
| Full `bun run test` | PASS, 170 files / 918 smoke tests; i18n lane PASS | `final-checks/test.log`, `commands.json` |
| `bun run test:smoke --coverage` | PASS, lines 75.23% (13074/17377) | `final-checks/coverage.log`, `coverage-command.json` |
| Final focused UI/config checks after focus-ring correction | PASS, 19 files / 87 tests | `final-checks/final-controls-focused-ui-configs.log` |
| Final `bun run type-check` | PASS | `final-checks/final-controls-type-check.log` |
| Final `bun run lint` | PASS, 0 errors / 2 pre-existing warnings | `final-checks/final-controls-lint.log` |
| Final `bun run build:with-budget`, including the last filter-wrapper correction | PASS, all budgets retained | `final-checks/final-filter-wrap-build-with-budget.log`, `final-filter-wrap-build-command.json` |
| `bun run check:cycles` | PASS, 738 files, no cycles | `final-checks/check-cycles.log` |
| Architecture equivalent local launcher | PASS, 4 expected boundary violations detected | `final-checks/arch-local-launcher.log` |
| `bun run check:style-lines` / `bun run docs:audit` | PASS | `final-checks/check-style-lines.log`, `docs-audit.log` |
| `git diff --check` | PASS after reviewer changes | tool receipt; no source autofix or whitespace suppression |

The full suite and coverage were collected before the final value-only palette strengthening and focus-ring correction. The final theme/UI/config checks plus type/lint/build cover those subsequent changes; coverage was not rerun for CSS/token/class-color-only updates. AG02's last approved filter-wrapper layout correction passed its own 4-file / 15-test narrow receipt and targeted ESLint/stylelint; the final build also includes it. The main session's final browser pass must include this correction.

### Final toolbar review handoff

After the build receipt above, the main session observed the refresh action still clipping in the populated 1200px screenshot and applied one final four-file correction: `ConfigsView.tsx`, `components/ConfigFilters.tsx`, `styles/config-filters.css`, and `styles/configs.css`. Read-only review found no actionable issue: the filter root now has `min-w-0 flex-1`, refresh does not shrink, labels remain unbroken while groups wrap, icons use semantic colors, and search/sort boundaries and focus indicators use solid tokens. No event handler, form registration, filter/sort calculation or query behavior changed. The main session owns the final targeted config/type/lint/build receipts and refreshed 1200/800/390 screenshots for this last correction; the earlier build must not be described as covering it.

AG02 separately verified the final opaque Checkbox/Select/Combobox focus rings in all four combinations, 12 samples / 12 PNGs: `../../10-26-ccr-ag02-workspaces/evidence/focus/2026-10-26T16-55-14.532Z/results.json`. The harness waits for the color transition to settle and requires the exact opaque accent RGB; its earlier immediate sample remains retained rather than overwritten.

ESLint warnings remain `scripts/measure-distribution.mjs:44` (`prefer-const`) and `tests/checkin/checkin-accounts-tab.smoke.test.tsx:82` (unused `editLabel`). No suppression or unrelated cleanup was introduced.

## Scope and acceptance

Reviewed runtime-source differences are palette/material values, solid reading surfaces, field/focus/badge classes, first-frame/native appearance constants, and the single shell pseudo-element. No Rust/SQL/API/IPC, query keys, form payloads, command lifecycle, chart options/series or virtual-list measurements were changed. The removed `providerKind` helper became unused solely because its hardcoded presentation palette was removed; filtering/counting logic remains unchanged.

Root `just frontend-check` / `just ci` include installation commands, which are outside authorization; they remain NOT_RUN rather than being replaced or edited. The no-install frontend subgates are recorded independently above. This review does not claim desktop final acceptance, native verification, full keyboard restoration, or performance regression acceptance; the main session owns the final route/material/performance evidence and the R1–R8 decision.
