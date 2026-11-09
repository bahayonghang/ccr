# Grok Profile design

Inherit parent [design](../10-12-tui-profile-workbench/design.md). Grok is a third source in the existing usage pipeline, not a new analytics subsystem.

## Data flow

Selected ProfileConfig plus existing Grok domain helpers -> typed engine/route fields. Current runtime uses platform/domain resolution in a background snapshot; the TUI does not infer logged-in status from `auth_mode=session`. Parent activity projection matches exact `(Grok, name)`.

Use `GrokPlatform::inspect_activation_state()` (`crates/ccr-cli/src/platforms/grok.rs:197-220`) inside the snapshot worker to retain Inactive/Active/Drifted/UnsafeMissingEntryState. `get_current_profile()` alone folds drift into no-current and is insufficient. These are activation states, separate from login/session validity; show login unknown if no existing read evidence establishes it. No new login probe in rendering.

Extend the existing UsageApp production source list from Claude/Codex to Claude/Codex/Grok, platform_rows mapping and profile-detail integration. Existing `Dashboard::provider_breakdown_by_source` performs one aggregate per source. Keep source+provider matching, single-flight loader, cached dataset, and schema capability outcomes. No additional database handle/query per row or selection.

## Display

Separate Grok identity via a high-contrast paired neutral token, rather than the current fallback to Codex blue. Header uses product name and no rocket. Follow parent revision-2 candidate A: compact model/connection summary, provider usage ledger, then vertical activation history. Details emphasize model + reasoning, backend, context window/search support, safe endpoint and auth source. Use configured-value/invalid/missing distinctions; hide unset optional rows in the normal view. The usage ledger shares common-scale independent category bars and an adjacent value column, with numeric-only narrow rendering; show unsupported values honestly. Latest activation appears only once, with switch count beside the history title.

Use `GrokPlatform::safe_base_url_for_display` for endpoints. Show env-key name only, never its value. Inline tokens are not displayed even in masked form. Usage scope/estimate and no-record/unavailable/errors remain visible in compact layouts.

## Compatibility / rollback

Primary files: `tui/ui.rs`, `tui/app.rs`, `tui/theme.rs`, `tui/usage/app.rs`; ccr-usage fixture extensions for source separation. Preserve Grok runtime CAS and secret write boundaries. Existing source support needs no schema migration. Revert source wiring/platform builder independently; do not remove canonical SourceKind::Grok.
