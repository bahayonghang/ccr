# Usage 子页表格样式优化

## Goal

Usage 子页上的用量排行与明细必须能被操作员扫读：名称、请求数、token、费用、占比分列对齐，而不是粘在一行或挤进 2rem 宽的格子。用户能在 Models / Projects / Providers 比较驱动因素，并在 Overview / Cost 的 Top 排行里认出项目与模型。

## Background

用户截图覆盖 Overview、Tokens、Cost、Providers、Models、Projects。根因是 Vue→React 迁移留下的 CSS/标记脱节，见 `research/table-markup-mismatch.md`。数据与 i18n 表头已经存在；缺的是列结构。

产品决策（2026-08-26）：**操作员密度表格**。Models / Projects / Providers 使用表头 + 多列（名称 / requests / tokens / cost / share；Models 加 pricing status）。数值右齐、`tabular-nums`、行 hover。项目路径主名可读，完整路径走 tooltip 或次行。

## Confirmed facts

- Models / Projects / Providers 把名称、token、费用放进无列网格的 `article`（`UsageTableTabs.tsx:14-20`、`:34-39`、`:53-59`）。对应 CSS 仍按 HTML `table` 或 `__item` 写，且没有 `__row` 规则。
- `.models-tab__table` 设了 `min-width: 106rem`（`usage-models-tab.css:81-86`），却套在 `div` 上，不是 `<table>`。
- Overview / Cost 排行 CSS 需要索引列 + 内容列（`usage-overview-tab.css:296-301`，`usage-cost-tab.css:101-106`），React 只渲染了内容节点，于是整行掉进 2.15–2.25rem 列。
- `ModelStatDto` 含 `request_count`、`total_tokens`、`cost_with_cache`、`pricing_status`；`ProjectStatDto` 含 `request_count` / `total_tokens` / `total_cost` / `project_path`；`ProviderBreakdownDto` 含 requests / tokens / `cost_with_cache_usd`；`SourceBreakdownDto` 含 `share_cost`。
- 表头文案已在 `usage.dashboard.table.*`（en-US `3140-3167` 与对应 zh-CN）。
- Tokens 日表明细的 CSS 还在（`usage-tokens-tab.css:123-167`），`toUsageTokenBreakdownRows` 可提供行数据，但 `UsageTokensTab.tsx` 不渲染表。
- Logs 子页已是可用 ledger（`UsageLogsTab.tsx:87-95`），作为视觉与结构对照，不作为本任务重写对象。
- `src/ui/` 没有 Table 原语。Usage 是 feature 域，共享表壳放在 `features/usage`，不新造 ui 原语、不加 token 名。

## Requirements

- R1：Models 工作区是操作员表格，不是粘连文本。列：Model、Requests、Tokens、Cost（`cost_with_cache`，缺省回退 `total_cost`）、Share（相对本表费用合计）、Pricing（现有 `usage.dashboard.table.status*` 文案 + 已有 status pill 样式）。行 hover；数值列右齐且 `tabular-nums`；模型名 ellipsis + `title` 全文。
- R2：Projects 工作区同样是操作员表格。列：Project、Requests、Tokens、Cost、Share。主名用末两段路径（与 `shortenPath` 同语义）；完整 `project_path` 在 `title` 和次行。
- R3：Providers 工作区同样是操作员表格。列：Provider、Requests、Tokens、Cost、Share。`provider` 空值走现有 `usageSourceFallbackLabel`。
- R4：三张表共用同一个 Usage ledger 壳（粘性表头、行网格、滚动、空态），列定义按页配置。禁止再套 `min-width: 106rem` 的假 `<table>` 类。桌面主列可读；窄于约 `60rem` 时表壳横向滚动，表头与行一起动。
- R5：Tokens 子页在现有柱状图下恢复每日 ledger。列：Date、Input、Output、Cache Read、Total（现有 `usage.dashboard.table.*` / tokens 文案）。空数据继续用 `usage.dashboard.table.noData`。柱状图走 datetime 工厂与 series 身份稳定（R11），keep-mounted 行为不变。
- R6：Overview「Model Cost Ranking / Project Cost Ranking」恢复可扫排行：索引、主名、费用、占比同一行左右对齐；`detail` 可见；`title` 为完整模型名或路径。项目主名至少能辨认仓库名，不能再被 2.25rem 列切成 `.../bah...`。
- R7：Cost「Source cost ranking」同样：索引、来源名、费用、占比（`share_cost`）对齐；来源名用 `usageSourceFallbackLabel`。
- R8：空列表三表 + Tokens 日表 + 两个排行都走现有 `noData` 文案，不渲染无表头的空网格冒充有数据。
- R9：契约测试放在 `ccr-ui/tests/**/*.smoke.test.ts(x)`。覆盖：三表表头与分格单元格、粘连回归（同一行文本不能把名称和 `$` 费用连在一起）、Projects 主名/完整路径分离、Overview/Cost 排行不只一个网格子节点、Tokens 日表在有 `trends` 时出现。
- R10：视觉验收在 `http://127.0.0.1:5173/` 的 Usage 路由、桌面宽度，dark × clay。不改 theme token 名，不新增 `@theme` 变量。
- R11：Tokens「Daily token composition」与 Cost「Cost concentration」日柱使用 `xaxis.type: 'datetime'`、`formatTrendAxisLabel`、`getTrendTickAmount`、`trim: false`。禁止 category 轴 + 全部 `YYYY-MM-DD`。options 不依赖 trends 数组身份；series 为 `{ x: UTC midnight, y }` 且按 join key 稳定引用。Breakdown 图例始终包含 Input / Output / Cache Read（`showForSingleSeries: true`）。
- R12：默认 `last_30d` 的 store `timeRange` 必须带本地起止日期。禁止 `rangePreset === last_30d` 且 `timeRange` 为空对象，以免查询变成 all-time、标签却显示「近 30 天」。

## Acceptance Criteria

- [ ] AC1（R1）：Models 渲染表头 Model / Requests / Tokens / Cost / Share / Pricing；每行至少六个独立单元格；`gpt-5.6-sol` 与 `$13,955.57` 不在同一文本节点里相邻。
- [ ] AC2（R2）：Projects 渲染表头 Project / Requests / Tokens / Cost / Share；主名是路径末两段；`title` 或次行为完整 `project_path`。
- [ ] AC3（R3）：Providers 渲染表头 Provider / Requests / Tokens / Cost / Share；空 provider 显示 fallback 标签，不显示空白主键。
- [ ] AC4（R4）：三表使用同一 ledger 壳 class；源码与产物都不再把 `min-width: 106rem` 用在这些表上；`60rem` 以下表壳可横向滚动。
- [ ] AC5（R5）：Tokens 在 `trends.length > 0` 时图下出现日表，列含 Date / Input / Output / Cache Read / Total；`UsageTokensTab` 的 ApexCharts `options` 工厂依赖不新增趋势数据身份（仍遵守 chart-stability）。
- [ ] AC6（R6）：Overview 每个排行项含索引节点 + 名称 + 费用 + 占比；费用与占比右齐；项目标签在桌面宽度下能读到最后一段路径。
- [ ] AC7（R7）：Cost 来源排行含索引 + 名称 + 费用 + 占比；费用右齐。
- [ ] AC8（R8）：对应列表为空时只出现 `usage.dashboard.table.noData`（或已有空态文案），不出现表头套零行的假表，除非该页明确用表头+空态组合且测试锁住。
- [ ] AC9（R9）：`cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/usage/usage-tabs.smoke.test.tsx tests/usage/usage-table-layout.smoke.test.tsx tests/usage/usage-daily-bar-chart.smoke.test.ts tests/state/state-store-actions.smoke.test.ts` 通过；`bun run type-check` 与 `bun run lint:ci` 通过。
- [ ] AC10（R10）：浏览器走查 Models / Projects / Providers / Tokens / Overview 排行 / Cost 排行；记录 viewport 与 `data-theme`/`data-flavor`；列对齐肉眼可扫。
- [ ] AC11（R11）：Tokens/Cost 日柱源码不含 `categories:`；工厂为 datetime 且 `tickAmount` 来自 `getTrendTickAmount`。`tests/usage/usage-daily-bar-chart.smoke.test.ts` 通过。
- [ ] AC12（R12）：`useUsageViewStore` 初态与 `resetFilters` 的 `timeRange` 等于 `getLocalDateRangeWindow(last_30d)`，含 start/end。

## Out of scope

- Overview 洞察卡缺色条、指标卡层级、饼图颜色未映射到排行点。
- Diagnostics / Logs ledger 重写与虚拟滚动改动。
- `PlatformUsageRankList`（平台首页，不是 Usage 子页）。
- 新 `src/ui` Table 原语、新 theme token、改 DTO / Tauri / llmusage。
- 排序点击、筛选、分页（Models/Projects/Providers 仍一次渲染当前窗口全部行）。
- 把 `glass-panel` 迁到 material surface 别名。

## Technical notes

- 层：`features/usage` 可新增域组件；禁止 `src/ui` 依赖 `features/`。
- 列表行 `memo`，禁止给行内联 object/function props（`react-rerender-discipline.md`）。
- 滚动表体不要再套一层 `backdrop-filter` glass。
- Share 在 Models/Projects/Providers 按本表 cost 合计计算；Cost 排行用 `share_cost`。
- 验证命令见 AC9；浏览器预览：`cd ccr-ui && bun run dev:web -- --host 127.0.0.1 --strictPort`，打开 `http://127.0.0.1:5173/` 再进 Usage。
