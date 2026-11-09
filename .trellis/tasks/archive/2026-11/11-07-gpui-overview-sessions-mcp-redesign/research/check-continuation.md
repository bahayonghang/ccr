# Trellis continuation review (2026-11-08)

Scope: current `911e9d7b` working-tree changes for handoff §5.1 A–F, matching tests, `ccr-desktop/DESIGN.md`, and task/spec amendments. This is a source and headless-contract review; native screenshots and final release acceptance are owned by the coordinating session.

## Findings resolved

1. **Failed list reads left stale counts visible.** Retaining the last successful Sessions snapshot correctly preserved provider availability, but `page_count()` still said “N on this page · older available” above an error panel with no rows. The implementer now returns an empty count when `view.error` is present. The failure regression asserts that the count is empty while the active provider chip remains visible and checked.
2. **Failed list reads retained a misleading empty detail pane.** `has_rows()` consumed the same retained snapshot, so the right pane claimed that N sessions were listed even though the left pane showed a read failure. The implementer now requires `view.error.is_none()` before treating retained rows as visible. The regression asserts that neither stale rows nor the empty detail pane render on failure; Codex trash visibility still follows the retained provider availability.
3. **Column sizing comments contradicted the new contract.** The two `column_size` comments still said inner panels always filled their column. The reviewer changed only those comments to state that content/pane type determines height.

## Contract review

- Short lists and unselected panes hug content; long MCP lists and the Sessions listbox retain bounded internal scrolling. Real detail/editor/transcript panes explicitly keep their fill behavior. Outer columns keep the existing zero-basis 3:2 sizing without padding or borders.
- Disabled ghost actions retain transparent fill/border and muted ink. The installed Kit replays instance styles after its disabled style, so these overrides survive rendering. `states()` preserves the prior disabled, busy and tab-stop behavior of the global button decorator.
- No MCP URL/args/credential projection was added. Editor, copy, batch ownership and masking paths are unchanged.
- Independent list/detail gates, debounce generations, delayed focus guards, stable IDs and Workbench/Usage single-snapshot behavior are preserved.
- New full-page regression cases are included in the existing 1 MiB Windows-stack guard. No stack increase or gate relaxation was introduced.
- Search copy, provider/trash visibility, group labels, narrow provider scrolling, localized muted message roles, message separators, merged detail counts and sparkline contrast match the handoff. The neutral sparkline color comes from the existing theme palette and has a light/dark ≥3:1 unit guard.
- Task PRD/design and three relevant code-spec documents were synchronized with the refined rules. The handoff's out-of-scope visual gaps remain separate from acceptance.

## Verification and limits

- Reviewer: `git diff --check` passed; `just desktop-fmt-check` passed with the installed Rustup Cargo prepended to process PATH.
- Coordinating session's full `just desktop-ci` run observed fmt/check/lint passing; Python command tests **16 passed**; AccessKit patch tests **3 passed**; Rust lib **294 passed, 0 failed, 4 ignored**; `native_shell` **12 passed, 1 ignored**; `native_ui` **2 passed**; shutdown **2 passed, 1 ignored**. Release compilation was still running when this receipt was written.
- That aggregate run began before the final stale-count/pane corrections. The coordinator owns a final fmt/lint and focused failed-list regression check on the resulting tree, plus release completion; see `integration.md` for the authoritative final receipt.
- Native pixels, screenshot geometry, actual keyboard traversal, UIA, IME, other DPI settings and other operating systems are not established by this review. See `native-visual.md` for actual native coverage and remaining gaps.

No unresolved source findings remain within the reviewed scope after the fixes above.
