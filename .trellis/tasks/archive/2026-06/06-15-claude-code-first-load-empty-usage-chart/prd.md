# Optimize Claude Code Page First-Load Empty Usage Chart

## Goal

Fix the first-open empty-looking area in the Claude Code page usage insight panel so users always see an honest loading, preparing, empty, or chart state instead of a blank chart container.

## What I Already Know

* The reported screenshot shows the Claude Code page renders the hero, usage metric cards, tabs, and the "last 30 days daily cost" chart title, but the chart body is visually blank on first open.
* The affected page is `ccr-ui/src/views/ClaudeCodeView.vue`.
* `ClaudeCodeView.vue` lazy-loads `UsageInsightPanel.vue` with `defineAsyncComponent`.
* `UsageInsightPanel.vue` fetches all Claude Observer slices on mount through `useClaudeObserverStore().fetchAll()`.
* `CostAttributionTab.vue` renders the 260px chart shell with an ApexCharts async component when `hasDaily && shouldRenderChart` is true.
* `ApexChartAsync` in `ccr-ui/src/components/claude-observer/apexChart.ts` lazy-loads `vue3-apexcharts` without a loading component.
* The main Usage Dashboard already has a safer pattern: when data exists but chart rendering is deferred, `UsageOverviewTab.vue` shows a "preparing chart" placeholder and has smoke coverage in `usage-overview-tab.smoke.test.ts`.

## Root Cause

The blank first-open area is caused by the Claude Observer chart path switching directly from data-ready to an async ApexCharts component without an interim loading/preparing fallback: `CostAttributionTab.vue` creates the chart shell, `ApexChartAsync` starts loading `vue3-apexcharts`, and Vue renders no visible child while the async chart chunk and Apex layout initialize.

This explains the screenshot:

* The page is not globally blank, so router, layout, i18n, and hero rendering are working.
* The metric cards and chart title are visible, so `UsageInsightPanel` reached the ready template instead of staying in `AsyncStatePanel`.
* The blank rectangle is isolated to `.cost-tab__chart-shell`, where the only visible alternatives are the async chart component or the "no trend" empty state.
* Because the screenshot shows neither a chart nor the "no trend" copy, the blank state fits the async component/loading gap, not a true no-data state.

## Requirements

* Add a visible "preparing chart" state for Claude Observer chart containers when data exists but the chart component is not ready or rendering is intentionally deferred.
* Keep true no-data states distinct from chart-loading states. Do not show "暂无趋势数据" when data exists and the chart is merely preparing.
* Apply the same pattern consistently to:
  * `CostAttributionTab.vue` daily cost chart.
  * `TokenDetailTab.vue` daily token chart.
  * `BehaviorAnalysisTab.vue` heatmap chart.
* Preserve the current page hierarchy and card density. This is a surgical state-handling fix, not a redesign.
* Add i18n copy under `claudeCode.observer.chart` or `claudeCode.observer.empty` for English and Chinese.
* Add smoke coverage that fails if the chart area becomes blank when data exists but chart rendering is deferred.

## Acceptance Criteria

* [ ] Opening `/claude-code` never shows an empty chart body while data exists and the chart component is still loading.
* [ ] When `daily.length > 0` and chart rendering is not ready, `CostAttributionTab` shows a preparing placeholder.
* [ ] When `daily.length === 0`, `CostAttributionTab` still shows the true no-trend empty state.
* [ ] Equivalent deferred placeholders exist for token and behavior charts.
* [ ] The placeholder has stable height equal to the chart shell, preventing layout shift.
* [ ] The smoke test covers the deferred-data state for at least the cost tab, and preferably all three chart tabs.
* [ ] `bun run test:smoke -- claude` or the narrow Vitest target passes.
* [ ] `bun run type-check` passes after implementation.
* [ ] Browser verification captures `/claude-code` in web preview or Tauri runtime and confirms no blank chart area on first open.

## Technical Approach

Recommended approach: reuse the Usage Dashboard's deferred-chart placeholder pattern.

1. Introduce a shared local condition in each Claude Observer chart tab:
   * data exists + `!shouldRenderChart` -> preparing placeholder.
   * data exists + `shouldRenderChart` -> async Apex chart.
   * no data -> no-trend empty state.
2. Consider adding `loadingComponent` to `ApexChartAsync` only if the async chunk itself still produces a visible blank after `shouldRenderChart` flips true.
3. Add i18n keys such as:
   * `claudeCode.observer.chart.preparingTrend`
   * `claudeCode.observer.chart.preparingHeatmap`
4. Add smoke tests around `CostAttributionTab.vue` / `TokenDetailTab.vue` / `BehaviorAnalysisTab.vue` with a stubbed chart component or existing async mock.

## Alternative Options

**Option A: Deferred Placeholder In Tabs (Recommended)**

* How it works: mirror `UsageOverviewTab.vue`; show "chart preparing" while data is present but chart rendering is not active.
* Pros: small, local, already proven in the repo, easy to test.
* Cons: does not cover a late blank inside Apex itself after the async component resolves.

**Option B: Add `loadingComponent` to `ApexChartAsync`**

* How it works: define a visible loading component in `apexChart.ts`.
* Pros: covers all Claude Observer Apex imports from one place.
* Cons: less context-aware; it cannot distinguish cost trend, token trend, and heatmap copy without more plumbing.

**Option C: Eager-load ApexCharts on Claude Code Page**

* How it works: remove lazy loading for the observer charts.
* Pros: reduces the visible async gap.
* Cons: worsens page first-load cost and contradicts the existing comment that the usage insight panel is heavy and intentionally lazy.

## Decision

Use Option A first, and add Option B only if runtime verification still shows an async-chart blank after the deferred placeholder is in place.

## Out of Scope

* Reworking Claude Observer backend commands.
* Changing pricing, llmusage, or aggregation behavior.
* Redesigning the Claude Code page layout.
* Replacing ApexCharts.
* Starting a full usage import flow from this page.

## Verification Plan

* Unit/smoke:
  * Add or extend a Vitest smoke test for `CostAttributionTab.vue` to assert:
    * data + `shouldRenderChart=false` shows preparing text.
    * data + `shouldRenderChart=true` shows chart stub.
    * no data shows no-trend text.
  * Extend the same shape to `TokenDetailTab.vue` and `BehaviorAnalysisTab.vue` if the test scaffolding remains small.
* Static checks:
  * `cd ccr-ui && bun run type-check`
  * `cd ccr-ui && bun run test:smoke`
* Visual runtime:
  * `cd ccr-ui && bun run dev:web -- --host 127.0.0.1 --strictPort`
  * Open `http://127.0.0.1:5173/claude-code`.
  * Confirm the first visible chart body is never blank; it should show chart, preparing state, no-data state, loading state, or error state.

## Technical Notes

Evidence from current code:

* `ccr-ui/src/views/ClaudeCodeView.vue` lazy-loads `UsageInsightPanel.vue`.
* `ccr-ui/src/components/claude-observer/UsageInsightPanel.vue` calls `store.fetchAll()` on mount and passes `shouldRenderChart` into tab components.
* `ccr-ui/src/components/claude-observer/CostAttributionTab.vue` uses `v-if="hasDaily && shouldRenderChart"` for the Apex chart and falls straight to no-trend for every other case.
* `ccr-ui/src/components/claude-observer/apexChart.ts` async-imports `vue3-apexcharts` with no loading component.
* `ccr-ui/src/components/usage/UsageOverviewTab.vue` and `ccr-ui/tests/usage-overview-tab.smoke.test.ts` provide an existing deferred-placeholder pattern to copy.
