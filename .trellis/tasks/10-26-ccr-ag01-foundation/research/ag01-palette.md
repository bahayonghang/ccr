# AG01 palette and implementation boundary

This is the original candidate preflight, not the final palette. Final all-reading-role calibration darkened light inks, dark overlays and semantic text; `ccr-ui/src/styles/tokens.css`, `ccr-ui/DESIGN.md` and AG04 integration-summary own the current values. Original candidate calculations below are retained as provenance.

Prepared before product mutation while the baseline owner captures the original UI.

## Candidate palette (sRGB contrast preflight)

| Combination | base | elevated | surface | overlay | primary | secondary | muted |
|---|---|---|---|---|---|---|---|
| neutral light | #e7f1ee | #f2f7f5 | #ffffff | #e3ece8 | #24303a | #43515c | #5c6b76 |
| clay light | #eae6f2 | #f5f2fa | #fefcff | #e9e4f1 | #292d38 | #4c4d60 | #686477 |
| neutral dark | #10191b | #172225 | #1d292c | #2a383b | #f0f8f5 | #c5d5d0 | #9fafa9 |
| clay dark | #171722 | #20212e | #282a38 | #343746 | #f5f4fc | #d4d2e3 | #b0adbf |

Primary/secondary/muted against the solid surface respectively:
- neutral light: 13.47 / 8.17 / 5.50
- clay light: 13.48 / 8.11 / 5.60
- neutral dark: 13.84 / 9.82 / 6.53
- clay dark: 13.01 / 9.56 / 6.48

All exceed the existing 12 / 7 / 4.5 floors. This calculation is not runtime verification.

Default accent (existing clay key): light #187a56 / hover #146a4a / active #105b40 / white contrast; dark #85d5b1 / hover #9ce0c3 / active #6bc39a / #102b20 contrast. Light pair is 5.31:1; dark pair is 8.75:1. Secondary output: light #6d4bd6, dark #b9a4f4. Red/warning and platform identity families retain their roles; their surface/contrast values are rechecked against the new palette rather than blindly replaced.

Chart heat candidates: light #e4e9e7 / #b0bdb7 / #8a9f95 / #647f72 / #405f50; dark #394349 / #596971 / #7a8b91 / #9badb2 / #c0d0d3. Heat-0 contrast against the four card faces is 1.23 / 1.20 / 1.47 / 1.40, above the existing 1.15 floor. These are neutral sequential colors, not success/output encoding. Existing categorical source dot colors retain at least 3:1 against all proposed card faces (checked for the four newest sources). Full chart tests remain required.

## Minimum dedicated material tokens

Register eight new layer-1 names, no Tailwind namespace mirror:
- --surface-app-field-start / --surface-app-field-end: light #e7f6f1 / #ede7f8; dark #102b26 / #25213d.
- --surface-app-sheet-bg: light white 0.78; dark elevated RGB 0.90.
- --surface-app-sheet-solid: existing solid elevated/surface recipe used for fallback.
- --surface-app-sheet-blur: blur(20px) saturate(1.15), only the unique layout shell consumes this.
- --surface-app-sheet-border: light white 0.80; dark existing subtle border.
- --surface-app-sheet-shadow: one light 0 20px 48px rgb(70 90 120 / 8%) shadow; dark one low-key shadow.
- --surface-app-sheet-radius: 1.25rem (20px), only shell.

Eight counts treat field-start and field-end separately. Inset uses existing --space-3, so it does not need another token. Color + matching rgb values remain consistent; all existing shared bg/text/card/workspace/status roles remain opaque. Floating material becomes solid/no-blur. Existing radius and font value domains remain intact.

## Code ownership after BASELINE_READY

- tokens.css: value migration, minimal paper/field aliases, material reset and fallbacks.
- shell.css, App.tsx and MainLayout.tsx if needed: unique non-scrolling paper with gradient visible around it, existing content-scroll-area remains owner. No remount/key/data-flow changes.
- index.html, flavorPreview.ts, nativeWindowAppearance.ts: visual value mirrors only; all preference/native gating logic preserved.
- Theme/shell smoke tests: migrate old appearance values, add paper composition/fallback guards, retain behavior and thresholds.

Keep titlebar, dialogs, sticky and ordinary cards solid. App.tsx currently paints bg-bg-base over StageBackground, so the gradient must be attached to the actual app field rather than only the background component. The independent tray keeps its compact solid surface.

Main session owns DESIGN.md, AGENTS.md, spec and sidecar synchronization. Product edits wait for explicit baseline completion.
