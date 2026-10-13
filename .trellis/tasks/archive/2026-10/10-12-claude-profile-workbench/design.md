# Claude Profile design

Inherit parent [design](../10-12-tui-profile-workbench/design.md). This child owns Claude presentation and scoped naming only.

## Architecture and data

Cached ClaudeRuntimeSummary establishes current mode/profile/auth status. Selected ProfileConfig supplies model mappings and configured knobs. Parent ccr-store activity projection matches explicit Claude + profile name. Usage uses SourceKind::Claude and exact provider label. No UI-layer TOML/credential parsing or new SQL.

## Visual and naming decisions

Use existing peach Claude identity token with semantic success/warning/error styles. Runtime state occupies the common compact strip; API-key and official modes retain their distinct contracts. Focus is selected identity only. Follow parent revision-2 candidate A: compact connection summary, provider usage ledger, then vertical activation history. Engine starts with configured primary model, then relevant Opus/Sonnet/Haiku/Fable/subagent and effort fields; show only present optional values by default, retaining expanded inspection. Usage shares the common independent category-bar scale and adjacent numeric column, with a numeric fallback for narrow widths. Keep the latest activation timestamp once, switch count beside the history title, and tags beside identity. No separate second wide group.

Rename visible profile identity at its owning label function and fixture expectations; do not rename auth tabs, CLI product names or TuiTabId. Compact navigation can remain `Claude` because the full current-page identity says `Claude Profile`. Remove robot branding through the TUI-local brand helper.

## Boundaries and rollback

Reuse current Claude auth semantics, profile-scoped env override rules, snapshot ownership and CAS. New runtime display does not change or claim credential ownership. Keep safe endpoint/token presentation and typed error severity. Any runtime summary read remains in the parent's background snapshot.

Primary files: `tui/app.rs`, `tui/ui.rs`, optional common theme tokens. Domain services are read-only references unless parent measurements prove a scoped read dedup change is needed. No schema migration; rollback is platform presentation and naming only.
