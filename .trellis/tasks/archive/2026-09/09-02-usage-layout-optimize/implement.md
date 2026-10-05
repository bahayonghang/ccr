# 执行计划

## Steps

1. `ccr-ui/src/features/usage/UsageDashboardView.tsx`：移除 PageHeader `status`（StatTile）及未再使用的导入。
2. `ccr-ui/src/features/usage/components/UsageCostConclusionCard.tsx`：重排为 身份区 / 趋势区（Sparkline + average/peak `dl`）/ embed 区三段。
3. `ccr-ui/src/features/usage/styles/usage-cost-conclusion-card.css`：带内两行 grid、row1 双分区（5fr / 7fr）、趋势区与 sparkline 尺寸类、<80rem 纵向堆叠。
4. `ccr-ui/src/features/usage/styles/usage-dashboard-view.css`：`.usage-hero-row` 改 flex 纵向堆叠；`.usage-metric-grid` 改 `repeat(auto-fit, minmax(16rem, 1fr))`；清理 hero 双列媒体查询，保留 <56.25rem 指标单列规则。
5. 对照 `design.md` 拓扑与护栏清单自检（squint test：主元素 = 成本带，次元素 = 指标行，分组顺序清晰）。
6. 验证（按序）：
   - `cd ccr-ui && bun run type-check`
   - `cd ccr-ui && bun run lint`
   - `cd ccr-ui && bun run test`
   - impeccable 机械检测（完成后跑一次）：`node C:\Users\lyh\.skillsmanage\skills\impeccable\scripts\detect.mjs --json --scope layout <changed files>`；若该路径不存在，回退 `C:\Users\lyh\.agents\skills\impeccable\scripts\detect.mjs`
7. 视觉验收：用户在桌面端自行确认（不在本任务内代跑）。

## Review Gates

- trellis-check 对照 `prd.md` AC1–AC6 与 `design.md` 护栏逐项核对，发现问题直接修复。
- layout 验证：DOM 顺序与视觉顺序一致；≤80rem、≤56.25rem 断点无溢出、无空轨；机械检测无未解释发现。

## Rollback

```bash
git checkout -- ccr-ui/src/features/usage/UsageDashboardView.tsx \
  ccr-ui/src/features/usage/components/UsageCostConclusionCard.tsx \
  ccr-ui/src/features/usage/styles/usage-cost-conclusion-card.css \
  ccr-ui/src/features/usage/styles/usage-dashboard-view.css
```
