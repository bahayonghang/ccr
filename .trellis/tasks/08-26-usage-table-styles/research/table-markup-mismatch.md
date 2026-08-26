# Usage 子页表格 / 排行：Vue CSS 与 React 标记脱节

日期：2026-08-26。证据来自当前 `dev` 源码与用户提供的 Usage 截图（Overview / Tokens / Cost / Providers / Models / Projects）。

## 根因

`08-22-views-usage` 把 Vue scoped CSS 迁到 `ccr-ui/src/features/usage/styles/*.css`，但 React 标记被收成「名称 + token + 费用」内联堆叠。CSS 仍按 `<table>` / 双列 rank 网格写，选择器对不上节点，于是：

1. Models / Providers / Projects 三列文字粘成一行（截图：`gpt-5.6-sol17.07B$13,955.57`）。
2. Overview / Cost 排行被挤进第一列 `2.15–2.25rem`，项目名变成 `.../bah...`。

这不是缺数据：`ModelStatDto` / `ProjectStatDto` / `ProviderBreakdownDto` / `SourceBreakdownDto` 已有 requests、tokens、cost、pricing status、share 字段。i18n `usage.dashboard.table.*` 表头也已存在。

## 标记 vs CSS

| 表面 | React | CSS 期望 | 可见后果 |
|---|---|---|---|
| Models | `UsageTableTabs.tsx:14-20`：`div.models-tab__table` > `article.models-tab__row` > 三个 inline 子节点 | `usage-models-tab.css:81-109`：真正的 `table`，`min-width: 106rem`，`thead th` / `tbody td`；**没有** `__row` | 名称/用量/费用粘连；容器被 106rem 最小宽度拉宽 |
| Projects | `UsageTableTabs.tsx:34-39`：`article.projects-tab__row` | `usage-projects-tab.css:28-48`：`.projects-tab__item` 网格；**没有** `__row` | 路径、token、费用粘连 |
| Providers | `UsageTableTabs.tsx:53-59`：同上 | `usage-providers-tab.css:129-173`：`.providers-tab__table` HTML table；**没有** `__row` | `unknown14.91B$4,997.16` |
| Overview 排行 | `UsageOverviewTab.tsx:92-97`：`li.rank-item` 只有 `rank-main` | `usage-overview-tab.css:296-301`：`grid-template-columns: 2.25rem minmax(0, 1fr)`，需要 `rank-index` | 唯一子节点进第一列 2.25rem，ellipsis 切到不可辨 |
| Cost 来源排行 | `UsageCostTab.tsx:79-84`：`li` 只有 `ranking-main` | `usage-cost-tab.css:101-106`：同样的 2.15rem + 1fr | 名称/费用挤在左缘，右侧大片空白 |
| Tokens 日表 | `UsageTokensTab.tsx` 只有柱状图 | `usage-tokens-tab.css:123-167`：完整 `tokens-tab__table` | 子页没有可扫表格 |
| Logs | `UsageLogsTab.tsx:87-95`：header + 6 列 cell | `usage-logs-tab.css:180-184`：匹配的 grid | **对照面**：可读 ledger |

## 截图问题如何映射到代码

- **粘连字符串**：inline `strong`/`span` 无列网格（Providers/Models/Projects）。
- **排行不可扫**：缺 `rank-index` / `rank-row`，数值与名称无法左右对齐。
- **项目路径**：`usageOverviewInsights.ts:37-39` 的 `shortenPath` 在排行里合理；真正不可读的是 2.25rem 列。Projects 全表应主行显示末两段、次行/tooltip 完整路径。
- **Tokens/Cost X 轴重叠**：`UsageTokensTab` / `UsageCostTab` 未调用已有 `getTrendTickAmount`（`usageChartOptions.ts:127-132`）。产品决策：本任务 **不做** 轴标签，避免碰到 `usage-chart-stability-contracts.md`。

## 对照实现

`UsageLogsTab` 已是操作员 ledger：粘性表头、列网格、`tabular-nums`、右齐数值、`title` 溢出、行 hover。新表应复用这个拓扑，而不是复活 `min-width: 106rem` 的 HTML `<table>`。

## 不在本文件结论内的项

- Overview 洞察卡缺 `insight-accent` 竖条（CSS `overview-tab__insight-tile` 双列 vs 单子节点）。
- `PlatformUsageRankList.tsx` 同样 CSS/标记脱节，但不在 Usage 子页。
