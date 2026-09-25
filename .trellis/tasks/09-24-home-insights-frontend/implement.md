# Implement: 首页 Insights 前端实现

启动前确认 prd "前置条件" 两项已满足。

## Checklist

### 1. 首屏图表高度回归保护
- [ ] 读 design 子任务 `research/chart-overflow.md`（结论：越界来自旧桌面包，dev 代码已受限，不改首屏图表代码）。
- [ ] 挂载 Insights 后，用 design 子任务的 Playwright 复现脚本（`research/chart-overflow.mjs`；design 任务归档后位于 `.trellis/tasks/archive/2026-09/09-24-home-insights-design/research/`）重测 1920×1080 下 `.dashboard-usage__chart` 高度，仍为 256px。
- [ ] 验证：`cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/dashboard/`

### 2. 数据接入

- [ ] `src/api/domains/usage.ts` 导出 `getHomeInsights`。
- [ ] `queries.ts`：`homeUsageKeys.insights()`、`useHomeInsights({ enabled })`。
- [ ] 验证：`bun run type-check`

### 3. 纯展示逻辑

- [ ] `insightsSources.ts`、`insightsPresentation.ts`（design §3）。
- [ ] `tests/dashboard/insights-presentation.smoke.test.ts`（prd AC2 第一组）。
- [ ] 验证：`bunx vitest run --config vitest.smoke.config.ts tests/dashboard/insights-presentation.smoke.test.ts`

### 4. 区块组件与样式

- [ ] 按 DESIGN.md Insights 条目实现 6 个区块与 `DashboardInsights` 容器；样式 `dashboard-insights.css` 只用 token，px 字面量须在豁免登记内。
- [ ] `DashboardView.tsx` 挂载 `<DashboardInsights isNativeRuntime={...} />`。
- [ ] i18n：`zh-CN.ts`、`en-US.ts` 与 `*.keys.txt`。
- [ ] `tests/dashboard/dashboard-insights.smoke.test.tsx`（prd AC2 第二组）。
- [ ] 验证：`just frontend-check-quick`、`bun run check:i18n`

### 5. 构建与包体

- [ ] `bun run build`、`bun run check:bundle-budget`

### 6. 视觉核对

- [ ] Web 预览（`bun run dev:web -- --host 127.0.0.1 --strictPort`）核对空态与布局；`just ui-dev` 下用真实数据核对首屏高度与 Insights 区块。
- [ ] 对照设计画布（https://claude.ai/code/artifact/d0871e3f-04ba-4a2d-a82f-2f2788af83d1）逐区块截图，差异记入任务 notes 与 PR 描述。已知差异：DeepSeek Harness 身份色已由画布的靛色 `#635dab` 改为天蓝 `#1e8dfe`（2026-09-24 用户确认），以 `DESIGN.md` 为准。

### 7. 审查

- [ ] dispatch `frontend-quality-reviewer`（改动跨多文件并触及 `queries.ts` 与 i18n）。
- [ ] 若步骤 1 改动了 `DashboardUsageMovement`，按 `dashboard-presentation-contracts.md` §Tests Required 重跑三个 dashboard smoke 测试。

## Review Gates

- 步骤 3 完成后：纯函数测试全部通过再写组件。
- 步骤 6 完成后：用户确认视觉差异可接受。

## Rollback Points

- 步骤 2-5 一个提交；撤回时 Insights 整体移除，首屏不受影响。
