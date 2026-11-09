# Fresh-context visual review: triage (2026-11-08)

The reviewer saw only the screenshots in `research/native/` and `ccr-desktop/DESIGN.md`, with no implementation conclusions. Page verdicts:
- Overview: pass with nits.
- Agent sessions: pass with nits, but too tight in narrow windows.
- MCP: the redesign passes, but at wide windows the large blank areas remain, so the user's core complaint is not resolved.

Stale evidence: `after2b-overview-light-en-2000x1080.png` (no `L` suffix) comes from an older build. It shows wording that was later removed ("not imported after"). Judge the Overview from the `*L.png` files; the old image has been deleted.

## Fixing in this round

| # | Problem | Fix |
| --- | --- | --- |
| M1 / S2 / S13 | At wide windows the list panel stretches to full height with nothing below 6 rows, and the empty pane and empty trash are large white panels | Size list panels to their content. Empty states (unselected pane, empty trash) also size to their content, so the background shows through below them. Only real content (detail, editor, transcript) stretches the pane and scrolls inside it |
| M2 | Disabled ghost buttons become gray filled pills and look more clickable than enabled ones | Header and toolbar ghost buttons on MCP and Sessions keep a transparent background and muted text when disabled. The global `semantics::button` is not changed |
| M3 | Batch: no heading for the preset group, and "Replace existing entries" reads like a fourth destination | Add "Preset" and "Options" headings |
| M4 | Inconsistent icons in the detail action row | Either give all three buttons icons or remove them all, depending on which icons Kit provides |
| O2 | Overview "Refresh" is an outlined pill, while the same button on MCP and Sessions is ghost with an icon | Make it ghost with the `refresh-cw` icon |
| O4 | "Install llmusage" still appears after data has loaded | Relabel to "Install or update llmusage" |
| O5 | The stale warning dot in the header has no visible explanation | Show the explanation in a tooltip |
| O6 | Neutral bars in the platform sparkline are about 1.4:1 against the background | Raise neutral bars to ≥3:1 and add a contrast test |
| S1 | The 40px icon block appears only on Sessions, so the h1 jumps 65px between pages | Remove it; all three pages share one h1 baseline |
| S3 | The two segmented controls both start with "All" and have no group label | Rename the first option to "All records" / "All sources", and add an aria group label |
| S4 | At 960×640 the toolbar takes 3 rows, provider chips take 2 more, and chrome uses 54% of the window | Below `SPLIT_WIDTH`, provider chips become one horizontally scrolling row and the count sits on the same row as the filters |
| S6 | Detail shows "60 messages" and "42 messages on this page" on two lines | Merge into one line: "42 of 60 shown" |
| S7 | Role names use decorative colors, and Chinese UI still shows user/assistant/tool | Role names use muted ink and are localized as 用户 / 助手 / 工具 |
| S9 | The identity dot is centered across both lines instead of aligned with the title | Align it with the first line (the title) |
| S10 | "Older sessions" is plain text and doesn't look clickable | Make it a ghost button with a right-chevron icon |
| S12 | Message cards inside the pane card | Separate messages with hairlines; no nested borders |

## Not fixing in this round (reasons)

| # | Problem | Why not now |
| --- | --- | --- |
| O1 | At 2000 wide, the Activity title sits about 75px above the grid | Vertical centering was decided last round to remove the blank area under the chart; it was a balance. Revisit together with an adaptive cell size |
| O3 | The lens shows a focus ring after launch | Comes from the previous round's focus anchor on the lens (material task); out of scope here. Recorded as a remaining gap |
| O7 | Heat map plus four streak counts reads as a template | The user chose "add real insight modules"; the data is real. Keep it |
| O8 / S8 | CJK fallback font renders 500 weight as bold | System font fallback, outside this task's scope. Recorded as a remaining gap |
| S5 | In Chinese at 1280, the Kimi chip wraps onto its own line | Minor; revisit after S4's narrow-window strategy lands |
| S11 | Raw variant ids (claude-jsonl, …) are shown | These are honest identifiers and useful for disambiguation |
| M5 | The health strip at 960 has no edge fade | Native fade capability is limited; recorded |
| M6 / M7 | Counts repeated; MCP and Sessions details have different heading styles | DESIGN.md specifies both deliberately |
| X1 | At the minimum window height, the "Usage" item in the sidebar is clipped by the Settings dock | Existing problem, recorded in the previous round; out of scope |
