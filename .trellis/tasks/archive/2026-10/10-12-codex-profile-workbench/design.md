# Codex Profile design

Inherit the parent [design](../10-12-tui-profile-workbench/design.md), including cell geometry, date semantics, async scheduling, safety and icon fallback. This child specializes the Codex builders only.

## Data flow and owners

Existing CodexRuntimeSummary -> semantic Mode/Profile/Auth spans. Selected ProfileConfig -> Engine/Routing fields. Parent activity snapshot keyed `(Codex, profile_name)` -> last recorded activation/count/recent timestamps. Existing UsageApp dataset keyed `(SourceKind::Codex, provider)` -> metrics.

Render from cached state; never call CodexAuthService or parse config in draw. Runtime current profile comes from validated runtime summary, not the highlighted row. A raw auth source name does not imply logged-in validity.

## Presentation

- Runtime: blue identity, clear bound/unbound message, model emphasized, expected auth deficiency warning, neutral runtime-only mode. Error separate from unknown data.
- Details: use parent revision-2 candidate A. Model and reasoning first, followed by provider, endpoint and auth; keep label/value pairs close. Long models move reasoning to a separate row. Present configured wire API/env-key information where relevant; expanded inspection retains unset fields, while invalid required fields stay visible.
- Usage: requests/total/approximate official-equivalent cost strip, then aligned input, cache read/create, assistant output and reasoning counts with independent common-scale bars. Preserve the scope note, numeric fallback at narrow widths and source semantics. Follow with vertical activation history and adjacent switch count; do not duplicate the newest timestamp.
- Focus owns name/current/enabled and last action exactly once. Existing masked token contract applies; sanitize any new endpoint summary so credential-bearing URL components never reach the view.

## Files and compatibility

Primary: `crates/ccr-tui/src/tui/ui.rs`, `app.rs`, `theme.rs`, `usage/app.rs`; domain reads continue in ccr-codex. No Codex runtime schema or public CLI changes planned. Shared Codex banner is also consumed by Codex Auth; test that consumer for geometry/content regressions without redesigning its body.

Codex Auth owns another cached `runtime_summary` (`codex_auth/app.rs:326`), and its activation hook currently does not reload it. After a Profile apply/off, the parent must mark this summary stale or replace it with the matching refreshed typed snapshot. Cover initialized Auth -> Profile apply -> Auth and verify the banner shows the new actual runtime without resetting Auth selection or repeating mutation.

## Tradeoffs / rollback

Provider totals are informative but not profile-exclusive. Last use is named-profile activation history, not API activity. Revert the platform builder changes independently from shared read/scheduling work; no persistent migration.
