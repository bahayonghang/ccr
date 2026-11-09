# Fresh native visual review

## Review

fix

- **P2 — Overview / status line** · “7 days ending 2026-11-09” and “Last successful sync 10-21 23:20” have no visible report timezone, although DESIGN.md requires the cutoff/timezone in this line and defines sync time in the report timezone · Add the actual report timezone beside this date/time context.

Independent subagent review. Only the design contract, the fresh-review prompt, the judged checklist and the three supplied screenshots were inspected. No implementation, prior conclusions, UI interaction, tests or builds were inspected or performed.

## Evidence

- `native/after5-overview-light-en-initial.png`: the status line below the heading shows the date and sync time without a timezone. Cost leads the KPI band; the remaining metrics are subordinate. The chart and activity panel are separate reading surfaces, without inner bordered cards. Chart space after November 5 represents the visible later dates rather than a stretched empty container. Activity content has deliberate vertical breathing room consistent with the contract's centering rule.
- `native/after5-mcp-light-en-wide-observed.png`: the six-row reading panel ends immediately after its contents. The unselected pane ends after its explanatory text. The large area below them is the outer work surface; filling it with larger empty reading panels would conflict with the content-tight contract. Row names, command/scope metadata, platform identity and separators use consistent positions. The sole filled primary action is “Add server”.
- `native/after5-sessions-light-en-wide-observed.png`: rows use a consistent two-line pattern, platform dots align with titles, timestamps align at the right, and the unselected pane is content-tight. The long list occupies its available reading area; no accidental card-within-card treatment is visible. The sole filled primary action is “Refresh index”.

No additional visible product finding was established. In particular, the approved system fonts, static mint/lavender scene, pill controls and solid reading panels are contract choices, not review defects.

## Judged checklist (§6)

| Gate | Result |
| --- | --- |
| 8 | pass: section order follows usage analysis, server management and archive reading tasks. |
| 9 | pass: spacing, ruled rows, panel surfaces and compact toolbars provide distinct separation. |
| 46a | n/a: screenshot-only review cannot establish data provenance or independently verify the displayed counts. No obviously invented promotional metric is visible. |
| 54 | pass: visible short labels identify metric scope, source, platform or navigation context; no redundant marketing eyebrow is visible. |
| 58 | n/a: this is the contract's dense native data workbench, with lists and analytical tables, not a marketing hero. The roughly 80-word web-screen ceiling is not an appropriate content limit here. |
| 63 | pass: visible middle dots separate live state, source metadata or measurement captions. |
| 69 | pass: repeated dates/counts carry distinct explicitly named aggregation or source contexts. |
| 70 | n/a: no submitted form state is supplied. |
| 84 | pass: inputs are native application screenshots, not drawn browser/product mockups. |
| 85 | n/a: no footer or email signup. |
| 3 | pass: no three equal icon-over-title tiles or thick coloured card-side stripes. |
| 6b | n/a: no hero. |
| 11 | n/a: static captures do not establish hover behavior. |
| 29 | pass: the static mint/lavender scene is explicitly declared by DESIGN.md; no additional visible glow treatment contradicts it. |
| 38iii | pass: no single decorative highlighted word inside a heading. |
| 40 | n/a: exact contrast pairs and all themes were not measured in this screenshot-only review. Normal-state text is visibly readable; that is not a WCAG contrast certification. |
| 41 | pass for visible normal-state buttons: white text is visibly distinct on the green fill. Exact token mapping and contrast ratio remain unverified. |
| 44b | n/a: exact native logical dimensions were not verified. Supplied wide captures show page headings and their main actions without clipping. |
| 47c | pass: no imitation product object or fabricated CSS product illustration. |
| 49a | n/a for the full width range: visible button/navigation labels stay on one line; 320–1920 was not tested and widths below 960 are outside this native contract. |
| 65 | pass: no undeclared cream/serif/clay theme. |
| 66 | pass: visible text conveys context, measurements, actions or the app version. |
| 68 | n/a: no empty image-slot composition. |
| 73 | n/a for all tested widths/states: no application element visibly covers text in the supplied states, but an external floating input toolbar covers part of Overview's comparison area. |
| 76 | n/a for image rights: the brand mark visibly loads, but asset provenance was not inspected. |
| 80 | n/a: no photographic subject. |
| 83 | n/a: no photographic composition. |
| F4 | pass: at most one filled primary action per supplied view. |
| F21 | n/a for semantic heading levels: the visual hierarchy is clear, but accessibility structure was not inspected. |
| F23 | n/a: native GPUI application, not an HTML document; language/accessibility metadata was not inspected. |
| F26 | n/a for requested web sizes: 375 is outside the native contract and exact 1280 logical size is unverified. No visible prose orphan requires a fix in these captures. |

## Unverified scope

Only light/en wide initial or unselected states are represented. Screenshot dimensions are normalized by the image tool; exact native logical window size and DPI are unknown. Dark theme, Chinese UI, narrower layouts, minimum-window behavior, scaling, cross-display behavior, other platforms, hover/pressed/focus/disabled/loading states, keyboard interaction, tooltips, accessible semantics, actual scrolling, Overview below the fold, expanded exact values, selected MCP details/edit/batch flows, opened session transcripts, and trash states were not reviewed.

Overview includes an external floating input toolbar over the lower-right comparison area. That obscured region cannot support a clean visual acceptance claim. This is a screenshot evidence limitation, not an established application layout defect.

This verdict is limited to the visible composition and contract finding above; it is not full native acceptance.
