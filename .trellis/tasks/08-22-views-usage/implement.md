# 执行计划：Usage 与 Dashboard 视图迁移

> 父任务：`08-22-react-migration`（阶段 5，七个视图子任务并行）。
> 分支：`feature/react-migration/views-usage`，PR 目标 `feature/react-migration`。

## 前置确认

- [ ] 父任务统一层门已通过（本域范围不受影响，无需回填）。
- [ ] 前置阅读完成，另加 `usage-chart-stability-contracts.md`（7.1 KB）、`dashboard-presentation-contracts.md`（10.9 KB）、`environment-scoped-dashboard-contracts.md`（5.0 KB）。
- [ ] `08-22-design-system` 已迁移 `chart-colors.css`（5 变量），耦合项已核对。
- [ ] `08-22-state-logic-port` 已提供 `usageKeys` 与 `homeUsageKeys` 的 Query hook，且 `usageDashboardPayload.ts` 与 `usageImportNormalization.ts` 已移入 `utils/`。
- [ ] `08-22-dep-upgrade` 已确定 `apexcharts` 核心版本。
- [ ] `git checkout -b feature/react-migration/views-usage feature/react-migration`

## 批次 0：稳定性契约转为可执行断言

先做。PRD Notes：建议先重写契约为可执行断言，再改实现。

- [ ] 按 `design.md` §3.2 的四条把 `usage-chart-stability-contracts.md` 的条目写成断言：数据更新不重构造、主题切换走 `updateOptions`、resize 有节流、卸载调 `destroy()`。
- [ ] 断言以 mock ApexCharts 构造与 update 方法的方式实现，不依赖真实渲染。
- [ ] 断言先失败（红），实现完成后转绿。
- [ ] 与 `08-22-test-contract-rebuild` 提供的重写稿对齐，不产生两份断言。

## 批次 1：图表桥接层

- [ ] `vue3-apexcharts` → `react-apexcharts` 2.1.1 的桥接。
- [ ] `apexChartsCore.ts` 的按需入口形态保留，全程 `await import()`。
- [ ] `manualChunks` 的 `charts-vendor` 成员改写。确认 `react-apexcharts` 是否有 `/core` 子路径。
- [ ] `options` 用 `useMemo`，`series` 保持稳定引用（`design.md` §3.2 末段）。
- [ ] 每个图表包图表级 `ErrorBoundary`（R5）。
- [ ] 主题切换时读 CSS 变量的方式确认并保留（`design.md` §3.4 末段）。

验证：批次 0 的断言全绿；`bun run check:bundle-budget` 通过（core 未被复制多份）；`apexcharts-style-contract` 与 `chart-error-boundary` smoke 测试通过（AC6）。

## 批次 2：`src/components/usage/`（16 文件 6,446 行）

拆 2 个提交批次（PRD Notes）。

- [ ] 2a：图表类组件。
- [ ] 2b：表格、明细、导入类组件。
- [ ] `usageTokenBreakdown.ts` 与 `UsageTokenBreakdownStrip`、`UsageTokensTab` 的主题 token 耦合按 `token-classification.md` 核对。
- [ ] 虚拟滚动接线写成可复用的 hook 或组件（`design.md` §4），供 `08-22-views-codex` 复用。
- [ ] 超过行数上限的文件拆分。

## 批次 3：`components/dashboard/` 与 `components/platform-usage/`

- [ ] `src/components/dashboard/`（5 文件 1,890 行）。
- [ ] `src/components/platform-usage/`（3 文件 1,013 行）。
- [ ] `Sparkline` 与 `StatTile` 消费点适配（R10）。`Sparkline` 是否消费 `chart-colors.css` 需确认。

## 批次 4：4 个视图

- [ ] `PricingView`(1,038)、`BudgetView`(797)、`DashboardView`(451)、`UsageDashboardView`(360)。
- [ ] `usage` 路由的缓存行为：数据走 Query，时间范围与平台维度入 Zustand，切回保留（`design.md` §6 末段）。
- [ ] 环境作用域过滤的实现位置确认并保留（`design.md` §5）。

验证：4 个视图路由可达（AC2）；缓存行为在真实视图上验证（补 `08-22-shell-port` AC4 的界面级验证）。

## 批次 5：稳定性与性能验证

- [ ] 图表稳定性：连续切换时间范围 20 次、窗口缩放 20 次、明暗主题切换 20 次，无闪烁与尺寸抖动（AC4）。
- [ ] 单个图表注入渲染错误后页面其余部分仍可用（AC5）。
- [ ] 虚拟滚动 10,000 行滚动流畅，无空白帧（AC8）。
- [ ] 三份契约的验证项全部通过（AC7）。

## 批次 6：收口

- [ ] 本批次组件内 px 与 `rgba()` 归零，图表内联样式可豁免并登记（AC9）。
- [ ] `rg --files -g '*.vue' src/components/usage src/components/dashboard src/components/platform-usage src/views/PricingView.vue src/views/BudgetView.vue src/views/DashboardView.vue src/views/UsageDashboardView.vue` 无匹配（AC1）。
- [ ] `git diff --stat src/api src/types/generated`（应为空，AC10）。

## 验证命令

| 时机        | 命令                                           |
| ----------- | ---------------------------------------------- |
| 每批次后    | `bun run type-check`、`bun run lint`（AC11）   |
| 批次 0–3 后 | `bun run test:smoke`                           |
| 批次 1 后   | `bun run build`、`bun run check:bundle-budget` |
| 批次 6 后   | 上表 AC1 的 `rg` 命令                          |
| 交付前      | `just frontend-check-quick`、`bun run lint:ci` |

## 交付门（父任务视图门的一部分）

- [ ] AC1–AC11 全部满足。
- [ ] AC3 的 8 条核心操作路径逐条验证并记录：看板加载、平台维度切换、时间范围切换、环境作用域切换、Token 明细展开、预算设置、定价查看、用量导入。
- [ ] 批次 0 的可执行断言全绿，与 `08-22-test-contract-rebuild` 的重写稿无重复。
- [ ] 图表稳定性三项 20 次验证通过（AC4）。
- [ ] `charts-vendor` chunk 未被复制多份，bundle 预算通过。
- [ ] 虚拟滚动接线已写成可复用形态，通知 `08-22-views-codex`。
- [ ] 硬编码豁免登记落盘（图表内联样式）。

## 回滚点

七个批次各自独立提交。批次 1（桥接层）的回滚会使全部图表不可用，其余批次独立。批次 2 拆 2a / 2b 两次提交。

## 协同点

| 编号 | 内容                                                 | 对方                                           | 时机         |
| ---- | ---------------------------------------------------- | ---------------------------------------------- | ------------ |
| D    | 三份契约的重写稿                                     | `08-22-test-contract-rebuild`                  | 前置与批次 0 |
| I    | i18n 调用形式                                        | `08-22-i18n-port`                              | 全程         |
| —    | `chart-colors.css` 迁移与耦合核对                    | `08-22-design-system`                          | 前置         |
| —    | `usageKeys` / `homeUsageKeys` 与两个纯变换模块的落位 | `08-22-state-logic-port`                       | 前置         |
| —    | `apexcharts` 核心版本；`manualChunks` 分组           | `08-22-dep-upgrade`、`08-22-arch-quality-perf` | 批次 1       |
| —    | 虚拟滚动接线形态                                     | `08-22-views-codex`                            | 批次 2 后    |
