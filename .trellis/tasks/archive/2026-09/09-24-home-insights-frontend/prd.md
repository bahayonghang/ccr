# 首页 Insights 前端实现

父任务：`09-24-home-insights-redesign`（决策 D1-D3 与现状事实见父任务 `prd.md`）。

## 前置条件

- `09-24-home-insights-backend` 已完成：`HomeInsightsResponse` 的 ts 绑定与 `getHomeInsights` generated client 已提交。
- `09-24-home-insights-design` 已完成：`ccr-ui/DESIGN.md` 已有 Insights 签名组件条目，token 已登记，`research/chart-overflow.md` 已给出首屏图表越界的原因或复现条件。

两项未完成时不得 `task.py start`。

## Goal

按设计定稿实现首页：修复首屏已知缺陷，在首屏之下渲染 Insights 区块，数据来自一次 `getHomeInsights` 调用。

## Requirements

- R1 首屏图表高度：design 子任务 `research/chart-overflow.md` 已确认越界来自已安装桌面包的旧前端（早于 `51ea4b1b`），dev 分支代码已受限（1920×1080 实测 256px）。本任务不改首屏图表代码；加入 Insights 后须保证 `.dashboard-usage__chart` 仍为 `clamp(10rem, 26vh, 16rem)`，并用当前分支重新构建的桌面包验收。首屏其余构图不变。
- R2 数据接入：
  - `src/api/domains/usage.ts`（或 `stats.ts`，与 `getHomeUsageOverviewV2` 同处）导出 `getHomeInsights`；feature 代码只经 domain facade 调用（`.trellis/spec/ccr-ui/frontend/api-facade-boundary.md`）。
  - `src/features/usage/queries.ts` 新增 `homeUsageKeys.insights()` 与 `useHomeInsights()`；`staleTime` 用现有 `USAGE_STALE_TIME`。导入完成后随 `homeUsageKeys.all` 失效（`DashboardView.tsx` 的 `refreshHomeUsage` 已失效该前缀）。
  - 非 Tauri 运行时不发起调用，渲染 Web 预览说明态（与首屏 `backendStatus === 'unsupported'` 一致）。
  - Insights 查询在首屏数据之后发起（例如区块进入视口或空闲时），不得拖慢首屏 `dashboard:*` perf mark。
- R3 组件（放 `src/features/usage/dashboard/insights/`，每个区块一个 `PascalCase.tsx`，样式放 `src/features/usage/styles/`）：总量行、最近 7 天对比、53 周热力图与 streak 说明行、按周分 agent 堆叠趋势、分布面板（小时/星期/月切换）、榜单（Agents/Projects/Models 切换 + 度量切换）。周趋势与热力图各带「图表 / 表格」切换，表格视图给出完整数值（2026-09-24 用户决定，作为低于 3:1 颜色的替代通道）。形式、高度、颜色、空态全部以 `DESIGN.md` Insights 条目为准。
- R4 展示逻辑（排序、前 N、合并 "其他"、百分比变化、streak 文案、最忙时段）放纯函数模块 `insightsPresentation.ts`，组件只渲染。
- R5 来源显示名与颜色：来源键集合用 `USAGE_PLATFORM_IDS`（`src/types/usage.ts:55-65`）；显示名用 i18n `usage.platforms.<key>`，回退 `usageSourceFallbackLabel`（`src/views/usage/usageSources.ts`）；颜色用 design 子任务登记的 `--color-platform-<token>` token（来源键 `kimi_code`、`deepseek_harness` 在 token 名中写作 `kimi-code`、`deepseek-harness`），建一张来源键 → token 名的表。未登记来源（`unmapped = true`）显示原始键，使用中性色 `--color-chart-other`。不在组件中按平台名写分支（`.trellis/spec/ccr-ui/frontend/platform-surface-contracts.md`）。
- R6 会话诚实状态：`sessions_indexed === false` 时，所有会话数显示 `—`（U+2014）并带 `title="会话尚未索引"`，不显示 0。
- R7 图表实现：用 CSS/SVG 自绘，与现有 `DashboardUsageMovement` 同方式；不引入新的图表依赖；不为 Insights 引入 ApexCharts。`bun run check:bundle-budget` 通过。
- R8 可访问性：每个图表有 `aria-label` 摘要；热力图格与柱有可读 `title`；视图/度量切换控件可键盘操作并有可见焦点；动画遵守 reduced-motion。
- R9 i18n：新增文案同时写入 `src/i18n/locales/zh-CN.ts` 与 `en-US.ts`，并更新对应 `*.keys.txt`；文案不出现 "prompts"。
- R10 重渲染纪律：遵守 `.trellis/spec/ccr-ui/frontend/react-rerender-discipline.md`（含 `t()` 不放空依赖 memo 的条款）。

## Acceptance Criteria

- AC1 桌面端 1920×1080：首屏用量图高度不超过 16rem；Insights 全部区块渲染；对照设计画布逐区块截图，差异列入 PR 描述。
- AC2 smoke 测试（`tests/dashboard/`）：
  - `insights-presentation.smoke.test.ts`：前 N 与 "其他" 合并；百分比变化（前 7 天为 0 时的显示）；streak 文案；榜单按度量重排；未登记来源回退。
  - `dashboard-insights.smoke.test.tsx`：四种空态（空库、仅会话、仅用量、Web 预览）；`sessions_indexed=false` 显示 `—`；度量切换、视图切换与「图表 / 表格」切换的键盘操作；周趋势与热力图表格视图的行列内容（最新周在前、晚于 `as_of` 的日为空格）。
  - 现有 `dashboard-usage-movement` 与 `route-view-mount` smoke 测试通过。
- AC3 `just frontend-check-quick`、`bun run check:i18n`、`bun run check:bundle-budget`、`bun run build` 通过。
- AC4 `frontend-quality-reviewer` 审查无未处理的阻断项。

## Out of Scope

- 后端改动（发现 DTO 缺字段时回到 backend 子任务处理，不在前端推算）。
- 首屏平台行情带扩展、"会话 0" 根因修复。
