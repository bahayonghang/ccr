# Batch 5: 40 "other" tests — disposition

Source: PRD 122 = 63 mount + 19 source-text + 40 other.

Method: each current non-mount, non-source-text smoke file is either **keep**
(intent unchanged, implementation already React-safe), **rewritten** (this task
or a view task), or **covered by batch 1**.

## Keep (intent unchanged)

api-facade-coverage, checkin-balance-queue, checkin-records-api, checkin-state,
checkin-waf-event-wait, claude-profiles, codex-auth-accounts,
command-runtime-policy, config-domain-api, dashboard-presentation,
error-message, font-preferences, frontend-dependency-audit, grok-settings-api,
install-opaque-handle, logger, native-window-appearance, perf-telemetry,
platform-usage-presentation, profile-diff, providers-catalog, router,
sanitize-terminal, settings-i18n, ssh-hardening, stage-landing-theme-contract,
startup-recovery, sync-encryption-contract, tauri-runtime, theme-bootstrap,
theme-contrast-contract, typed-*-client / typed-*-boundary, usage-dashboard-payload,
usage-dashboard-presentation, usage-date-window, usage-import-normalization,
usage-overview-insights, usage-summary-cards, usage-token-breakdown,
use-codex-tray-panel, window-chrome, zod-pilot, cache-route,
codex-route-loaders, configs-form-drafts, configs-route-loaders,
gemini-route-loaders, grok-route-loaders, opencode-route-loaders,
sync-tools-route-loaders, generic-views-reuse, opencode-theme-namespace,
overlay-single-implementation, platform-surface-unify,
command-runtime-policy, tauri-event-inventory, config-form-schema.

## Rewritten this task (Vue path / SFC pattern)

| File | Disposition |
| --- | --- |
| `v-html-allowlist.smoke.test.ts` | Scan `dangerouslySetInnerHTML` allowlist (3 audited sites) |
| `platform-usage-trend-chart.smoke.test.ts` | Read `.tsx` chart host (variable `chartSource`) |
| `apple-glass-surface-contract.smoke.test.ts` | Walk `tsx/css/ts/html` |
| `api-facade-boundary.smoke.test.ts` | Walk `ts/mts/tsx` only |
| `helpers/apiInvokeScan.ts` | Same suffix set |
| `dev-tooling-resource.smoke.test.ts` | Drop old SFC warmup path |
| `i18n.test.cjs` | Collect `ts/tsx` |
| `profiles-shared-layer.smoke.test.tsx` | Assert TSX modules |
| `grok-dashboard.smoke.test.tsx` | Restore Local-only dashboard assertions |

## Covered by batch 1 (do not redo)

`api-facade-coverage.smoke.test.ts`, `tauri-event-inventory.smoke.test.ts`,
`router.smoke.test.ts` (75 paths; this task only renamed the case title).

## Deleted Vue-toolchain files (legacy-tests-disposition)

67 Vue Test Utils files were deleted in `08-22-react-foundation` batch 3.
View tasks rewrote the mount subset as `*.smoke.test.tsx`. Store-only files
(`usage.store`, `home-usage-overview.store`) are replaced by Query-hook tests
(`state-query-hooks.smoke.test.tsx`, `state-store-actions.smoke.test.ts`).

No "other" test was dropped without a replacement of the same intent.
