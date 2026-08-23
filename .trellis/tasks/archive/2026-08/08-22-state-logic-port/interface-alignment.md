# 接口对齐记录（08-22-state-logic-port 批次 7）

> 消费方：`08-22-shell-port`（§1、§3）、`08-22-views-claude`（§2）、`08-22-views-profiles-config`（§3）。
> 所有 API 名均经源码 grep 复核（2026-08-23）；本文只做记录，不改 src。

## 1. shellPreferences ↔ themeBootstrap / fontPreferences 接口对齐

源：`ccr-ui/src/shell/stores/shellPreferences.ts`、`ccr-ui/src/utils/themeBootstrap.ts`、`ccr-ui/src/utils/fontPreferences.ts`。

### 1.1 localStorage 键清单（逐字节契约）

| 键 | 写入/读取方 | 说明 |
| --- | --- | --- |
| `ccr-theme` | themeBootstrap `readStoredTheme` / `persistTheme` | `'light' \| 'dark' \| 'system'` |
| `ccr-flavor` | themeBootstrap `readStoredFlavor` / `persistFlavor` / `migratePersistedFlavor` | 经 `FLAVOR_MIGRATION` 迁移表 + 白名单回退 |
| `ccr-accent` | themeBootstrap `readStoredAccent` / `persistAccent` / `migratePersistedAccent` | 经 `ACCENT_MIGRATION`，值域仅 `'clay'` |
| `ccr-font-ui` | fontPreferences `readStoredUiFont` / `persistUiFont` | 净化后字体族名（`sanitizeFontFamily`，≤64 字符） |
| `ccr-font-code` | fontPreferences `readStoredCodeFont` / `persistCodeFont` | 同上 |
| `ccr-sidebar-width` | shellPreferences `readStoredSidebarWidth` / 私有 `persistSidebarWidth` | 数字字符串，clamp 到 [200, 480] |

shellPreferences 不用 zustand/persist 中间件（批次 4 偏差 1）：持久化全部经上述工具逐 key 写入。`ccr-ui/index.html` 首帧 IIFE 内联读取同一组键（`ccr-theme` / `ccr-flavor` / `ccr-accent` / `ccr-font-ui` / `ccr-font-code`）并内联同一份迁移逻辑；键布局由 `theme-bootstrap.smoke.test.ts` 行为锁锁定。**任何一侧改键名或键值格式都会破坏首帧无闪契约——shell-port 不得新增/改名这些键。**

### 1.2 从 bootstrap 值水合的 store 字段

store 创建时（`useShellPreferencesStore` 初值）直接读存储：

| store 字段 | 来源 |
| --- | --- |
| `theme` | `readStoredTheme()` |
| `effectiveTheme` | `resolveThemeMode(readStoredTheme())` |
| `flavor` | `readStoredFlavor()` |
| `resolvedFlavor` | `resolveFlavorMode(resolveThemeMode(readStoredTheme()), readStoredFlavor())` |
| `accent` | `readStoredAccent()` |
| `uiFont` | `readStoredUiFont()` |
| `codeFont` | `readStoredCodeFont()` |
| `sidebarWidth` | `readStoredSidebarWidth()` |

不来自 localStorage 的字段：`locale` / `localeLabel`（i18n 层 `readStoredLocale`）、`confirmBeforeExit` / `closeToTray` / `openPanelOnTrayClick`（后端 runtime 偏好，初值硬编码 true/false/true，经 `hydrateRuntimePreferences` 覆盖）、`perfTelemetryEnabled`（`isPerfTelemetryEnabled()`）、`runtimeHydrated`（false）。

### 1.3 shell-port 启动接线要求

1. **主题/字体首帧应用**：import `@/shell/stores/shellPreferences` 即完成——模块尾执行
   `useShellPreferencesStore.getState().initializeTheme()`（与原 Pinia 创建时机等价）。`initializeTheme()`
   依次执行 `migratePersistedFlavor()` → `migratePersistedAccent()` → `applyThemeToDocument(theme, flavor)` →
   `applyAccentToDocument(accent)` → `applyFontsToDocument(ui, code)` 并写回 state。**shell-port 无需再调
   `themeBootstrap.applyInitialTheme` / `fontPreferences.applyInitialFonts`**（两者为独立原语，src 内无消费方）。
2. **OS 偏好跟随**：`THEME_RESOLUTION_CHANGE_EVENT`（`'ccr-theme-resolution-change'`）监听在模块级注册一次，
   system 模式下自动同步 `effectiveTheme` / `resolvedFlavor`。shell-port 无需自行订阅。
3. **runtime 偏好水合**：外壳挂载后调用一次 `useShellPreferencesStore.getState().hydrateRuntimePreferences(): Promise<void>`
   （幂等：`runtimeHydrated` 为 true 时直接返回；失败时落默认值并置位）。
4. **runtime 偏好 flush**：导出 `syncRuntimePreferences(patch?: Partial<DesktopShellPreferences>): Promise<DesktopShellPreferences | null>`，
   读当前 state 合并 patch 后经 `shellSetPreferences` 写后端并回写 state；失败返回 null。供退出前/托盘路径兜底 flush。
5. **写路径约定**：UI 改偏好一律走 store action（`setTheme` / `toggleTheme` / `setFlavor` / `setAccent` /
   `setUiFont` / `setCodeFont` / `updateSidebarWidth(nextWidth, persist?)` / `commitSidebarWidth` / `resetLayout` /
   `setLocalePreference` / `setConfirmBeforeExit` / `setCloseToTray` / `setOpenPanelOnTrayClick` / `setPerfTelemetryPreference`），
   action 内部已含 persist + DOM 应用；视图层不得绕过 store 直接调 `persistXxx` / `applyXxxToDocument`。

### 1.4 对齐结论

未发现接口错位：store 引用的工具函数名与两个 utils 的导出逐一对应；键名两侧一致且受 smoke 测试锁定。

## 2. claudeObserver Query key 与事件失效范围

源：`ccr-ui/src/features/claude/queries.ts`、`ccr-ui/src/shell/eventBridge.ts`、`event-adjudication.md` §1。
数据源 wrapper：`ccr-ui/src/api/generated/claudeObserver.ts` 的 `claudeObserver` 对象（9 个方法，全部经既有 IPC command）。

### 2.1 key 工厂全集（`claudeObserverKeys`）

根前缀 `['claude-observer']`：

| 工厂 | 签名 | key 形态 |
| --- | --- | --- |
| `all` | — | `['claude-observer']` |
| `insight` | `(range?: 'today' \| 'month' \| 'all')` | `[all, 'insight', range ?? null]` |
| `dailyTrend` | `(days?: number)` | `[all, 'daily-trend', days ?? null]` |
| `costBreakdown` | `(dim: 'project' \| 'model', days?: number, limit?: number)` | `[all, 'cost-breakdown', dim, days ?? null, limit ?? null]` |
| `cacheStats` | `()` | `[all, 'cache-stats']` |
| `topSessions` | `(limit?: number, by?: 'cost' \| 'calls')` | `[all, 'top-sessions', limit ?? null, by ?? null]` |
| `toolHeatmap` | `(days?: number)` | `[all, 'tool-heatmap', days ?? null]` |
| `topTools` | `(days?: number, limit?: number)` | `[all, 'top-tools', days ?? null, limit ?? null]` |
| `subscription` | `()` | `[all, 'subscription']` |

可选参数缺省时以 `null` 占位入 key——同参数缺省形态必须一致，否则缓存分裂；views-claude 传参请走 hook 封装。

### 2.2 事件失效范围

桥接层（`shell/eventBridge.ts` `useTauriEventBridge`）中唯一相关事件：

- `claude_observer:updated` → `queryClient.invalidateQueries({ queryKey: claudeObserverKeys.all })`
  ——单事件整体失效全部切片（含 subscription），等价于原 store「单事件驱动全切片 refetch」语义。

无其他事件触碰该前缀；views-claude 自身无需再订阅 `claude_observer:updated`。staleTime 兜底为
`OBSERVER_STALE_TIME = 30_000`（30s），主新鲜度来源是事件失效。

### 2.3 订阅写入行为

`useSetClaudeObserverSubscription` mutation：入参 `{ mode: string; plan: string; monthlyUsd: number }`，
透传 `claudeObserver.subscriptionSet(mode, plan, monthlyUsd)`；onSuccess 仅失效
`claudeObserverKeys.subscription()` 切片（其余切片不受写操作影响，靠下一次 `claude_observer:updated` 收敛）。

### 2.4 views-claude 可直接消费的全部导出 hook

| Hook | 参数 | 类型 |
| --- | --- | --- |
| `useClaudeObserverInsight` | `(range?: 'today' \| 'month' \| 'all')` | useQuery |
| `useClaudeObserverDailyTrend` | `(days?: number)` | useQuery |
| `useClaudeObserverCostBreakdown` | `(dim: 'project' \| 'model', days?: number, limit?: number)` | useQuery |
| `useClaudeObserverCacheStats` | `()` | useQuery |
| `useClaudeObserverTopSessions` | `(limit?: number, by?: 'cost' \| 'calls')` | useQuery |
| `useClaudeObserverToolHeatmap` | `(days?: number)` | useQuery |
| `useClaudeObserverTopTools` | `(days?: number, limit?: number)` | useQuery |
| `useClaudeObserverSubscription` | `()` | useQuery |
| `useSetClaudeObserverSubscription` | `()`（mutation 入参见 §2.3） | useMutation |

另有常量 `claudeObserverKeys` 导出（自定义 select/精确失效时可引用）。原 store 无残余 UI 态（批次 4 判定修正），
订阅/面板 UI 态由 views-claude 自行以组件态承载。

## 3. configs 表单草稿键约定

源：`ccr-ui/src/features/configs/stores.ts`（Zustand，批次 4）。

### 3.1 存储形态

- store：`useConfigsViewStore`，字段 `formDrafts: Record<string, unknown>`——**键为配置 id（configName 字符串），
  值为草稿内容**（JSON 字符串或任意可序列化值，具体形态归消费视图决定，store 不约束）。
- 同 store 另承载路由视图态两项：`currentConfig: string | null`（选中配置）与 `searchQuery: string`，
  与草稿共同构成外壳门 AC4 的「configs 缓存路由视图态」六项状态之一组。

### 3.2 set/clear API

| API | 签名 | 行为 |
| --- | --- | --- |
| `setFormDraft` | `(configId: string, draft: unknown) => void` | 按 id 覆盖式写入草稿 |
| `clearFormDraft` | `(configId: string) => void` | 删除对应草稿；id 不存在时保持 state 引用不变（无多余渲染，批次 6 用例锁定） |
| （配套）`setCurrentConfig` / `setSearchQuery` | 见上 | 路由切回时的选中态与搜索词恢复 |

### 3.3 持久化行为

**memory-only**：store 无 persist 中间件、无 localStorage 键——草稿仅在会话内存活，刷新即失。
设计意图是「切换路由后返回时未提交表单可恢复」（design.md §5），非跨会话持久化。views-profiles-config
若需要跨会话草稿须另行决策，不得默认本 store 提供。

### 3.4 外壳门 AC4 的 store 侧验证将覆盖

外壳门对 AC4 的六项状态核验中，configs 组将走 `useConfigsViewStore` 实测：`setFormDraft` 后按 id 可取回、
`clearFormDraft` 后该键消失且缺失 id 时引用不变、`currentConfig` / `searchQuery` 在路由往返后保持。
shell-port 侧只需保证路由容器不卸载该 store 所在模块（模块级单例），无额外接线。

### 3.5 对齐结论

未发现 API 缺口。三个消费方（shell-port AC4、views-profiles-config 批次 2）按上述键约定与 memory-only
语义接入即可；如与 views-profiles-config 的表单实现存在形态分歧（unknown vs JSON string），由其批次 2
在消费侧定形并在自身记录中登记。
