# Overview 首页重构：图表比例与信息可信度

## Goal

按新视觉世界重构 Overview 首页（`运行概览`）：修复图表比例失控，重建信息层级，让数据展示诚实可信，并按新世界重组页面结构（页头检查项、平台卡、用量面板、Action queue、Event stream）。

## Requirements

1. **图表固定尺度**（核心缺陷）：当前 `.dashboard-usage__chart` 只有 `min-height: 8rem` 无上限，`flex:1` 吞掉右栏撑出的全部高度（膨胀链见 `research/overview-page-analysis.md`）。改为确定高度（clamp 或固定行高网格），右栏自行滚动；峰值柱恒 100% 但不超出图表区。保持 `usage-chart-stability-contracts.md` 既有契约。
2. **信息层级重建**：按方向契约重组 DashboardView 骨架（`DashboardView.tsx:232-317`）；状态检查项、平台卡、用量面板、右栏的比例与视觉权重按新世界重排。
3. **Sessions 诚实态**：sessions 走 ccr-db `session_archive` 独立通路，未索引时（`needs_session_index`，`services/usage.rs:1151`）显示"未索引/索引中"状态而非静默 0；评估是否在后端响应中显式携带索引状态字段（如需改 IPC 契约，先在 design.md 中评审）。
4. **平台卡指标完整性**：`buildPlatformRows` 构造的 sessions 指标当前被渲染层丢弃（`DashboardPlatformMatrix.tsx:82-83,138-147`）——要么渲染，要么从构造中移除，不留半截。
5. **图表色与平台色**：改用确权后的平台色 token（依赖 theme-token-world）；Antigravity 不再映射 Gemini 蓝。
6. **死键清理**：`dashboard.usage.peakLabel/hoverHint/metricSelectLabel/metricPlatforms`（`en-US.ts:779-795`）无组件引用，确认后删除并同步 zh-CN 与 key 计数（`check-i18n.mjs` 的 `EXPECTED_LEAF_COUNT`）。
7. 本页 i18n 已完整，保持 `dashboard.*` 双语言齐全；遵循全局中文化决策检查 eyebrow。

## Acceptance Criteria

- [ ] 窗口从最小到 4K 任意高度，图表区域高度有界；右栏内容超出时右栏自滚动，左列不被撑高
- [ ] 7D/30D/90D 切换下柱形比例正确，峰值柱不溢出图表区
- [ ] Sessions 未索引时页面显式提示，而非显示 `0`；索引完成后显示真实值
- [ ] 四个平台卡使用各自确权平台色；Antigravity ≠ Gemini 蓝
- [ ] `bun run type-check|lint|test|build` 全绿；相关 smoke test 更新/新增
- [ ] 新视觉方向在本页按方向契约完整落地（页面结构、色彩纪律、状态表达）

## Dependencies / Ordering

- 依赖 `09-03-theme-token-world` 的 token 落地后再做视觉层收尾；结构性修复（图表尺度、Sessions 诚实态）可先开工。

## Notes

- 分析：`../09-03-ui-visual-world-replacement/research/overview-page-analysis.md`
- 关键文件：`features/usage/dashboard/{DashboardView,DashboardUsageMovement,DashboardCostMetric,DashboardPlatformMatrix,DashboardNextActions,DashboardSignalStream}.tsx`，样式 `features/usage/styles/dashboard-*.css`，后端 `src-tauri/src/services/usage.rs`
