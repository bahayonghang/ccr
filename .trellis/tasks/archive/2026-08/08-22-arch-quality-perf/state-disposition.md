# 状态三分类判定表（state-disposition.md）

> 交付对象：**本文件是 `08-22-state-logic-port` 的直接输入**。该任务按本表执行，不重新决策归属。
> 来源：本任务 `08-22-arch-quality-perf` 批次 6（AC6）。判据与列定义见本任务 `design.md` §5，父任务 `08-22-react-migration/design.md` §4（10 个 store 处理映射）与 §5（缓存路由替代）。
> 分支：`react-migration/react-foundation`。未提交。

## 1. 判定摘要

### 1.1 计数（实际 vs 计划）

| 范围 | 计划 | 实际（wc -l 实测） | 判定 |
| --- | --- | --- | --- |
| `src/stores/` | 10 | 10 | 一致 |
| `src/composables/` | 35 | 35 | 一致 |
| 合计 | 45 | 45 | 一致，无未判定项 |

行数来源：`wc -l ccr-ui/src/stores/*.ts ccr-ui/src/composables/*.ts`（2026-08-23，分支 `react-migration/react-foundation`）。

### 1.2 类别计数（按主类别）

| 类别 | store | composable | 合计 | 承载位置 |
| --- | --- | --- | --- | --- |
| 服务端数据 | 5 | 16 | **21** | TanStack Query（含事件桥接失效） |
| 跨页面共享 | 3 | 2 | **5** | Zustand |
| 组件本地 | 0 | 9 | **9** | useState / react-hook-form / useEffect |
| 纯变换 | 2 | 8 | **10** | `utils/` |
| 合计 | 10 | 35 | 45 | |

### 1.3 跨承载 SPLIT 清单（一个文件映射两个承载，12 项）

| 文件 | 拆分 |
| --- | --- |
| `stores/usage.ts` | 数据 → Query，视图偏好/筛选 → Zustand |
| `stores/configs.ts` | 数据 → Query，选中态（current_config）→ Zustand |
| `stores/claudeObserver.ts` | 数据切片 → Query，订阅/对话框 UI 态 → Zustand |
| `composables/useAccessibility.ts` | useFocusTrap/useEscapeKey → 组件本地；ariaUtils/focusUtils/useUniqueId → utils/ |
| `composables/useCodexOAuthFlow.ts` | OAuth 命令 → Query（mutation），事件监听 → 事件桥接；流程瞬态 → useState |
| `composables/useCodexProviders.ts` | providers → Query，providerForm → react-hook-form |
| `composables/useCodexTrayPanel.ts` | snapshot → Query（事件桥接失效），screen/isDragging → useState |
| `composables/useMainLayoutShell.ts` | sidebarWidth 委派 shellPreferences → Zustand；isSidebarOpen/isResizing → useState |
| `composables/useMcpManager.ts` | 服务器数据经 useUnifiedMcp → Query；panelMode/selectedKeys → useState |
| `composables/usePlatformMcp.ts` | 服务器数据 → Query，表单 → react-hook-form |
| `composables/usePlatformPlugins.ts` | 插件数据 → Query，表单 → react-hook-form |
| `composables/useUnifiedMcp.ts` | 服务器数据 → Query，表单/筛选 → react-hook-form + useState |

### 1.4 与父任务 `design.md` §4 的核对

父任务 §4 的 10 个 store 映射与本表逐行一致：`usage`（Query+Zustand）、`configs`（Query+Zustand）、`commands`（Query）、`claudeObserver`（Query+Zustand）、`homeUsageOverview`（Query）、`ui`（Zustand）、`shellPreferences`（Zustand）、`commandsView`（Zustand）、`usageDashboardPayload`（utils）、`usageImportNormalization`（utils）。两个「纯变换」文件确认仍位于 `stores/`，迁移时移入 `utils/`（`08-22-state-logic-port` 批次 4 已登记该动作）。

## 2. 偏差记录（代码现实 vs 计划文档）

1. **父任务 `design.md` §4 的 store 结构断言不准确**：§4 称「10 个 Pinia setup store，含 21 处 `ref`、13 处 `computed`、0 处 `watch`、0 处 `reactive`」。实测：**8 个 setup + 2 个 Options-API**（`configs.ts`、`commandsView.ts` 用对象式 `state/getters/actions` 写法，含 `$patch` / `$state` 使用）。响应式原语实测 `ref`=61（另有 `shallowRef`=9）、`computed`=14、`watch`=0、`reactive`=0。`watch`=0 与 `reactive`=0 两处断言**确认成立**；`ref` 与 `computed` 计数**低估**（计数方法见下注）。对迁移的影响：`configs` / `commandsView` 的 `this.$state`、`this.$patch` 需按 Options-API 语义映射到 Zustand。
   - 注：`ref` 计数含 `<T>` 泛型调用（`ref<Toast[]>([])`），故比 `grep -c 'ref('` 高；此偏差只影响文档数字，不影响归属判定。
2. **`composables/useStream.ts` 是无消费方（死代码）**：全仓（含 tests）无任何 `import ... from '@/composables/useStream'`，仅有 `CommandsView.vue` 的两处注释引用。`CommandsView` 的实际流式输出走 Tauri `listen` 事件 + `startCcrCommandJob`/`cancelCcrCommandJob` IPC（直接订阅），不经 `useStream`。归类为组件本地（流式缓冲）并标注：建议删除，或按父任务 §5 的 `commands/:client` 流式缓冲入 Zustand 的规则复活。
3. **计数无偏差**：10 store + 35 composable = 45，与计划一致，无需修正计划数字。

## 3. store 判定表（`src/stores/`，10 项）

| 名称 | 行数 | 类别 | 承载位置 | 依据 |
| --- | --- | --- | --- | --- |
| `src/stores/usage.ts` | 991 | 服务端数据 + 跨页面共享（SPLIT） | TanStack Query + Zustand | 全部数据切片（summary/trends/modelStats/projectStats/providerStats/sourceStats/heatmap/logs/archive/snapshot/capabilities）来自 `getUsage*V2` IPC 命令，30s TTL 缓存 + 30s 自动刷新 + `usePolledData` 轮询，有明确新鲜度概念 → Query；`platform`/`timeRange`/`logsPage`/`logsModelFilter` 为 `usage` 路由视图偏好 → Zustand（父 §5 usage 缓存路由）；Tauri 事件（`usage:snapshot-updated`、`usage:job-progress/finished/failed`）→ 事件桥接失效 Query |
| `src/stores/configs.ts` | 92 | 服务端数据 + 组件本地（SPLIT） | TanStack Query + Zustand | `listConfigs()` IPC 拉取配置列表，5 分钟缓存（`isCacheValid`）→ Query；`current_config` 选中态 → Zustand（父 §4：数据 → Query，选中态 → Zustand）。Options-API 写法，无 setup |
| `src/stores/commands.ts` | 86 | 服务端数据 | TanStack Query | `listCommands()` IPC + `useCachedFetch`（2min TTL）→ Query（`staleTime` 等效）；`executeCommand` 运行 → `useMutation` + `invalidateQueries`。`running`/`currentCommand`/`lastOutput` 为执行瞬态，随命令页（Query）承载 |
| `src/stores/claudeObserver.ts` | 251 | 服务端数据 + 跨页面共享（SPLIT） | TanStack Query + Zustand | 9 个数据切片全部来自 `claude_observer_*` IPC 命令，订阅 `claude_observer:updated` 事件驱动 refetch → Query + 事件桥接失效（父 §4：事件流数据 → Query）；订阅/面板 UI 态 → Zustand |
| `src/stores/homeUsageOverview.ts` | 357 | 服务端数据 | TanStack Query | `getHomeUsageOverviewV2` / `getUsageCapabilitiesV2` IPC，30s TTL 缓存 + 事件（`usage:snapshot-updated`、`usage:job-*`、`usage:session-index-*`）驱动失效 → Query + 事件桥接失效 |
| `src/stores/ui.ts` | 134 | 跨页面共享 | Zustand | toast / confirm 对话框 / 全局 loading 为全局 UI 编排态，全路由读写，非服务端数据 → Zustand（父 §4：toast/收藏/历史 → Zustand） |
| `src/stores/shellPreferences.ts` | 294 | 跨页面共享 | Zustand（persist 中间件，存储键不变） | theme/flavor/accent/字体/locale/sidebarWidth 等为持久化 UI 偏好，全壳层共享 → Zustand + persist（父 §4）。`confirmBeforeExit` 等 runtime 偏好经 `shellGetPreferences`/`shellSetPreferences` IPC 同步为异步写副作用（flush 到后端），不升级为 Query 承载 |
| `src/stores/commandsView.ts` | 73 | 跨页面共享 | Zustand（persist，键 `ccr-commands-view`） | 排序/视图模式/折叠等视图偏好，localStorage 持久化，非服务端数据 → Zustand。Options-API 写法（`$state`/`$patch`） |
| `src/stores/usageDashboardPayload.ts` | 171 | 纯变换 | `utils/`（移出 stores） | 纯类型 + 无状态映射函数（`normalizeDashboardPayload`、`buildDashboardFetchKey`、`buildUsageLogsQuery`、`normalizePaginatedLogs`、`isCapabilityUnsupported` 等），输入→输出，无响应式状态 → utils/（父 §4，确认仍在 stores/） |
| `src/stores/usageImportNormalization.ts` | 83 | 纯变换 | `utils/`（移出 stores） | 纯归一化函数（`normalizeImportResponse`、`buildImportSummary`、`normalizeUserVisibleImportJob`），无状态 → utils/（父 §4，确认仍在 stores/） |

## 4. composable 判定表（`src/composables/`，35 项）

### 4.1 服务端数据（16）

| 名称 | 行数 | 类别 | 承载位置 | 依据 |
| --- | --- | --- | --- | --- |
| `src/composables/useAgents.ts` | 136 | 服务端数据 | TanStack Query + useMutation | agents 列表 CRUD 全部走 IPC（`listAgents`/`listGeminiAgents`/`addAgent`/`updateAgent`/`deleteAgent`/`toggleAgent`），含 `listConfigs`/`getHistory` 辅助加载 → Query + mutations。`currentAgent` 为选中瞬态 |
| `src/composables/useBackendHealth.ts` | 94 | 服务端数据 | TanStack Query（refetchInterval 自适应 30s/5min） | `health_check` IPC 轮询，有新鲜度与退避语义（健康 5min、异常 30s）→ Query 轮询；模块级单例 ref + `usePolledData` 的共享语义由 Query 缓存承担。批次 4 已为 rules-of-hooks 误判登记豁免（归本迁移） |
| `src/composables/useCachedFetch.ts` | 90 | 服务端数据（机制） | TanStack Query（staleTime/gcTime 替代） | 通用 TTL 缓存 fetch 包装器，不持有具体业务数据；是命令缓存机制的等价物 → Query 的缓存语义直接覆盖，包装器本身随 `commands.ts` 一起消解 |
| `src/composables/useCodexAgentSources.ts` | 192 | 服务端数据 | TanStack Query + useMutation | Codex agent source 的 IPC CRUD + catalog（`listCodexAgentSources`/`getCodexAgentSourceCatalog`/`add`/`remove`/`sync`/`install`/`untrack`）→ Query + mutations；`selectedSourceId` 为列表选中瞬态 |
| `src/composables/useCodexAgents.ts` | 283 | 服务端数据 | TanStack Query + useMutation | agents/diagnostics/context/models/sessions 全部来自 IPC（`listCodexAgents`/`listCodexModels`/`addCodexAgent`/`update`/`rename`/`delete`/`copy`/`validateCodexAgentToml`）→ Query + mutations；`lastProjectRoot` localStorage 为 UI 偏好（次要） |
| `src/composables/useCodexDashboard.ts` | 657 | 服务端数据 | TanStack Query（staleTime 30s/60s，select 派生卡片） | overview/usage/version 均来自 IPC（`getCodexDashboardOverview`/`getCodexDashboardUsageSummary`/`getCliVersion`），模块级共享 TTL 缓存 → Query 缓存；readiness/nextActions/inventory 等派生卡片 → Query `select` 或 `useMemo` |
| `src/composables/useCodexProviders.ts` | 241 | 服务端数据 + 组件本地（SPLIT） | TanStack Query + react-hook-form | providers 列表 CRUD 走 IPC（`codexListModelProviders`/`codexSaveModelProvider`/`codexDeleteModelProvider`）→ Query + mutations；`providerForm`（reactive）为单表单瞬态 → react-hook-form |
| `src/composables/useCodexTrayPanel.ts` | 182 | 服务端数据 + 组件本地（SPLIT） | TanStack Query + useState（事件桥接失效） | snapshot 来自 `getCodexTraySnapshot` IPC + `codex-tray:refresh` Tauri 事件（`win.listen`）→ Query + 事件桥接失效（事件订阅型）；`screen`/`isDragging` 为托盘窗口内瞬态 → useState。运行于独立 `codex-tray-panel` 窗口 |
| `src/composables/useGrokDashboard.ts` | 580 | 服务端数据 | TanStack Query（staleTime 30s/60s，select 派生） | overview/version/environment 来自 IPC（`getGrokDashboardOverview`/`getCliVersion`/`getCurrentEnvironment`），模块级共享 TTL 缓存 → Query；readiness/actions/management 派生 → `select`/`useMemo` |
| `src/composables/useMonitoringFeed.ts` | 400 | 服务端数据（Tauri 事件流） | 事件桥接层 + TanStack Query（setQueryData 批量提交） | 高频 Tauri 事件 `app:monitoring` + `token-stats` + 前端 logger 订阅，初始快照走 `getMonitoringFeed`/`getRecentEvents` IPC → 事件桥接层按 state-logic-port 批次 3 约定：高频事件 ref 累积 + 定时批量 `setQueryData`（父 §4 末段）。多消费者（MonitoringView/DashboardView/DashboardSignalStream）各自挂载独立实例 |
| `src/composables/usePlatformMcp.ts` | 383 | 服务端数据 + 组件本地（SPLIT） | TanStack Query + react-hook-form | Gemini MCP 服务器 CRUD 走 IPC（`listGeminiMcpServers`/`add`/`update`/`delete`）→ Query + mutations；formData/env/arg 等表单瞬态 → react-hook-form。批次 4 已为裸 `useUIStore()` 登记豁免（归本迁移） |
| `src/composables/usePlatformPlugins.ts` | 287 | 服务端数据 + 组件本地（SPLIT） | TanStack Query + react-hook-form | Gemini 插件 CRUD 走 IPC（`listGeminiPlugins`/`add`/`update`/`delete`/`toggle`）→ Query + mutations；`formData`/`configJson` 表单瞬态 → react-hook-form。批次 4 豁免同上 |
| `src/composables/usePlatformUsageInsight.ts` | 100 | 服务端数据 | TanStack Query（enabled/days 参数） | `getUsageDashboardV2` IPC 拉取平台用量洞察卡，loading/error/dashboard 均为服务端数据形态 → Query（`useQuery` + `select` 派生 presentation）。消费者 CodexView/OpenCodeView/GeminiCliView 各挂独立实例 |
| `src/composables/usePolledData.ts` | 243 | 服务端数据（机制） | TanStack Query（refetchInterval/refetchOnWindowFocus 替代） | 通用轮询包装器（interval/隐藏暂停/key 共享 in-flight），是 polling 机制而非业务状态 → Query 的 `refetchInterval` + `refetchOnWindowFocus` 直接覆盖，包装器消解（消费方 useBackendHealth、usage.ts 自动刷新） |
| `src/composables/useUnifiedMcp.ts` | 534 | 服务端数据 + 组件本地（SPLIT） | TanStack Query + react-hook-form + useState | 统一 MCP 服务器 CRUD 走 IPC（`listUnifiedMcp`/`add`/`update`/`delete`/`toggle`）→ Query + mutations；`filter*` 筛选与 `formData`/arg/env/header 表单瞬态 → useState + react-hook-form。批次 4 已为裸 `useUIStore()` 登记豁免（归本迁移） |
| `src/composables/useCodexOAuthFlow.ts` | 279 | 服务端数据（事件订阅）+ 组件本地（SPLIT） | 事件桥接层 + TanStack Query + useState | OAuth 命令（`codexOAuthLoginStart`/`codexOAuthSubmitCallbackUrl`/`codexOAuthLoginCompleted` 等 IPC）→ Query mutations；Tauri 事件 `codex-oauth-login-completed`/`codex-oauth-login-timeout`（`listen`）→ 事件桥接层失效/推进；oauthLoginId/pending/busy 为单弹窗瞬态 → useState（事件订阅型） |

### 4.2 跨页面共享（2）

| 名称 | 行数 | 类别 | 承载位置 | 依据 |
| --- | --- | --- | --- | --- |
| `src/composables/useProfilesQuickSwitch.ts` | 171 | 跨页面共享 | Zustand（persist localStorage） | 钉选/最近使用为 UI 偏好（非服务端数据），localStorage 按平台键持久化；消费方横跨 ClaudeCodeProfilesView / CodexProfilesView / GrokProfilesView / ProfilesQuickRail / useProfilesHotkeys 三个路由 + 共享组件 → Zustand + persist（与 ui.ts 收藏同语义） |
| `src/composables/useProviderTemplates.ts` | 46 | 跨页面共享 | Zustand（persist localStorage） | 模块级单例 `customTemplates`（localStorage 持久化）+ 内置/自定义模板合并，供共享 ProviderTemplateSelector（Codex 等多平台消费）→ Zustand + persist |

### 4.3 组件本地（9）

| 名称 | 行数 | 类别 | 承载位置 | 依据 |
| --- | --- | --- | --- | --- |
| `src/composables/useAccessibility.ts` | 225 | 组件本地 + 纯变换（SPLIT） | useState/useEffect + `utils/` | `useFocusTrap`/`useEscapeKey` 为 DOM 生命周期钩子（keydown 监听，弹窗内瞬态）→ useState/useEffect；`ariaUtils`/`focusUtils`/`useUniqueId` 为无状态纯工具 → utils/ |
| `src/composables/useAnimationVisibility.ts` | 66 | 组件本地 | useState + useEffect | 视口/页面可见性/reduced-motion 标志由 IntersectionObserver/matchMedia/visibilitychange 驱动，属单个动画宿主组件的瞬态 |
| `src/composables/useConfirmAction.ts` | 61 | 组件本地 | useState | 二次确认弹窗的 isOpen/busy/dialog 为消费视图内瞬态 |
| `src/composables/useFuzzySearch.ts` | 53 | 组件本地 | useState + useMemo | `query` 为搜索框瞬态；Fuse 实例与 results 派生 → useMemo |
| `src/composables/useMainLayoutShell.ts` | 162 | 组件本地 + 跨页面共享（SPLIT） | useState + Zustand（委派 shellPreferences） | isSidebarOpen/isResizing/isMobileSidebar 为 MainLayout 内瞬态 → useState；sidebarWidth 已委派 `useShellPreferencesStore` → Zustand（批次 4 已为裸 `useShellPreferencesStore()` 登记豁免，归本迁移） |
| `src/composables/useMcpManager.ts` | 266 | 组件本地 + 服务端数据（SPLIT） | useState + useMemo + TanStack Query | panelMode/selectedKeys/isMultiSelectMode 为 MCP 管理页内瞬态（路由未在父 §5 缓存清单）→ useState；grouping/search 派生 → useMemo；服务器数据经 `useUnifiedMcp` → Query |
| `src/composables/usePageTransition.ts` | 84 | 组件本地（壳层瞬态） | useState（React Router 集成，置于 shell） | transitionName 由导航上下文（depth/group/back）派生，单一消费者 MainLayout，为路由切换瞬态 → React Router location 逻辑 + useState；不再有 vue-router beforeEach 守卫 |
| `src/composables/useProfilesHotkeys.ts` | 52 | 组件本地 | useEffect | window keydown 快捷键监听（⌘K/⌘1-9），纯生命周期 DOM 钩子 |
| `src/composables/useStream.ts` | 297 | 组件本地（**死代码，无消费方**） | 删除；如复活 → 见依据 | 全仓无任何 `import`（含 tests），仅 `CommandsView.vue` 两处注释引用；CommandsView 实际流式输出走 Tauri `listen` 事件。建议直接删除；若复活，其 lines 缓冲按父 §5 `commands/:client` 流式输出累积缓冲入 Zustand（切回续读） |

### 4.4 纯变换（8）

| 名称 | 行数 | 类别 | 承载位置 | 依据 |
| --- | --- | --- | --- | --- |
| `src/composables/useClaudeProfilesFilter.ts` | 57 | 纯变换 | `utils/` | 注入 Claude 平台差异后委托 `useProfilesFilter` 的薄包装，无自有状态，纯输入→派生输出 |
| `src/composables/useClaudeProfilesInsights.ts` | 59 | 纯变换 | `utils/` | 委托 `useProfilesInsights` 的薄包装，无状态 |
| `src/composables/useCodexProfilesFilter.ts` | 58 | 纯变换 | `utils/` | 同上（Codex 搜索字段注入） |
| `src/composables/useCodexProfilesInsights.ts` | 65 | 纯变换 | `utils/` | 同上（四 auth 模式/弃用判定注入） |
| `src/composables/useGrokProfilesFilter.ts` | 47 | 纯变换 | `utils/` | 同上（Grok 搜索字段注入） |
| `src/composables/useProfilesFilter.ts` | 187 | 纯变换 | `utils/` | 平台无关过滤/排序/分组核心：接收输入 ref、返回派生 computed，不自持状态 → 纯函数（组件内 `useMemo` 或 `utils/` 选择器） |
| `src/composables/useProfilesInsights.ts` | 241 | 纯变换 | `utils/` | 分布统计与健康审计核心，全部派生自传入 profiles，无状态 |
| `src/composables/useTf.ts` | 16 | 纯变换 | `utils/` | i18n translate-with-fallback 包装，无状态 |

## 5. 事件订阅型 composable 汇总（承载 = 事件桥接层）

按父任务 `design.md` §4 末段：后端 `emit` 事件在监听回调中调用 `queryClient.invalidateQueries` / `setQueryData`，订阅建立与解绑保持在组件生命周期内。本判定表内的事件订阅型 composable 及其事件：

| 文件 | 事件 | 桥接动作 |
| --- | --- | --- |
| `composables/useCodexOAuthFlow.ts` | `codex-oauth-login-completed`、`codex-oauth-login-timeout` | 推进/失效对应 OAuth mutation 的 Query |
| `composables/useCodexTrayPanel.ts` | `codex-tray:refresh` | `setQueryData`（snapshot）或失效 |
| `composables/useMonitoringFeed.ts` | `app:monitoring`、`token-stats` | 高频 → ref 累积 + 定时批量 `setQueryData`（state-logic-port 批次 3 落地，间隔取值待本任务批次 7 场景 3 数据） |

store 内的事件监听（`usage.ts`、`homeUsageOverview.ts`、`claudeObserver.ts`）同样走事件桥接，其事件清单由 `08-22-state-logic-port` 批次 3 的 inventory 产出，本表不重复登记。

## 6. 交付说明

- 本文件 45 项全部判定，无未判定项（AC6）。零空单元格。
- `08-22-state-logic-port` 按本表执行：批次 2 承接 21 项服务端数据（Query 层）、批次 3 承接事件桥接、批次 4 承接 5 项 Zustand + 2 项 utils 迁移、批次 5 承接 composable → hooks 重写。
- 归类依据按「代码实际做什么」判定：IPC/Tauri 事件数据 → Query；跨路由共享的本地/偏好状态 → Zustand；单组件/单表单瞬态 → useState/react-hook-form；无状态映射 → utils/。
