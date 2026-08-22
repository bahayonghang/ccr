# 架构约定、质量门与性能基线

> 父任务：`08-22-react-migration`

## Goal

在视图迁移开始之前定稳架构约定、落地质量门与性能预算，使 7 个视图子任务在同一套可强制执行的约束下工作。本任务的产出是规则与工具，而非业务代码。

## 背景

当前代码库的三项缺口（数据来自父任务测量基准）：

- **架构**：29 个文件、20,218 行是同类功能在多平台的独立实现，占 `views/` 总量 43%。`src/views/generic/`（5 文件 2,635 行）是一次未完成的抽象尝试。`src/api/tauri.ts` 的冻结门面边界只有文档与 smoke 测试保护，无静态强制。
- **质量**：`eslint.config.js`（133 行）只有 `@typescript-eslint/no-explicit-any: error`。无文件规模、复杂度、嵌套深度、导入边界规则。最大 `.vue` 为 1,744 行，39 个根级视图平均 739 行。覆盖率门已存在（`just frontend-coverage`，lines ≥70%），但未纳入 `just ci`，阈值写在 justfile 而非 `vitest.smoke.config.ts`。139 / 185 个组件带局部样式，合计 24,434 行。
- **性能**：现有基础设施为 `check-bundle-budget.mjs`、`measure-vite-route.mjs`、`warm-vite-deps.mjs`、`perfTelemetry.ts`、三层 CSS 加载策略、`corePlugins.preflight: false`。React 的组件级重渲染替代 Vue 的细粒度响应式后，大表单、长列表、日志流、图表四类场景存在回退风险，且 353 处 `v-model` 落点集中在配置表单。

## Scope

### 架构约定（产出：契约文档 + 强制工具）

- 分层与依赖方向：视图 → 域逻辑 → API → 类型。反向依赖由 ESLint 导入规则拦截。
- 门面边界：`src/api/tauri.ts` 只读，新 wrapper 只能落 `src/api/domains/<domain>.ts`。两侧分别强制：消费侧由 lint 规则拦截直接 import；定义侧由既有 smoke 测试 `api-facade-boundary.smoke.test.ts` 的冻结用例拦截（该测试断言 `tauri.ts` 内的 `invoke()` 命令名序列恰好等于 9 条允许集合）。lint 单独无法冻结定义侧——`src/api/index.ts` 有 `export * from './tauri'`。
- 组件分层：原语（`src/ui/`）→ 复合组件 → 域组件 → 页面。原语不得导入域逻辑与 store，由 lint 规则强制。
- 状态归属：服务端数据、UI 瞬态、跨页面共享三类各有唯一承载位置。判定表落盘，供 `08-22-state-logic-port` 使用。
- 循环依赖检查纳入 CI。
- 跨平台共享层边界：`views/generic` 与平台专属视图的职责划分写入契约。共享层接口变更的通知机制写明。

### 质量门（产出：lint 配置 + CI 门）

| 规则 | 当前状态 | 本任务动作 |
|---|---|---|
| 单文件行数上限 | 无 | 定值并强制。参考现状：最大 1,744 行 |
| 圈复杂度上限 | 无 | 定值并强制 |
| 嵌套深度上限 | 无 | 定值并强制 |
| 参数个数上限 | 无 | 定值并强制 |
| 组件内样式行数上限 | 无 | 定值并强制（lint 规则或检查脚本） |
| `no-explicit-any` | error | 保留 |
| `no-unsafe-*` 系列 | 无 | 启用 |
| `react-hooks/rules-of-hooks` | 不适用 | error |
| `react-hooks/exhaustive-deps` | 不适用 | error |
| 导入边界 | 无 | 按分层规则强制 |
| 循环依赖 | 无 | CI 检查 |
| 测试覆盖率阈值 | 已有 lines ≥70%，在 justfile 内，未纳入 ci | 阈值移入 `vitest.smoke.config.ts`，纳入 `just ci`，按现状数据复核取值 |

全部规则以 error 级别加入 `bun run lint:ci`，不使用 warning 级别软约束。

超出上限的既有文件在迁移过程中逐批处理，不使用全局豁免。豁免需逐文件登记并说明。

### 性能基线与预算（产出：测量脚本 + 预算值）

五项测量场景：

1. 配置大表单输入延迟（`AppSettingsView`、`ClaudeCodeSettingsView`、`CodexSettingsView` 三个最大的表单页）。
2. 10,000 行虚拟列表滚动帧率。
3. 实时日志流（`MonitoringView`）持续输出时的帧率与内存。
4. 图表数据更新与主题切换时的渲染耗时。
5. 路由切换耗时（75 条路由，采样覆盖各域）。

三项产物指标：启动耗时、首屏渲染耗时、bundle 体积。

React 重渲染纪律：memo 边界、状态切分粒度、context 拆分、选择器使用的约定写入契约，并尽可能落为 lint 规则，避免在 7 个视图子任务中逐个补救。

## Requirements

- R1 分层与依赖方向、门面边界、组件分层、循环依赖四项由 lint 或检查工具以 error 级别强制。门面边界的定义侧由既有 smoke 测试冻结，不由 lint 承担。
- R2 文件行数、圈复杂度、嵌套深度、参数个数、组件内样式行数五项上限确定并强制。上限值需给出依据，不使用无根据的整数。取值分两段：阶段 2 给暂定值（排除将被统一层接管的 20 个文件后取 P90），阶段 4 结束后按统一层实际分布冻结最终值。
- R3 `react-hooks/rules-of-hooks` 与 `react-hooks/exhaustive-deps` 为 error。
- R4 `no-unsafe-*` 系列规则启用，`no-explicit-any: error` 保留。
- R5 测试覆盖率阈值移入 `vitest.smoke.config.ts` 并纳入 `just ci`。现有门为 `just frontend-coverage` 的 lines ≥70%（阈值写在 justfile，未进 ci）。取值按迁移前 122 个 smoke 测试的实际覆盖数据复核，调整需给出依据。
- R6 状态三分类判定表落盘，`08-22-state-logic-port` 的 10 个 store 与 35 个 composable 逐个归类。
- R7 五项性能场景的迁移前基线数据采集完成并落盘。测量方法可重复执行。
- R8 React 重渲染纪律写入契约文档，7 个视图子任务在动手前需阅读该文档。
- R9 bundle 预算值确定。React 产物与 Vue 产物的对比基准写明。
- R9.1 预算需为两项新增运行时依赖显式预留额度并记录预留前后对比：`motion` 13.1.1（动画）与 `zod` 4.4.3（校验）。两者均为选型决策的结果（父任务 `design.md` §9、§7），预算超出不构成回退选型的理由，但超出量需落盘。
- R10 路由级代码分割与三层 CSS 加载策略的等价方案确定，写入契约。
- R11 全部规则与预算写入 `.trellis/spec/ccr-ui/frontend/` 新增契约文档，由 `08-22-test-contract-rebuild` 纳入重写范围（该任务的契约基线 16 份 → 本任务新增 2 份 → 18 份；`08-22-platform-unify` 再加 1 份为 19 份）。
- R12 既有超限文件的处理计划写明：逐批处理，不使用全局豁免，豁免逐文件登记。

## Acceptance Criteria

- [ ] AC1 `bun run lint:ci` 退出码 0，全部新规则为 error 级别。
- [ ] AC2 四类违规各构造一个用例并报错：反向依赖（lint）、跨层导入（lint）、门面消费侧绕过（lint）、门面定义侧新增 wrapper（既有 smoke 冻结用例）。第四项不能由前三项替代。
- [ ] AC3 循环依赖检查可运行，构造一个循环用例后报错。
- [ ] AC4 五项规模与复杂度上限的**暂定值**与依据落盘（`thresholds.md`）。最终值在阶段 4 → 5 门冻结，同文件追加第二段数据。
- [ ] AC5 `just frontend-coverage` 退出码 0，阈值已在 `vitest.smoke.config.ts` 内且已纳入 `just ci`，取值与依据落盘。
- [ ] AC6 状态三分类判定表落盘，10 个 store 与 35 个 composable 全部归类。
- [ ] AC7 五项性能场景的基线数据落盘，测量脚本可重复执行并给出一致结果。
- [ ] AC8 React 重渲染纪律契约文档落盘。
- [ ] AC9 bundle 预算值与对比基准落盘，`bun run check:bundle-budget` 可运行。
- [ ] AC10 代码分割与 CSS 分层的等价方案落盘。
- [ ] AC11 既有超限文件清单落盘，含处理批次归属，无全局豁免。
- [ ] AC12 新增契约文档已登记到 `08-22-test-contract-rebuild` 的范围表。

## 前置与后续

- 前置：`08-22-react-foundation`（需要 React 侧的 lint 与测试管线已就位）。
- 后续：`08-22-design-system`。本任务必须在 7 个视图子任务之前完成，否则规则无法约束迁移过程。
- 性能基线采集与父任务的「Phase 1 收尾前置动作」协同：视觉基线在 Phase 1 采集，性能基线在本任务采集，两者共同构成 `08-22-regression-release` 的对比依据。

## Out of Scope

- 业务代码改动。本任务只产出规则、工具与文档。
- 既有超限文件的实际拆分。拆分在对应视图子任务中随迁移完成。
- 跨平台重复实现的统一。该项的范围决策见父任务「跨平台重复」一节。
- 性能优化实施。本任务只建立基线与预算，优化在对应子任务与 `08-22-regression-release` 中进行。
- Rust 侧的架构与质量约束。

## Notes

- 本任务的价值在于时序：规则若在视图迁移之后才落地，7 个子任务已产出的约 78,000 行代码需要返工。规则必须先行。
- 规模与复杂度上限的取值需要平衡：定得过严会导致迁移期大量拆分工作，定得过松则无约束效果。建议按现状分布的分位数取值，并记录分布数据作为依据。
- React 重渲染是本次迁移最可能出现的非功能性回归。Vue 的细粒度响应式在大表单场景下天然不触发无关组件更新，React 需要显式的 memo 与状态切分。这一点在 353 处 `v-model` 落点上影响最大。
