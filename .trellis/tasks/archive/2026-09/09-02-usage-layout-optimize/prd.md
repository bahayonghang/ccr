# 优化 Usage 页面布局：消除大片空白

## Goal

修复 Usage 页（数据看板）hero 区的大片空白：桌面宽屏（≥80rem）下左侧成本结论卡被拉伸到右侧指标卡列高度、中部约 40% 为死空间；右侧 3 张指标卡在 2 列栅格中留下一个空格子。同时移除页头与 hero 卡重复展示的 Total Cost 数字。方向已经用户拍板：**结构重排 + 内容填充**（通栏成本带内嵌成本趋势 + 指标卡整行排开）。

## Context / Evidence

- 用户提供的桌面端截图（2532×1531，深色主题）：TOTAL COST 大卡中部空白；右列 2×2 栅格右下角为空格子；页头右上 StatTile 与 hero 卡重复展示 `$12,742.56`。
- 根因（源码证据）：
  - `ccr-ui/src/features/usage/styles/usage-dashboard-view.css` `.usage-hero-row` 为 `7fr / 5fr` 双列 + `align-items: stretch`；`.usage-cost-conclusion`（usage-cost-conclusion-card.css）`height: 100%` 被动撑高。
  - 指标卡 `min-height: 11.25rem`（usage-metric-card.css），3 张卡撑出两行高度，左卡内容仅约一半。
  - `usageSummaryCards.ts` 实际产出 4 张卡（tokens / cost / activeDays / requests），cost 被抽进 hero 后右列只剩 3 张。
- 已确认的用户决策：
  1. 修复方向：结构 + 内容（通栏成本带内嵌成本趋势，指标卡补齐成整行）。
  2. 移除页头重复的 Total Cost StatTile。
  3. 视觉验收由用户在桌面端自行确认；交付侧负责类型检查、lint、测试与 impeccable 机械检测。

## Requirements

- R1 hero 区重排为通栏成本带：成本数字 / 环比 / 单价为身份区，成本趋势（该卡已有的 sparkline 数据 + average/peak 统计）为趋势区，既有 Token Breakdown Strip 为构成区；任何内容量下带内不出现死空间。
- R2 3 张指标卡（Total Tokens / Total Requests / Active Days）在一行内排开，栅格在任意卡片数量、任意视口宽度下不得出现空格子。
- R3 移除 PageHeader 右上角与 hero 卡重复的 Total Cost StatTile；页头仅保留标题与描述。
- R4 不新增数据管道、不新增 i18n key：趋势区复用 `UsageSummaryCard.sparkline / averageLabel / peakLabel / sparklineLabel` 既有字段、既有 `Sparkline` 组件与既有文案 key（average / peak）。
- R5 保持既有视觉世界（DESIGN.md「Editorial Control Room」）：沿用既有 surface / border / radius / token 词汇，不引入新颜色、新圆角规格、新阴影或装饰效果；Operate 模式的密度与可扫描性优先。
- R6 响应式：<80rem 时带内分区纵向堆叠；指标卡栅格在任意宽度下不产生空轨道、无横向溢出。
- R7 其余标签页（Overview / Tokens / Cost / Providers / Models / Projects / Logs）内容与行为不变；UsageTokenBreakdownStrip 组件内部不变。

## Acceptance Criteria

- [ ] AC1 ≥80rem 视口：成本带通栏展示，身份区 / 趋势区 / 构成区排布紧凑，无可见死空间；指标卡 3 张一行、无空格子（对照截图基线）。
- [ ] AC2 页头不再出现 Total Cost StatTile，成本数字全页仅出现一次（hero 带内）。
- [ ] AC3 成本带内渲染成本 sparkline 趋势与 average / peak 统计，数据来自既有卡片字段，无新增 API / i18n key。
- [ ] AC4 <80rem 与 <56.25rem 断点下布局纵向堆叠、无横向溢出、无空栅格轨。
- [ ] AC5 `cd ccr-ui && bun run type-check`、`bun run lint`、`bun run test` 全部通过。
- [ ] AC6 impeccable 机械检测 `detect.mjs --json --scope layout`（对改动文件）无未解释发现。
- [ ] AC7 用户在桌面端确认 hero 区空白消除（用户自行验收项）。

## Notes

- 技术方案见 `design.md`，执行计划见 `implement.md`。
