# composable 三类归类 + watch/computed 排查（批次 1）

> AC3 / R6。归类依据 `state-disposition.md`（arch-quality-perf 批次 6，45 项判定表），
> 本表按 `design.md` §5 的方法复核（`rg -l "from 'vue'"` + 生命周期特征）后落盘。

## 1. 三类归类（35 项，主类别 + 承载；与 state-disposition.md §4 一致，无重新决策）

- **服务端数据（16）**：useAgents、useBackendHealth、useCachedFetch（机制，随 Query 消解）、useCodexAgentSources、useCodexAgents、useCodexDashboard、useCodexProviders（SPLIT）、useCodexTrayPanel（SPLIT）、useGrokDashboard、useMonitoringFeed（事件流）、usePlatformMcp（SPLIT）、usePlatformPlugins（SPLIT）、usePlatformUsageInsight、usePolledData（机制，随 Query 消解）、useUnifiedMcp（SPLIT）、useCodexOAuthFlow（SPLIT，事件订阅）
- **跨页面共享（2）**：useProfilesQuickSwitch（persist）、useProviderTemplates（persist）
- **组件本地（9）**：useAccessibility（SPLIT，工具部分 → utils）、useAnimationVisibility、useConfirmAction、useFuzzySearch、useMainLayoutShell（SPLIT）、useMcpManager（SPLIT）、usePageTransition（→ shell）、useProfilesHotkeys、useStream（**死代码，删除**）
- **纯变换（8）**：useClaudeProfilesFilter、useClaudeProfilesInsights、useCodexProfilesFilter、useCodexProfilesInsights、useGrokProfilesFilter、useProfilesFilter、useProfilesInsights、useTf → `utils/`

复核：`rg -l "from 'vue'" src/composables` = 34/35 文件导入 vue（唯 useTf 不导入，与其纯变换归类一致）；2 个机制类（useCachedFetch/usePolledData）无具体业务数据，随 Query 语义消解。

## 2. watch 排查与选项映射（composable 内 7 处；store 内 0 处）

| 位置 | 原代码 | 选项 | React 替代（design.md §6.3） |
| --- | --- | --- | --- |
| `useMainLayoutShell.ts:132` | watch(routeFullPath, () => { | 见批次 5 逐点登记 | 见批次 5 |
| `useMainLayoutShell.ts:136` | watch(hasSidebar, (value) => { | 见批次 5 逐点登记 | 见批次 5 |
| `useMainLayoutShell.ts:142` | watch([isMobileSidebar, isSidebarOpen], ([mobile, open]) => { | 见批次 5 逐点登记 | 见批次 5 |
| `useMcpManager.ts:196` | watch(groupedServers, (groups) => { | 见批次 5 逐点登记 | 见批次 5 |
| `usePlatformUsageInsight.ts:83` | `watch(() => [unref(platform), unref(days), unref(enabled)], ([, , isEnabled]) => { if (isEnabled) void refresh() })` | 无选项（非 immediate、默认 flush pre） | 批次 5：watch 目标即查询参数 → Query key（platform+日期窗口）/enabled 变化自动 refetch，无需 effect；onMounted 初始拉取由 Query 挂载拉取覆盖（`usePlatformUsageInsight.ts`） |
| `usePolledData.ts:200` | `watch(pauseWhen as WatchSource<boolean>, (paused) => { paused ? stopTimer() : isActive && (doFetch(), startTimer()) })` | 无选项（非 immediate、默认 flush pre） | 批次 5：布尔 pauseWhen → hook effect 调用 `poller.onPauseChange(paused)`（分支逻辑逐行保留在核心）；函数源由核心每 tick 求值（`usePolledData.ts` + `utils/poller.ts`） |
| `useProfilesQuickSwitch.ts:96` | watch( | 见批次 5 逐点登记 | 见批次 5 |

选项（immediate/deep/flush）逐点判定在批次 5 各文件转换时补充至此表。

## 3. computed 排查（store + composable 共 63 处；store 内 13 处 / composable 内 50 处）

R6 逐个列出响应式来源：store 内 13 处随批次 4 转选择器/Query select 时登记；composable 内 50 处随批次 5 各文件转换时登记（`react-hooks/exhaustive-deps` error 级 lint 拦截遗漏）。

排查清单（文件:行）：

```
ccr-ui\src\stores\commands.ts:20
ccr-ui\src\stores\shellPreferences.ts:95
ccr-ui\src\stores\usage.ts:165
ccr-ui\src\stores\usage.ts:167
ccr-ui\src\stores\usage.ts:173
ccr-ui\src\stores\usage.ts:175
ccr-ui\src\stores\usage.ts:177
ccr-ui\src\stores\usage.ts:187
ccr-ui\src\stores\usage.ts:194
ccr-ui\src\stores\usage.ts:195
ccr-ui\src\stores\usage.ts:196
ccr-ui\src\stores\usage.ts:197
ccr-ui\src\stores\usage.ts:198
ccr-ui\src\composables\useAnimationVisibility.ts:9
ccr-ui\src\composables\useCachedFetch.ts:35
ccr-ui\src\composables\useCodexAgents.ts:82
ccr-ui\src\composables\useCodexAgents.ts:83
ccr-ui\src\composables\useCodexAgents.ts:84
ccr-ui\src\composables\useCodexAgents.ts:250
ccr-ui\src\composables\useCodexAgentSources.ts:26
ccr-ui\src\composables\useCodexDashboard.ts:299
ccr-ui\src\composables\useCodexDashboard.ts:303
ccr-ui\src\composables\useCodexDashboard.ts:305
ccr-ui\src\composables\useCodexDashboard.ts:310
ccr-ui\src\composables\useCodexDashboard.ts:314
ccr-ui\src\composables\useCodexDashboard.ts:318
ccr-ui\src\composables\useCodexTrayPanel.ts:45
ccr-ui\src\composables\useCodexTrayPanel.ts:46
ccr-ui\src\composables\useCodexTrayPanel.ts:47
ccr-ui\src\composables\useCodexTrayPanel.ts:48
ccr-ui\src\composables\useFuzzySearch.ts:41
ccr-ui\src\composables\useGrokDashboard.ts:300
ccr-ui\src\composables\useGrokDashboard.ts:304
ccr-ui\src\composables\useGrokDashboard.ts:310
ccr-ui\src\composables\useGrokDashboard.ts:316
ccr-ui\src\composables\useGrokDashboard.ts:327
ccr-ui\src\composables\useMainLayoutShell.ts:21
ccr-ui\src\composables\useMainLayoutShell.ts:22
ccr-ui\src\composables\useMainLayoutShell.ts:23
ccr-ui\src\composables\useMainLayoutShell.ts:26
ccr-ui\src\composables\useMainLayoutShell.ts:29
ccr-ui\src\composables\useMcpManager.ts:117
ccr-ui\src\composables\usePlatformMcp.ts:149
ccr-ui\src\composables\usePlatformMcp.ts:346
ccr-ui\src\composables\usePlatformMcp.ts:347
ccr-ui\src\composables\usePlatformMcp.ts:348
ccr-ui\src\composables\usePlatformPlugins.ts:65
ccr-ui\src\composables\usePlatformPlugins.ts:256
ccr-ui\src\composables\usePlatformPlugins.ts:257
ccr-ui\src\composables\usePlatformPlugins.ts:258
ccr-ui\src\composables\usePlatformPlugins.ts:259
ccr-ui\src\composables\usePlatformUsageInsight.ts:38
ccr-ui\src\composables\usePlatformUsageInsight.ts:39
ccr-ui\src\composables\usePlatformUsageInsight.ts:40
ccr-ui\src\composables\useProfilesQuickSwitch.ts:102
ccr-ui\src\composables\useProfilesQuickSwitch.ts:106
ccr-ui\src\composables\useProfilesQuickSwitch.ts:114
ccr-ui\src\composables\useProviderTemplates.ts:19
ccr-ui\src\composables\useUnifiedMcp.ts:88
ccr-ui\src\composables\useUnifiedMcp.ts:133
ccr-ui\src\composables\useUnifiedMcp.ts:136
ccr-ui\src\composables\useUnifiedMcp.ts:145
ccr-ui\src\composables\useUnifiedMcp.ts:151
```

### 3.1 批次 5 已转换文件的 computed 登记（响应式来源 → useMemo 依赖）

| 文件 | computed（原行） | 响应式来源 | React 形态 |
| --- | --- | --- | --- |
| `useCodexAgentSources.ts` | selectedSource（26） | sources、selectedSourceId | `useMemo([sources, selectedSourceId])` |
| `useCodexAgents.ts` | currentContextRequest（71） | activeContext.mode、activeContext.projectRoot | `useMemo`（原始值依赖，避免对象身份抖动） |
| `useCodexAgents.ts` | hasProjectShortcut / isProjectMode / contextLabel（82–84） | lastProjectRoot；activeContext.mode；activeContext.label | 各自 `useMemo` |
| `useCodexAgents.ts` | activeMode（250，返回处内联） | activeContext.mode | `useMemo` |
| `useCodexDashboard.ts` | loading（299）/ error（303） | 三查询 isFetching；overviewError/usageError | `useMemo`（error 为普通派生值） |
| `useCodexDashboard.ts` | currentAccountLabel（305）/ currentProfileLabel（310） | overview.auth.current；overview.profiles.current_profile；t | `useMemo` |
| `useCodexDashboard.ts` | usageTotalRequests（314）/ usageTotalTokens（318） | usageSummary.all_time | `useMemo` |
| `useCodexDashboard.ts` | readinessItems/nextActions/primaryAction/compactInventory/managementLinks（324–627，排查清单外多行声明） | overview、usageSummary、isFetching 标志、versionStatus/versionLabel、t | 各自 `useMemo`；healthItems 为 readinessItems 别名 |
| `useGrokDashboard.ts` | loading（300）/ initialLoading（304） | environment/overview/version 查询 isFetching、localOnly、overview | `useMemo` |
| `useGrokDashboard.ts` | currentProfileLabel（310）/ activationLabel（316）/ authModeLabel（327） | overview.current_profile/activation_name/activation/auth_mode；t | `useMemo` |
| `useGrokDashboard.ts` | activationWarning/versionTone/readinessItems/nextActions/primaryAction/managementItems（337–553，排查清单外多行声明） | overview、versionStatus/versionLabel/versionTone、currentProfileLabel、activationLabel、t | 各自 `useMemo` |
| `usePlatformUsageInsight.ts` | dateWindow / resolvedLabels / presentation（38–40） | days；labels；dashboard 数据 + resolvedLabels + tone | 各自 `useMemo` |

### 3.2 批次 5b-ii 已转换文件的 computed 登记（响应式来源 → useMemo / 常量）

| 文件 | computed（原行） | 响应式来源 | React 形态 |
| --- | --- | --- | --- |
| `useCodexProviders.ts` | codexTemplateDraft（69） | providerForm 表单值（name/baseUrl/websiteUrl/apiKeyUrl，RHF watch） | `useMemo([providerForm])` |
| `useCodexTrayPanel.ts` | currentAccount（45）/ accounts（46）/ canManageAccounts（47） | snapshot（Query data） | 各自 `useMemo([snapshot])`；canOpenSwitchScreen 为 canManageAccounts 别名 |
| `usePlatformMcp.ts` | config（149） | platformConfigs[platform]，platform 为挂载期常量 | 直接查表，无响应性需求 |
| `usePlatformMcp.ts` | moduleColor/i18nPrefix/parentPath（348–350） | config 常量字段 | 普通常量 |
| `usePlatformPlugins.ts` | config（65）；moduleColor 等（256–259） | 同上 | 同上 |
| `useUnifiedMcp.ts` | filteredServers（90） | servers、filterPlatform/filterKeyword/filterProtocol/filterScope | `useMemo` |
| `useUnifiedMcp.ts` | scopeCounts（127）/ platformCounts（138） | servers | `useMemo([servers])` |
| `useUnifiedMcp.ts` | sourceDiagnostics（135） | diagnostics | 普通别名 |
| `useUnifiedMcp.ts` | currentCapability（147） | capabilities、formData.platform | `useMemo` |
| `useUnifiedMcp.ts` | hasActiveFilters（153） | filter* 四项 | `useMemo` |
| `useMonitoringFeed.ts` / `useCodexOAuthFlow.ts` | —（0 处 computed） | — | — |

§2 watch 寄存器补充：本批 7 个文件原实现 watch 计数为 **0**（复核），无新增选项映射行。
