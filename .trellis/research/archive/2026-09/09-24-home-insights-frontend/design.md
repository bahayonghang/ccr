# Design: 首页 Insights 前端实现

## 1. 文件布局

```
ccr-ui/src/features/usage/dashboard/
├── DashboardView.tsx                 # 首屏之后挂载 <DashboardInsights />
└── insights/
    ├── DashboardInsights.tsx         # 取数、状态分派（加载/错误/空态/Web 预览），按 DESIGN.md 顺序排列子区块
    ├── InsightsTotalsLine.tsx
    ├── InsightsWeekCompare.tsx
    ├── InsightsActivityHeatmap.tsx
    ├── InsightsAgentTrend.tsx
    ├── InsightsDistribution.tsx      # 小时/星期/月三视图
    ├── InsightsLeaderboards.tsx      # Agents/Projects/Models + 度量切换
    ├── insightsPresentation.ts       # 纯函数：排序、前 N、合并、变化率、文案键、网格坐标
    └── insightsSources.ts            # 来源键 → i18n 键 / 颜色 token 名
ccr-ui/src/features/usage/styles/dashboard-insights.css
```

`DashboardView.tsx` 已 322 行，只增加挂载点与所需的 props（`isNativeRuntime`），不在其中写 Insights 逻辑。

## 2. 数据流

```
getHomeInsights (generated, src/api/generated/usageV2.ts)
  → src/api/domains/usage.ts 导出
  → useHomeInsights()  (queries.ts, key = ['home-usage', 'insights'])
  → DashboardInsights  (enabled = isNativeRuntime && 已进入视口或空闲)
  → buildInsightsPresentation(data, { metric, view, topN })  (纯函数)
  → 各区块组件（只读 presentation）
```

- query key 放在 `homeUsageKeys` 下，复用 `refreshHomeUsage` 对 `homeUsageKeys.all` 的失效，导入完成后自动刷新。
- 延迟取数：`IntersectionObserver` 观察 Insights 容器，或 `scheduleWhenIdle`（`src/utils/scheduling`，首屏已用）。二选一，按 perf mark 实测选择；默认 `scheduleWhenIdle`，与首屏现有模式一致。
- 首屏的 `DashboardView` 被路由缓存（`routeCatalog.ts:15` 的 `cache: true`），Insights 查询在路由重新激活时不重复发起（TanStack Query staleTime 控制）。

## 3. 展示逻辑要点

- 热力图：53 列 × 7 行，列 = 周（`trend_start` 起），行 = 周一至周日。由 `daily` 构建 `Map<date, requests>`；晚于 `as_of` 的格不渲染。共 5 档（DESIGN.md）：0 请求为 `heat-0`；非零日按非零请求数的 25/50/75 分位切为 `heat-1`…`heat-4`（≤p25 → 1，≤p50 → 2，≤p75 → 3，其余 → 4）。分级规则写在 `insightsPresentation.ts`。窗口内无请求的格（含 `first_date` 之前）为 `heat-0`。
- 周趋势：前 4 来源（DESIGN.md N = 4）按 53 周窗口总量选取，同量按 `USAGE_PLATFORM_IDS` 顺序（与 `SourceKind::ALL` 一致）。后端 `trend` 同量按来源键字母序，所以前端不能直接取前 4 项，须自行重排。未登记来源不参与前 4，计入 "其他"。堆叠顺序与图例顺序按 DESIGN.md 的固定顺序。柱高按全窗最大周总量归一。
- 7 天对比：请求数与会话数显示百分比变化 `delta = (last - prev) / prev`，格式 `+21.7%` / `−17.6%`（U+2212），相等显示 `持平`；活跃天数显示天数差（`+2 天` / `−1 天`）。`prev = 0` 且 `last > 0` 显示 `新增`；两者都为 0 显示 `—`。颜色保持 secondary ink。
- 表格视图：周趋势与热力图各一个「图表 / 表格」分段切换（`role="radiogroup"`，默认图表）。周趋势表：每周一行、最新周在前，列为周起始日、前 4 来源 + "其他"（按堆叠顺序）、周合计；热力图表：每周一行、最新周在前，列为周起始日、周一至周日、周合计，晚于 `as_of` 的日为空格。数值为完整值（千分位分隔），行列由 `insightsPresentation.ts` 生成，与图表视图共用同一份 presentation 数据。
- 最忙时段：`hourly` 最大值下标；全零时不显示该说明。
- 榜单：度量切换后重排再取前 8 行（DESIGN.md 上限）；会话度量只对 Agents 开放，切到 Projects/Models 时回退到请求数；`sessions_indexed = false` 时会话选项禁用并带 `title` 说明；无请求时默认度量为会话。`unmapped` 行排在已映射行之后同度量排序，显示原始名，色条用 `--color-chart-other`。`agents` 行数可能多于 `totals.agents`（后端契约见 `.trellis/spec/ccr/backend/llmusage-provider-adapter.md` Home Insights 场景），总量行显示 `totals.agents`。
- 会话诚实：`sessions_indexed === false` 时会话数统一经一个格式化函数输出 `—`，并带 `title="会话尚未索引"`。
- 大数：沿用首屏 `compactLabel`（`src/features/usage/dashboard/DashboardCostMetric.tsx:148`，`Intl.NumberFormat(undefined, { notation: 'compact', maximumFractionDigits: 1 })`），完整值放 `title`。该格式跟随运行时默认 locale，不跟随应用语言设置，与首屏一致（2026-09-24 用户决定）。
- 空态判定：空库 `totals.requests === 0 && totals.sessions === 0`；仅会话 `totals.requests === 0 && sessions_indexed && totals.sessions > 0`；仅用量 `!sessions_indexed && totals.requests > 0`；Web 预览为非 Tauri 运行时，不发起调用。各状态的呈现以 DESIGN.md Empty states 为准。

## 4. 与现有契约的关系

- `dashboard-presentation-contracts.md`：首屏五个子组件不改接口；首屏越界修复若需改 `DashboardUsageMovement` 的 CSS，按该契约与 `usage-chart-stability-contracts.md` 执行并更新对应 smoke 测试。
- `api-facade-boundary.md`：`invoke()` 只在 generated client；不改 `src/api/tauri.ts` 兼容门面。
- `layering-contracts.md`：`features/usage/dashboard/insights/*` 只依赖 `@/api` domain、`@/ui`、`@/types`、`@/utils`，不反向依赖 `views/`；`usageSourceFallbackLabel` 位于 `src/views/usage/usageSources.ts`，若层级规则不允许 feature 依赖 `views/`，改为把该映射移到 `src/features/usage/` 下并保留原路径的 re-export。实施前用 `just frontend-lint` 的层级规则确认。

## 5. 取舍

- 不用 ApexCharts：Insights 图形是离散格与离散柱，CSS/SVG 自绘可控、无包体增量；首屏已采用同方式。
- 不做首屏平台行情带扩展：属于父任务不在范围内的项。
