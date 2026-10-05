# 旧 Vue 测试处置清单（批次 3）

> 判定标准：测试文件（含其 helper）是否导入 `vue` / `pinia` / `vue-router` /
> `vue-i18n` / `@vue/test-utils` / `@tanstack/vue-virtual` 或任何 `.vue` 单文件组件。
> 上述包均已从 dependencies/devDependencies 移除，此类测试无法运行。
> 方法：对 tests/** 全量正则扫描 import，再以 vitest 实跑验证保留集全部通过。

## 保留（框架无关，实测通过）

共 **56 个既有文件 + 2 个新增 React 测试**。vitest 实跑：58 个测试文件、291 个用例全部通过
（`bun run test:smoke` exit 0）。

保留的既有 smoke 测试（54 个）：
apexcharts-style-contract、api-facade-boundary、api-facade-coverage、apple-glass-surface-contract、
checkin-balance-queue、checkin-records-api、checkin-state、checkin-waf-event-wait、claude-profiles、
codex-auth-accounts、command-runtime-policy、config-domain-api、dashboard-presentation、
dev-tooling-resource、error-message、font-preferences、frontend-dependency-audit、grok-settings-api、
icon-registry、install-opaque-handle、logger、native-window-appearance、perf-telemetry、
platform-usage-presentation、platform-usage-trend-chart、profile-diff、providers-catalog、router、
sanitize-terminal、settings-i18n、ssh-hardening、stage-landing-theme-contract、startup-recovery、
sync-encryption-contract、tauri-runtime、theme-bootstrap、theme-contrast-contract、typed-auth-client、
typed-claude-client、typed-codex-system-prompts、typed-command-boundary、typed-config-client、
typed-json-boundary、typed-open-config-client、typed-small-domain-client、usage-chart-diagnostics、
usage-dashboard-payload、usage-dashboard-presentation、usage-date-window、usage-import-normalization、
usage-overview-insights、usage-summary-cards、usage-token-breakdown、v-html-allowlist、window-chrome、use-codex-tray-panel

保留的其他文件（4 个）：helpers/usageFixtures.ts（纯数据）、setup/localStorage.ts、
setup/react-cleanup.ts（新增，@testing-library/react cleanup）、i18n.test.cjs
（纯文本断言，走 `test:i18n`，不在 vitest include 内）。

本批次改动说明：dev-tooling-resource.smoke.test.ts 的 warm manifest 断言从旧 Vue 入口
（main.ts/App.vue/MainLayout.vue 等）更新为 React 入口（main.tsx/shell/App.tsx/shell/router.tsx），
与批次 2 重写后的 scripts/dev-warm-targets.json 对齐；其余保留文件零改动。

## 删除（依赖 Vue 工具链，git 历史可追溯；08-22-test-contract-rebuild 负责重写）

共 **69 个文件**（67 个 smoke 测试 + 2 个 helper），全部因导入 vue 系包或 `.vue`
组件而无法在 React 测试环境下运行：

app-settings、app-window-chrome、base-modal、chart-error-boundary、checkin-accounts-tab、
checkin-cookie-fix、checkin-progress-modal、checkin-runtime-coverage、claude-auth-view、
claude-code-view、claude-observer-tabs、claude-profile-editor-sections、claude-profiles-view、
claude-settings、code-source-editor、codex-agent-sources-panel、codex-auth-view、
codex-command-palette、codex-dashboard、codex-profile-editor、codex-profiles-view、
codex-tray-panel、commands-view、config-card-a11y、config-source-panel、dashboard-view、
environment-switcher、grok-auth-view、grok-dashboard、grok-profile-editor、grok-profiles-view、
grok-settings、home-usage-overview.store、hooks、i18n-format-message、legacy-shells、
main-layout-theme-stage、mcp-manager、monitoring-view、opencode-view、platform-target-pruning、
pricing-view、profiles-hotkeys、profiles-quick-rail、profiles-quick-switch、
profiles-raw-editor-panel、profiles-toolbar、provider-templates、skills-migration-view、sync-view、
system-prompts-view、ui-primitives、usage-cost-tab、usage-dashboard-state、usage-dashboard-toolbar、
usage-diagnostics-drawer、usage-logs-tab、usage-models-tab、usage-overview-tab、usage-providers-tab、
usage-source-summary-card、usage-stale-banner、usage-tokens-tab、usage.store、use-stream、
use-unified-mcp-coverage、wsl-management（以上均为 `tests/<名>.smoke.test.ts`）；

helper：tests/helpers/i18n-stub.ts（导入 vue-i18n）、tests/helpers/usageDashboardContextStub.ts
（导入 vue 响应式 API），二者仅被上述已删测试引用。
