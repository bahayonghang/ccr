# Usage 与 Dashboard 视图迁移

> 父任务：`08-22-react-migration`

## Goal

将用量、看板、预算与定价视图从 Vue 迁移到 React，约 11,995 行，并完成 ApexCharts 从 `vue3-apexcharts` 到 `react-apexcharts` 的桥接重写。

## Scope

| 文件 / 目录 | 行数 |
|---|---|
| `src/components/usage/`（16 文件） | 6,446 |
| `src/components/dashboard/`（5 文件） | 1,890 |
| `src/components/platform-usage/`（3 文件） | 1,013 |
| `src/views/PricingView.vue` | 1,038 |
| `src/views/BudgetView.vue` | 797 |
| `src/views/DashboardView.vue` | 451 |
| `src/views/UsageDashboardView.vue` | 360 |
| 合计 | 11,995 |

关联的框架无关资产（原样复用，只改调用点）：`src/utils/apexChartsCore.ts`、`src/views/usage/usageTokenBreakdown.ts`、`src/stores/usageDashboardPayload.ts`、`src/stores/usageImportNormalization.ts`（后两者的归属由 `08-22-state-logic-port` 判定）。

关联的契约：`usage-chart-stability-contracts.md`、`dashboard-presentation-contracts.md`、`environment-scoped-dashboard-contracts.md`。

关联的样式：`src/styles/chart-colors.css`（5 个变量，由 `08-22-design-system` 迁移）。

## Requirements

- R1 上表 27 个文件迁移为 React 组件，对应 `.vue` 文件删除。
- R2 ApexCharts 桥接从 `vue3-apexcharts` 改为 `react-apexcharts` 2.1.1，`apexcharts` 核心版本由 `08-22-dep-upgrade` 决定。
- R3 图表配色继续消费 `chart-colors.css` 的 token，不在组件内写死颜色。
- R4 `usage-chart-stability-contracts.md` 定义的稳定性行为在迁移后成立：图表在数据更新、窗口缩放、主题切换时不出现闪烁、尺寸抖动或重复渲染。
- R5 图表错误边界行为保留（对应 `chart-error-boundary.smoke.test.ts`），单个图表渲染失败不影响页面其余部分。
- R6 `apexcharts-style-contract.smoke.test.ts` 断言的样式契约在迁移后成立。
- R7 `dashboard-presentation-contracts.md` 与 `environment-scoped-dashboard-contracts.md` 定义的展示与环境作用域行为不变。
- R8 用量数据读取沿用 `src/api/domains/stats.ts` 与 ts-rs 生成的 DTO 类型（`src/types/generated/usage/`），不修改。
- R9 `@tanstack/vue-virtual` 到 `@tanstack/react-virtual` 3.14.10 的替换在本批次落地，长列表虚拟滚动行为不变。
- R10 `Sparkline` 与 `StatTile` 原语的消费点适配 `08-22-design-system` 的产出。

## Acceptance Criteria

- [x] AC1 上表 27 个文件全部迁移，`rg --files -g '*.vue' src/components/usage src/components/dashboard src/components/platform-usage src/views/PricingView.vue src/views/BudgetView.vue src/views/DashboardView.vue src/views/UsageDashboardView.vue` 无匹配。
- [ ] AC2 4 个视图的路由可达，页面渲染无报错。——`routeLoaders.ts` 已导出；未改 `router.tsx`/`routeCatalog.ts`。
- [ ] AC3 核心操作路径手动验证通过并记录：看板加载、平台维度切换、时间范围切换、环境作用域切换、Token 明细展开、预算设置、定价查看、用量导入。
- [ ] AC4 图表稳定性验证：连续切换时间范围 20 次、窗口缩放 20 次、明暗主题切换 20 次，无闪烁与尺寸抖动。——mock 断言已绿（构造一次 / updateOptions / 节流 resize / destroy）。
- [x] AC5 单个图表注入渲染错误后，页面其余部分仍可用。——`tests/chart-error-boundary.smoke.test.tsx` 通过。
- [x] AC6 `apexcharts-style-contract` 与 `chart-error-boundary` smoke 测试通过。
- [ ] AC7 `usage-chart-stability-contracts.md`、`dashboard-presentation-contracts.md`、`environment-scoped-dashboard-contracts.md` 三份契约的验证项全部通过。——dashboard-presentation 与 chart-stability smoke 已绿；环境作用域看板契约仍由 grok-dashboard 测试覆盖，本域未改其实现。
- [ ] AC8 虚拟滚动长列表在 10,000 行数据下滚动流畅，无空白帧。——`useVirtualList` 已落地并接到 Usage 日志表。
- [ ] AC9 本批次组件内 px 字面量与 `rgba()` 数量为 0（登记豁免除外，图表内联样式可豁免并登记）。——CSS 已把 Vue scoped `px` 转为 rem；未逐条人工点检。
- [x] AC10 `src/api` 与 `src/types/generated` 的 git diff 为空。
- [ ] AC11 `bun run type-check` 与 `bun run lint` 退出码 0。——`src/features/usage` eslint 0；全仓 `tsc` 被并行 checkin 文件 `ProgressHero` 缺失挡住；全仓 lint 仍有 checkin/codex 既有错误。

## 前置与后续

- 前置：`08-22-shell-port`。
- 可与其余六个视图子任务并行。
- i18n 调用点在本批次内同步转换，运行时切换与收尾校验属 `08-22-i18n-port`。

## Out of Scope

- 新增功能与信息架构调整。
- `src/api`、`src/types` 的修改。
- `src-tauri/src/commands/stats.rs` 的改动。
- `crates/ccr-usage` 的改动。
- llmusage CLI 集成与 schema 版本门（`MIN_SUPPORTED_SCHEMA_VERSION`、provider schema 14）的改动。
- 更换图表库。本任务只替换 Vue 绑定，保留 ApexCharts。

## Notes

- 图表稳定性是本批次的主要风险。建议先重写 `usage-chart-stability-contracts.md` 为可执行断言，再改实现。
- `src/components/usage/` 的 16 个文件是全仓单目录行数最多的组件目录（6,446 行），建议在 `implement.md` 中拆为 2 个提交批次。
- `usageTokenBreakdown.ts` 与 `UsageTokenBreakdownStrip.vue`、`UsageTokensTab.vue` 存在主题 token 耦合，迁移时同步核对。
