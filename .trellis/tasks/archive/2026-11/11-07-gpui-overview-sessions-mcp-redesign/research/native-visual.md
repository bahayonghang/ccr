# Native visual receipt — continuation

Code: `a145abba166eef4828efdaeba8bccf2b7783cf0f`. Release hash and complete gate results: `integration.md`.

## Actual coverage and verdict

**Implementation/check complete; native acceptance incomplete. Keep the task in progress.**

The final release was opened through Computer Use after the user requested a retry of the initial application-approval timeout. The three final pages were observed in light/en at a wide window:

- `native/after5-overview-light-en-initial.png` (capture 2048×1104).
- `native/after5-mcp-light-en-wide-observed.png` (capture 2048×1104).
- `native/after5-sessions-light-en-wide-observed.png` (capture 2048×1104, after loading completed).

These are captures of the real final release and local data. They are not synthetic UI mockups. The capture API normalizes image dimensions; saved window geometry changed during concurrent user activity. **No claim is made that these frames are exactly 2000×1080 or 1280×800 logical pixels.** The first MCP capture (`after5-mcp-light-en-initial.png`, 2042×1152) is additional uncontrolled evidence, not a matrix cell.

The MCP list ends after its six real rows, and the unselected right pane ends after its short explanation; the large white reading panels no longer stretch to the column bottom. In the 2048×1104 observed image the list is approximately y=237–585 and the empty pane y=237–387, with the outer column continuing toward y=1055. This establishes the qualitative content-sizing improvement at the observed width. It does not replace the requested controlled-size acceptance. Sessions has 100 real entries, so its long list appropriately fills and scrolls; its unselected right pane is approximately y=232–364 and leaves the scene visible below it.

Overview's daily chart ends shortly below its axis labels (approximately 20 normalized capture pixels), and Activity/Data source content reaches the first-screen boundary. Exact logical-pixel measurements of the chart gap and first-screen bottom gap remain **not verified**, because the capture scale/window size was not calibrated. The headless geometry assertions passed; these are separate kinds of evidence.

The captured Overview has an external floating input toolbar over a small part of the comparison panel. A yellow capture indicator/cursor markers are also tool artifacts. They are not CCR UI defects. No foreign-application screenshot was saved into task evidence.

The independent fresh-context review used only these screenshots and DESIGN.md; see `native-fresh-review.md`. Its `fix` verdict flags a missing explicit local-timezone label and otherwise applies only to the supplied light/en wide initial states. The source-audit disposition is below.

## Capture boundary and preference recovery

After initial successful captures the tool repeatedly reported `user input was detected in this window`, and a state request returned a different foreground application's picture despite the selected CCR window still being listed. One window-selection/activation recovery produced correct CCR captures; subsequent control/capture became unstable again. Further coordinate/foreground cycling was stopped.

This follows the native handoff boundary in [ui/SKILL.md](C:/Users/lyh/.agents/skills/ui/SKILL.md), expanded in [mode-screenshot-iteration.md](C:/Users/lyh/.agents/skills/ui/references/mode-screenshot-iteration.md): “Make one bounded automation attempt. If it is flaky” hand off the specific screen. This limit applies to the remaining visual automation, not to implementation or build verification, which are complete.

Before the successful reopen, the temporary preference preparation was restored byte-for-byte in the settings helper's `finally`. Original/restored SHA-256: `5a72f2dd62026dd21d86ea210454255007265db4045cc161135dad4958c5b7f0`. Later user geometry changes were not overwritten. The final release instance was left available for inspection.

## Not run / not established

- Controlled three-page light/dark × 2000×1080 and 1280×800 logical-pixel matrix.
- Controlled 960×640 native capture, including first Sessions row <45% of window height. The corresponding headless regression passed.
- Chinese final-release screenshots; native dark-theme pixels.
- Calibrated native Overview chart/bottom ≤24px gap measurements at both requested sizes.
- Final-release selected detail/transcript, batch/editor, empty-trash and error-state pixel inspection. Automated source, masking, focus, failure, paging and geometry regressions passed.
- Actual end-to-end keyboard navigation through page controls; a quit/cancel interaction is not that acceptance path.
- Screen-reader/UIA acceptance. An accessibility-tree query showed the expected search description and separate filter group names; this does not establish assistive-technology usability.
- IME composition, other DPI settings, macOS/Linux and cross-platform native acceptance.

## design-atlas judged checklist §6

Read the installed `C:/Users/lyh/.claude/skills/design-atlas/references/anti-slop.md`. `slop_check.mjs` scans HTML/CSS/JSX; this product is Rust GPUI, so its mechanical scanner is **not applicable**, not a passing zero-findings run. The following is a scoped manual walk-through, supplemented by source/tests and the independent screenshot review.

| ID | Result and evidence boundary |
| --- | --- |
| 8 | pass: usage insight, MCP source management and indexed transcripts are product-specific utility sequences. |
| 9 | pass: surfaces, spacing and hairlines distinguish sections; no repeated ornamental dividers. |
| 46a | pass: screenshots show existing local usage/config/session data; no invented metrics or customers. |
| 54 | pass: dates, scope/source identifiers carry facts; no decorative eyebrow labels. |
| 58 | n/a: the 80-word marketing-hero ceiling does not fit a data workbench; headings and utility hierarchy are reviewed instead. |
| 63 | pass: middle dots separate live source/state and record metadata, as DESIGN.md specifies. |
| 69 | pass with known gap: MCP/source counts repeat where documented; M6/M7 remains below. |
| 70 | pass by source/tests: mutations use real existing owners; no new fake-success form. Native submission was not exercised. |
| 84 | pass: actual release captures. |
| 85 | n/a: no marketing footer/signup. |
| 3 | pass: one KPI band and a list/detail composition, no equal icon-tile wall. |
| 6b | n/a: no centered marketing hero. |
| 11 | unverified native hover matrix; existing controls retain their normal immediate interaction styles. |
| 29 | pass against DESIGN.md: static mint/lavender scene is the approved material; no new animated orb or overlay was introduced. |
| 38iii | pass: neutral headings, no highlighted heading word. |
| 40 | pass in token tests for both palettes, including neutral sparkline ≥3:1; native dark pixels remain unverified. |
| 41 | pass by theme/source contracts and light capture; native dark pixels unverified. |
| 44b | unverified at controlled 1280×800; wide capture shows heading, actions and primary content. |
| 47c | pass: GPUI is the actual product, not styled imitation chrome. |
| 49a | unverified native supported-size matrix; 320px is outside the desktop product's 960px minimum. |
| 65 | pass: follows the existing airy-glass workbench, no unsolicited cream/serif/clay recipe. |
| 66 | pass: labels communicate scope, state, identity or actions. |
| 68 | n/a: no stock-image/placeholder grid. |
| 73 | unverified across sizes; known minimum-window sidebar clipping remains out of scope. External input toolbar obscures a small Overview region in the capture. |
| 76 | pass for visible bundled icons; no new remote image assets. |
| 80 | n/a: no drawing pretending to be a photograph. |
| 83 | n/a: no photography. |
| F4 | pass: existing primary action treatment; secondary actions are ghost. |
| F21 | pass by source/semantics and review: one h1 per page. |
| F23 | n/a: native GPUI, no HTML metadata; window title/language use the native owners. |
| F26 | unverified final Chinese/default-size wrapping; 375px is unsupported. |

## Fresh review finding disposition

The independent reviewer rated the absent Overview timezone label P2. Source audit confirmed `OverviewView::zone()` appends ` UTC` for `ReportTimezone::Utc` and an empty suffix for `Local`; both header and rail reuse it. The behavior already exists in `dd8956ff^` before this repair, and current tests explicitly expect no Local suffix. Runtime freshness/date conversion correctly uses `ReportTimezone::Local`. The owning source review therefore classifies it as a pre-existing P3 presentation/contract gap, not a wrong-timezone calculation or fabricated freshness. It remains open rather than widening handoff §5.1. A later fix can localize a `(local time)` suffix without changing the runtime/report DTO.

## Deliberately retained gaps

These remain the handoff's out-of-scope list, not newly accepted behavior:

- O1: Activity title/grid separation at wide sizes; O7: heat map plus streak counts can still look templated.
- Additional existing gap: local report dates/sync times omit an explicit local-timezone suffix (source-audit disposition above).
- O3: launch lens focus ring from the previous material/focus task.
- O8/S8: CJK fallback may render weight 500 as bold.
- S5: Chinese Kimi chip wrapping at 1280 remains unverified after the narrow-chip change.
- S11: raw variant identifiers remain visible for disambiguation.
- M5: no edge fade on the minimum-width source-health strip.
- M6/M7: repeated counts and different MCP/Sessions detail headings follow the existing design.
- X1: minimum-height sidebar Usage item may be covered by the Settings dock.

## Next acceptance step

Capture the final release at the requested logical dimensions with a stable target window and run the remaining native checklist. Use the existing `research/tools/capture.ps1` procedure in a later authorized native session, or provide those screenshots for comparison. Do not archive on the strength of the headless gate or these three uncontrolled-size captures alone.

## User-requested archive — 2026-11-09

The user explicitly requested committing all changes and archiving this task. The earlier in-progress/do-not-archive notes are historical and superseded by that instruction. Implementation and the recorded desktop gates passed. The unchecked native theme/size/Chinese matrix, calibrated gap measurements and other native acceptance remain **UNVERIFIED**; archival does not turn them into passes. Preserve the unchecked acceptance items and remaining gaps for future work. No push.
