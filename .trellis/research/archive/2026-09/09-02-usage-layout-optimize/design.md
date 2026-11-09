# 技术设计：Usage 页 hero 区结构重排 + 成本趋势填充

## 根因

1. `.usage-hero-row`（`usage-dashboard-view.css`）为 `grid-template-columns: minmax(0,7fr) minmax(0,5fr)` + `align-items: stretch`。
2. `UsageCostConclusionCard` 根元素 `.usage-cost-conclusion`（`usage-cost-conclusion-card.css`）设 `height: 100%`，被动撑到右列高度。
3. 右列 `.usage-metric-grid` 为 2 列 × 3 张卡（每张 `min-height: 11.25rem`）≈ 23rem；左卡实际内容 ≈ 12rem → 中部死空间；且 3 卡在 2 列栅格留下第 4 空格子。
4. `UsageDashboardView.tsx` 页头 `status` 渲染 StatTile 展示 Total Cost，与 hero 卡重复。

关键事实：`buildUsageSummaryCards`（`ccr-ui/src/views/usage/usageSummaryCards.ts`）为 cost 卡**已计算好** `sparkline`（按桶日成本）、`averageLabel`、`peakLabel`、`deltaLabel`、`sparklineLabel`，`UsageCostConclusionCard` 目前未渲染它们 —— 趋势填充零数据管道改动、零新增 i18n key。Cost tab 已有完整日成本柱状图（`buildDailyBarChartOptions`），hero 不复刻它，改用与指标卡一致的 sparkline 词汇，避免图表重复。

## 空间命题（Spatial Thesis）

- 主阅读路径：工具栏过滤 → 成本结论带（本页主角）→ 辅助指标行 → 标签页明细。
- 分组：成本故事（数字 + 趋势 + token 构成）聚合在通栏带内；辅助指标为等权卡片行；标签切换器是明细工作区边界。
- 密度：Operate 模式，带内为紧凑横向分区；间距沿用现有 `0.8rem / 0.85rem` 节奏，不新增间距刻度。
- 适配：≥80rem 横向分区；<80rem 纵向堆叠；指标栅格 `auto-fit` 消除空轨。

## 目标拓扑（≥80rem）

```
usage-hero-row（flex column, gap 0.8rem）
├─ .usage-cost-conclusion（通栏带，grid 两行）
│   ├─ row1: [身份区: icon+label / value+delta / detail]  [趋势区: sparkline + average/peak dl]
│   └─ row2: children embed（UsageTokenBreakdownStrip，组件内部不动）
└─ .usage-metric-grid: repeat(auto-fit, minmax(16rem, 1fr)) —— 3 卡一行、无空轨
```

- 身份区与趋势区列比约 `5:7`（`minmax(0,5fr) minmax(0,7fr)`），趋势区给足横向空间。
- 趋势区结构对齐 `UsageMetricCard` 词汇：sparkline（复用 `../Sparkline`，新 CSS 类给约 4.5rem 高度）+ `dl`（average / peak，复用 `usage.dashboard.cards.average` / `.peak` 既有文案 key）。
- 带高约 15–17rem（内容驱动）；移除 `height: 100%` 被动撑高。

## 组件契约变更

- `UsageCostConclusionCard.tsx`：props 不变（`card` + `children`）；渲染扩展为身份区 / 趋势区 / embed 区三段，新增对 `card.sparkline / sparklineLabel / averageLabel / peakLabel` 的渲染，复用 `useUsageT` 既有 key。
- `UsageDashboardView.tsx`：删除 PageHeader `status` 属性与未再使用的 `StatTile` 导入；hero 结构类名不变（`.usage-hero-row` / `.usage-metric-grid`）。
- `usage-cost-conclusion-card.css`：根元素改两行 grid（row1 双区、row2 embed）；移除 `height: 100%` 依赖；新增趋势区与 sparkline 尺寸类。
- `usage-dashboard-view.css`：`.usage-hero-row` 改 flex 列；`.usage-metric-grid` 改 `repeat(auto-fit, minmax(16rem, 1fr))`；清理 80rem 下 hero 双列媒体查询，保留 <56.25rem 指标单列规则。
- 圆角 `1.35rem` 保留：该带是本页主面板（DESIGN.md 允许主面板 20px+）。

## 设计护栏（impeccable craft-floor / DESIGN.md）

- 不新增 eyebrow / kicker 文案；不新增 i18n key；不引入新色、新圆角、新阴影、装饰性 glass、渐变文字。
- sparkline 是承载真实趋势的既有组件词汇（指标卡同款），不是内容替身。
- 保留 reduced-motion 与既有交互行为；不改任何数据获取逻辑；不改 UsageTokenBreakdownStrip 内部。
- DOM 顺序与视觉顺序一致：身份区 → 趋势区 → 构成区 → 指标行。

## Out of Scope

- 标签页内容（Overview 趋势大图、SOURCE MIX 等）、UsageTokenBreakdownStrip 内部、主题 token、数据层、`usageSummaryCards.ts`。
