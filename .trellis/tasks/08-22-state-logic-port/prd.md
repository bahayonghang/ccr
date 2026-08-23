# Pinia store 与 composable 迁移

> 父任务：`08-22-react-migration`

## Goal

将 Vue 响应式状态层迁移到 React 状态模型：10 个 Pinia store（2,531 行）迁到选定状态库，35 个 composable（6,894 行）迁为 hooks。

## Scope

> **状态归属划分（父任务 `design.md` §4）**：10 个 store 不是整体迁到 Zustand，而是按 R6.6 三分类拆分。
>
> | store | 处理 |
> |---|---|
> | `usage.ts` | 数据 → TanStack Query，视图偏好 → Zustand |
> | `configs.ts` | 数据 → Query，选中态 → Zustand |
> | `commands.ts` | → Query |
> | `claudeObserver.ts` | 事件流数据 → Query（配合 Event 订阅失效），UI 态 → Zustand |
> | `homeUsageOverview.ts` | → Query |
> | `ui.ts` | → Zustand |
> | `shellPreferences.ts` | → Zustand |
> | `commandsView.ts` | → Zustand |
> | `usageDashboardPayload.ts`（171 行） | 纯变换，移入 `utils/`，不进状态层 |
> | `usageImportNormalization.ts`（83 行） | 同上 |
>
> Tauri Event 与 Query 的衔接：后端 `emit` 的事件在监听回调中调用 `queryClient.invalidateQueries` 或 `setQueryData`，store 不再直接持有服务端数据。订阅的建立与解绑保持在组件生命周期内。

### Pinia store（10 个 / 2,531 行）

| 文件 | 说明 |
|---|---|
| `claudeObserver.ts` | Claude 观测数据 |
| `commands.ts` | 命令数据 |
| `commandsView.ts` | 命令视图状态 |
| `configs.ts` | 配置列表 |
| `homeUsageOverview.ts` | 首页用量概览 |
| `shellPreferences.ts` | 外壳偏好 |
| `ui.ts` | UI 状态（收藏、历史） |
| `usage.ts` | 用量数据 |
| `usageDashboardPayload.ts` | 171 行，用量看板载荷 |
| `usageImportNormalization.ts` | 83 行，导入归一化 |

`usageDashboardPayload.ts` 与 `usageImportNormalization.ts` 为纯数据变换，可能无需状态库承载，逐个判定。

### composable（35 个 / 6,894 行）

按依赖性质分三类处理：

1. **纯逻辑**：不使用 `ref` / `computed` / `watch`，可原样复用或轻改。
2. **响应式状态**：`ref` / `computed` → `useState` / `useMemo`；`watch` / `watchEffect` → `useEffect`。
3. **生命周期与副作用**：`onMounted` / `onUnmounted` → `useEffect` 清理函数；Tauri `listen()` 订阅的解绑时机需逐个复核。

### 语义差异需逐点处理

| Vue | React | 风险 |
|---|---|---|
| `ref` 深层响应式 | `useState` 需不可变更新 | 现有代码可能依赖就地修改 |
| `computed` 缓存 | `useMemo` 依赖数组 | 依赖遗漏导致陈旧值 |
| `watch` 的 `immediate` / `deep` / `flush` | `useEffect` 无对应选项 | 时序与触发次数变化 |
| `nextTick`（全仓 52 处） | 无等价物 | 依赖 DOM 更新时序的逻辑需改写 |
| Pinia store 跨组件单例 | 状态库的 store 语义 | 订阅粒度与重渲染范围变化 |
| `provide` / `inject`（各 1 处） | React Context | 直接映射 |

## Requirements

- R1 10 个 store 全部迁移，跨组件单例语义保留。
- R2 35 个 composable 全部迁移，产出三类归类清单。
- R3 落在 store 与 composable 中的 `nextTick` 调用逐点登记，记录原始时序意图与替代实现。
- R4 Tauri `listen()` 订阅的建立与解绑时机在迁移前后一致，无泄漏。
- R5 状态更新采用不可变模型，原代码中依赖就地修改的位置逐点改写并登记。
- R6 `computed` 到 `useMemo` 的依赖集合逐个核对，无遗漏依赖。
- R7 store 与 hooks 的公开 API 命名沿用原名，减少调用点改动面。

## Acceptance Criteria

- [x] AC1 10 个 store 迁移完成，`src/stores` 下无 Pinia 引用。——**偏差 2**：9/10 完成，`src/stores/usage.ts` 暂留（其消费方深度耦合 monolith API，属 `08-22-views-usage` 转换范围），删除随 views-usage 落地，外壳门前复核（implement.md 批次 4 偏差记录）。其余判定：7 store 删除 + usage.ts 数据切片已入 Query、视图偏好入 Zustand。
- [x] AC2 35 个 composable 迁移完成，`src/composables` 下无 `vue` 导入。（主线程 grep 复核零匹配；35 = 8 纯变换迁 utils + useCachedFetch/useStream 消解删除 + useMainLayoutShell 入 shell/hooks + 其余 24 原地转 hooks）
- [x] AC3 composable 三类归类清单落盘，35 个文件全部归类（`composable-classification.md`）。
- [x] AC4 `nextTick` 登记表落盘，落在本任务范围内的调用点全部有替代实现说明（范围内 0 处，`next-tick-register.md`）。
- [x] AC5 事件订阅泄漏检查通过：挂载与卸载 100 次后监听器数量回到基线（`tests/event-bridge-leak.smoke.test.tsx` 3 用例：立即 resolve ×100、卸载后迟到 resolve 仍解绑、StrictMode 双挂载延迟 resolve；经变异验证非平凡）。
- [x] AC6 就地修改改写清单落盘，无未处理项（46/46 行判定完毕）。
- [x] AC7 `bun run type-check` 退出码 0（每批次主线程复验）。
- [x] AC8 store 与 hooks 的单元测试通过，覆盖每个 store 的核心状态转移（48 用例 / 7 store 全 action + Query hook 用例；391 测试全绿）。

## 前置与后续

- 前置：`08-22-design-system`。
- 后续：`08-22-shell-port`。

## Out of Scope

- 视图与组件的迁移。
- 状态库选型。已在父任务 `design.md` §1 定稳为 Zustand 5.0.15，服务端数据层为 TanStack Query 5.101.4。
- 数据获取层的引入本身。`08-22-react-foundation` 完成 TanStack Query 接入，本任务只做 store 的归属划分与迁移。
- 数据获取层改动。`src/api` 保持原样。
- store 的功能重构。语义等价迁移，不改变行为。

## Notes

- `usageDashboardPayload.ts` 与 `usageImportNormalization.ts` 若判定为纯变换，应移到 `src/utils` 并从 store 目录移除。该判定需记录依据。
- `shellPreferences.ts` 与 `themeBootstrap.ts`、`fontPreferences.ts` 存在耦合，后两者的接线属 `08-22-shell-port`，本任务需与其对齐接口。
