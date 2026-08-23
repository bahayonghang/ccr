# Usage Chart Stability Contracts

> ApexCharts (`react-apexcharts`) reference discipline and keep-mounted tab interaction
> for the usage dashboard. Breaking any rule remounts the chart canvas on tab re-entry
> or window switch (flash, replayed entrance animation, several times the cost), which
> cancels the keep-mounted tab cache.
> Distilled from `07-07-ui-usage-dashboard` (`research/after/perf-comparison.md` in the
> archived task holds the full cause chain and before/after numbers).

---

## 1. react-apexcharts prop update semantics (fact base)

The modular wrapper (`react-apexcharts/core`, assembled in `src/utils/apexChartsCore.ts`)
and the usage host `src/features/usage/charts/ApexChart.tsx` decide the upper-layer rules:

| prop | update path | effect |
| --- | --- | --- |
| `options` | identity change | `chart.updateOptions()` → ApexCharts full update, **rebuilds `.apexcharts-canvas`** |
| `series` | identity change | `chart.updateSeries()` → the same full update rebuilds the canvas |
| `type` / `width` / `height` | value change | `destroy()` + `init()` (heaviest path) |

Consequences:

- "updateSeries is the fast path" is only relative to remounting the React tree; a new
  series **identity** (even with equal values) still rebuilds the canvas. Identity
  stability must hold at the data source.
- Do not bind `type` / `width` / `height` to expressions that change. Height is a
  fixed value or CSS.

## 2. options build discipline

- All usage chart options go through `src/views/usage/usageChartOptions.ts` factories
  (`buildTrendChartOptions` / `buildDistributionPieOptions`) or the same shape:
  **static skeleton as module-level `Object.freeze` constants**; the factory only
  injects theme colors, locale, and axis scalars.
- Options memo dependencies may only be: theme, locale, axis shape
  (`tickAmount` / granularity), and labels/seriesNames memoized by a join key.
  **Read data through a getter closure (for example `getBuckets`) at render time.**
  Data must not be a build-time dependency of options.
- **Every chart options object must include:**

  ```ts
  redrawOnParentResize: false,
  redrawOnWindowResize: false,
  ```

  ApexCharts defaults `redrawOnParentResize: true`. A keep-mounted tab that is
  hidden and shown again still fires parentResize → full rebuild. TREND/PIE frozen
  bases already include the flags; **local options inside tab hosts
  (`UsageTokensTab` / `UsageCostTab` bar charts) are the easy miss.** Check each
  new chart.
- Animations go through `buildChartAnimations()` (exported; on by default,
  `prefers-reduced-motion` degrades). Do not add hardcoded
  `animations: { enabled: false }`.

## 3. series identity discipline

- Every series fed to a chart (`trendSeries` / `pieSeries` / `modelTokenPieSeries`)
  is **memoized by value**: a join key covering name + every data point; equal
  values return the previous reference (`useMemo` plus an identity check, or an
  equivalent helper). React will otherwise pass a new array every render.
- Why this is required: dashboard presentation input includes copy such as
  `selectedWindowLabel`. Window switches, locale changes, and the 30s auto-refresh
  (new array, same values) recompute presentation and emit "equal values, new
  identity" series. Without memoization the canvas rebuilds for free
  (measured inside 37ms, before the 300ms filter debounce refetch; data did not
  change).
- New charts use the same rule. The join key must cover every rendered field.

## 4. Keep-mounted tab contract (`UsageDashboardView`)

- Tab component references must be a **module-level stable map** (`TAB_COMPONENTS`
  with static imports). Do not allocate the map during render.
- Visited tabs stay in the tree: `visitedTabs` plus `hidden={tab !== activeTab}`.
  Do not unmount a visited tab on switch. That is the React stand-in for the
  former keep-alive cache.
- Chart hydration gates (`shouldRenderTrendChart` and siblings) follow `*Ready`
  monotonic flags (false→true once). **Do not couple them to `activeTab`.** A
  hidden tab instance is still mounted; flipping the gate off unmounts the chart
  and remounts it on return.
- Do not tear the tree during refresh: the loading panel takes over only when
  there is no renderable data (`hasDashboardData` gate). Refresh goes through
  series memoization + `updateSeries`.

## 5. ApexCharts complete CSS dual-path delivery

`src/utils/apexChartsCore.ts` is the only modular assembly entry. That entry must
satisfy both:

```ts
import ReactApexChart from 'react-apexcharts/core'
import 'apexcharts/dist/apexcharts.css'

import 'apexcharts/area'
// continue registering chart types / features actually used
```

- Build path: statically import the upstream complete `apexcharts.css`; Vite
  delivers it with the `apexChartsCore` async chunk.
- Runtime path: keep ApexCharts default `chart.injectStyleSheet: true`. Do not
  turn off `#apexcharts-css` injection to remove duplicate rules. The two paths
  carry the same content; either path must satisfy the full layout contract.
- Lazy-load boundary: the CSS import must sit with the assembly entry. Do not
  lift it to `main.tsx`. A production build must show the CSS as a preload
  dependency of chart callers, and `index.html` must not link it on first paint.

This is not a single-marker visual patch. Complete styles also own tooltip
absolute positioning, series-group initial hide, and the marker host `12x12`
size with inner SVG scale. Do not copy a private ApexCharts selector set, patch
the global SVG reset, or only add marker width/height. Those leave static
placeholders, wrong tooltip layout, or upgrade drift.

On a dependency upgrade or assembly-entry change,
`tests/apexcharts-style-contract.smoke.test.ts` must still assert:

- `react-apexcharts/core` wrapper and every used module registration stay unique;
- the complete CSS import stays unique;
- `.apexcharts-tooltip` is absolutely positioned;
- `.apexcharts-tooltip-series-group` starts hidden;
- `.apexcharts-tooltip-marker` is `12x12`, its SVG is `100% x 100%`.

Fault retest must block the runtime `#apexcharts-css` append **before** the first
chart mount, not delete it after mount. Build-managed CSS must still satisfy
every rule above, and tooltip hover must not change card height. That injection
only proves dual-path tolerance; it does not prove a natural field trigger that
drops runtime styles.

## 6. Verification method (regression retest)

- Browser harness + measure scripts live in the archived task
  `07-07-ui-usage-dashboard/research/perf-harness/`:
  `tauri-shim.js` (fake Tauri IPC fixture), `measure-after.mjs` (same-baseline
  retest: tab switch ×3, window switch ×2, 20 round-trip memory),
  `diagnose-after.mjs` (canvas / component-root identity probe).
- Pass criterion: **node identity** (`data-perf-id` markers) — tab re-entry
  rebuilt=false; window switch keeps old canvases alive. Duration is a helper
  metric only.
- The store has `DASHBOARD_CACHE_TTL_MS = 30s` snapshot cache: switching back to
  the same window inside 30s does not issue IPC. Window-switch tests must
  separate the refetch path from the cache path.

## 7. Horizontal date labels

Platform home trend charts (`PlatformUsageTrendChart`) and the Usage dashboard
daily trend must use `xaxis.type: 'datetime'`. Labels go through
`formatTrendAxisLabel` + `parseUtcDate` (`YYYY-MM-DD` → UTC midnight). Forbidden:

- category axis + `labels.trim: true` + ISO `YYYY-MM-DD` strings. ApexCharts
  judges overflow from the full category slot width; a 30-day window clips
  `2026-07-22` to `2026-07...` even when `tickAmount` shows only 6 ticks.
- A second month/day formatter. Locale short form is already covered by
  `formatTrendAxisLabel` (en-US `Jul 22`, zh-CN `7月22日`).

`tests/platform-usage-trend-chart.smoke.test.ts` freezes datetime, `trim: false`,
and `redrawOnParentResize: false`. `tests/usage-chart-diagnostics.smoke.test.ts`
freezes `parseUtcDate` and the daily label copy.
`tests/usage-chart-stability.smoke.test.tsx` freezes the chart controller mount
path.

## Known deviations (accepted; fold in when touching the area)

- `UsageTokensTab` / `UsageCostTab` local options that hardcoded
  `animations: { enabled: false }` — already folded (07-07-ui-consistency-sweep
  R2-6); both go through exported `buildChartAnimations()`.
- Cost tab options that depend directly on `ctx.trends` (data refresh fires
  `updateOptions` on an off-screen cached chart; the user does not see it). Fold
  those options into the factory in the same change.
