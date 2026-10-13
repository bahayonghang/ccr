# Planning validation

Status: revision-2 planning complete, latest user review pending. Product implementation and performance measurements have not started.

## Delivered

- One parent with direct shared work, three linked platform children.
- Each task has a converged PRD, design, execution plan and curated implement/check manifests.
- Source-grounded visual/data/performance research, including explicit limits and proposed benchmarks.
- Original three 2032×1088 fixture previews retained as the revision-1 baseline. Revision 2 adds thirteen artifacts: three platform-A full previews, full and focused B/C comparisons, a focused A, and narrow/short/Chinese/empty/long-content cases. They are not screenshots from a running or modified TUI.

## Checks performed

- `task.py validate` passed for all four tasks after replacing oversized injection entries with spec indexes and an explicit full-file reading fallback. No context-size warnings remain.
- Parsed all task records and manifests; verified planning status, reciprocal parent/child links, artifact existence and local markdown/context paths.
- Checked planning documents for remaining template placeholders; none remain.
- `git diff --check` passed for tracked changes; no product code changed in this task. Pre-existing build-cache/spec/journal changes were preserved.
- Opened Codex, Claude and Grok design previews for visual inspection. Corrected the initial preview's current-vs-selected marker and column rules overlapping its legend; all three use consistent column coordinates and distinct platform labels/colors.
- Static layout proposal uses a 200-cell canvas: left panel 91 cells, right panel 107 cells, 2-cell gap; table guides at cells 5/30/53/74. Actual before/after Ratatui cell geometry remains implementation acceptance work.
- Independent source-grounded planning review caught three omissions, now covered: exit readiness cannot use on_tick's redraw bool; initialized Codex Auth runtime cache needs invalidation; Grok drift requires inspect_activation_state rather than only get_current_profile.

Visual consumer accounting: checked 3 / changed 0 / left 3. Here changed/left refer to product pages; this phase delivered design artifacts only.

## Revision-2 verification

- Applied the explicitly requested frontend-design skill to the user's rejected right-panel screenshot. Compared three arrangements at the same 107×32-cell panel and recommended A: connections, usage ledger, activation history.
- Visually inspected the ten distinct design cases: A/B/C, Claude-A, Grok-A, 74-cell rail, 60×20 short rail, Chinese, missing usage/history, and long model/endpoint. Full/focused duplicate renders are not counted as separate cases. Fixed a candidate-B rule extending below its content-fit frame, an awkward split in its USD qualifier, and hidden reasoning/switch-count values at narrow widths before final review.
- All preview text passes cell-boundary assertions. Short panels explicitly show a scrolling cue; this is a design requirement, not proof of an implemented scrolling widget. Long endpoints wrap and complete timestamps retain their timezone.
- Connection value offset reduces from 16 to 11 cells in the English fixture. Token labels share one ledger; its five tracks use the same 59.4M fixture maximum and the adjacent common numeric column. The former 46-cell Switches key/value scan is replaced by a count beside the activation heading. No fabricated trend, composition percentage or savings figure was added.
- Pixel comparison against each platform's revision-1 image confirms every pixel outside the right-panel rectangle (x=946..2015, y=232..999 in the 2032×1088 canvas) is unchanged. Runtime and table geometry are therefore held constant for this comparison.
- Updated parent PRD/design/implementation acceptance, all three child designs, and all eight implement/check manifests. Marked the original visual audit's right-panel arrangement superseded. All four task records remain planning with design_revision=2 and implementation_started=false; task/context existence and parent/child links passed validation.
- Re-ran `task.py validate` for all four tasks: passed, with no size warnings. Product runtime tests and both-theme terminal verification remain implementation work.

Revision-2 design accounting: checked 3 platform consumers / revised 3 design previews / remaining runtime implementations 3. No product code was changed in this revision.

## Not claimed

No Rust build/test, real terminal run, real profile switch, credential/database read, benchmark result or speedup was performed/claimed. Full validation commands are planned in implement.md. The mockup contains synthetic data and a portable text brand fallback.

## Final review decisions

Approve the complete layout and execution plan with the revision-2 candidate-A right panel, including Last used as last recorded successful activation, independent category comparisons and provider-scoped approximate costs, plain product names where official artwork cannot be rendered, and safe deferred exit during a write. Runtime/UI/data semantics in the parent design are authoritative over alternative ideas listed by research.
