# 首页 1b 运行时布局：栅格重排与平台卡阵列

父任务：`.trellis/tasks/08-25-react-home-style-redesign`（决策 D1：首页采用 1b 运行时卡方案；D5：就绪 pill 留在内容区；D8：未跟踪判据用 source_health）
设计输入：父任务 `research/claude-design-source.html` 的 `1b` 区块
审阅裁定：父任务 `research/plan-review-adjudication.md` 的 TPR-03 / TPR-04 / TPR-09 / TPR-11

## Goal

按设计稿 `1b` 重建首页的栅格骨架与上半屏：首屏第一块是四张平台运行时卡，下方为左大图右侧栏两栏。
本子任务是首页栅格的唯一负责人，其余首页子任务在其骨架内改各自组件内部。
`DashboardReadinessLedger` 承载的信息在移除前逐项重新落位，不做静默丢失。

## 前置

`08-25-design-token-consolidation` 必须先合入。本子任务只用既有语义令牌，不自造颜色值。

## Background and Confirmed Facts

- `DashboardReadinessLedger.tsx` 消费两组数据：`readiness`（`status`、`labelKey`、`titleKey`、`descriptionKey`、`reasons[]`）与 `statusMetrics[]`。两者均由 `buildDashboardPresentation` 产出（`dashboardPresentation.ts:652-654`）。
- `MainLayoutTopbar` 位于 shell 层，不持有 dashboard presentation。把 readiness 提升到顶栏需要引入跨层 store 依赖。父任务 D5 决定不提升。
- 后端 `usage.rs` 的 `empty_home_platform_map()` 为每个首页平台插入零值统计，`build_home_date_range_from()` 逐日补齐。**未跟踪平台同样得到长度等于所选天数的全零序列。** 无法用「序列全零」或 `DashboardPlatformRow.state`（表示 CLI 安装/运行状态）判断平台是否被跟踪。
- 可用的跟踪信号：`HomeUsageOverviewResponse.archive.source_health: Array<UsageSourceHealth>`，其中 `UsageSourceHealth = { source: string, state: "live" | "degraded" | "missing", ... }`。`source` 字段的取值域与 `usageKey` 的对照关系尚未验证。
- `--color-platform-opencode` 由令牌子任务新增；`antigravity` 平台复用 `--color-platform-gemini`（名称漂移，本任务不重命名）。
- 响应式的仓库既定写法是 CSS Media Queries Level 4 区间语法加 px 字面量，例：`ccr-ui/src/styles/components/profiles-page.css:184` `@media (width >= 1280px)`、`:190` `@media (width <= 1279px)`。`tokens.css` 的 `--breakpoint-*` 位于标注「仅用于参考」的 `:root` 块，不在 `@theme` 中，既不生成 Tailwind 变体也不能用于 `@media` 条件。
- `ccr-ui/tests/dashboard-presentation.smoke.test.ts` 覆盖 `buildDashboardPresentation` 的输出契约，是 sparkline 新字段的落测位置。
- 设计稿 `1c` 要求 chrome 层实色。仓库现状 `--surface-shell-bg` → `--material-glass-chrome-bg` → `--color-bg-elevated`（`blur: none`）已满足该要求。组件侧是否消费该语义别名尚未验证。

## Requirements

- R1：先核查 `shell.css` 的 `sidebar-glass` / `topbar-glass` 是否已消费 `--surface-shell-*`。已消费则 chrome 层为零改动；未消费则改为消费该语义别名。不新增 chrome 颜色令牌，不改 `--material-glass-chrome-bg` 的定义。
- R2：侧栏分组标题（平台 / 配置 / 工具）用 mono 标签样式（`--font-mono`、`letter-spacing: 0.16em`、`font-weight: 600`）；平台条目前置 6px 平台色识别块。侧栏保留现有 5 个平台入口与既有路由，不增删导航项。
- R3：顶栏保留面包屑与既有 `EnvironmentSwitcher`，不新增元素。就绪 pill 与主行动按钮落在 `DashboardView` 的区块标题行（父任务 D5）。
- R4：`DashboardReadinessLedger` 的每一项信息在移除前逐项落位。落位表见 `design.md` §7，每项结论为「迁移到 X」或「删除，理由 Y」二选一，不留悬空项。
- R5：`DashboardPlatformMatrix` 原地改写为四张平台卡：顶部 3px 平台色条、平台名与版本、状态 chip、sparkline、请求与 TOKEN 两项数据。props 契约（`rows` / `installedCliCount` / `runtimeCliCount` / `className`）保持不变。
- R6：sparkline 数据由 `buildDashboardPresentation` 从 `overview.series` 派生，作为 `DashboardPlatformRow.sparkline?: number[]` 新增可选字段。只加字段，不改已有字段。
- R7：平台跟踪状态判据用 `archive.source_health[]`，不用全零序列，不用 CLI 安装状态。实施第一步先验证 `source` 字段取值能否对应 `usageKey`；若不能对应，则不显示占位态并在 `design.md` 记录原因，不得用零值冒充占位（父任务 D8）。
- R8：`DashboardView` 栅格改为「平台卡阵列（1 行 4 列）＋ 下方 1.85fr / 1fr 两栏」；响应式断点用 CSS Level 4 区间语法加 px 字面量，三档为 ≥1440px、1025–1439px、≤1024px。窄窗口下平台卡降为 2 列、两栏堆叠为单列。
- R9：所有新增样式用语义令牌，不写硬编码十六进制颜色与 px 圆角字面量。
- R10：本子任务必须包含测试改动：`dashboard-presentation.smoke.test.ts` 扩展 sparkline 派生断言（含 `overview == null` 与 `series` 为空的分支）；平台卡占位分支的 DOM 断言落在组件测试中。测试文件列入 change list。

## Acceptance Criteria

- [x] AC1（R1,R2）：暗色 clay 下侧栏、顶栏、内容区、卡片四层可辨；侧栏分组标题为 mono 标签样式，平台条目有平台色识别块。若 R1 核查结论为零改动，本条只验证视觉结果。
- [x] AC2（R3）：顶栏可见面包屑与环境切换，无新增元素；就绪 pill 与主行动按钮在首页区块标题行可见，pill 数字与 `readiness` 一致。
- [x] AC3（R4）：`design.md` §7 的落位表每一行都有结论，无「待定」；被删除的项有具体理由。
- [x] AC4（R5,R6）：四张平台卡各自渲染平台色条、版本、状态 chip、sparkline 与两项数据；sparkline 柱数等于当前 `activeDays`。
- [x] AC5（R6,R10）：`DashboardPlatformRow` 仅新增 `sparkline` 可选字段，既有字段与消费者不受影响；`dashboard-presentation.smoke.test.ts` 新增断言覆盖正常派生、`overview == null`、`series` 为空三种输入。
- [x] AC6（R7）：占位分支的触发条件是 `source_health` 中该平台的 `state`，可在测试中构造 `state: "missing"` 的 fixture 触发，且构造全零 `series` 不触发占位。若结论为不实现占位态，则本条改为验证 `design.md` 已记录原因且界面未显示误导性的 0。
- [x] AC7（R4）：`DashboardView.tsx` 不再引用 `DashboardReadinessLedger`；被移除的信息按 AC3 的落位表可在新界面上找到或有删除理由。
- [x] AC8（R8）：1440px、1280px、1024px 三档下无横向滚动、无重叠、无截断，平台卡在 1024px 降为 2 列。CSS 中媒体查询用 px 字面量，不引用 `--breakpoint-*`。
- [x] AC9（R9）：本子任务改动的 CSS 文件中无硬编码十六进制颜色与 px 圆角字面量。
- [x] AC10（R10）：change list 与提交包含测试文件改动。
- [x] AC11：中英文文案键完整，新增文案两种语言都有键，无缺键回退。

## Out of Scope

- 用量大图内部实现（归 `08-25-home-usage-chart`）。
- 右侧「下一步 / 事件流」内部实现（归 `08-25-home-side-rail`）。
- 顶栏 PROFILE 下拉：应用无全局 profile 概念，按父任务 D4 不实施。
- 把 readiness 提升到 shell 层 store（父任务 D5）。
- 把 Grok 加入首页卡阵列。
- 修改路由、IPC 命令或 `src/api/` 封装。
- 重命名 `--color-platform-gemini`。

## Open Questions

以下两项在实施第一步内解决，不阻断启动：

- `shell.css` 的 chrome 类是否已消费 `--surface-shell-*`（决定 R1 是否为零改动）。
- `UsageSourceHealth.source` 的取值域能否对应 `usageKey`（决定 R7 走占位态还是走记录原因分支）。
</content>
