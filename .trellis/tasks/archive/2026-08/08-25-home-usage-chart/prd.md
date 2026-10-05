# 首页用量与成本图表区

父任务：`.trellis/tasks/08-25-react-home-style-redesign`
设计输入：父任务 `research/claude-design-source.html` 的 `1b` 左下「用量与成本」卡

## Goal

把 `DashboardUsageMovement` 的呈现层改写为设计稿 `1b` 的用量卡：标题行带 7D/30D/90D 切换，指标行给出请求（hero 数字）、TOKEN、成本、会话与平台图例，主体是按天按平台堆叠的柱状图，底部是日期轴与「打开完整报表」入口。

## 前置

`08-25-design-token-consolidation` 与 `08-25-home-runtime-layout` 先合入（与父任务 Task Map 一致）。
本子任务改 `DashboardUsageMovement.tsx`、`dashboard-usage-movement.css`、一个新增的成本子组件与对应测试，不改 `DashboardView.tsx` 与 `dashboard-view.css`。

## Background and Confirmed Facts

- `HomeUsageOverviewResponse` 无 cost 字段。成本只能来自既有命令 `get_usage_summary_v2` 的 `UsageSummaryDto.total_cost_usd`。
- `useUsageSummary(platform?, startDate?, endDate?)`（`ccr-ui/src/features/usage/queries.ts:80`）的 `queryKey` 为 `usageKeys.summary(platform, startDate, endDate)`，**随三个参数变化**。显式传入区间即可让成本与首页天数联动并在切换档位时重取。无参调用取的是另一区间语义，与 `activeDays` 无关。
- 该 hook **没有 `enabled` 参数**。延迟发起不能靠开关，只能靠条件挂载消费组件。修改 hook 签名会波及所有既有调用方，不在本任务范围。
- 首页区间口径由后端 `local_usage_date_window(days)` 定义：`end = 本地今天`，`start = end - (days - 1)`。前端构造 `startDate` / `endDate` 时按同一口径，避免与 overview 错位。
- `HomeOverviewSeriesItem` 按天携带四平台的 `sessions` / `requests` / `tokens`，堆叠柱为纯前端派生，不需要改 IPC 契约。
- 令牌子任务的结论：不引入 `--text-data-lg` / `--text-data-md` 角色令牌（除非其名称增量审计判定确需）。数据字号用既有 `--text-*` 档位，具体档位以令牌子任务 `research/token-name-delta.md` 的结论为准。

## Requirements

- R1：柱状图按天渲染，每根柱按平台分层堆叠，层色取平台色令牌；图例列出参与堆叠的平台。数据源为 `HomeUsageOverviewResponse.series[].{claude,codex,antigravity,opencode}`，纯前端派生，不改 IPC 契约。
- R2：指标行给出请求、TOKEN、成本、会话四项。请求为 hero 数字，其余为次级数据档。整个首页只允许存在这一个 hero 档数字。具体字号档位引用令牌子任务的结论，不自造令牌名。
- R3：成本取自既有 hook `useUsageSummary(undefined, startDate, endDate)`，`startDate` / `endDate` 按 `end = 本地今天`、`start = end - (activeDays - 1)` 构造，与 overview 同区间。不改 hook 签名，不新增 `src/api/` 封装，不新增 IPC 命令。
- R4：成本请求延后到首屏绘制之后发起。实现方式为条件挂载：父组件在首屏 perf mark 之后翻转标志位，再渲染调用该 hook 的成本子组件。
- R5：成本取不到、加载中或不适用时显示 `—`；有数据且 `total_cost_usd === 0` 时显示 `$0.00`。两者必须可区分。会话数为 0 时按真实 0 显示，不与「无数据」混淆。
- R6：7D/30D/90D 切换沿用既有 `activeDays` / `onChangeDays` props 契约，不改变父组件接口。切换后成本与其余三项指标同区间刷新。
- R7：保留既有的 loading 与 error 分支，且两者在新版式下都有明确呈现，不出现空白卡。
- R8：图表需给出非颜色的可辨手段（图例文字、`title` / `aria-label` 或数值标注），状态不只依赖颜色。
- R9：`prefers-reduced-motion: reduce` 下柱状图不做入场动画。
- R10：所有样式用语义令牌与平台色令牌，不写硬编码十六进制颜色与 px 圆角字面量。
- R11：本子任务必须包含测试改动，至少覆盖：堆叠柱派生纯函数（含 `maxDailyTotal === 0` 分支）、成本三态（`—` / `$0.00` / 正常值）、区间构造与 `activeDays` 的对应关系。测试文件列入 change list。

## Acceptance Criteria

- [x] AC1（R1）：暗色与亮色下柱状图按天渲染且按平台分层，层色与图例、平台卡的色条一致；柱数等于当前档位天数。
- [x] AC2（R2）：指标行四项齐备；全页扫描确认仅有一个 hero 档数字。
- [x] AC3（R3）：`rg -n 'invoke\(' ccr-ui/src/features/usage/dashboard/DashboardUsageMovement.tsx` 无命中；`src/api/` 与 `queries.ts` 无改动。
- [x] AC4（R3,R6）：成本使用的 `startDate` / `endDate` 与 `activeDays` 严格对应（`end = 今天`，`start = end - (activeDays - 1)`），切换档位后 query key 变化并重取。此项由单元测试断言，不靠肉眼。
- [x] AC5（R4）：成本子组件在首屏 perf mark 之后才挂载；首屏 perf mark 不因成本请求推迟。
- [x] AC6（R5）：Web 预览（无 Tauri IPC）下成本显示 `—`；有数据但成本为 0 显示 `$0.00`，两者在界面上可区分。
- [x] AC7（R6）：切换 7D/30D/90D 后图表、指标行（含成本）同步刷新，父组件 props 契约未变。
- [x] AC8（R7）：loading 与 error 各自有可见呈现，均不出现空白卡。
- [x] AC9（R8）：关闭颜色区分（灰度模拟）后仍能判断各层归属。
- [x] AC10（R9）：`prefers-reduced-motion: reduce` 下无柱状入场动画。
- [x] AC11（R10）：本子任务改动的 CSS 中无硬编码十六进制颜色与 px 圆角字面量；`just frontend-check-quick` 通过。
- [x] AC12（R11）：change list 与提交包含测试文件改动，且断言覆盖 R11 列出的三类。
- [x] AC13：中英文文案键完整。

## Out of Scope

- 修改 `get_home_usage_overview_v2` 或 `get_usage_summary_v2` 的后端实现与返回类型。
- 引入图表库。堆叠柱用现有 CSS/SVG 手法实现，不新增运行时依赖。
- 完整用量报表页（`/usage`）的改版。
- `DashboardView.tsx` 的栅格改动。
