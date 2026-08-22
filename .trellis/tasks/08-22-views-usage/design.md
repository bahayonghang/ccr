# 技术设计：Usage 与 Dashboard 视图迁移

> 父任务：`08-22-react-migration`。本域 11,995 行，不进统一层。主要风险为 ApexCharts 桥接与图表稳定性。

## 1. 本批次共性转换

与 `08-22-views-claude` 的 `design.md` §1 同表，不重复。前置阅读同，另加 `usage-chart-stability-contracts.md`。

## 2. 范围

27 个文件，11,995 行。`src/components/usage/`（16 文件 6,446 行）是全仓单目录行数最多的组件目录。

本域不受 `08-22-platform-unify` 影响，范围表不回填。

框架无关资产（原样复用，只改调用点）：`src/utils/apexChartsCore.ts`、`src/views/usage/usageTokenBreakdown.ts`。`src/stores/usageDashboardPayload.ts`（171 行）与 `usageImportNormalization.ts`（83 行）由 `08-22-state-logic-port` 判定为纯变换并移入 `utils/`，本任务只改调用点。

## 3. ApexCharts 桥接（本域主要风险）

`vue3-apexcharts` 1.10.0 → `react-apexcharts` 2.1.1。`apexcharts` 核心版本由 `08-22-dep-upgrade` 决定（现 ^5.3.6）。

### 3.1 按需加载入口保留

现状：图表库统一走 `src/utils/apexChartsCore.ts` 的按需入口，全程 `await import()`；`vite.config.ts` 的 `manualChunks` 固定为单一 `charts-vendor` chunk，避免多个懒加载点各自复制一份 core（该文件 :35–:37 的注释说明）。

React 侧等价：`charts-vendor` 的成员从 `['apexcharts/core', 'vue3-apexcharts/core']` 改为 `['apexcharts/core', 'react-apexcharts']`（`react-apexcharts` 是否有 `/core` 子路径需在实施时确认）。`apexChartsCore.ts` 的按需入口形态保留。

若 `manualChunks` 分组失效导致 core 被复制多份，产物体积上升，`bun run check:bundle-budget` 会报警。该项是本批次的构建验证要点。

### 3.2 稳定性契约先重写为可执行断言

`usage-chart-stability-contracts.md`（7.1 KB）定义：图表在数据更新、窗口缩放、主题切换时不出现闪烁、尺寸抖动或重复渲染（R4）。

PRD Notes 的建议：先重写该契约为可执行断言，再改实现。本设计采纳该顺序。

可执行断言的形态：

| 契约条目           | 断言方式                                                                                        |
| ------------------ | ----------------------------------------------------------------------------------------------- |
| 数据更新不重复渲染 | mock ApexCharts 的构造与 `updateOptions` / `updateSeries`，断言数据变化时只调 update 不重新构造 |
| 主题切换不重新构造 | 同上，断言主题变化走 `updateOptions`                                                            |
| 窗口缩放不抖动     | 断言 resize 处理有节流，且不触发重新构造                                                        |
| 卸载时销毁         | 断言 `destroy()` 被调用                                                                         |

**React 侧的具体风险**：`react-apexcharts` 的 `options` 与 `series` 若在每次渲染时传入新对象字面量，会触发不必要的 update。因此 `options` 需 `useMemo`，`series` 需稳定引用。这是 `react-rerender-discipline.md` 中「props 不传内联对象」条目在本域的具体落点。

### 3.3 错误边界（R5）

对应 `chart-error-boundary.smoke.test.ts`。单个图表渲染失败不影响页面其余部分。

React 侧实现：每个图表包一个 `ErrorBoundary`。`08-22-shell-port` 已建 `MainLayout` 级与 tray 级两个边界，本任务加图表级边界——粒度更细，因此不复用上层边界。

### 3.4 配色 token（R3）

图表配色继续消费 `chart-colors.css` 的 5 个变量，不在组件内写死颜色。

`08-22-design-system` 批次 1 迁移该文件时已核对与 `apexcharts-style-contract.smoke.test.ts` 的耦合。本任务确认迁移后变量名不变，图表读取路径不变。

主题切换时 ApexCharts 需要读到新的颜色值——CSS 变量不会自动传入 JS 配置。现状的读取方式（`getComputedStyle` 或预定义映射）需在迁移时确认并保留。

## 4. 虚拟滚动（R9）

`@tanstack/vue-virtual` 3.13.18 → `@tanstack/react-virtual` 3.14.10。

行为不变。AC8：10,000 行数据下滚动流畅，无空白帧。

本任务的接线形态是其他子任务的复用参照（`08-22-views-codex` 的 `CodexSessionsView` 引用本域形态）。因此接线需写成可复用的 hook 或组件，不内联在某个视图里。

## 5. 展示与环境作用域契约（R7）

`dashboard-presentation-contracts.md`（10.9 KB）与 `environment-scoped-dashboard-contracts.md`（5.0 KB）定义的行为不变。

环境作用域指看板数据按执行环境（local / wsl / ssh）过滤。该过滤的实现位置需确认：若在前端，迁移时保留过滤逻辑；若在 Rust 侧，前端只传参数。

## 6. 数据读取（R8）

用量数据读取沿用 `src/api/domains/stats.ts` 与 ts-rs 生成的 DTO 类型（`src/types/generated/usage/`），不修改。

Query 层由 `08-22-state-logic-port` 已建（`usageKeys` 与 `homeUsageKeys`）。本任务只消费其 hook，不新建 Query。

`usage` 路由是 5 条缓存路由之一：数据走 Query，时间范围与平台维度入 Zustand（父任务 `design.md` §5）。切回时筛选条件保留。该行为已由 `08-22-shell-port` AC4 验证一次，本任务在真实视图上再验一次。

## 7. token 耦合

`usageTokenBreakdown.ts` 与 `UsageTokenBreakdownStrip.vue`、`UsageTokensTab.vue` 存在主题 token 耦合（PRD Notes），迁移时按 `token-classification.md` 核对其消费的变量名仍存在。

## 8. `Sparkline` 与 `StatTile` 原语（R10）

两者在 `08-22-design-system` 的 `primitive-disposition.md` 中初判为「保留不变」与「保留并改消费新 token」。本任务适配其消费点。

`Sparkline` 需核对是否消费 `chart-colors.css`（`08-22-design-system` §6 的待核对项）。

## 9. 不变量

- `src/api` 与 `src/types/generated` 不改（AC10）。
- `src-tauri/src/commands/stats.rs` 不改。
- `crates/ccr-usage` 不改。
- llmusage CLI 集成与 schema 版本门（`MIN_SUPPORTED_SCHEMA_VERSION`、provider schema 14）不改。
- 不更换图表库，只替换 Vue 绑定。

## 10. 未决项

- `react-apexcharts` 是否有 `/core` 子路径（第 3.1 节），决定 `manualChunks` 的写法。
- 主题切换时 ApexCharts 读取 CSS 变量的现状方式（第 3.4 节末段）。
- 环境作用域过滤的实现位置（第 5 节）。
- `Sparkline` 是否消费 `chart-colors.css`（第 8 节）。
