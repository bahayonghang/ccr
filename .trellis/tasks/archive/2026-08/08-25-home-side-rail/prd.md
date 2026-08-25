# 首页右侧栏：下一步与事件流

父任务：`.trellis/tasks/08-25-react-home-style-redesign`（决策 D9：事件流保留既有筛选、channel 与聚合）
设计输入：父任务 `research/claude-design-source.html` 的 `1b` 右侧两块
审阅裁定：父任务 `research/plan-review-adjudication.md` 的 TPR-06

## Goal

按设计稿 `1b` 改写首页右侧栏的两块呈现：上半「下一步」为按风险排序的动作列表（首条为强调态），
下半「事件流」为紧凑行列表。
设计稿的三列行式**扩充**现有事件流，不替换——现有的筛选、`channel` 列与相邻聚合全部保留。

## 前置

`08-25-design-token-consolidation` 与 `08-25-home-runtime-layout` 先合入。
本子任务只改 `DashboardNextActions.tsx`、`DashboardSignalStream.tsx`、其对应 CSS 与测试，
不改 `DashboardView.tsx` 与 `dashboard-view.css`。

## Background and Confirmed Facts

`DashboardSignalStream.tsx` 现有能力，设计稿中均无对应元素，**全部保留**：

- `PillToggleGroup` 三档筛选 `all` / `warn` / `error`，每档标签自带计数。
- 独立 `channel` 列（`dashboard-signal__channel`）。
- 相邻同 `message` + `channel` + `level` 的条目聚合为一行并显示 `×N`。
- 空态含 `/monitoring` CTA；非空时页脚另有 `/monitoring` 链接。
- `matchesFilter` 的语义：`warn` 档同时包含 `warn` 与 `error` 两级，不是只看 `warn`。

现有计数口径（**本任务不改动**）：

- 计数基于 `aggregatedEntries`，即**聚合后、筛选前、截断前**。
- 可见行为 `aggregatedEntries.filter(matchesFilter).slice(0, limit)`，`limit` 默认 6。
- 因此「标题计数」与「可见行数」本来就可以不相等（聚合与截断造成），这是既有正确行为，不是缺陷。

## Requirements

- R1：「下一步」列表首条为强调态，其余为静默态；沿用既有 `actions` / `showOnboarding` props，不改父组件接口。
- R2：「事件流」行改为设计稿的紧凑三列视觉，但保留 `channel` 列——实际为四列栅格（时间戳 / 状态点 / channel / 文本 + 聚合计数）。沿用既有 `entries` / `limit` props。
- R3：保留 `all` / `warn` / `error` 三档筛选与其计数标签。计数口径维持现状（聚合后、筛选前、截断前），不改为可见行数。
- R4：保留相邻条目聚合与 `×N` 标记。
- R5：保留空态 CTA 与页脚 `/monitoring` 链接。
- R6：错误行与警告行各有 tint 底色，信息行无底色。tint 取值以令牌子任务 `research/token-name-delta.md` 的结论为准，不自造令牌名。
- R7：状态不只依赖颜色——错误/警告/信息三档需另有文字或图标区分。
- R8：两块的空态与截断行为明确：无动作、无事件时各自给出可读空态，不出现空白卡；文本超长省略而不撑破布局。
- R9：所有样式用语义令牌与 tint 令牌，不写硬编码十六进制颜色与 px 圆角字面量。
- R10：本子任务必须包含测试改动，至少覆盖：三档筛选切换后可见行集合正确、计数口径为聚合后筛选前截断前、相邻聚合产生 `×N`、`warn` 档包含 `error` 级。测试文件列入 change list。

## Acceptance Criteria

- [ ] AC1（R1）：首条动作为强调态且视觉上明显区别于其余条目；`actions` / `showOnboarding` props 契约未变。
- [ ] AC2（R2）：事件流行渲染为四列栅格且 `channel` 仍可见；时间戳为 mono。
- [ ] AC3（R3）：三档筛选控件存在且可切换；`warn` 档结果包含 `error` 级条目。
- [ ] AC4（R3,R10）：计数口径由测试断言固定为「聚合后、筛选前、截断前」。构造超过 `limit` 条数据时，标题计数大于可见行数，且此差异不判为缺陷。
- [ ] AC5（R4）：构造相邻重复条目时渲染出 `×N`，且聚合后计为一行。
- [ ] AC6（R5）：空态显示 CTA；非空时页脚 `/monitoring` 链接可见。
- [ ] AC7（R6,R7）：错误与警告行有 tint 底色、信息行无；灰度模拟下仍能区分三档。
- [ ] AC8（R8）：无数据时两块各自显示空态；超长文本省略且不产生横向滚动。
- [ ] AC9（R9）：本子任务改动的 CSS 中无硬编码十六进制颜色与 px 圆角字面量。
- [ ] AC10（R10）：change list 与提交包含测试文件改动。
- [ ] AC11：`just frontend-check-quick` 通过；中英文文案键完整。

## Out of Scope

- `useDashboardSignals` 的采集逻辑与日志来源。
- 监控页本身的改版。
- `DashboardView.tsx` 的栅格改动。
- 改动计数口径、删除筛选控件、删除 `channel` 列、删除聚合行为——这些是能力削减，父任务 D9 已排除。
</content>
