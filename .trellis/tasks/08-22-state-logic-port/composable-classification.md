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
| `usePlatformUsageInsight.ts:83` | watch( | 见批次 5 逐点登记 | 见批次 5 |
| `usePolledData.ts:200` | watch(pauseWhen as WatchSource<boolean>, (paused) => { | 见批次 5 逐点登记 | 见批次 5 |
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
