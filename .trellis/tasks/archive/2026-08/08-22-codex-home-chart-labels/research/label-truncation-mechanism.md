# 平台用量趋势图横轴截断机制

日期：2026-08-22  
证据截图：`evidence/codex-home-cost-trend-labels-truncated.jpg`（用户标注，CCR UI v7.1.5）  
当前代码：工作区 v7.2.0，问题配置仍在。

## 现象

Codex 首页 Cost trend，窗口 30 days：

- 可见刻度约 6 个。
- 标签渲染为 `2026-07...`、`2026-08...`。日丢失。
- 红框把「截断文字 + 右侧空白」一起圈住。空白属于槽宽，不是额外 DOM。

## 调用链

```
CodexView
  → PlatformUsageInsightPanel
      → PlatformUsageTrendChart   (cost | tokens | requests)
```

同面板还被 `GeminiCliView`（Antigravity）和 `OpenCodeView` 使用。

`DailyTrend.date` 为 `YYYY-MM-DD` 字符串（`DailyTrendDto`）。

## 直接配置

`PlatformUsageTrendChart.vue` 的 `xaxis`：

- `categories = trends.map(t => t.date)` → 约 30 个 ISO 日字符串
- `type` 未设 → category 轴
- `labels.rotate = 0`
- `labels.trim = true`  ← 溢出改 ellipsis
- `tickAmount = categories.length > 16 ? 6 : undefined`
- `fontSize: 11px`

ApexCharts category 轴按 **全部 category 个数** 分配槽宽，不是按可见刻度个数。30 个点时槽宽约 `plotWidth / 30`。`2026-07-22` 在 11px 下宽于该槽，`trim: true` 砍到 `2026-07...`。`tickAmount: 6` 只隐藏部分标签，不重新分配槽宽，所以刻度之间看起来空，字却被截。

## 仓库内已有正确做法

`ccr-ui/src/views/usage/usageChartOptions.ts`：

- `xaxis.type = 'datetime'`
- `formatTrendAxisLabel(timestamp, granularity, locale)`：日粒度 en-US → `Jan 5`
- `getTrendTickAmount(n)`：`n > 16` → `6`
- `parseUtcDate` + `timeZone: 'UTC'`，避免 `YYYY-MM-DD` 被当成 UTC 午夜再偏到前一天
- `tests/usage-chart-diagnostics.smoke.test.ts` 已锁 `Jan 5`

平台趋势图应复用这套函数，不要再写格式化。

## 稳定性合同偏差（改图表时一并收口）

`usage-chart-stability-contracts.md` 要求：

- `redrawOnParentResize: false`
- `redrawOnWindowResize: false`
- 动画走 `buildChartAnimations()`
- options 不把原始 series 数据编进构建期依赖

`PlatformUsageTrendChart` 三项都未满足。本任务改 options 时必须对齐，避免首页 KeepAlive 切回时 canvas 重建。

## KPI 图标对照

| 表面 | 图标色 | 容器 |
| --- | --- | --- |
| 平台用量 KPI（现状） | `--color-text-primary`，三卡相同 | 2.15rem、12% accent 底、无边 |
| Usage `UsageMetricCard` | `rgb(var(--usage-metric-rgb))`，每卡 tone | 1.9rem、10% 语义底 + 14% 语义边 |

映射：cost=`Wallet`，tokens=`Layers`，requests=`Activity`（`solar:graph-up-bold-duotone`）。不改 `iconMap` 全局语义，除非产品要求换图标。

## 已确认产品决定

- D1（2026-08-22）：横轴日粒度用 locale short，复用 `formatTrendAxisLabel`。en-US `Jul 22`，zh-CN `7月22日`。
- D2（2026-08-22）：图标范围是整个 Codex 首页已有图标表面（KPI、面板按钮与 notice、页头按钮、readiness / next action / management）。不为 StatTile 新增图标。
- D3（2026-08-22）：KPI 三卡独立语义色，对照 UsageMetricCard。

## 明确排除

- `/usage` 的 `UsageCostTab` / `UsageTokensTab` 同样 `trim: true` + ISO category。同机制，不同页面，本任务不做。
- Claude Observer 已用 datetime 轴，不是本截图来源。
