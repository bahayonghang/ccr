# 执行计划：Pinia store 与 composable 迁移

> 父任务：`08-22-react-migration`（阶段 3，在 `08-22-shell-port` 之前）。
> 分支：`feature/react-migration/state-logic-port`，PR 目标 `feature/react-migration`。

## 前置确认

- 输入：`.trellis/tasks/08-22-arch-quality-perf/state-disposition.md`（45 项判定表）。
- [ ] 父任务约束门已通过（`08-22-arch-quality-perf` AC1–AC12 与 `08-22-design-system` AC1–AC11）。
- [ ] `08-22-arch-quality-perf` 的 `state-disposition.md` 已落盘，45 项（10 store + 35 composable）全部归类。本任务按该表执行，不重新决策归属。
- [ ] `react-rerender-discipline.md` 已阅读。
- [ ] `08-22-react-foundation` 的订阅写法参照与 `queryClient.ts` 已就位。
- [ ] `git checkout -b feature/react-migration/state-logic-port feature/react-migration`

## 批次 1：归类与排查

先测量再动手，避免边迁边发现语义差异。

- [ ] 35 个 composable 按 `design.md` §5 的方法三类归类，`composable-classification.md` 落盘（AC3）。
- [ ] 全仓 `rg` 就地修改形态（`.push(`、`.splice(`、`.sort(`、`[i] =`、`.field =`），筛出落在 store 与 composable 内的，`mutation-rewrite.md` 建表（AC6）。
- [ ] 52 处 `nextTick` 中落在本任务范围的逐点登记原始意图，`next-tick-register.md` 建表（AC4）。
- [ ] composable 内 `watch` 的数量与所用选项统计，按 `design.md` §6.3 的映射表逐点登记。
- [ ] 13 处 `computed` 逐个列出其读到的响应式来源（R6）。

## 批次 2：Query 层

- [ ] 五个 queryKey 工厂建立：`usageKeys`、`configsKeys`、`commandsKeys`、`claudeObserverKeys`、`homeUsageKeys`。
- [ ] 按 `design.md` §4 的表，把 `usage` / `configs` / `commands` / `claudeObserver` / `homeUsageOverview` 的数据部分改为 `useQuery` hook。
- [ ] `queryFn` 只调 `src/api` 下现有 wrapper。`git diff --stat src/api` 须为空。
- [ ] 写操作改 `useMutation` + `invalidateQueries`。
- [ ] 逐 query 设 `staleTime`，取值记录。

验证：`bun run type-check`；每个 hook 的测试用 mock `queryFn` 通过。

## 批次 3：Event 桥接层

- [ ] 按 `design.md` §3 建 `shell/eventBridge.ts`，全局事件名清单集中可见。
- [ ] **取消协议落地**：`listen()` 返回 `Promise<UnlistenFn>`，按 `design.md` §3 的 `disposed` + `track()` 写法实现；cleanup 已跑过时迟到的 unlisten 立即调用。
- [ ] 逐事件判定 `setQueryData` 与 `invalidateQueries`，判定记录落盘。
- [ ] 高频事件（`app-log`、`token-stats`）走 ref 累积 + 定时批量提交。间隔取值待 `08-22-arch-quality-perf` 场景 3 数据；数据未到则先设一个保守值并标注待复核。
- [ ] cleanup 中完整解绑，StrictMode 下不双订阅。
- [ ] 产出前端事件 inventory 的**全局部分**：事件名、所有者、生命周期、对应 Rust `emit` 位置。交 `08-22-test-contract-rebuild` 合并（协同点 M）。局部事件（CheckIn 的 WAF 等待）由 `08-22-views-checkin` 登记，本任务不代登记。

验证：`bun run test:smoke` 中的订阅泄漏测试（批次 6 交付）。

## 批次 4：Zustand store

- [ ] 按 `design.md` §1 建 store：`ui`、`shellPreferences`（带 `persist`，存储键不变）、`commandsView`，以及 `usage` / `configs` / `claudeObserver` 拆出的 UI 态部分。
- [ ] 公开 API 命名沿用原名（R7）。
- [ ] 选择器返回对象处用 `useShallow`。自检：无选择器返回新引用导致的无限重渲染。
- [ ] 13 处 `computed` 转为选择器内计算或 Query 的 `select`。
- [ ] `usageDashboardPayload.ts`（171 行）与 `usageImportNormalization.ts`（83 行）判定为纯变换后移入 `utils/`，判定依据记录（PRD Notes）。
- [ ] `src/stores` 下无 Pinia 引用（AC1）。

验证：`rg 'pinia|defineStore' src` 无匹配；`bun run type-check`。

## 批次 5：composable → hooks

按批次 1 的三类分组推进，每类一批提交。

- [ ] 纯逻辑类：原样复用或改签名。
- [ ] 响应式状态类：`ref` → `useState`，`computed` → `useMemo`，`watch` → `useEffect`（按 `design.md` §6.3 的选项映射）。
- [ ] 生命周期类：`onMounted` / `onUnmounted` → `useEffect` + cleanup，`listen()` 解绑时机逐个复核（R4）。
- [ ] `provide` / `inject` 各 1 处 → React Context。
- [ ] 就地修改逐点改为不可变写法，`mutation-rewrite.md` 填完（AC6）。
- [ ] `nextTick` 逐点按 `design.md` §6.4 的替代表改写，`next-tick-register.md` 填完（AC4）。
- [ ] `src/composables` 下无 `vue` 导入（AC2）。

验证：`rg "from 'vue'" src/composables src/stores` 无匹配；`bun run type-check`（AC7）；`bun run lint`（`exhaustive-deps` 为 error，拦截依赖遗漏）。

## 批次 6：测试

- [ ] 每个 store 的每个 action 至少一个用例（AC8）。
- [ ] 订阅泄漏测试：按 `design.md` §7 的三个用例写（AC5）。用例 1 立即 resolve；用例 2 Promise 在卸载后才 resolve，断言该 unlisten 仍被调用；用例 3 StrictMode 下挂载 → 卸载 → 再挂载 + 延迟 resolve。**只过用例 1 不构成 AC5 满足。**
- [ ] Query hook 测试用 `QueryClientProvider` + mock `queryFn`。

验证：`bun run test:smoke` 退出码 0。

## 批次 7：接口对齐

- [ ] `shellPreferences` 与 `themeBootstrap.ts`、`fontPreferences.ts` 的接口与 `08-22-shell-port` 对齐（PRD Notes）。后两者的接线归对方，本任务提供 store 侧接口。
- [ ] `claudeObserver` 的 Query key 与事件失效范围通知 `08-22-views-claude`。
- [ ] `configs` 的表单草稿键（配置 id）通知 `08-22-shell-port`（其 AC4 的表单草稿验证）与 `08-22-views-profiles-config`。

## 验证命令

| 时机           | 命令                                                             |
| -------------- | ---------------------------------------------------------------- |
| 每批次后       | `bun run type-check`、`bun run lint`                             |
| 批次 2、4–6 后 | `bun run test:smoke`                                             |
| 批次 4 后      | `rg 'pinia\|defineStore' src`（应无匹配）                        |
| 批次 5 后      | `rg "from 'vue'" src/composables src/stores`（应无匹配）         |
| 交付前         | `just frontend-check-quick`、`git diff --stat src/api`（应为空） |

## 交付门（父任务外壳门的一半）

- [ ] AC1–AC8 全部满足。
- [ ] 四份记录落盘：`composable-classification.md`（35 行）、`mutation-rewrite.md`、`next-tick-register.md`、逐事件的 `setQueryData` / `invalidateQueries` 判定。
- [ ] `usageDashboardPayload.ts` 与 `usageImportNormalization.ts` 的纯变换判定依据记录。
- [ ] `src/api` git diff 为空。
- [ ] 高频事件的批量提交间隔已按基线数据复核，或已标注待复核项。

## 回滚点

| 批次 | 回滚方式                                               |
| ---- | ------------------------------------------------------ |
| 1    | 只产出表格，revert 无副作用                            |
| 2–4  | 每层单独提交（Query / 桥接 / Zustand）。可只回退某一层 |
| 5    | 按 composable 三类分三次提交，可按类回退               |
| 6–7  | 测试与接口对齐                                         |

Pinia 与 Zustand 在批次 4 前可并存（旧 store 未删除时），因此批次 2–3 的回滚不影响可运行性。批次 4 删除 Pinia 引用后回滚需一并恢复。

## 协同点

| 编号 | 内容                                                               | 对方                                              | 时机      |
| ---- | ------------------------------------------------------------------ | ------------------------------------------------- | --------- |
| —    | `state-disposition.md` 是本任务的输入                              | `08-22-arch-quality-perf`                         | 前置      |
| —    | 高频事件批量提交间隔的基线数据                                     | `08-22-arch-quality-perf`                         | 批次 3    |
| —    | `shellPreferences` / `themeBootstrap` / `fontPreferences` 接口对齐 | `08-22-shell-port`                                | 批次 7    |
| —    | 事件名清单供事件名断言                                             | `08-22-test-contract-rebuild`                     | 批次 3 后 |
| —    | `claudeObserver` 的 key 与失效范围                                 | `08-22-views-claude`                              | 批次 7    |
| —    | `configs` 表单草稿键                                               | `08-22-shell-port`、`08-22-views-profiles-config` | 批次 7    |
