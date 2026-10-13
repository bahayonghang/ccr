# Profile TUI visual audit

Revision note: this is the original source audit. Its right-panel arrangement is superseded by [right-panel-v2.md](right-panel-v2.md) and the parent design's revision-2 candidate A. Runtime/table/source findings remain applicable.

## Evidence and repair contract

User screenshot: 2203×1313 source image, displayed scaled to 2048×1221. Pixel dimensions do not determine terminal columns without font/cell metrics; do not claim an exact terminal size from the image.

Target: Codex, Claude and Grok Profile runtime/list/detail presentation and shared header brands. Preserve Auth content/flows, switching semantics, key bindings, per-tab selection, localization and persisted theme. Verify at terminal cell sizes.

## Source findings

| Finding | Evidence | Proposed change |
|---|---|---|
| Runtime text is stacked inside a full-width 4-row box, mostly neutral | `crates/ccr-tui/src/tui/ui.rs:140-221` | Structured Mode/Profile/Auth regions; explicit value tones and wrapping budget |
| Selection background has gaps between cells | `crates/ccr-tui/src/tui/ui.rs:355-395` | One full-row selected surface, current and disabled remain separate signals |
| Only name/description, no header | `crates/ccr-tui/src/tui/ui.rs:516-610` | Shared table geometry with header, guides and last-used column |
| All inner height counts as data rows | `crates/ccr-tui/src/tui/ui.rs:517-523` | Subtract header/rules; retain actual data-row geometry for click mapping |
| Mouse handler uses list-area hit test | `crates/ccr-tui/src/tui/app.rs:1415-1423` | Ignore headers/rules; verify x and y bounds |
| Right rail gets 54% width but remains a single field column | `crates/ccr-tui/src/tui/ui.rs:264-269,676-786` | Internal two-column details when inner width >=74, long endpoint/notice full-width |
| Heavy section marker, underline, blank rows and repeated unset summaries | `crates/ccr-tui/src/tui/ui.rs:991-1050,1713-1733` | Thin group rules, compact headings, one missing count per group |
| Label width scans repeat for every field | `crates/ccr-tui/src/tui/ui.rs:1736-1747,1766-1787` | Compute once per layout/group if profiling supports it |
| Claude title differs | `crates/ccr-tui/src/tui/app.rs:65-85` | Claude Profile EN, Claude 配置 ZH, auth title preserved |
| Platform emoji shared across consumers | `crates/ccr-config/src/models/platform.rs:72`, `tui/ui.rs:414,462-464` | TUI-local brand treatment, not a global model icon change |
| Grok falls through to Codex blue | `crates/ccr-tui/src/tui/theme.rs:380-388` | Separate high-contrast Grok identity using paired theme tokens |

## Visual thesis

Precise terminal workbench, calm existing Catppuccin surfaces, thin neutral grid and semantic color. Density comes from alignment and useful summaries, not tiny text or filler.

Content: current runtime -> comparable profiles -> selected identity/configuration + activity/provider totals. Interaction: no keyboard animation; immediate selection; pending/completed/error state for async operations.

Terminal owns font family/size. Keep host monospace font and `unicode-width` measurements. CSS and browser font rules do not apply to Ratatui.

## Existing tokens

Mocha: background #1e1e2e; secondary #181825; surface #313244; rule #585b70; text #cdd6f4; secondary text #a6adc8; muted #7f849c; good #a6e3a1; warning/cost #f9e2af; error #f38ba8; Codex #89b4fa; Claude #fab387; selected row #cba6f7 with #11111b foreground (`theme.rs:89-116`). These are deliberate existing product colors. Latte retains its paired tokens (`theme.rs:118-145`).

## Table geometry proposal

- Keep outer wide split 46/54; first solve sparse detail use internally.
- Mandatory: status, Profile, Last used. Optional: model and description.
- Inner width >=88: status 3, profile 24, last-used 14, model 20, description remainder; account for guides/gaps.
- 62..87: status 3, profile >=22, last-used 14, description remainder; omit model.
- 40..61: status 3, profile remainder, last-used 14; model/description in detail.
- <40: status+profile; last-use remains in selected detail. Important dates cannot end in ellipsis.
- One header and one thin header rule; vertical guides; no additional body separator rows. Parent design settles this to keep dense rows and simple hit testing.
- Sample `now` once per render/update, relative table time and exact time+offset in detail. See data research for the actual source and wording.

## Responsive verification

Existing modes: Compact width<90 or height<22; Wide width>=120 and height>=22; otherwise Standard (`theme.rs:21-28`).

Cell matrix: 60×16, 80×20, 89×22, 90×22, 100×30, 119×30, 120×22, 140×30, 200×50; two languages/themes and three profile platforms. Stress long Latin/CJK names/models/endpoints, current != selected, disabled, empty, malformed config, query error, switching, ancillary warning, missing provider/source, missing/future/invalid timestamps.

Measure header/body boundaries, body Rect, visible page size, first/last clickable row, total wrapped detail lines and last reachable scroll line. Per-tab selection must survive resize and reload.

## Brand feasibility

Current dependencies: Ratatui 0.30.2/Crossterm 0.29, no image widget (`crates/ccr-tui/Cargo.toml`). SVG logos are not printable Unicode characters. Random stars/robots/computers or third-party font glyphs must not be described as official artwork.

Proposal: portable product names and platform colors, no emoji. Official graphics require a validated terminal path, source provenance and fallback. Do not add terminal-protocol/font probing on startup just for decoration during this performance task.

Primary sources checked:

- [Ratatui rendering](https://ratatui.rs/concepts/rendering/): immediate mode and app-owned redraw scheduling.
- [Ratatui backends](https://ratatui.rs/concepts/backends/): styled text/cursor backend contract.
- [ratatui-image](https://github.com/ratatui/ratatui-image): terminal-specific graphics protocols and off-thread encoding requirements.
- [OpenAI design guidelines](https://openai.com/brand/), [xAI brand guidelines](https://x.ai/legal/brand-guidelines): official asset source entry points if terminal graphics are added.

No official Claude/Codex/Grok terminal font glyph mapping has been verified. Do not assume a Nerd Font is installed.

## Verification gap

This is a source-grounded audit, not a post-change screenshot. Implementation needs Ratatui TestBackend evidence and real-terminal smoke. Any design preview is illustrative, uses fixture data, and does not prove runtime behavior.
