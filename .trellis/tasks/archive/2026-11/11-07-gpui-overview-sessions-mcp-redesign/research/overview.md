# Research: GPUI Overview 与 Tauri Dashboard 对照（空白根因、数据可用性、重排建议）

- **Query**: 对照 Tauri 首页 Dashboard 与 GPUI Overview，定位 GPUI 大片空白的根因、11-06/11-07 无柱原因、可复用数据与必须保留的合同，并给出 2000×1080 与 960×640 的推荐版式
- **Scope**: internal（代码 + 本机 llmusage 数据库只读查询）
- **Date**: 2026-11-07

---

## 1. Tauri 首页（Dashboard）构成

### 1.1 路由

- `/` 的 index 子路由 id 为 `dashboard`，`cacheKey: 'DashboardView'`：`ccr-ui/src/shell/routeCatalog.ts:15`；挂在 `MainLayout` 下：`ccr-ui/src/shell/router.tsx:94-98`。
- 规格引用：`react-shell.smoke.test.tsx` 断言根路由挂载 `.dashboard-view`（`.trellis/spec/ccr-ui/frontend/dashboard-presentation-contracts.md` StatTile 场景 Tests Required）。

### 1.2 自上而下的区块顺序（`ccr-ui/src/features/usage/dashboard/DashboardView.tsx:240-333`）

| 序 | 区块 | 位置 | 内容 | 数据来源 |
|---|---|---|---|---|
| 1 | 页头 `dashboard-header` | :243-298 | `PageHeader`（标题、eyebrow、readiness 描述）+ readiness 徽章（带未通过项计数）+ 主操作链接 + “全部平台”链接；下方 readiness 理由清单（✓/⚠ 行） | `buildDashboardPresentation`（`ccr-ui/src/views/dashboard/dashboardPresentation.ts`）：systemInfo、cliVersions、overview、logs |
| 2 | `DashboardPlatformMatrix` | :299-304 | 4 个平台格（Claude Code / Codex / Antigravity / OpenCode，`DashboardView.tsx:142-191`），每格：图标+名称、CLI 版本、状态点、sparkline、3 列指标（请求 / Token / 会话，`dashboard-platform-matrix.css:219-224`） | `platformRows`（含 `version`、`sparkline`、`trackingHealth`，`dashboardPresentation.ts:67-75`） |
| 3 | `dashboard-lower` 两列 | :305-324 | 左：`DashboardUsageMovement`；右 rail：`DashboardNextActions` + `DashboardSignalStream limit=6` | 见下 |
| 3a | `DashboardUsageMovement` | `DashboardUsageMovement.tsx:211-276` | 标题 + “Updated {time}”（`overview.last_updated`）+ 7/30/90 范围 pill；`UsageMetricsRow`（成本/请求/Token/会话 + 图例）；堆叠柱图；footer 轴标签 + “完整报告”链接 | `HomeUsageOverviewResponse`（`useHomeUsageOverview(activeDays)`，`DashboardView.tsx:52`）；柱数据 `deriveStackedUsageBars(overview.series)`（`DashboardCostMetric.tsx:117-129`） |
| 3b | `DashboardNextActions` | `DashboardNextActions.tsx:78+` | 动作队列；首启时内联 3 步 onboarding（`ONBOARDING_STEPS` :15-41） | `presentation.actions` / `isFirstRun` |
| 3c | `DashboardSignalStream` | `DashboardSignalStream.tsx:39-63` | 最近 6 条事件（同消息聚合、按级别过滤） | `useDashboardSignals(24)`：Tauri `app-log` 事件 + `getMonitoringFeed`/`getRecentEvents`（`useDashboardSignals.ts:1-3`） |
| 4 | `DashboardInsights` | `insights/DashboardInsights.tsx:24-149` | 标题 + 范围 `trend_start – as_of`（:52）；`InsightsTotalsLine`（全历史 totals）；宽行 1.85:1 `InsightsActivityHeatmap` \| `InsightsWeekCompare`（:125-132）；`InsightsAgentTrend`（有请求时）；1:1 行 `InsightsDistribution` \| `InsightsLeaderboards`（:134-137） | `useHomeInsights`（空闲时才取，:31-35）→ `HomeInsightsResponse` |
| 5 | `DashboardStatusBar` | :330 | 后端状态 + 信号计数 | backendStatus、logs |

注意：Tauri 的 KPI（成本/请求/Token/会话）在用量面板内部（`UsageMetricsRow`），不是独立 KPI 带；GPUI 把它拆成了独立 KPI 带。

### 1.3 网格与纵向空间如何被填满

- `.dashboard-view { min-height: 100% }`、`.dashboard-workbench { flex: 1; flex-direction: column }`：`ccr-ui/src/features/usage/styles/dashboard-view.css:1-15`。
- `.dashboard-lower { flex: 1; grid-template-columns: 1.85fr 1fr; align-items: start }`：`dashboard-view.css:178-186`；≤1024px 单列：:205-212。rail `grid-template-rows: auto 1fr`：:192-197。
- 两列 `align-items: start`，不拉伸等高；主图高度随视口：`.dashboard-usage__chart { height: clamp(10rem, 26vh, 16rem) }`（160–256px，`dashboard-usage-movement.css:140-149`）。
- 信号流 `max-height: 16rem`（`dashboard-signal-stream.css:45`）。
- **纵向主要靠内容量填满**：lower 之后还有 Insights 整段（热力图 53 周网格 `dashboard-insights.css:280-287`、趋势图 `clamp(10rem,26vh,16rem)` :425-431、分布图 `10rem` :468-472、排行榜 8 行 `insightsPresentation.ts:234`），首屏之下自然滚动。没有靠 flex 拉伸面板去“铺满”视口。
- Insights 行网格：`.insights-row` 1:1、`.insights-row--wide` 1.85:1、<1024 单列：`dashboard-insights.css:129-149`。

### 1.4 各 Insights 模块的数据需求（`HomeInsightsResponse`，`ccr-ui/src/types/generated/usage/HomeInsightsResponse.ts`）

| 模块 | 字段 |
|---|---|
| `InsightsTotalsLine` | `totals.{requests,tokens,sessions,agents,projects,active_days}`、`sessions_indexed`（未索引显示 `—`，`insightsPresentation.ts:39-62`） |
| `InsightsActivityHeatmap` | `daily[]`（53 周内有请求的日）、`current_streak`、`longest_streak`、`busiest_day`、`totals.requests`；图 / 表双视图 |
| `InsightsWeekCompare` | `last_7_days` / `previous_7_days`（`InsightsWindow{requests,sessions,active_days}`）、`sessions_indexed`（`buildWeekCompare` :109） |
| `InsightsAgentTrend` | `trend_start`、`trend[]{source, weekly[53]}` |
| `InsightsDistribution` | `hourly[24]` / `weekday[7]` / `monthly[12]`；峰值小时 caption |
| `InsightsLeaderboards` | `agents[]{source,requests,tokens,sessions,unmapped}` / `projects[]` / `models[]`（`InsightsTally{key,label,requests,tokens}`），8 行；会话维度受 `sessions_indexed` 约束 |

全部是**全历史 / 53 周窗口**，不随 7/30/90 范围变化（Tauri 用独立标题范围 `trend_start – as_of` 标明，`DashboardInsights.tsx:52`）。

---

## 2. GPUI Overview 现状与空白根因

### 2.1 结构（`ccr-desktop/src/features/overview.rs:233-271`）

```
overview-root  (size_full, flex_col, overflow_y_scroll, p=24, gap=16)   :240-252
├─ header       (h1 + lens 胶囊 | 7D/30D/90D + Refresh；状态行)         :299-386
├─ overview-kpis           (report 存在时)                              :416-523
├─ overview-platform-band  (report 存在时)                              :577-607
└─ 下排 div (flex, flex_shrink_0, gap 16; 宽: items_stretch / 窄: flex_col) :260-269
   ├─ overview-usage  (flex_1, p=16)   → chart()                        :609-644
   └─ overview-rail   (w=240, p=16)    → Next actions ×4 + Data source  :817-870
```

壳层：页面挂在 `shell-main`（`flex_1 min_h_0`）里，下面是 28px `shell-status`（`ccr-desktop/src/shell/mod.rs:1157-1185`），所以 `overview-root.size_full()` 的高度等于“纸面高 − 28px”。

### 2.2 根因 A：图表面板内部 ~130px 空白

1. 柱区是**固定高度**：`bars()` `.h(px(theme::OVERVIEW_CHART_HEIGHT))`（`overview.rs:1417-1425`），每个日列同高（:1467-1476），段高按常量换算像素（:1514-1516）。`OVERVIEW_CHART_HEIGHT = 216.`（`ccr-desktop/src/theme.rs:45`）。
2. `chart()` 返回的是普通 `flex_col`，没有 `flex_1`（`overview.rs:677`），`usage_panel` 只是 `flex_col + gap + p`（:616-629），面板内没有任何子项吸收多余高度。
3. 下排宽屏用 `items_stretch()`（:266），所以 `overview-usage` 被拉到与 `overview-rail` 同高；右栏的内容高度决定了这一排的高度：标题 + 4 个动作行（每行 `mt 8 + h 36`，`action_row` :976-1006，`ROW_HEIGHT=36` theme.rs:17）+ “数据源”小节（`mt 16 + pt 16` + 标题 + 状态 + “Report as of” + 覆盖说明，:873-957）。覆盖说明是一整句英文（:940-943），在 240px 栏宽（`OVERVIEW_RAIL_WIDTH=240`，theme.rs:49，内容宽 208px）里折成约 4 行。估算右栏 ≈ 400px，而图表面板内容 ≈ 16 + 标题行 ~32 + 8 + 216 + 8 + 刻度 16 + 16 ≈ 312–340px。差值即面板内 x 轴下方的空白（截图约 130px）。
4. 单测把“等高”写死：`overview-rail.bottom == overview-usage.bottom`（`overview.rs:1862-1865`）；DESIGN.md 也要求“两列等高（stretch），不留死区”（`ccr-desktop/DESIGN.md:137`）。等高本身达成了，但被拉高的是面板外框，不是图表，所以视觉上仍是死区。

### 2.3 根因 B：页面底部 ~200px 空白

- `overview-root` 是 `size_full + flex_col`（:245-247），但所有子项都是 `flex_shrink_0` 且**没有任何子项 `flex_1`/`flex_grow`**：header `flex_shrink_0`（:328）、KPI `flex_shrink_0`（:490）、平台带 `flex_shrink_0`（:593）、下排 `flex_shrink_0`（:263）。内容总高小于视口时，剩余高度全部落在最后一项之下。
- 2000×1080 估算：外场景边距 `clamp(min(1040,440)/8,0,20)=20`（DESIGN.md Elevation 末段）→ 纸面 ≈ 1040，减状态栏 28、上下 padding 48，可用 ≈ 964（若有原生标题栏再减）。header ~56 + KPI ~110 + 平台带 ~140 + 下排 ~400 + 3×16 gap ≈ 754 → 底部剩 ≈ 200px，与截图一致。
- 对照：Tauri 不靠拉伸，靠下方 Insights 整段内容填满并滚动（§1.3）。GPUI Overview 目前没有任何 Insights 类模块（`overview-insights` 这个 id 实际是“数据源”小节，`overview.rs:877`）。

### 2.4 11-06 / 11-07 无柱：数据缺失，不是渲染 bug

- 读取链：`read_workbench` 以报告时钟日期为 `as_of`，`since = as_of − (days−1)`（`ccr-desktop/src/runtime/insights.rs:559-574`），`home_overview` 只返回有桶的日期（`crates/ccr-usage/src/db.rs:831-915`，SQL 按 `usage_bucket_30m` 分组）；`dense_days` 对缺失日期补 0（`insights.rs:189-216`）；全零日在 `day_stack` 画虚线基线（`overview.rs:1484-1493`）。
- 本机库只读查询（`%USERPROFILE%\.llmusage\llmusage.db`）：`max(hour_start) = 2026-11-05T06:30:00Z`；10-30 起逐日只有 claude/codex 数据，11-05 之后无任何桶。`run_log` 最近一条：`sync failed 2026-11-05T06:34:13Z`，错误 `no such table: message: Error code 1: SQL logic error`。
- GPUI 打开页面从不触发导入（`insights.rs:38-40` 文档注释 “opening the page never starts an import”）；Refresh 只是重读库（`overview.rs:111-176`）。Tauri 首页有 bootstrap 导入（`DashboardView.tsx:73-80`，`needs_usage_import` 时自动导入）。
- GPUI 状态行写的是“7 days ending {as_of}”（`overview.rs:200-230`），as_of 是时钟日期而不是最后数据日；`last_sync_at` 在 `read_workbench` 已查询的 `dashboard.overview(&filter)`（`insights.rs:577-584`）里有（`OverviewPayload.last_sync_at`，`crates/ccr-usage/src/queries.rs:33-45`），但只取了 `total_cost_usd`，新鲜度被丢弃。Tauri 用户面板显示 “Updated {time}”（`DashboardUsageMovement.tsx:224-227`）。
- 相关已有合同：“空窗口 + 过期 llmusage 同步不是 usageReady”（`dashboard-presentation-contracts.md` 中 “Empty selected window with stale llmusage sync is not usageReady” 场景，禁止删除用户 `llmusage.db` 来隐藏空窗口）。

### 2.5 其他尺寸细节

- 窄宽判断用窗口宽 `viewport_size().width < NARROW_WIDTH(1040)`（`overview.rs:239`，theme.rs:29），不是内容宽。
- Overview 固定用 `CONTENT_PADDING=24`（`overview.rs:250`），DESIGN.md Layout 写“窄窗 16px”（DESIGN.md:128）——当前页面未按窄窗切换 padding。
- 平台 sparkline 高 34px（`SPARK_HEIGHT`，theme.rs:51），格最小宽 158px（theme.rs:50）。截图中“小 sparkline”即此常量。
- 精确值表 `max_h(240)`、内部独立滚动（`overview.rs:755-768`）；展开时主列变高、右栏被拉伸。

---

## 3. 数据可用性：Tauri 模块能否由 GPUI 已有数据驱动

关键事实：`read_workbench` 每次刷新**已经**调用了 `dashboard.insights(全历史)` + `assemble_home_insights`（`insights.rs:606-645` 的 `session_window`），得到完整的 `HomeInsightsSnapshot`（`crates/ccr-app/src/usage/home.rs:48-76`），但只取会话计数后丢弃。`WorkbenchReport` 结构见 `insights.rs:151-178`。另外 GPUI 的 Usage 页已能读 `InsightsQuery::Home`（`insights.rs:497-510`，`features/insights/mod.rs:334-339`），但只渲染成 agents 行表与一行 summary（`features/insights/rows.rs:68-80,352-376`）。

| Tauri 模块 | 所需字段 | GPUI 状态 | 结论 |
|---|---|---|---|
| 活动热力图（53 周）+ streak / busiest day | `daily`、`current_streak`、`longest_streak`、`busiest_day` | 已在 `session_window` 计算（`insights.rs:614-628`），未透传 | **数据现成**（需给 `WorkbenchReport` 加字段透传；无新 SQL） |
| 周对比（近 7 日 vs 前 7 日） | `last_7_days` / `previous_7_days`（requests/sessions/active_days） | 同上；`last_7_days.sessions` 已用于 KPI（`insights.rs:632-637`） | **数据现成** |
| 分布（小时 / 星期 / 月） | `hourly[24]`、`weekday[7]`、`monthly[12]` | 同上 | **数据现成** |
| 排行榜（模型 / 项目 / Agent，全历史） | `models`、`projects`、`agents` | 同上 | **数据现成** |
| 排行榜（按 7/30/90 窗口） | 窗口内模型 / 项目 | `Dashboard::model_breakdown` / `project_breakdown` 接受同一 `QueryFilter`（`crates/ccr-usage/src/db.rs:424,547`），`read_workbench` 未调用 | **需新增调用**（已有投影，无新 SQL） |
| Agent 周趋势 | `trend_start`、`trend[]` | 同上 | **数据现成** |
| 全历史 totals 行 | `totals.{agents,projects,active_days,…}` | 同上 | **数据现成** |
| 新鲜度 “Updated / 最后同步” | `OverviewPayload.last_sync_at` | `dashboard.overview(&filter)` 已调用（`insights.rs:577-584`），字段被丢弃 | **数据现成** |
| 平台矩阵的 CLI 版本 / 安装状态 | Tauri `getCliVersions` / `getSystemInfo` | Overview 无此数据；GPUI 其他运行时（`runtime/platforms.rs`）未在本次调查中确认对应投影 | **需新接线**（未深入，不确定） |
| 信号流（最近事件） | app-log / monitoring feed | GPUI 有 `runtime/monitoring.rs` `Report.entries`（:57-62），由同一 Dashboard 目的地的 Monitoring lens 消费 | **需新接线**；归档合同写“右栏不新增查询” |
| 首启 onboarding | CLI 计数 + 用量是否为 0 | Overview 有 `report.requests` | 可行但依赖 CLI 计数（同上） |

限定条件：
- `assemble_home_insights` 需要会话归档 pool；归档不可用时 `session_window` 返回 `Unavailable` 并丢弃整个快照（`insights.rs:623-628`）。若要在“无归档”时仍显示热力图等用量模块，需改为在 assemble 前保留 `InsightsPayload`（`crates/ccr-usage/src/insights.rs:90-120` 起，含 `daily/hourly/weekday/monthly/trend/models/projects/sources`）。
- 这些字段都是全历史 / 53 周，与页头 7D/30D/90D 胶囊（`overview.rs:388-413`，DESIGN.md:134 “整页作用”）口径不同，渲染时必须单独标注口径（Tauri 做法：`trend_start – as_of`）。
- 会话相关数值在 `sessions_indexed=false` 时必须显示未索引状态，不能显示 0。

---

## 4. 必须保留的合同

来源：`ccr-desktop/DESIGN.md`、`.trellis/spec/ccr-ui/frontend/dashboard-presentation-contracts.md`、`.trellis/tasks/archive/2026-11/11-07-gpui-airy-glass-material/{prd.md,design.md}`。

1. **不铺零 / 不编数**：无记录给原因、库不存在或查询失败不铺零；会话未索引写“未索引”（DESIGN.md:124、:130、:136、:217、:245；归档 prd R4 平台带）。spec：sessions honesty 场景、平台 sparkline “all-zero 不等于 untracked”，`overview == null` 时不造零数组。
2. **派生值必须标注**：日均 = 总量 ÷ 窗口天数，必须标“日均 / avg”（DESIGN.md:135；`overview.rs:424-431`）。
3. **KPI 带**：单面板 4 栏 hairline，Cost 32/600 为每屏唯一主读数，其余 24/500；<1040 改 2×2（DESIGN.md:120、:135）。
4. **平台带**：只有有逐日记录的平台单独成格，其余合并“其他来源”，窄宽横向滚动不换行（DESIGN.md:136；`overview.rs:575-607`）。
5. **图表**：高度 160–240px（Overview 逐日图 216px）；自动收紧到首个有数据日前一天并标 “since MM-DD”；全窗口无记录保留完整虚线基线；最上段 2px 圆角、段间 1px；系列色 `theme::chart_series`；数据更新无入场动画（DESIGN.md:243）。**若要让图表随面板拉伸超过 240px，需要先改 DESIGN.md。**
6. **精确值表焦点**：默认收起；最新在上；连续全零日折叠为一行且可聚焦；第 i 个显示行对应表格第 i+1 个子项；Tab/Shift+Tab 同时 reveal 表格与外层页面（DESIGN.md:244；`overview.rs:273-297`、:1309-1337；单测 :1588-1607、:1760-1800 区间）。
7. **右栏**：下一步 + “数据源”小节，数据来自视图已有状态，不新增查询；两列等高，不留死区；窄宽右栏堆到主列下方，不横向滚动整页（DESIGN.md:137；归档 design.md §4 “右栏…不新增查询”；归档 prd R4 “不能留死区，也不能编造数字”）。
8. **页头**：唯一 h1 + 内嵌 lens 胶囊同一行；范围胶囊作用整页（DESIGN.md:134；`tests/native_shell.rs:190-205`）。
9. **材质**：阅读面（表格、列表）一律实色，不叠玻璃（DESIGN.md Elevation 1-3 条）。
10. **数据层边界**：归档任务约束“Overview 只能使用已有 report / services 数据，不新增伪造统计”；用量 SQL 只在 `crates/ccr-usage`（AGENTS.md）。
11. **演示数据标识**：fixture 带“Isolated demo / 隔离演示”（`overview.rs:377-384`、:911-917）。
12. **先改 DESIGN.md，再改 theme.rs，最后改视图**（归档 prd 约束节）。

---

## 5. 推荐版式

> 以下为调用方要求的布局建议；每个模块都映射到已存在的数据。尺寸为逻辑 px 估算。

### 5.1 宽屏 ~2000×1080（≥1040，非 stacked）

可用内容宽 ≈ 2000 − 侧栏 216 − 外边距 40 − padding 48 ≈ 1696；可用高 ≈ 964。

```
[页头] Overview [Overview|Monitoring]         [7D|30D|90D] [Refresh]
       ● Local usage · 7 days ending 11-07 · Last sync 11-05 14:30     ← last_sync_at（已查询，现被丢弃）
[KPI 带] Cost | Requests | Tokens | Sessions                  ~110
[平台带] Claude Code | Codex | Other sources                   ~140
[下排: 主列 flex_1  :  右栏 ~340–380px（约 2:1 至 1.85:1）]       ~540
  主列 (flex_col, gap 16)                    右栏 (flex_col, gap 16)
  ├ 逐日请求面板（图 216，不拉伸）~330        ├ 下一步（4 行）              ~220
  └ 活动热力图面板（53 周、streak、           ├ 近 7 日 vs 前 7 日（周对比）~140
     busiest day；标“全历史”）~190            └ 数据源（配置、截止、最后同步、覆盖）~160
[首屏之下 · 可滚动] 洞察行 1:1：
  分布（小时/星期） | 排行榜（模型/项目/Agent，8 行；标“全历史”）  ~300
```

- 合计首屏 ≈ 56 + 110 + 140 + 540 + 3×16 ≈ 894 ≤ 964：首屏填满且无面板内死区；主列 ≈ 330+16+190 = 536 与右栏 ≈ 220+140+160+32 = 552 接近，等高拉伸只多出几像素。
- 右栏变宽后“数据源”覆盖说明不再折 4 行，右栏高度主要由三个真实模块组成，不再靠一段长文案撑高。
- 图表保持 216px（DESIGN.md:243），不需要改动图表高度合同；空白由真实模块（热力图）吸收。
- 若坚持右栏 240px：周对比可移入主列热力图面板右侧（Tauri 同为 1.85:1 宽行 `Heatmap | WeekCompare`，`DashboardInsights.tsx:125-132`）。

数据映射：

| 模块 | 数据 | 现状 |
|---|---|---|
| 状态行“最后同步” | `OverviewPayload.last_sync_at` | 已查询，需透传 |
| KPI / 平台带 / 逐日图 / 精确值表 | 现有 `WorkbenchReport` | 不变 |
| 活动热力图 | `HomeInsightsSnapshot.daily/current_streak/longest_streak/busiest_day` | 已计算，需透传 |
| 周对比 | `last_7_days` / `previous_7_days` | 已计算，需透传（会话受 `sessions_indexed` 约束） |
| 下一步 | 现有 4 个 `OverviewLink` | 不变 |
| 数据源 | 现有 + `last_sync_at` | 小改 |
| 分布 | `hourly` / `weekday`（/ `monthly`） | 已计算，需透传 |
| 排行榜 | `models` / `projects` / `agents`（全历史）；窗口版需调用 `model_breakdown` / `project_breakdown` | 全历史已计算；窗口版需新增调用 |

### 5.2 最小窗口 960×640（stacked，<1040）

可用内容宽 ≈ 960 − 窄侧栏（`NARROW_SIDEBAR=184`，theme.rs:28）− padding ≈ 728；可用高 ≈ 640 − 28 − 48 ≈ 564（外边距为 0）。

```
[页头]（可换行）
[KPI 带 2×2]                                   ~200
[平台带]（横向滚动）                             ~140
—— 首屏到此约满，以下滚动 ——
[逐日请求面板] 全宽，图 216                      ~330
[活动热力图] 全宽；53 列 × 10px + 间距 ≈ 636 ≤ 728，必要时面板内横向滚动
[下一步 | 周对比] 1:1 两列（728 宽可容纳），或单列堆叠
[数据源] 全宽
[分布] / [排行榜] 单列堆叠
```

- 窄宽不追求填满视口，内容已超过一屏；只需保证不横向滚动整页（DESIGN.md:137）。

### 5.3 实现注意（事实性提示）

- `overview-root` 内若要让某一行吸收余量，需要 `flex_1`/`min_h`，并验证在 `overflow_y_scroll` 容器内的 shrink 行为；当前所有子项都是 `flex_shrink_0`（§2.3）。
- 若改为“图表随面板高度伸缩”，段高目前按 `OVERVIEW_CHART_HEIGHT` 换算像素（`overview.rs:1514-1516`），需要改为相对高度，而且与 DESIGN.md 的 160–240px 合同冲突。
- `overview-insights` 这个 id 目前指“数据源”小节（`overview.rs:877`），新增真正的洞察模块时命名会冲突。

---

## 6. 会受影响的测试与 test id

### 6.1 `ccr-desktop/src/features/overview.rs` 内单测

- `exact_rows_are_newest_first_and_fold_every_quiet_run`（:1588-1607），`chart_trims_to_one_day_before_the_first_record`（:1609-1619）：纯函数测试，不受版式影响。
- `overview_ranges_hide_stale_totals_and_preserve_exact_metrics`（:1621 起，1280×800 窗口，:1643-1646）：手工构造 `WorkbenchReport`（:1659-1680 等），**给 `WorkbenchReport` 加字段时每个字面量都要补**。依赖的 id / 断言：
  - `overview-total-tokens / -requests / -sessions / -cost`、`overview-codex-requests`、`overview-grok-requests`（不存在）、`overview-other-grok`、`overview-other-pi`
  - `overview-exact-toggle`、`overview-exact-table`、`overview-exact-{MM-DD}`、`overview-exact-quiet-{MM-DD}`（焦点顺序 :1760-1800、:1888-1940 区间）
  - `overview-root` bounds（:1787，精确值行 reveal 验证）
  - `overview-heading`、`overview-window-7/30/90`、`overview-refresh`（同行断言 :1849-1858）
  - **`overview-rail.bottom == overview-usage.bottom`（:1862-1865）** —— 主列改为多面板容器时，此断言的目标 id 需要随之调整
  - `overview-insights` 可见（:1866）
  - `overview-chart-since`（:1883）、`overview-day-10-31` 不存在 / `overview-day-11-01` 存在（:1886-1887）
  - `overview-open-usage / -sessions / -commands / -install` → `take_pending`（:1974-1981 区间）
- `ccr-desktop/tests/native_shell.rs`：
  - `f6_tab_navigation_does_not_intercept_search_editing`（:157）内 `overview-heading` 与 `tool-overview` 同行（:190-205）
  - `overview_states_missing_usage_without_sample_numbers`（:442-501）读 `overview-status` 文案（:469-475），断言 “No local usage source”
- `ccr-desktop/src/runtime/insights/tests.rs`：`workbench_reads_seeded_usage_and_does_not_invent_sessions`（:293）、`workbench_folds_stored_source_aliases_without_inventing_daily_bars`（:352）—— `WorkbenchReport` 增字段时需更新
- `ccr-desktop/src/runtime/benchmark/tests.rs:843`：使用 `InsightsQuery::Home`，改动 Home 读取路径时相关
- `ccr-desktop/tests/test_commands.py`：未引用 overview id

### 6.2 验证命令

- `just desktop-test`（`ccr-desktop/justfile:49-53`）、`just desktop-test-ui`（:56-58，`--test native_ui --test native_shell`）、`just desktop-lint`、`just desktop-fmt-check`。

---

## Caveats / Not Found

- 高度数值是按常量与布局代码估算的，未做原生像素测量；截图所述 ~130px / ~200px 与估算量级一致。
- 是否存在原生标题栏占用高度，本次未确认。
- 11-06/11-07 结论基于本机 `llmusage.db` 只读查询；`run_log` 失败原因（`no such table: message`）属于 llmusage CLI 侧，本次未追查。
- GPUI 是否已有可复用的 CLI 版本 / 安装状态投影（对应 Tauri 平台矩阵的版本列）未深入调查。
