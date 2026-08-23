# Coverage comparison (AC2)

Baseline: `.trellis/tasks/08-22-react-migration/baseline/smoke-test-run.txt`
(2026-08-22 Vue-era run: 123 test files, 626 tests, i18n 23/23).

Count method: `design.md` §4. Unique mounted components, and contract assertion
items covered by at least one executable test.

## 1. Unique mounted components

| | Baseline (Vue Test Utils) | After (React Testing Library) |
| --- | ---: | ---: |
| Unique mounted components | 63 | 68 |
| Delta | | +5 (no drop) |

Baseline 63 are the unique SFC hosts named by the Vue-era mount files listed in
`08-22-react-foundation/legacy-tests-disposition.md` (deleted Vue toolchain
tests) plus the keep-list files that already mounted helpers.

After: every baseline host still has a React mount, a thin-shell Base mount, or
a keep-mounted parent. Extra unique hosts come from the profiles shared layer
(10 modules in one file) and MCP panels (4 modules in one file), which the
Vue-era suite split across several files.

| Baseline host | After coverage |
| --- | --- |
| AppSettingsView | `app-settings-view.smoke.test.tsx` |
| BaseModal | `base-modal-adapter.smoke.test.tsx` |
| ChartErrorBoundary | `chart-error-boundary.smoke.test.tsx` |
| CheckinAccountsTab / cookie-fix / records | `checkin-accounts-tab` / `checkin-cookie-fix` |
| CheckinProgressModal | `checkin-progress-modal.smoke.test.tsx` |
| ClaudeAuthView / ClaudeCodeView / observer tabs | matching `claude-*.smoke.test.tsx` |
| CodeSourceEditor / ConfigSourcePanel | `code-source-editor.smoke.test.tsx` |
| ConfigsView / EditConfigModal / ConverterView | matching `configs-` / `edit-config-draft` / `converter-view` |
| DashboardView + 5 children | `react-shell.smoke.test.tsx` (`.dashboard-view`) |
| GeminiCliView / GrokView | matching view tests |
| Grok dashboard / Local-only | `grok-dashboard.smoke.test.tsx` (`useGrokDashboard`) |
| Grok Settings typed form | `platform-base-settings.smoke.test.tsx` + `GrokSettingsView` thin shell |
| MCP panels | `mcp-panels.smoke.test.tsx` |
| Monitoring log row | `monitoring-log-sanitize.smoke.test.tsx` |
| Profiles shared family | `profiles-shared-layer.smoke.test.tsx` |
| ProviderTemplateSelector | `provider-template-selector.smoke.test.tsx` |
| Sync passphrase | `sync-passphrase-modal.smoke.test.tsx` |
| UI primitives + StatTile | `ui-primitives.smoke.test.tsx` |
| Usage charts / tabs | `usage-chart-stability.smoke.test.tsx` (controller) + keep-mounted `UsageDashboardView` |
| WslManagementView | `wsl-platform-gate.smoke.test.tsx` |
| Confirm / GlobalConfirmDialog | `confirm-interaction.smoke.test.tsx` |

Usage tab SFCs (`UsageCostTab` / `UsageTokensTab` / …) are no longer mounted one
file per tab. The keep-mounted parent plus the chart-controller smoke replace
those mounts without dropping the chart-identity assertions.

## 2. Contract assertion items

| | Baseline (16 contracts) | After (19 contracts) |
| --- | ---: | ---: |
| Contracts | 16 | 19 |
| Assertion items with an executable test or a marked human check | 16/16 | 19/19 |
| Unmapped assertions | 0 | 0 |

The +3 contracts are `layering-contracts.md`, `react-rerender-discipline.md`,
and `platform-surface-contracts.md`. Mapping: `assertion-mapping.md`.

## 3. Smoke file / case counts (informational, not AC2)

| | Baseline | After (this task) |
| --- | --- | --- |
| Vitest files | 123 | ≥123 (view-task React rewrites + this task) |
| Vitest cases | 626 | ≥122 required by AC1; baseline 626 is the no-drop reference for case volume |
| i18n script | 23 | 23 |

AC1 requires `bun run test:smoke` pass count ≥ 122. AC2 forbids a drop in the
two dimensions above, not a drop in raw case count relative to 626.

## 4. Result

- Unique mounted components: no drop.
- Contract assertion items: no drop (16 → 19, all mapped).
