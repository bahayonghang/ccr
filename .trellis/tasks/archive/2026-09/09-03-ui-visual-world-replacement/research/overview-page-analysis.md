# Overview 首页深度分析（2025-09-03）

> 来源：3 个 explore 子代理对 `ccr-ui/src/` 的只读分析。注：ccr-ui 前端实际是 **React 18 + react-i18next + react-router**（TSX），仓库级 AGENTS.md 的 "Vue 3" 描述已过时。

## 路由与组成

- 首页路由 `''` → `dashboard`：`ccr-ui/src/shell/routeCatalog.ts:15`，懒加载于 `ccr-ui/src/features/usage/routeLoaders.ts:1-2`。
- 页面根组件：`ccr-ui/src/features/usage/dashboard/DashboardView.tsx:47`；骨架在 `:232-317`。
- 头部（标题/副标题/就绪徽标）：`DashboardView.tsx:235-273`（共享 `PageHeader`）；状态检查项 pills 在 `:274-289`（`.dashboard-header__reasons`），数据来自 `ccr-ui/src/views/dashboard/dashboardPresentation.ts:371-466` 的 `buildReadiness`。
- 平台统计卡：`ccr-ui/src/features/usage/dashboard/DashboardPlatformMatrix.tsx`（矩阵 `:153-177`，卡 `:73-151`，迷你柱状 `PlatformSparkline :36-71`）。
- "Usage and cost" 面板：`ccr-ui/src/features/usage/dashboard/DashboardUsageMovement.tsx`（面板 `:286-345`，7D/30D/90D 切换 `:293-312`，指标行 `:184-217`，图表 `:139-157`，单柱 `UsageStackBar :74-97`）。
- Action queue：`DashboardNextActions.tsx:78-143`（队列 `.slice(0, 4)` 于 `dashboardPresentation.ts:555`）。
- Event stream：`DashboardSignalStream.tsx:39-144`。

## 柱状图失控的根因（"柱状图太大了"）

图表是**手写 flexbox div，无图表库、无 SVG**：柱 = `div.dashboard-usage-stack` + 内联 `--stack-height: N%`，段 = `span.dashboard-usage-segment` + `--segment-height: N%`（`DashboardUsageMovement.tsx:80-96`）。

百分比数学本身没问题（`DashboardCostMetric.tsx:98-124`：柱高 = 当日总量/窗口内最大日总量，峰值日恒为 100%）。**真正的问题是图表容器没有任何 max-height**：

```css
/* dashboard-usage-movement.css:131-141 */
.dashboard-usage__chart { display:flex; flex:1; align-items:flex-end; min-height: var(--space-32); /* 仅 8rem 下限 */ }
/* dashboard-usage-movement.css:1-11 */
.dashboard-usage { display:flex; flex-direction:column; height:100%; }
/* dashboard-view.css:170-189 */
.dashboard-lower { display:grid; flex:1; grid-template-columns: 1.85fr 1fr; align-items: stretch; }
.dashboard-rail  { display:grid; grid-template-rows: auto 1fr; }
```

膨胀链：`align-items: stretch` 让左列被右栏（Action queue + 6 条 Event stream，固有高度远超 128px）撑高 → `.dashboard-usage{height:100%}` 跟随 → `flex:1` 的图表吞掉全部余量 → 峰值柱 `--stack-height:100%` 渲染成数百 px 巨柱。另：`.dashboard-view{min-height:100%}`（`dashboard-view.css:1-4`）因父级 `.route-page` 无高度（`styles/shell-critical.css:38-40`）实际是 no-op。

修复方向：给图表一个确定高度（clamp/max-height 或固定行高网格），让右栏自己滚动；不要靠百分比柱去适应被撑开的容器。

## 信息可信度问题（"首页信息不对"）

- **Sessions 恒为 0**：用量总览的请求/token 来自外部 `llmusage` crate（`src-tauri/src/services/usage.rs:709-740`），而 sessions 走**另一条数据通路**——ccr-db `session_archive` 表（`services/usage.rs:847-895`，合并于 `:1153-1169`，`total_sessions` 于 `:1190`）。session 归档未索引时（`needs_session_index`，`:1151`）requests 非零而 sessions=0，页面上表现为"假的 0"。需要诚实的状态表达（未索引提示）而不是静默 0。
- **Cost 是独立查询**：不在 overview 响应里；`DashboardCostMetric.tsx:55-74` 在 idle 后单独发 `get_usage_summary_v2`（llmusage `overview().total_cost_usd`），未就绪显示 `—`。
- **平台卡丢弃了 sessions 指标**：`buildPlatformRows`（`dashboardPresentation.ts:307-341`）构造了 requests/sessions/tokens 三个指标，但卡片只渲染 requests + tokens（`DashboardPlatformMatrix.tsx:82-83,138-147`）。
- **Antigravity 颜色张冠李戴**：专用的 `--color-platform-antigravity` token（`tokens.css:133-137`，dark `:275-279`，`#98afc9`）**全 `src/` 无人消费**；所有 Antigravity 表面都映射到 Gemini 蓝（dark 下 `#7d97b6` ≠ `#98afc9`）：图表段/图例 `dashboard-usage-movement.css:123-125,200-202`，平台卡 `dashboard-platform-matrix.css:51-52`，图标 `DashboardView.tsx:164`，导航色样 `MainLayoutNav.tsx:15`。

## 本页 i18n

本页翻译完整（`useUsageT()`，`dashboard.*` 双语言齐全：`en-US.ts:653-822` / `zh-CN.ts:~638-790`）。仅有 `tf()` 回退参数里的英文兜底（`DashboardUsageMovement.tsx:299,336`）会被真实词条覆盖。疑似死键：`dashboard.usage.peakLabel/hoverHint/metricSelectLabel/metricPlatforms`（`en-US.ts:779-795`，无组件引用）。

## 相关既有 spec

- `.trellis/spec/ccr-ui/frontend/usage-chart-stability-contracts.md`
- `.trellis/spec/ccr-ui/frontend/dashboard-presentation-contracts.md`
- `.trellis/spec/ccr-ui/frontend/environment-scoped-dashboard-contracts.md`
