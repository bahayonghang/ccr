# Usage 子页表格 — 技术设计

## Architecture and boundaries

本任务只改 `ccr-ui` Usage feature 的呈现。数据仍来自 `useUsageDashboard` / `UsageDashboardContext` 已有 DTO。不改 API。日柱图接到已有 datetime 工厂；store 初态 `timeRange` 与 `last_30d` 对齐。

```
UsageDashboardView (keep-mounted tabs)
  ├── UsageOverviewTab     → ranking list markup aligned to existing CSS
  ├── UsageCostTab         → ranking list markup aligned to existing CSS
  ├── UsageTokensTab       → existing chart + new daily ledger
  ├── UsageModelsTab       ┐
  ├── UsageProjectsTab     ┼→ UsageLedger (new, features/usage)
  └── UsageProvidersTab    ┘
UsageLogsTab               → unchanged (reference only)
```

共享壳放在 `ccr-ui/src/features/usage/components/UsageLedger.tsx` + `styles/usage-ledger.css`。不放进 `src/ui/`：列内容含定价 pill、路径双行、share bar，且 `src/ui` 没有 Table 原语可扩。三处以上重复时抽 feature 组件，符合 code-reuse 指南，同时满足 `layering-contracts.md`（feature → ui，反向禁止）。

## Ledger contract

`UsageLedger` 只接收已格式化的行模型（string / 可选 secondary / 可选 tone），不调用 `formatCost` / i18n。父 tab 负责映射 DTO。

行模型：

- `id: string`（稳定 key：model / project_path / provider ?? `'unknown'` / date）
- `cells: UsageLedgerCell[]`，与 `columns` 等长
- `UsageLedgerCell`：`text`、可选 `title`、`secondary`、`align: 'start' | 'end'`、`kind: 'text' | 'share' | 'status'`
- `share`：`ratio` 0–1，文案由父级格式化为百分比
- `status`：`pricing_status` 原值，父级给出已翻译 label；样式复用现有 `models-tab__status--*` 映射（迁到 ledger class，避免再依赖未挂上的 table 选择器）

列定义：`id`、`header`、`align`、`colTemplate`（CSS grid track，如 `minmax(12rem, 1.6fr)`）。父级把 tracks join 成 `--usage-ledger-cols`。

壳行为：

- 外层 `overflow: auto`、`max-height: 38rem`（Tokens 日表可用 `34rem`，与旧 CSS 一致）
- 表头 `position: sticky; top: 0`
- 行：`display: grid; grid-template-columns: var(--usage-ledger-cols)`；`min-width` 取各列 minmax 之和，**上限远低于 106rem**（目标桌面约 `52–64rem` 可从名称列读到费用列）
- 数值：`font-variant-numeric: tabular-nums`；`align: end` → `text-align: right`
- 名称：`min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap`
- hover：现有 `rgb(var(--color-accent-primary-rgb) / 6%)`
- 空态：壳不渲染 header+0 行；tab 渲染现有 empty 文案（R8）

`UsageLedgerRow` 必须 `memo`。父级 `map` 不得传入新函数/新 object 字面量到行组件；在 tab 内先把 DTO 收成 `UsageLedgerRowData[]`（`useMemo` 依赖 stats + formatters 引用）。

## Per-tab mapping

### Models (R1)

| Column | Source |
|---|---|
| Model | `item.model` |
| Requests | `item.request_count` locale string |
| Tokens | `formatTokens(item.total_tokens)` |
| Cost | `formatCost(item.cost_with_cache ?? item.total_cost)` |
| Share | `cost / sum(cost)` |
| Pricing | `item.pricing_status` → 已有 i18n `statusPriced` 等 |

### Projects (R2)

主名：复用 `shortenPath`（`usageOverviewInsights.ts:37-39`），不要另写第三套截断。`title` + `secondary` = 原始 `project_path`。

### Providers (R3)

名称：`usageSourceFallbackLabel(item.provider ?? 'unknown')`。Cost：`formatCost(item.cost_with_cache_usd)`。Share：相对本表该 cost 合计。

### Tokens daily (R5 + R11)

行：`toUsageTokenBreakdownRows(ctx.trends)`（tab 已 memo `rows`）。Total：现有 `getUsageTokenRowChartTotal`。图表走 `buildDailyBarChartOptions` + `toDailyBarPoints`；options 只依赖 theme / locale / `rows.length` / stacked。ledger 是第二个 `article`。

### Overview rankings (R6)

不要改成 6 列表。修好现有排行拓扑：

```
li.overview-tab__rank-item
  span.overview-tab__rank-index
  div.overview-tab__rank-main
    div.overview-tab__rank-row
      span.overview-tab__rank-label  title={item.title}
      strong.overview-tab__rank-value
      span.overview-tab__rank-share
    span.overview-tab__rank-detail
    div.overview-tab__rank-bar > span width share
```

这样 CSS 第二列吃到 `rank-main`，名称不再掉进 2.25rem。`item.detail` 已经算好（`usageOverviewInsights.ts:248-251`、`:273`），只是没渲染。

### Cost source ranking (R7)

同样补 `cost-tab__rank`、`cost-tab__ranking-row`、`cost-tab__bar`。占比：`item.share_cost`（已是 0–1）。名称：`usageSourceFallbackLabel(item.source)`。日柱（R11）与 Tokens 共用 `usageDailyBarChart.ts`：datetime、`getTrendTickAmount(trends.length)`、series 身份稳定。options 不得依赖 `ctx.trends` 数组。

### Default window (R12)

`useUsageViewStore` 初态 `rangePreset` 是 `last_30d`。`timeRange` 必须是 `getLocalDateRangeWindow('last_30d')`。`resetFilters` 同样。空 `timeRange` 会让 `useUsageDashboard(platform, start, end)` 不带日期，后端返回全部历史。

## CSS strategy

- 新增 `usage-ledger.css`：壳、行、单元格、share bar、status pill。颜色只用现有 `--color-*`。
- Models / Providers 里针对 `table`/`thead`/`min-width: 106rem` 的死规则删掉或改接到 ledger，避免以后再把 `models-tab__table` 套到 div 上。保留仍被引用的 title/empty。
- Projects 的 `__item` 卡片规则：若 Projects 改为 ledger，未引用的 `__item` 规则删除，避免两套视觉。
- Overview / Cost 排行 CSS 大体已正确，缺的是标记；只在对不齐时改 CSS。
- 不加新 token 名（`theme-token-contracts.md`）。

## Compatibility

- Keep-mounted：`TAB_COMPONENTS` 与 `hidden=` 不变（`usage-chart-stability-contracts.md` §4）。
- i18n：只消费已有 `usage.dashboard.table.*`；缺 key 才加中英对照。
- 长列表：当前窗口模型约几十行，不用虚拟滚动。Logs 的 `useVirtualList` 不动。
- `glass-panel` 留在 section 外壳；ledger 内部用 opaque `bg-elevated`，滚动行不上 glass。

## Trade-offs

| 选择 | 取 | 舍 |
|---|---|---|
| Feature `UsageLedger` 而非 `src/ui` Table | 无跨层、可含 status/share | 其他域暂时不能复用 |
| Grid ledger 而非 HTML `<table>` | 与 Logs 一致、易做双行路径 | 原生 table 语义略弱；用 `role="table"` / `row` / `columnheader` 补 |
| 排行保持 list 拓扑 | Overview/Cost 仍是 Top-N，不是全量六列 | 与全量表视觉不完全同一组件 |
| 日柱 datetime 工厂 + 默认 30 日窗口 | 轴标签可读，且「近 30 天」与查询一致 | all-time 仍可能是细柱；那是刻意的全历史 |

## Rollback

回退 `features/usage/components/Usage*.tsx`、对应 `styles/*`、`tests/usage/usage-table-layout.smoke.test.tsx` 与 `usage-tabs.smoke.test.tsx` 的断言即可。无数据迁移。
