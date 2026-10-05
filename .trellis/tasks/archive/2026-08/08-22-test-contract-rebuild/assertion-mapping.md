# Contract assertion ↔ test mapping (AC4)

Every assertion in the 19 frontend contracts maps to an executable test or a
marked human check. Human checks belong to `08-22-regression-release` unless a
view task already owns them.

| Contract | Assertion | Test / human check |
| --- | --- | --- |
| `api-facade-boundary.md` | invoke stays in API layer; tauri.ts 9-command freeze; manifest-typed commands behind generated clients | `tests/api-facade-boundary.smoke.test.ts` |
| `api-facade-boundary.md` | wrapper set covers command-manifest.json | `tests/api-facade-coverage.smoke.test.ts` |
| `checkin-ux-contracts.md` | per-origin queue / 30s throttle | `tests/checkin-balance-queue.smoke.test.ts` |
| `checkin-ux-contracts.md` | event wait, no polling | `tests/checkin-waf-event-wait.smoke.test.ts` |
| `checkin-ux-contracts.md` | 4-state results / cookie-fix / toast-only errors | `tests/checkin-state.smoke.test.ts`, `tests/checkin-cookie-fix.smoke.test.tsx`, `tests/checkin-accounts-tab.smoke.test.tsx`, `tests/checkin-progress-modal.smoke.test.tsx` |
| `checkin-ux-contracts.md` | WAF WebView cookie inject | Human: `08-22-views-checkin` / `08-22-regression-release` (WebView) |
| `provider-template-contracts.md` | selector + non-secret persistence | `tests/provider-template-selector.smoke.test.tsx`, `tests/state-store-actions.smoke.test.ts`, `tests/providers-catalog.smoke.test.ts` |
| `theme-token-contracts.md` | three-layer theme / 0.75rem exception / token switch | `tests/theme-bootstrap.smoke.test.ts`, `tests/theme-switch.smoke.test.tsx`, `tests/token-single-point.smoke.test.tsx`, `tests/theme-domain-extension.smoke.test.tsx`, `tests/theme-contrast-contract.smoke.test.ts`, `tests/font-preferences.smoke.test.ts`, `tests/apple-glass-surface-contract.smoke.test.ts` |
| `theme-token-contracts.md` | visual contrast of custom accent | Human: `08-22-regression-release` |
| `monitoring-log-contracts.md` | sanitizer / redact | `tests/monitoring-log-sanitize.smoke.test.tsx` |
| `dashboard-presentation-contracts.md` | signal gating / reasons / first-run | `tests/dashboard-presentation.smoke.test.ts` |
| `dashboard-presentation-contracts.md` | compact card empty state / StatTile tone | `tests/react-shell.smoke.test.tsx`, `tests/ui-primitives.smoke.test.tsx` |
| `usage-chart-stability-contracts.md` | options/series identity, CSS dual path, datetime axis | `tests/usage-chart-stability.smoke.test.tsx`, `tests/apexcharts-style-contract.smoke.test.ts`, `tests/platform-usage-trend-chart.smoke.test.ts`, `tests/usage-chart-diagnostics.smoke.test.ts` |
| `usage-chart-stability-contracts.md` | canvas identity across tab/window switch | Human: `08-22-regression-release` (perf harness in archived `07-07-ui-usage-dashboard`) |
| `confirm-interaction-contracts.md` | requestConfirm / no native dialogs | `tests/confirm-interaction.smoke.test.tsx` |
| `raw-config-editor-contracts.md` | editor mount / CSP nonce | `tests/code-source-editor.smoke.test.tsx` |
| `raw-config-editor-contracts.md` | Local-only raw commands / CAS | Tauri `commands::*raw*` tests; human window chrome N/A |
| `brand-asset-pipeline.md` | generated icon hashes / formats | `bun run icons:ensure` (build path); human visual: `08-22-regression-release` |
| `sync-security-contracts.md` | typed sync IPC / passphrase | `tests/sync-encryption-contract.smoke.test.ts`, `tests/sync-passphrase-modal.smoke.test.tsx` |
| `development-resource-contracts.md` | watcher ignore / warmup / worker budget | `tests/dev-tooling-resource.smoke.test.ts` |
| `profiles-page-contracts.md` | shared skeleton / serialization | `tests/profiles-shared-layer.smoke.test.tsx`, `tests/claude-profiles.smoke.test.ts`, `tests/profile-diff.smoke.test.ts` |
| Environment-Scoped Dashboard Contracts | Local-only gates / Query keys / stale refresh | `tests/grok-dashboard.smoke.test.tsx` |
| `grok-settings-contracts.md` | BaseSettings shared | `tests/platform-base-settings.smoke.test.tsx`, `tests/platform-surface-unify.smoke.test.ts` |
| `grok-settings-contracts.md` | dirty patch / raw unsupported | `tests/grok-settings-api.smoke.test.ts` |
| `grok-settings-contracts.md` | CAS / managed lock / no-backup | Tauri `commands::grok::tests`; human: `08-22-views-secondary-platforms` |
| `layering-contracts.md` | dependency graph / facade split | `bun run check:arch-boundaries`, `bun run check:cycles`, `tests/api-facade-boundary.smoke.test.ts`, `tests/frontend-dependency-audit.smoke.test.ts` |
| `react-rerender-discipline.md` | four lint rules | `bun run lint:ci` (`app/rerender-*` blocks) |
| `react-rerender-discipline.md` | three review-gate rules | Human: code review on view PRs (`08-22-arch-quality-perf`) |
| `platform-surface-contracts.md` | 75 paths / no platform-name branch / thin shells | `tests/platform-surface-unify.smoke.test.ts`, `tests/router.smoke.test.ts`, `tests/platform-base-settings.smoke.test.tsx` |

No unmapped assertion remains.
