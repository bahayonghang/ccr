# Codex 首页趋势图坐标标签与 KPI 图标优化

## Goal

Codex 首页 30 天趋势图的横轴日期完整可读，不得出现 `2026-07...` 截断。Codex 首页上已有的图标（用量 KPI、面板按钮、页头按钮、readiness / next action / management 的 tone 图标）对比度足够扫读。

用户打开 Codex 首页看成本趋势时，能直接读出每个刻度的月和日。Antigravity 与 OpenCode 首页共用用量面板，该面板的图表与 KPI 行为与 Codex 一致。

## Background

2026-08-22 用户截图（CCR UI v7.1.5，30 days，Cost trend）用红框标出横轴三处：`2026-07...`、`2026-07...`、`2026-08...`。日丢失，末尾刻度右侧留白。当前仓库 v7.2.0，同一配置仍在。证据：`evidence/codex-home-cost-trend-labels-truncated.jpg`。

`ccr-ui` 是 Vue 3 + Vite + Pinia + vue-router + Tauri 2，图表为 `vue3-apexcharts` + ApexCharts 5。

## Confirmed Facts

- 用量面板是 `PlatformUsageInsightPanel`，接入点：`CodexView.vue`、`GeminiCliView.vue`（Antigravity）、`OpenCodeView.vue`。改面板即三端一起变。
- 趋势图在 `PlatformUsageTrendChart.vue`。横轴是 category，`categories` 为 `DailyTrend.date`（`YYYY-MM-DD`）。`labels.rotate = 0`，`labels.trim = true`，点数 `> 16` 时 `tickAmount = 6`。窗口文案固定 30 天（`platformUsageSpecs.ts` 的 `window30`）。
- ApexCharts category 轴按全部 category 槽宽（约 `chartWidth / 30`）判断溢出。`trim: true` 把 `2026-07-22` 裁成 `2026-07...`。`tickAmount: 6` 只隐藏部分标签，不重分槽宽，所以刻度之间有空、字仍被截。
- Usage 仪表盘已有日轴方案：`usageChartOptions.ts` 的 `type: 'datetime'`、`formatTrendAxisLabel`（en-US 日粒度 `Jan 5`）、`getTrendTickAmount`、UTC 日解析。`tests/usage-chart-diagnostics.smoke.test.ts` 锁了这些输出。
- `PlatformUsageTrendChart` 未走该工厂：options 编入 categories/series；未设 `redrawOnParentResize: false` / `redrawOnWindowResize: false`；动画未走 `buildChartAnimations()`。`usage-chart-stability-contracts.md` 要求 usage 图表遵守这些纪律。
- KPI 图标：`buildCards()` 中 cost=`Wallet`、tokens=`Layers`、requests=`Activity`。容器 `.platform-usage-panel__kpi-icon` 为 2.15rem、12% accent 底，图标色 `--color-text-primary`，三卡同色。对照 `UsageMetricCard`：每卡语义色图标 + 浅底 + 1px 语义边。
- Codex 首页其它图标表面：页头 `Button` 内 `SIcon`（Refresh / primary / Auth / Profiles）；readiness、next action、management 共用 `.codex-tone-icon` + `CodexDashboardTone`。顶栏 `StatTile`（Version / Home / Account）没有图标。Grok 首页有平行的 `.grok-tone-icon`，本任务不改。
- `/usage` 的 `UsageCostTab` / `UsageTokensTab` 同样 `trim: true` + ISO category，不是本页。

## Requirements

### R1 趋势图横轴日期完整可读

- 30 天窗口下，Cost / Tokens / Requests 三个趋势 tab 的横轴不得使用 ellipsis。用户必须能读出月和日。
- 标签保持水平，不旋转。
- 约 30 个点时可见刻度约 6 个，不得把 30 个日期挤成一行。
- 提示框日期完整。日粒度与横轴同一套 `formatTrendAxisLabel`；若提示框需要范围，用已有 `formatTrendTooltipLabel`。
- 日粒度格式：en-US `Jul 22`，zh-CN `7月22日`。不显示年份。locale 跟 UI 走，不得写死英文月份。
- 空数据、1–8 个点、跨月都必须可读。

### R2 改图表时对齐 usage 图表稳定性合同

- 横轴格式化复用 `formatTrendAxisLabel` / `getTrendTickAmount` 和现有 UTC 日解析，不得再写一套日期函数。
- chart options 必须含 `redrawOnParentResize: false` 与 `redrawOnWindowResize: false`。动画走 `buildChartAnimations()`。
- 不得把 `type` / `height` 绑到会变的表达式。高度继续固定 `286`。
- 不把 Usage 仪表盘整页 KeepAlive 方案搬进平台首页；只在本组件内收口 options/series 纪律。

### R3 Codex 首页图标可扫读

- 范围内表面：用量 KPI 三卡；面板 Refresh / Open Usage dashboard 与 notice 图标；Codex 页头 Refresh / primary / Auth / Profiles；readiness、next action、management 的 `.codex-tone-icon`。
- KPI 三卡：每卡独立语义色（cost=`--color-accent-primary-rgb`，tokens=`--color-info-rgb`，requests=`--color-accent-secondary-rgb`）。图标色 = 该 rgb，浅底 10%，1px 语义边 14%。对照 `UsageMetricCard`。不改 `Wallet` / `Layers` / `Activity` 的 `iconMap` 映射。
- Codex tone 图标：继续用 `success` / `warning` / `danger` / `neutral` token。标准格 2.25rem，glyph `w-5 h-5`。不把数字涂成语义色。
- 页头与面板按钮图标：继承所在 `Button` / 面板按钮的 `currentColor`。页头图标走 `Button` 的 leading 槽，避免默认槽里再叠 `mr-2`。
- 继续用 `SIcon` + `iconMap`。不改 `SIcon.vue` API，不引入第二套图标库，不改全局 `iconMap`。
- 不为 Version / Home / Account 的 `StatTile` 新增图标。不改侧栏。不改 `Button.vue` 原语。不改 Grok 首页。不改 `EmptyState.vue` 原语。
- 不写字面 hex/rgb。不加重投影或 glow。深浅色 × `neutral` / `clay` 下图标对比不低于现有 text/surface 阈值。

### R4 一致性与验证

- 用量面板的图表与 KPI 在 Codex / Antigravity / OpenCode 三个首页一致。Codex 页头与 readiness 只在 `/codex` 生效。
- 锁横轴标签格式的单元或 smoke（扩 `usage-chart-diagnostics.smoke.test.ts`，或给平台图表 options 单独加测试）。
- `platform-usage-presentation.smoke.test.ts` 与 `apexcharts-style-contract.smoke.test.ts` 继续通过。
- 视觉证据放 `evidence/`。至少：Codex 首页 Cost trend，dark × zh-CN 与 en-US，宽屏（与截图同量级）。横轴完整可读，KPI 三卡图标色不同。

## Acceptance Criteria

- [ ] Codex 首页 30 天 Cost trend 横轴不再出现 `YYYY-MM...` ellipsis。月和日可见。
- [ ] Tokens 与 Requests 两个趋势 tab 同样不截断。
- [ ] 提示框日期完整。
- [ ] 标签保持水平。约 6 个可见刻度，不出现 30 个日期挤叠。
- [ ] 日粒度横轴：en-US `Jul 22`，zh-CN `7月22日`。无年份。locale 跟 UI 走。
- [ ] 横轴格式化复用 `formatTrendAxisLabel` / `getTrendTickAmount`。
- [ ] 本组件 chart options 含 `redrawOnParentResize: false`、`redrawOnWindowResize: false`，动画走 `buildChartAnimations()`。
- [ ] KPI 三卡图标为语义色 + 浅底 + 描边，三卡颜色不同。
- [ ] Codex 页头按钮图标走 leading 槽；readiness / next action / management 的 tone 图标 glyph 为 `w-5 h-5`。
- [ ] Antigravity、OpenCode 首页同一用量面板同步生效。
- [ ] `StatTile`、侧栏、Grok 首页、`Button.vue`、`SIcon.vue` API、`iconMap` 映射未改。
- [ ] `cd ccr-ui && bun run type-check`、`bun run lint`、相关 smoke 通过。
- [ ] `evidence/` 至少有 Codex 首页 Cost trend 的 dark 截图，标签完整可读。

## Out of Scope

- `/usage` 的 `UsageCostTab` / `UsageTokensTab` 同类截断。
- Claude Observer 图表。
- 侧栏、Grok 首页、`EmptyState` 原语、`Button` 原语、`StatTile` 加图标。
- 换图表库或升级 ApexCharts 大版本。
- 改 30 天窗口、汇总口径、IPC。
- 全局 `iconMap` 换库或换 Solar 风格。

## Decisions

- **D1 横轴日期格式（2026-08-22）**：locale short。复用 `formatTrendAxisLabel`。en-US `Jul 22`，zh-CN `7月22日`。
- **D2 图标范围（2026-08-22）**：整个 Codex 首页已有图标表面（KPI、面板按钮与 notice、页头按钮、readiness / next action / management）。不为 `StatTile` 新增图标。不改侧栏与 Grok。
- **D3 KPI 色（2026-08-22）**：每卡独立语义 token，对照 `UsageMetricCard`，不是三卡共用平台 accent。
