# C4 — ccr-tui usage/statistics tab

Parent: `07-01-provider-usage-stats` · Design: parent `design.md` §6 · Order: after C2

## Goal

Add a Usage/Statistics tab to the TUI showing per-provider token/cost for Claude
Code and Codex, following the existing tab and usage-panel conventions.

## Requirements

- Register a new tab via the existing system:
  - `crates/ccr-config/src/managers/tui_config.rs:16-40`: add `TuiTabId::Usage`,
    include in `DEFAULT_TAB_ORDER`, add `as_str()` arm.
  - `crates/ccr-tui/src/tui/app.rs:33-60`: add `TabVariant::Usage`; build the tab
    in `with_task_executor`; route in `handle_key` and `ui::draw`.
- New module `crates/ccr-tui/src/tui/usage/{app.rs,ui.rs}` rendering a per-provider
  table (provider | requests | input/output/cache tokens | ≈cost) for Claude and
  Codex; follow the Codex Auth usage panel (`codex_auth/ui.rs:542-607`).
- Async load via the existing `AsyncTaskExecutor` + message channel (no blocking
  the UI thread); key `r` = refresh; respect footer/status/keys conventions.
- Cost column labeled/footnoted as "≈ official-equivalent price".
- Graceful state when provider data is unavailable (old llmusage / empty).
- Prefer sharing C2's read-only projection over `llmusage.db` + the activation log
  rather than duplicating SQL — decide the exact data path in this child's design
  (parent design §6 lists the two options).

## Acceptance Criteria

- [ ] New tab appears in the configured tab order and is reachable via Tab/Shift+Tab.
- [ ] Per-provider token/cost table renders for Claude and Codex with an
      unattributed row; `r` refreshes; layout holds at narrow widths.
- [ ] No blocking/jank on load; matches existing TUI styling.
- [ ] `just test` (tui crate) passes; manual TUI smoke verified.

## Notes / dependencies

- Depends on C2's per-provider read path (adapter projection preferred).
- Independent of C3; can proceed in parallel once C2 lands.
- Cross-check TUI tab-order specs (`.trellis/spec/ccr-tui/backend/index.md`) and
  the prior tab-order tasks for conventions.
