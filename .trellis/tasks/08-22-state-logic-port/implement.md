# 执行计划：Pinia store 与 composable 迁移

> 父任务：`08-22-react-migration`（阶段 3，在 `08-22-shell-port` 之前）。
> 分支：`feature/react-migration/state-logic-port`，PR 目标 `feature/react-migration`。

## 前置确认

- 输入：`.trellis/tasks/08-22-arch-quality-perf/state-disposition.md`（45 项判定表，已随该任务归档至 `archive/2026-08/`）。
- [x] 父任务约束门已通过（`08-22-arch-quality-perf` AC1–AC12 与 `08-22-design-system` AC1–AC11；`react-migration/phase-2` tag，2026-08-23）。
- [x] `08-22-arch-quality-perf` 的 `state-disposition.md` 已落盘，45 项（10 store + 35 composable）全部归类。本任务按该表执行，不重新决策归属。
- [x] `react-rerender-discipline.md` 已阅读。
- [x] `08-22-react-foundation` 的订阅写法参照与 `queryClient.ts` 已就位。
- [x] ~~`git checkout -b feature/react-migration/state-logic-port`~~ **偏差（沿用既有先例）**：分支命名空间冲突（父任务 §7 执行偏差记录），继续工作在 `react-migration/react-foundation` 分支，不新建分支。
- [ ] `08-22-arch-quality-perf` 的 `state-disposition.md` 已落盘，45 项（10 store + 35 composable）全部归类。本任务按该表执行，不重新决策归属。
- [ ] `react-rerender-discipline.md` 已阅读。
- [ ] `08-22-react-foundation` 的订阅写法参照与 `queryClient.ts` 已就位。
- [ ] `git checkout -b feature/react-migration/state-logic-port feature/react-migration`

## 批次 1：归类与排查

先测量再动手，避免边迁边发现语义差异。

- [x] 35 个 composable 按 `design.md` §5 的方法三类归类，`composable-classification.md` 落盘（AC3；与 state-disposition.md §4 一致 + 复核记录）。
- [x] 全仓 `rg` 就地修改形态，`mutation-rewrite.md` 建表（AC6；46 处排查行，批次 5 逐行填判定与改写）。
- [x] 52 处 `nextTick` 中落在本任务范围的逐点登记：**0 处**（全部在视图层），`next-tick-register.md` 落盘（AC4 的排查证据）。
- [x] composable 内 `watch` 统计：7 处（5 文件），选项映射表建在 `composable-classification.md` §2，批次 5 补充逐点判定。
- [x] `computed` 逐个列出：63 处（store 13 / composable 50），清单在 `composable-classification.md` §3；R6 的响应式来源登记随批次 4/5 转换时逐个完成（`exhaustive-deps` error 级 lint 拦截遗漏）。

## 批次 2：Query 层

- [x] 五个 queryKey 工厂建立：`usageKeys`、`configsKeys`、`commandsKeys`、`claudeObserverKeys`、`homeUsageKeys`（`src/features/{usage,configs,commands,claude}/queries.ts`）。
- [x] 按 `design.md` §4 的表，五个 store 的数据部分改为 `useQuery` hook（usage 9 切片 + capabilities + importJob 轮询 + homeOverview；configs list；commands list + executeCommand mutation；claudeObserver 8 切片 + subscription mutation）。
- [x] `queryFn` 只调 `src/api` 下现有 wrapper。`git diff --stat src/api` 为空（本轮零改动）。
- [x] 写操作改 `useMutation`：executeCommand（不失效 list，清单不含运行结果）、claudeObserver subscriptionSet（失效 subscription key）。
- [x] 逐 query 设 `staleTime` 并在文件头记录取值：usage 切片 30s（原 TTL）/ capabilities 5min / configs 5min / commands 2min（原 useCachedFetch TTL）/ claudeObserver 30s。

验证：`bun run type-check`；每个 hook 的测试用 mock `queryFn` 通过。

## 批次 3：Event 桥接层

- [x] 按 `design.md` §3 建 `shell/eventBridge.ts`，全局事件名清单 `TAURI_GLOBAL_EVENTS` 集中可见（12 项）。
- [x] **取消协议落地**：`disposed` + `track()` 写法照 design.md §3 实现；cleanup 已跑过时迟到的 unlisten 立即调用。泄漏断言三用例在批次 6 交付（`tests/event-bridge-leak.smoke.test.tsx`）。
- [x] 逐事件判定 `setQueryData` 与 `invalidateQueries`：12 个全局事件全部判定为 `invalidateQueries`（payload 均不含完整切片），记录落盘 `event-adjudication.md` §1。
- [x] 高频事件（`app-log`、`token-stats`、`app:monitoring`）：`createEventBatcher` 原语（ref 累积 + 定时批量提交）已交付；间隔 250ms 保守值并标注待复核（场景 3 React 侧数据由 regression-release 步骤 7 补测）。Monitor feed Query 缓存的接线随批次 5 `useMonitoringFeed` 落地（`event-adjudication.md` §2）。
- [x] cleanup 中完整解绑；非 Tauri 环境走 noop 桩保持协议形状。StrictMode 不双订阅由泄漏用例 3 断言（批次 6）。
- [x] 前端事件 inventory 全局部分落盘（`event-adjudication.md` §4：桥接常驻 12 + 高频 3 + 组件级 17 项归属登记，Rust emit 位置标注）。交 `08-22-test-contract-rebuild` 合并（协同点 M）。

验证：`bun run test:smoke` 中的订阅泄漏测试（批次 6 交付）。

## 批次 4：Zustand store

- [x]（部分，见偏差）按 `design.md` §1 建 store：`ui`（`shell/stores/ui.ts`）、`shellPreferences`（`shell/stores/shellPreferences.ts`）、`commandsView`（`features/commands/stores.ts`）、`usage` 视图偏好（`features/usage/stores.ts`）、`configs` 选中/搜索/草稿（`features/configs/stores.ts`）。
  - **偏差 1（persist 中间件未用）**：持久化沿用 themeBootstrap/fontPreferences 的逐 key 写入（ccr-theme / ccr-flavor / ccr-accent / ccr-font-* / ccr-sidebar-width / ccr-commands-view 键全部不变）。理由：首帧 IIFE 与迁移表和这些 key 逐字节对齐（theme-bootstrap.smoke.test.ts 行为锁），persist 中间件的单一 blob 会改变 key 布局破坏契约。「存储键不变」以原语义满足。
  - **偏差 2（AC1 未完全满足）**：`src/stores/usage.ts` 暂留。其消费方 `src/views/usage/state/*`（5 个 .ts）与 `useUsageDashboardState.ts` 深度耦合 monolith API（数据切片 + import 任务 + auto-refresh + 派生标志），属 `08-22-views-usage`（阶段 5）转换范围，本任务无法在不越界的前提下解除。其余 7 个 Pinia store 已删除（含 2 个纯变换移入 utils）。usage.ts 的删除随 views-usage 落地，外壳门前复核。
  - **claude 无 Zustand 侧**：claudeObserver store 拆解后仅剩数据切片（已入 Query）与事件监听（已入桥接层），无残余 UI 态——state-disposition.md 预判的「订阅/面板 UI 态」实际不存在，无需建 store（记录为判定修正）。
- [x] 公开 API 命名沿用原名（R7）：useUIStore / useShellPreferencesStore / useCommandsViewStore 的属性与 action 名与 Pinia 版一致。
- [x] 选择器约束遵守（全部单值选择器，无对象返回，无 useShallow 需求点）；「裸 useUIStore()」的 4+2 个存量调用点已过渡接线到 getState()（批次 5 重写）。
- [x] store 内 `computed`（14 处实测）转选择器内计算：localeLabel 为选择器内派生；usage/configs 的 computed 随数据入 Query select。响应式来源登记随批次 5 完成（exhaustive-deps error 级拦截）。
- [x] `usageDashboardPayload.ts` 与 `usageImportNormalization.ts` 判定为纯变换（判据：无跨调用存活状态、无 ref/computed，仅类型 + 输入→输出函数）移入 `utils/`（git mv），消费测试路径同步更新，eslint 豁免路径更新。
- [ ] `src/stores` 下无 Pinia 引用（AC1）——**7/8 完成**，usage.ts 暂留（偏差 2），views-usage 转换时删除。

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


## 批次 2 证据（补记）

改动：新增 `src/features/{usage,configs,commands,claude}/queries.ts` + `tests/state-query-hooks.smoke.test.tsx`（4 用例：key 工厂形态、参数透传、wrapper 调用、mutation 成功态）。

| 命令 | 退出码 | 结果 |
| --- | --- | --- |
| `bun run type-check` | 0 | ✓ |
| `bun run lint:ci` | 0 | ✓ |
| `vitest run --config vitest.smoke.config.ts tests/state-query-hooks.smoke.test.tsx` | 0 | 4/4 通过 |
| `bun run test:smoke` | 0 | 67 文件 / 333 测试全绿 |
| `git diff --stat src/api` | — | 空 |


## 批次 3 证据（补记）

改动：新增 `src/shell/eventBridge.ts`（桥接 + 取消协议 + `createEventBatcher`）与 `event-adjudication.md`。

| 命令 | 退出码 | 结果 |
| --- | --- | --- |
| `bun run type-check` | 0 | ✓ |
| `bun run lint:ci` | 0 | ✓ |
| `rg -c "listen\(" src/shell/eventBridge.ts` | — | 12 处 track（全部经取消协议） |


## 批次 4 证据（部分完成，偏差见上）

改动：新增 `shell/stores/{ui,shellPreferences}.ts`、`features/{commands,usage,configs}/stores.ts`、`shell/hooks/useMainLayoutShell.ts`（首个 composable 转换，批次 5 开头）；`git mv` 两个纯变换入 `utils/`；删除 7 个 Pinia store；6 个存量 uiStore 调用点过渡接线（getState()，批次 5/视图子任务重写）。

| 命令 | 退出码 | 结果 |
| --- | --- | --- |
| `bun run type-check` | 0 | ✓ |
| `bun run lint:ci` | 0 | ✓（exhaustive-deps 经 ref 镜像修正，无豁免注释） |
| `bun run test:smoke` | 0 | 67 文件 / 333 测试全绿 |
| `just frontend-check-quick` | 0 | 全绿 |
| `rg 'pinia\|defineStore' src` | — | 仅剩 `src/stores/usage.ts`（偏差 2）与死 .vue |

## 批次 5a 证据（纯变换 composable → utils，部分完成）

改动：新增 `src/utils/{profilesFilter,profilesInsights,tf}.ts` 纯变换核心与 `{claude,codex,grok}Profiles{Filter,Insights}.ts` 平台薄包装（8 个文件，此前已起草，本轮与 composable 逐一比对语义后收口）；重接消费方 `utils/claudeProfiles.ts`、`codexProfiles.ts`、`grokProfiles.ts`（useInsights 改为纯 builder，grok 移除 vue Ref 导入）、`vite-env.d.ts` 垫片类型（useInsights 入参 Ref→数组、返回 ProfilesInsightsResult）；过渡期 composable `useCodexProviders.ts`、`useCodexOAuthFlow.ts` 的 `useTf()` 重接为 `createTf(t)`（语义等价：同为 translateWithFallback(t, …)）；删除 8 个源 composable。语义比对结论：8 个草稿与其 composable 行为一致，无行为缺失；仅两处嵌套循环因 max-depth lint 展平（`(tags ?? []).filter(Boolean)`，守卫语义不变）。

| 命令 | 退出码 | 结果 |
| --- | --- | --- |
| `bun run type-check` | 0 | ✓ |
| `bun run lint:ci` | 0 | ✓（2 处 max-depth 经循环展平修正，无豁免注释） |
| `bun run test:smoke` | 0 | 67 文件 / 333 测试全绿 |
| `grep "from 'vue'" src/utils/*.ts` | — | 无匹配 |
| `ls src/composables` | — | 8 个纯变换 composable 已删除 |

mutation-rewrite.md 已填：useClaudeProfilesInsights（46–48）、useCodexProfilesInsights（50–51）、useProfilesFilter（100/114/157）、useProfilesInsights（128–216 共 9 行），共 17 行——全部判定为本地临时累积或新数组上排序（immutable 安全），无需改写，新写法列记「—」。

## 批次 5b-i 证据（服务端数据类 8 个 composable → Query/hooks）

改动：新增 `src/utils/poller.ts`（框架无关 `createPoller` 轮询核心，自 usePolledData 逐行搬运）；usePolledData 改薄 hook 包裹；`src/stores/usage.ts` 两处调用点改用 createPoller（偏差 2 存活期兼容）。Query 化：useBackendHealth（`features/system/queries.ts` 新建 systemKeys，refetchInterval 自适应 30s/5min）、useAgents（`features/agents/queries.ts` + mutations）、useCodexAgentSources、useCodexAgents、useCodexDashboard、useGrokDashboard（`features/{codex,grok}/queries.ts`，staleTime 30s/60s 按原 TTL）、usePlatformUsageInsight（usageKeys.insightDashboard 新增）。4 处 computed→useMemo（useCodexAgents），watch(:83) 由 Query key/enabled 变化覆盖（classification §2 登记）。签名变化（Ref→普通值、t 参数化、pauseWhen 类型收窄）均已在各文件头注登记。

| 命令 | 退出码 | 结果 |
| --- | --- | --- |
| `bun run type-check` | 0 | ✓ |
| `bun run lint:ci` | 0 | ✓ |
| `bun run test:smoke` | 0 | 67 文件 / 333 测试全绿 |
| `just frontend-check-quick`（主线程复跑） | 0 | 全绿 |
| `git diff --stat src/api` | — | 空 |
| `grep -l "from 'vue'" <8 文件>` | — | 无匹配 |

未决项登记：grok version queryFn 未透传 force（原实现即如此，仅越过前端 staleTime）；useCodexAgents.loadRuntimeSummary 复用 dashboard overview 缓存（原为绕 TTL 直连 IPC，头注已说明）；编排类多步命令保持 useCallback 编排非 mutation（useCodexAgentSources）。

## 批次 5b-ii 证据（服务端数据/SPLIT 类 7 个 composable → Query/hooks）

改动：新增 `src/features/monitoring/queries.ts`（monitoringKeys + 快照 fetcher，含 getRecentEvents 回退）与 `src/features/mcp/queries.ts`（mcpKeys：platformServers/plugins/unifiedList 三组）；`features/codex/queries.ts` 扩展 providers/tray 两组 key。Query 化：useMonitoringFeed（事件批量 setQueryData 接线，event-adjudication §2 落地）、useCodexProviders（providers Query + save/delete mutation + providerForm→react-hook-form）、useCodexTrayPanel（snapshot Query，force 经 forceRef 透传；`codex-tray:refresh` 组件级 listen 带取消协议，payload 全量快照→setQueryData）、usePlatformMcp/usePlatformPlugins/useUnifiedMcp（列表 Query staleTime Infinity + CRUD useMutation + 表单 react-hook-form/useState）、useCodexOAuthFlow（瞬态 useState，监听器 push 改取消协议 track）。下游适配：useMcpManager.ts 两处 `.value` 读取点改直接属性访问（批次 5c 整体重写前过渡，头注已登记）；`shell/eventBridge.ts` 补 `EventBatcher<T>` 具名类型导出。签名变化（消费方均为待迁移 .vue 视图）：Ref/computed → 普通值、t 参数化、activeManagerTab Ref→setter、OAuth 注入 Ref→setState 写入器、loadServers(platform?) 收窄为 loadServers()（唯一消费方从不传参），均已在各文件头注登记。

| 命令 | 退出码 | 结果 |
| --- | --- | --- |
| `bun run type-check` | 0 | ✓ |
| `bun run lint:ci` | 0 | ✓（仅存量 scripts/measure-distribution.mjs prefer-const warning） |
| `bun run test:smoke` | 0 | 67 文件 / 333 测试全绿 |
| `git diff --stat src/api` | — | 空 |
| `grep -l "from 'vue'" <7 文件>` | — | 无匹配 |
| dashboardPresentation 的 `MonitoringEntry` 类型导入 | — | 解析正常（type-check 通过佐证） |

语义判定登记：
- **useMonitoringFeed 共享缓存说明**：按 event-adjudication §2 以单一 queryKey（monitoringKeys.feed）承载 feed 缓存，事件写入走 createEventBatcher（250ms）批量 setQueryData；各消费者仍挂载独立实例（各自的监听器/batcher/logger 订阅与 start/pause/resume 生命周期），条目缓冲共享同一缓存——原 per-instance 缓冲的隔离性由共享缓存承担，maxEntries 取后 flush 实例值。去重键改为与当前缓存内容比对（与原 seenEntries 在 trim 后重建的行为等价）。token-stats 为替换语义，同用 batcher 合并渲染。
- **useCodexOAuthFlow 编排保持 useCallback**：OAuth 命令均为多步编排（端口检查+启动+打开浏览器等），沿用批次 5b-i「编排类多步命令非 mutation」先例；监听回调经 latestDepsRef 读最新注入依赖，避免闭包陈旧值。
- **useUnifiedMcp 加载失败 toast 移入 queryFn**：Query 去重保证单次拉取只弹一次（v5 无 query 级 onError）。
- **codex-tray:refresh 登记**：event-adjudication §4 已有该事件归属行，无需新增。

规则匹配备注：useMonitoringFeed 的局部 `isRecord` 守卫为原文件逐字保留（归一化逻辑语义不变更优先）；如需收口到共享守卫模块，随后续清理统一处理。
