# Design: Codex 首页趋势图坐标标签与 KPI 图标优化

## Boundaries

- 改：`PlatformUsageTrendChart.vue` 及其可测的 options/series 辅助函数；`usageChartOptions.ts` 只导出已有 UTC 日解析；`PlatformUsageInsightPanel.vue` 的 KPI / 按钮图标样式；`CodexView.vue` 页头 leading 槽与 `.codex-tone-icon` glyph 尺寸。
- 不改：`SIcon.vue` API、`iconMap` 映射、`Button.vue`、`StatTile.vue`、Grok 首页、侧栏、`/usage` 的 Cost/Tokens tab、IPC、窗口天数。

共享面板的图表与 KPI 会同时出现在 Codex / Antigravity / OpenCode。这是组件边界，不是额外产品范围。

## Chart axis

把平台趋势图从 category + `trim: true` 改成 datetime 轴，与 Usage 仪表盘同一套日标签。

1. 从 `usageChartOptions.ts` 导出 `parseUtcDate`（现有私有函数，签名保持 `YYYY-MM-DD` → UTC 午夜 `Date`）。不在平台组件里再写一份拆分。
2. series 点改为 `[timestamp, value]`。tokens 图仍 stacked bar，四条序列同一套 timestamp。
3. `xaxis.type = 'datetime'`，`labels.trim = false`，`rotate = 0`，`datetimeUTC: false`。`formatter` 调用 `formatTrendAxisLabel(timestamp, 'day', locale)`。`tickAmount` 用 `getTrendTickAmount(pointCount)`。
4. tooltip 的 x 用同一日格式。tokens 多序列仍用现有 y formatter（`formatTokens` / `formatCost` / 整数）。
5. locale 从 `useI18n().locale` 读。locale 变化才重建 options 的 formatter 闭包。

### Options / series 纪律

`chart` 段固定：

```ts
{
  toolbar: { show: false },
  animations: buildChartAnimations(),
  fontFamily: 'var(--font-sans)',
  background: 'transparent',
  redrawOnParentResize: false,
  redrawOnWindowResize: false,
  stacked: metric === 'tokens',
}
```

`height` 继续字面量 `286`。`type` 仍由 metric 决定（area / bar / line），metric 切 tab 时允许换 type；这是用户操作，不是 resize 闪烁。

options 的 computed 依赖只允许：theme、locale、metric、tickAmount（点数形状）。不得把每日 cost/token 值编进 options。

series 按 join key 记忆化（name + 每个 `x=y`）。值未变时返回上一引用，避免 vue3-apexcharts deep watch 重建 canvas。

### 可测缝

把「DailyTrend[] → datetime series + tickAmount」抽到 `ccr-ui/src/views/platform-usage/platformUsageTrendChart.ts`（或同目录纯 ts）。Vue 文件只组装 Apex options。Smoke 断言：

- `2026-07-22` → timestamp `Date.UTC(2026, 6, 22)`
- 30 个点 → `getTrendTickAmount` 为 6
- 不在辅助函数里出现 `trim: true`

`formatTrendAxisLabel` 的 `Jan 5` / zh-CN 输出继续由 `usage-chart-diagnostics.smoke.test.ts` 锁。

## Icon surfaces

三层，不抽全局新组件。

| 表面 | 文件 | 处理 |
| --- | --- | --- |
| KPI 三卡 | `PlatformUsageInsightPanel.vue` | `--kpi-icon-rgb` 按 `card.id`：cost=accent-primary，tokens=info，requests=accent-secondary。容器浅底 10%、边 14%、图标 `rgb(var(--kpi-icon-rgb))`。glyph `w-5 h-5`。 |
| 面板按钮 / notice | 同上 | 继承按钮/notice 的 `currentColor`。保持 `inline-flex` + gap。 |
| Codex 页头 | `CodexView.vue` | 四个页头 `SIcon` 改到 `Button` leading 槽，去掉默认槽里的 `mr-2`。尺寸 `w-4 h-4`。 |
| Codex tone | `CodexView.vue` | `.codex-tone-icon` 保持 2.25rem 与现有 tone token。readiness 与 management 的 glyph 从 `w-4` 升到 `w-5`。next action 已是 `w-5`，只对齐。 |

不改 `Wallet` / `Layers` / `Activity` 名称。不抽 `ToneIcon.vue`（Grok 有平行样式，抽公共组件超出本任务）。

## Compatibility

- 暗色 / 亮色 × `neutral` / `clay`：只用已有 rgb token。禁止字面 hex。
- 平台面板 `--platform-usage-accent-rgb` 继续给 tab 激活态和 cost 卡径向底。KPI 图标色不再绑这个变量。
- KeepAlive：Codex 页在 `mainLayoutShell` 的缓存名单里。options 引用纪律是为了切回首页时不闪。

## Rollback

还原上述 Vue/ts 与测试即可。无数据迁移，无 IPC 变更。
