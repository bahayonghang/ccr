# Actual Profile workbench implementation evidence

Date: 2026-10-13. Approved design: revision-2 candidate A. These are synthetic Ratatui/TestBackend results, not a native terminal or personal-runtime verification.

## Changed surface

- `crates/ccr-tui/src/tui/ui.rs`: shared workbench integration, portable product labels, exact tab-title hit geometry, compact footer, shared typed field assembly and safe endpoint projection, regression fixtures and render benchmark.
- `crates/ccr-tui/src/tui/ui/workbench.rs`: actual table, runtime strip, single scrolling detail panel, cell-width wrapping, provider ledger, activation timestamps, geometry/security/state tests, and fixture cell exporter.
- `crates/ccr-tui/src/tui/theme.rs`: paired neutral Grok identity accent.
- App/background state and history projection were implemented by their respective sibling owners; this renderer consumes their cached data only.

All three Profile pages use the same table and A reading order: selected identity/status/tags, compact connections, provider totals and independent category bars, then up to three recorded activations. Current runtime remains independent of the selected row. Grok uses its typed activation state and configured credential source; session mode never claims a verified login. Claude's title is supplied by the updated App label owner. Header product names replace generic emoji.

The normal view hides unset optional fields. Expanded inspection retains them and the complete current runtime text. Narrow category displays retain numeric values without bars. Provider scope is all-time and cost is approximate official-equivalent USD. No profile-exclusive bill, quota, category percentage, inferred API timestamp or fabricated empty/error zero is introduced.

## Geometry and visual corrections

At 140×32, each Profile now has a 3-row header, 4-row runtime, 22-row workspace, and 3-row footer. The list's cached rectangle is the exact body: y=10 and height=18. In the previous Codex composition the list frame started at y=7 and exposed 20 inner rows; the two new header/rule rows explain the capacity change. The former Claude page also lacked the 4-row runtime strip. Mouse clicks on header/rule/border rows no longer map to records.

Column budgets include 3 cells per inter-column guide/gap. Tests exhaust every inner width 0..219 and heights 0/1/2/3/16/40. Profile/last-use columns survive smaller widths; model/description enter only when their complete column budget fits. Header tab hit tests follow actual rendered text, padding and dividers rather than equally dividing the terminal width.

Details wrap before computing scroll extent, including long CJK/Latin models and credential-sanitized endpoints. The bottom line remains reachable and oversized scroll offsets clamp. Short views show line-position indicators and a scrollbar. Empty content shrinks the frame. Compact footers use both allocated rows for actions instead of spending one on a decorative border.

Actual cell inspection found a dropped line-level style while joining wrapped connection values. Values now carry styled spans through wrapping, and an actual-buffer regression checks model/effort colors and bold weight. Exported previews include all six metadata tabs without initializing embedded auth apps.

## Verification

- `cargo test -p ccr-tui --lib tui::ui -- --skip export_bindings`: **83 passed**, 0 failed, 2 ignored (explicit benchmark/export utilities). Default parallelism retained.
- `cargo clippy -p ccr-tui --all-targets --all-features -- -D warnings`: **passed**.
- `rustfmt --edition 2024 crates/ccr-tui/src/tui/ui.rs crates/ccr-tui/src/tui/ui/workbench.rs crates/ccr-tui/src/tui/theme.rs`: applied to owned files.
- `git diff --check` on the tracked owned files: passed; Git only reports the existing Windows LF/CRLF conversion notice.

The focused suite includes existing composed Codex Auth quota/cost/error regressions, secret masking, Grok no-token rendering, and new real-cell geometry/style checks. New workbench tests cover all three platforms and EN/ZH at 60×16, 80×20, 89×22, 90×22, 100×30, 119×30, 120×22, 140×30 and 200×50. Additional pure tests cover future/relative time boundaries, zero/partial/overflow-safe bars, provider/source isolation, unavailable usage, full URL sanitization, selected-vs-current identity and complete scroll reachability. The injected usage loader panics if rendering tries to load data.

## Actual visual artifacts

The ignored `export_profile_workbench_cells` fixture exports Ratatui cell symbols, RGB foreground/background and modifier bits to `research/render-cells/*.json`. It is not a hand-positioned mockup. There are **34 exports**: Mocha/Latte × three platforms × EN/ZH × 200×50/80×20, plus Codex long/error/empty/scroll-end/zero states in both themes. The main session converts these actual cells to image artifacts and performs the final visual inspection.

Run each theme in its own process, filtered to this export test. Theme is process-global; ordinary parallel tests do not mutate it.

```powershell
$env:CCR_TUI_RENDER_OUTPUT='D:\Documents\Code\Github\ccr\.trellis\tasks\10-12-tui-profile-workbench\research\render-cells'
$env:CCR_TUI_RENDER_THEME='mocha' # repeat with latte in a separate cargo invocation
cargo test -p ccr-tui --lib export_profile_workbench_cells -- --ignored --nocapture --skip export_bindings
```

## Render measurements

Baseline revision: `8115cd11`. Windows, `rustc 1.98.0 (88d9e12ae 2026-08-18)`, Cargo **debug test profile** using the repository's existing build-cache target. The same `profile_workbench_render_benchmark` fixture builds 25/1000 in-memory profiles and an injected provider dataset. Five warmups precede 40 measured draws at each size; each draw includes TestBackend rendering/diff work. No timing assertion is added to CI.

| Profiles | Viewport | Before p50 / p95 µs | After p50 / p95 µs |
|---|---|---:|---:|
| 25 | 140×40 | 334 / 382 | 428 / 485 |
| 25 | 200×50 | 444 / 483 | 664 / 839 |
| 1000 | 140×40 | 352 / 390 | 446 / 523 |
| 1000 | 200×50 | 507 / 654 | 729 / 864 |

```powershell
cargo test -p ccr-tui --lib profile_workbench_render_benchmark -- --ignored --nocapture --skip export_bindings
```

The richer layout adds a small measured rendering cost and remains below 1 ms p95 in this debug fixture. It is not a render speedup. The measurement does not justify a second render cache or provider indexing layer. The important interaction improvement is the sibling asynchronous I/O/mutation work; see its separate evidence. No release-mode claim or universal hardware guarantee follows from these samples.

## Remaining limits

- Final cross-crate gate and Trellis review belong to the coordinating session.
- TestBackend proves composition and state contracts; native terminal font/glyph behavior and real user configurations remain untested here.
- Official SVG logos are not portable terminal characters. Product-name fallback is implemented, with no font/protocol probing or invented official glyph.
- History remains named-profile successful CCR activation history, subject to legacy attribution/retention limits. Exact timestamps explicitly show UTC `+00:00` and future times warn.

Checked: three platforms, both locales, both palette export processes, responsive sizes, actual colors, geometry, scope and secret boundaries. Changed: approved runtime/table/A detail surface and the exact geometry needed for its interaction. Left: native terminal smoke and coordinating full gates; no personal data was read or switched.

## Final integration review

The main session regenerated all 34 actual cell artifacts after the compact-header fix and inspected the wide Codex view, Chinese/light Claude view, compact Grok view, long strings, error/empty/zero states and final scroll position. Six platform-qualified tab labels now fit the narrow view; the shared measured spacing also determines mouse hits. Review additionally corrected the width budget for localized invalid category counts. Tests for the actual six-tab 60/80/100-column EN/ZH matrix and invalid counts passed.

The final exact strict gate includes `-D clippy::unwrap_used`; new fixture/export helpers now propagate errors rather than using unqualified unwraps. See `check-final.md` for the independent review and `integration-validation.md` for aggregate gate status. Subsequent changes to test error propagation do not change exported product cells.
