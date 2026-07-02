# C3 — ccr-ui per-provider usage view

Parent: `07-01-provider-usage-stats` · Design: parent `design.md` §5 · Order: after C2

## Goal

Surface per-provider token/cost for Claude Code and Codex in ccr-ui, reusing the
existing `/usage` dashboard infrastructure, with an "unattributed" bucket and an
explicit "≈ official-equivalent price" cost label.

## Requirements

- Add a "By provider" surface: a new tab in `UsageDashboardView.vue` (alongside
  Overview/Tokens/Cost/Models/Projects/Logs) or a dedicated `/usage/providers`
  route — pick one in this child's design and justify.
- `stores/usage.ts`: add `providerStats` state + `fetchProviderStats()` calling
  `get_usage_by_provider_v2`; honor existing platform + time-range filters and the
  30s cache/auto-refresh conventions.
- Reuse existing components (StatCard, breakdown strips, ApexCharts) and the
  `usage.*` i18n namespace; add keys to **both** `zh-CN` and `en-US`.
- Show an "unattributed" row/segment for NULL-provider usage.
- Render "≈ official-equivalent price" next to any USD amount (FR5/§9).
- Capability-aware: when `ProviderBreakdown` is unsupported, show the existing
  unsupported/empty state with an "update llmusage" hint (AC3) — no error/crash.
- Match ccr-ui design language (Anthropic-like editorial, no legacy neko/purple);
  keep dark+light high-contrast and reduced-motion support.

## Acceptance Criteria

- [ ] Per-provider token & (labeled) cost render for Claude and Codex, filterable
      by time range; provider totals + unattributed reconcile with the source view.
- [ ] Cost is always labeled as official-equivalent; never shown as real billing.
- [ ] Old-llmusage / no-data states are graceful and localized (zh-CN + en-US).
- [ ] `just frontend-check-quick` passes; frontend-quality-reviewer clean.

## Notes / dependencies

- Depends on C2's `get_usage_by_provider_v2` (+ dashboard `provider` param).
- Independent of C4; both can proceed in parallel once C2 lands.
