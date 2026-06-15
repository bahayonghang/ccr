# Research: ccr-ui 性能问题深度分析

- **Query**: 深入分析 ccr-ui/（Vue 3 + TypeScript + Tauri 2 + Pinia + Vite 前端）的性能问题：路由懒加载与包体积、轮询与定时器、事件监听器泄漏、大列表渲染、响应式开销、重复请求、watch 滥用、启动路径、图表/重组件
- **Scope**: internal（全量代码审查 + dist 产物实测）
- **Date**: 2026-06-12

---

## 总体结论

ccr-ui 的性能工程基线**显著高于一般前端项目**：路由全量懒加载、九路 vendor 分包、CI 级 bundle 预算门禁（`scripts/check-bundle-budget.mjs`）、统一轮询基建 `usePolledData`（可见性暂停 + in-flight 去重）、启动路径多级延迟（boot locale 子集 / shell 图标子集 / deferred CSS / perfTelemetry 埋点）。**没有发现教科书式的 P0 级泄漏**（setInterval 不清理、listener 不配对等基本均已治理）。

剩余问题集中在四类：

1. **GPU 合成成本**（68 处 `backdrop-filter` + 大尺寸 `blur(88px)` 无限动画光晕）；
2. **一批确认无引用的死代码组件**，其中部分锁住了 `marked` + `highlight.js` 整条依赖链，且自身携带未清理的 rAF 动画反模式；
3. **keep-alive 常驻策略**（9 个大型视图 + 后台事件监听继续工作）；
4. **零 shallowRef** —— 大 JSON 载荷（heatmap 365 天、trends、logs、overview）全部深响应式。

另发现 1 个**功能性失效**（SessionsView 仍走 legacy HTTP fetch，Tauri 运行时必然失败），按用户可感知度列为 P0。

---

## 1. 路由懒加载与包体积

### 1.1 路由懒加载 —— 已全量达标

`ccr-ui/src/router/index.ts`：全部 40+ 路由组件均为 `component: () => import(...)` 动态导入（L39、L56、L70-453），含 `genericPlatformRoutes` 动态生成的平台路由（L33-64）。MainLayout 自身也是懒加载（L75）。App.vue 中 Titlebar / ToastContainer / AnimeBackground / GlobalConfirmDialog 均为 `defineAsyncComponent`（`src/App.vue` L20-38）。**无静态导入视图，达标。**

### 1.2 分包策略

`ccr-ui/vite.config.ts` L33-43 定义 9 路 manualChunks：

| chunk | 内容 | dist 实测 (raw) |
|---|---|---|
| `charts-vendor` | apexcharts + vue3-apexcharts | **467.6 KB**（最大单 chunk） |
| `vue-vendor` | vue / vue-router / pinia | 114.2 KB |
| `i18n-vendor` | vue-i18n | 61.7 KB |
| `search-vendor` | fuse.js | 23.8 KB |
| `markdown-vendor` | marked / dompurify / highlight.js | 21.9 KB |
| `tauri-vendor` / `virtual-vendor` / `term-vendor` / `ui-vendor` | — | 小 |

入口产物：`index.js` 105.4 KB、`index.css` **173.4 KB**（注释标注 gzip 约 26 KB，预算 28 KB，已贴线）。语言包懒加载：`en-US.js` 135.9 KB / `zh-CN.js` 129.5 KB。预算门禁见 `scripts/check-bundle-budget.mjs` L116-139（index ≤110 KB raw、core.css gzip ≤28 KB、UsageDashboardView ≤250 KB/80 KB gzip、shell-icons ≤40 KB）。

### 1.3 重依赖引用方式盘点

| 依赖 | package.json 版本 | 引用方式 | 落点 | 评价 |
|---|---|---|---|---|
| apexcharts / vue3-apexcharts | ^5.3.6 / ^1.10.0 | **全部动态 import** | `UsageDashboardView.vue:260`、`claude-observer/{BehaviorAnalysisTab:168, CostAttributionTab:139, TokenDetailTab:112}`、`platform-usage/PlatformUsageTrendChart.vue:60`（defineAsyncComponent） | 达标，主包零污染 |
| highlight.js | ^11.11.1 | 静态（core + 13 语言，`utils/highlightLanguages.ts:13-25`） | 仅被 `useMarkdownRender.ts` → `MarkdownEditor.vue` 引用，**而 MarkdownEditor 无任何消费者（死代码）** | 见 §10 |
| marked | ^17.0.1 | 静态（`composables/useMarkdownRender.ts:13`） | 同上，死代码链 | 见 §10 |
| dompurify | ^3.3.1 | 静态（`utils/sanitize.ts:1`） | 被 `ansiRenderer`（CommandsView）使用，落 markdown-vendor | 合理 |
| fuse.js | ^7.3.0 | 静态（`composables/useFuzzySearch.ts:15`） | 仅被懒视图（McpManagerView、ProviderTemplateSelector）引用 → search-vendor 按需加载 | 合理 |
| ansi_up | ^6.0.6 | 静态（`utils/ansiRenderer.ts:1`） | CommandsView chunk | 合理 |
| @tanstack/vue-virtual | ^3.13.18 | 静态 | 仅 `components/HistoryList.vue:182` | 合理 |

### 1.4 问题点

- **[P2] CheckinView 巨石 chunk**：`CheckinView-*.js` 100.2 KB + CSS 67.3 KB，为最大路由 chunk（源文件 `views/CheckinView.vue` 49.2 KB）。次大：`UsageDashboardView` 84.9 KB、`CodexAuthView` 75.6 KB（源文件 **133.9 KB** 单 SFC，3283 行）、`CodexProfilesView` 69.4 KB + CSS 42.1 KB。首次进入这些页面解析/编译耗时可感知（WebView2 下 100 KB JS 约 50-100 ms 解析 + 执行）。
- **[P2] 依赖分类错误**：`@types/dompurify`、`@types/marked`、`tailwindcss` 位于 `dependencies` 而非 `devDependencies`（`ccr-ui/package.json`）。不影响产物但污染依赖语义。

---

## 2. 轮询与定时器

### 2.1 轮询清单（全仓库 `setInterval` 仅 1 处源头）

唯一 `setInterval` 在 `composables/usePolledData.ts:109`，所有轮询统一走该基建。其能力：`pauseWhenHidden`（visibilitychange，L123-137）、key 级共享 in-flight Promise（L86-90、L214）、组件内自动 `onBeforeUnmount` 停止（L195-198）。**基建质量高。**

| 轮询点 | 间隔 | 启停控制 | 卸载清理 | 评价 |
|---|---|---|---|---|
| usage store `coreAutoRefresh`（`stores/usage.ts:813-830`） | 30 s（L64） | `immediate:false`，由 UsageDashboardView `onActivated` resume / `onDeactivated`+`onUnmounted` pause（`views/usage/useUsageDashboardState.ts:914-931`） | ✓ | 模范实现 |
| usage store `heatmapAutoRefresh`（L832-845） | 10 min（L65），默认 LAZY 模式下不创建 | 同上 | ✓ | 良好 |
| **`useBackendHealth`（`composables/useBackendHealth.ts:50-69`）** | **30 s，`immediate:true`，模块加载即启动** | **无任何 pause/stop 出口**（仅 hidden 时暂停） | 模块级，永不清理 | **[P2] 见下** |
| homeUsageOverview `retryProbeTimer`（`stores/homeUsageOverview.ts:108-114`） | 20 s 一次性 setTimeout，clearRetryProbe 配对（L58-63） | teardown 清理（L317-329） | ✓ | 良好 |

**[P2] useBackendHealth 永久后台轮询**：`BackendStatusBanner.vue:8` 引用该 composable，Banner 被 MainLayout 无条件渲染（`components/MainLayout.vue:253`），因此模块一旦加载，`healthCheck` invoke 每 30 s 执行一次直到进程退出——即使 Banner 仅在 `status === 'error'` 时可见（Banner L12）。健康检查本身廉价（SQLite ping），但属于"无消费者仍在烧后端"的模式；且 `options.auto === false` 分支（L74）形同虚设，poller 在模块求值时已经启动。

### 2.2 一次性 setTimeout（39 处 / 26 文件）

抽查高风险点均有清理：`GeminiCliView.vue:487-501`（copyResetTimer，onBeforeUnmount clearTimeout）、usage store `filterDebounceTimer`（300 ms 防抖，stopAutoRefresh 时清理，L791-801、L864-867）。`ConverterView.vue:683/733/758/781` 四处 toast 重置 setTimeout 无句柄保存，组件卸载后回调仍执行一次写已卸载 ref——Vue 容忍此写入，**P3 级瑕疵**。

### 2.3 多视图同时轮询同一数据

未发现两个活跃视图同时对同一数据开两路 interval（usage 仅在 Usage 页激活时轮询；Dashboard 走事件驱动的 homeUsageOverview）。但存在**事件扇出双查询**问题，见 §6.2。

---

## 3. 事件监听器（Tauri listen / addEventListener）

### 3.1 Tauri `listen()` —— 25 个调用点，配对情况

| 位置 | 清理 | 备注 |
|---|---|---|
| `App.vue:62`（shell:navigate） | onUnmounted ✓（L71-74） | App 永不卸载，形式正确 |
| usage store import job ×4（`stores/usage.ts:397-418`） | job finished/failed 时 `clearImportJobListeners`（L297-301、L382-389） | ✓ 动态生命周期管理 |
| usage store snapshot listener（L338-356） | 不清理（app 生命周期，幂等 guard L336） | 设计如此，可接受 |
| homeUsageOverview ×8（L88、L167-179、L194-203） | `teardown()` 全量清理（L317-329），DashboardView onBeforeUnmount 调用（`DashboardView.vue:210`） | ✓ 但见 §3.3 keep-alive 警告 |
| `useMonitoringFeed.ts:325-337` | onUnmounted 遍历 unlisten（L355-363） | ✓ |
| `CommandsView.vue:975-977` | onUnmounted（L1009-1013） | ✓ |
| `CodexAuthView.vue:3237-3259`（OAuth） | onBeforeUnmount `cleanupOauthListeners`（L3280-3282），且 listen 本身动态 import | ✓ |
| checkin `checkinJobRuntime.ts:165-176` / `checkinWafRecovery.ts:243-249` | 显式 unlisteners 数组 + 清理（L51、L231） | ✓ |
| `useCodexTrayPanel.ts:151-154` | onUnmounted（L157-160） | ✓ |
| `claudeObserver.ts:199` | `disposeEventListener`（L210-220） | ✓ |
| `LlmusageInstallDialog.vue:546` | L524/544 unlisten | ✓ |

**结论：无裸泄漏。**

### 3.2 DOM addEventListener —— 19 处注册 / 24 处移除，全部配对

抽查：`useMainLayoutShell.ts:87-88/116` ↔ L77-78/123-126（mousemove/mouseup/keydown + mediaQuery）；`ClaudeCodeProfilesView.vue:1029` ↔ 1032；`AccountActionsMenu.vue:154-156` ↔ 160-162（含 capture scroll）；`Titlebar.vue:311` ↔ 333。两处 app 生命周期监听不清理但合理：`stores/shellPreferences.ts:130`（主题事件，store 单例）、`utils/themeBootstrap.ts:296`（有 remove 路径）。

**[P3] `usePageTransition.ts:34`**：`router.beforeEach` 注册后从不调用返回的解注册函数。当前唯一消费者 MainLayout 是常驻单例所以无害；若未来在可卸载组件中复用，每次挂载都会累积一个全局守卫（闭包持有组件 ref）。

### 3.3 [P1] keep-alive 缓存视图的"后台监听"

`router/index.ts` 中 `meta.cache: true` 共 **9 个视图**（DashboardView、CodexView、CommandsView、ConfigsView、UsageDashboardView、McpManagerView、CodexMcpView、CodexProfilesView、CodexAuthView），`MainLayout.vue:260-263` 的 `<keep-alive :max="10">` 几乎不会发生 LRU 驱逐。后果：

1. **监听器生命周期 = 缓存期而非可见期**。CommandsView 的 3 个 job 监听（L975-977）、DashboardView 的 useMonitoringFeed 监听 + logger.subscribe（`useMonitoringFeed.ts:345-353`）在视图切走后继续接收并处理每一条事件（mergeEntries 含数组拷贝 + 二分插入，L262-287）。Dashboard 配置 maxEntries=24 成本低，但模式本身随事件频率线性放大。
2. **DOM 与 Pinia 大对象常驻内存**：9 个视图含 CodexAuthView（75.6 KB chunk、大表单树）、UsageDashboardView（含 ApexCharts 实例，deactivated 时不销毁）。WebView 进程常驻内存预计增加 50-150 MB 量级（取决于访问过的页面数）。
3. teardown 语义被架空：DashboardView `onBeforeUnmount` 的 `usageOverviewStore.teardown()`（L210）在 keep-alive 下基本永不执行。

---

## 4. 大列表渲染

| 列表 | 规模 | 防护 | 评价 |
|---|---|---|---|
| ConfigsView 历史 `HistoryList.vue:195-198` | 不限 | **唯一虚拟滚动**（@tanstack/vue-virtual，estimateSize 160） | ✓ |
| usage logs（usage store fetchLogs） | 50/页 cursor 分页（`logsPageSize` L142） | 分页 ✓ | ✓ |
| CodexSessionsView 会话列表 | `SESSION_LIMIT = 160`（L443） | 硬上限，无虚拟化 | 可接受，贴线 |
| CodexSessionsView 详情 messages | `DETAIL_LIMIT = 120`（L444），`<pre>` 全文渲染，key=`timestamp-index`（L387-388） | 上限保护 | 可接受 |
| **CommandsView 终端输出 `ledgerLines`（L851-866）** | **无前端行数上限**，逐行 `v-html`（L530-538） | 仅 ansi LRU cache（4000 行，`ansiRenderer.ts:13-34`） | **[P1] 见下** |
| MonitoringView feed（useMonitoringFeed） | maxEntries 500（L45） | 上限 + 去重 Set | 每事件 O(n) 拷贝（L224-248），P2 |
| SessionsView | limit=50（L354） | — | 页面本身失效，见 §11 |

**[P1] CommandsView 输出渲染双重放大**：

1. 后端每个 `commands:job-progress` 事件携带**全量快照**（`currentSnapshot.value = event.payload`，L968-971），stdout/stderr 数组随输出增长，IPC 序列化成本 O(n) /事件、累计 O(n²)。
2. `ledgerLines` computed 每次快照更新重建全部行对象数组（含模板字符串 key `${channel}-${index}-${text}`，L531——key 含整行文本，长行会生成超长 key 字符串）。ansi 渲染有 LRU 缓存兜底，但数组重建 + diff 仍是 O(n)/事件。
3. 无 `maxLines` 截断（对比 `useStream.ts:54` 的 maxLines=2000 设计）。一条产出数万行的命令（如 `ccr doctor -v` 或长 sync）会让 Commands 页随执行越来越卡。

**index-as-key 普查**（50 处）：绝大多数为有界小列表（warnings、rankings、breadcrumb、月标签等），风险可忽略。值得改的仅 `CodexSessionsView.vue:387`（消息列表，但有 timestamp 复合）与已属死代码的 `AnimatedCounter.vue:4-5`（每字符一个 index-key span，动画期间每帧 diff）。

---

## 5. 响应式开销

**全仓库 0 处 `shallowRef` / `markRaw` / `shallowReactive`**（rg 验证无命中）。这意味着以下大 JSON 载荷全部被递归 Proxy 化：

- `stores/usage.ts:115-124`：`heatmap`（365 天 × 单元格，`HEATMAP_DAYS = 365` L67）、`trends`（每日趋势数组）、`logs`（50 条/页记录）、`modelStats` / `projectStats` / `snapshot` / `usageCapabilities`。这些数据**只读展示、整体替换**，是 `shallowRef` 的标准适用场景。
- `stores/homeUsageOverview.ts:33`：`overview`（完整聚合响应）。
- `useMonitoringFeed.ts:255`：`logs` 数组 500 条，每条含 fields 任意对象。
- `CommandsView.vue` `currentSnapshot`（含全量输出行数组）。

实际代价：Vue 3.4+ 的 Proxy 是惰性创建（访问到才包装），渲染会访问全部展示字段，所以每次 30 s 自动刷新替换 payload 时都重新创建整棵 Proxy 树 + 触发依赖重算。单次约毫秒级，属**累积型 P2**，不是卡顿主因，但改造成本极低。

**computed 重计算**：`ActivityHeatmap.vue:71-118`（371 个 cell 对象 + 371 次 `toLocaleDateString`）——该组件已是死代码；`useFuzzySearch.ts:41-50` 每键击全量 Fuse.search 且 items 变化即重建索引——现有消费者数据量小（MCP 分组、模板列表），P3。`CodexAuthView.vue:2385-2394` filteredAccounts 每键击过滤排序，账号数通常 <50，P3。

**watch 滥用**：全仓库 **0 处 `deep: true`**；未发现 watch 链式触发请求风暴（`useUsageDashboardState.ts:281-310` 的 watch → loadLogs/hydrate 均有 TTL/去重兜底）。**此项整体达标。**

---

## 6. 重复请求

### 6.1 去重基建（达标）

- usage store：`dashboardCache` TTL 30 s + `inFlightKey`/`inFlightPromise` 去重（`stores/usage.ts:459-480、577-584`）+ `requestSerial` 防过期响应回写（L482、518）。
- `useCodexDashboard.ts:55-72`：模块级共享缓存（TTL 30 s/60 s）+ inflight Promise，`CodexView.vue:466-471` onMounted+onActivated 双调用被 TTL 吸收。
- `usePolledData` key 级共享 in-flight（`usage:auto-refresh:core` 等）。
- `homeUsageOverview.ts:44、280-284`：overviewCache TTL 30 s。
- 搜索/筛选：usage 筛选 300 ms 防抖（L788-802）。

### 6.2 [P2] `usage:snapshot-updated` 事件双 store 扇出

同一事件被两个 store 各自订阅并各自发起后端聚合查询：

- `stores/usage.ts:338-356` → `fetchAll`（getUsageDashboardV2 聚合）
- `stores/homeUsageOverview.ts:85-98` → `loadOverview`（getHomeUsageOverviewV2 聚合）

当 Dashboard（keep-alive 常驻）与 Usage 页都已初始化后，每次后端 snapshot 更新（导入进行中每 2 s 节流刷新一次，`IMPORT_PROGRESS_REFRESH_INTERVAL_MS` L68）触发 **2 个独立 SQLite 聚合查询**。导入大量历史数据期间 CPU 占用翻倍。两者数据高度重叠（home overview ⊂ dashboard）。

### 6.3 [P3] 未防抖的本地搜索输入

`CodexAuthView`、`CodexSessionsView`、`useFuzzySearch` 消费者的搜索均为同步 computed 过滤，数据量小（<200 条）无需网络请求，当前无感知；仅在列表规模增长后需要 debounce。

---

## 7. 启动路径（main.ts / App.vue / MainLayout.vue）

`src/main.ts`（271 行）的启动编排是**模范级**的：

- 同步关键路径仅：`initPerfTelemetry` → `applyInitialTheme`（防闪白）→ `registerShellIcons`（24.2 KB 子集）→ createApp + router ready（带 10 s 超时降级，L210）→ mount。
- 首帧后（`scheduleAfterPaint`，L128-170）才做：deferred-interactive CSS、完整 locale 水合、87.5 KB 完整图标子集、字体 CSS、decorations CSS、telemetry flush——全部 `scheduleWhenIdle` 分级。
- i18n 双层：`bootMessages.ts`（52.8 KB 源码，首屏导航/Dashboard/Settings keys）进入口包，完整语言包（130-136 KB）按 locale 懒加载（`i18n/index.ts:11-14`）；直接深链到二级页面时先水合再 mount 防 key 闪烁（main.ts L228-245）。
- DashboardView 自身再延迟：`scheduleWhenIdle` 后才发 system_info / cli_versions / usage overview 三个 invoke（`DashboardView.vue:192-205`）。

**残余改进点（均 P2/P3）**：

- `index.css` 173.4 KB raw 为同步阻塞资源，gzip ~26 KB 已贴 28 KB 预算上限；Tailwind utilities + 全局 tokens 继续增长会破线。
- `bootMessages.ts` 52.8 KB 包含**双语**启动子集（`bootLocaleMessages` 同时含 zh-CN 与 en-US），理论上可按 `readStoredLocale()` 只内联一种、另一种走懒加载，可省 ~20 KB 入口体积。

---

## 8. 图表 / 重组件实例管理

- **ApexCharts 加载**：所有图表组件动态 import（§1.3），且 UsageDashboard 有「先内容后图表」两级 idle 门控（`useUsageDashboardState.ts:205-224`：scheduleAfterPaint → scheduleWhenIdle → trend → distribution），models tab 图表首次切换才挂载（L286-297）。
- **销毁**：vue3-apexcharts 组件内部在 beforeUnmount 调用 `chart.destroy()`（上游实现），项目未直接 new ApexCharts，无手工 dispose 缺口。
- **主题切换**：MutationObserver 监听 `data-theme`（L886-892），`onUnmounted` disconnect（L930）✓。
- **[P2] keep-alive 例外**：UsageDashboardView 被缓存，deactivated 时 ApexCharts 实例（SVG 节点数千个）不销毁，常驻内存；ApexCharts 自带的 window resize 监听也保持活跃。
- **[P2] apexcharts 体积**：467.6 KB raw（约 120 KB gzip）只为折线/柱状/donut 服务，是产物中最大依赖。uPlot（~40 KB）或 chart.js（~70 KB）可覆盖现有图表类型，但迁移成本高，属长期项。

### 8.1 [P1] 装饰层 GPU 成本（额外发现）

- 全仓库 **68 处 `backdrop-filter`**（40 个文件；`styles/utilities.css` 5 处、MainLayout 5 处、Button 5 处等）。桌面 WebView 中每个 backdrop-filter 表面在其下层内容变化/滚动时都要重采样模糊，玻璃卡片列表页（Sessions、Plugins、ProviderHealth 等每卡一个 blur）滚动帧成本显著。
- `components/common/AnimatedBackground.vue:118-122`：光晕层 `filter: blur(88px)` + `ambient-drift 20s ease-in-out infinite` 动画（L192-203 同时动 `transform: scale` 与 `opacity`），主光晕 34vw×34vw、次光晕 30vw×30vw（L124-139）。**scale 动画会迫使已栅格化的模糊纹理反复重采样**，在 ClaudeCodeView（L3）、ConfigsView（L3）、OpenCodePageShell（L3）常驻运行。已有缓解：`useAnimationVisibility`（IntersectionObserver + visibilitychange 暂停，`--animation-state`）与 reduced-motion 降级（L205-209），但页面可见期间 GPU 持续做无限循环合成。
- `components/common/AnimeBackground.vue:38-41`（App 全局背景）：两个 38-44vw 光晕 `blur(88px)`，**静态**（无动画），一次性栅格化后成本可接受；真正的成本在其上层所有 backdrop-filter 表面需要穿透采样它。

---

## 9. 定时动画组件的 rAF 反模式（现均为死代码，见 §10）

- `components/usage/AnimatedCounter.vue:68-105`：rAF 循环驱动 `displayValue` ref，每帧触发 `displayChars` computed 重算 + N 个 index-key span 重渲染（60 fps × ~1 s）；**无 onUnmounted 取消**（仅在重启动画时 cancel，L69-71），卸载后循环继续跑完。
- `components/usage/RingProgress.vue:150-168`：rAF 循环**完全没有 cancelAnimationFrame**，props.value 快速变化时多个循环并发互相覆盖；外加 `animate-ping` 无限动画 + `blur-xl` 发光层 + `drop-shadow` filter（L8、L93、L201）。

这两个组件当前无消费者，列为"删除而非修复"。

---

## 10. 死代码清单（影响 dev/typecheck/认知成本，部分锁住依赖）

以下组件经全仓引用搜索确认**无任何消费者**：

| 文件 | 大小 | 连带影响 |
|---|---|---|
| `components/MarkdownEditor.vue` | 6.7 KB | **唯一引用 `useMarkdownRender.ts` → `highlightLanguages.ts` 的入口**，删除后 `marked@17` + `highlight.js@11`（core+13 语言）整条链可从依赖图移除 |
| `components/usage/StatCard.vue` / `RingProgress.vue` / `AnimatedCounter.vue` | 3 件套 | 含 §9 的 rAF 反模式 |
| `components/UsageStatsDashboard.vue` + `UsageStatsChart.vue` | 17.4 + 11.2 KB | 互引孤岛 |
| `components/TokenUsageChart.vue` | 22.1 KB | 无引用 |
| `components/ActivityHeatmap.vue`（+ `activity/*` 4 个子组件仅被其引用） | 5.1 KB+ | 371 cell computed |
| `components/Layout.vue` / `Navbar.vue` / `StatusHeader.vue` / `Table.vue` | 共 ~26 KB | 旧布局体系残留 |

注：Vite 构建对未引用 SFC 会 tree-shake，**运行时产物不受影响**，但这些文件参与 `vue-tsc` / ESLint / Stylelint 全量检查，拖慢 `just frontend-check-quick`，并让依赖清单虚胖（marked/highlight.js 仍需安装、审计、升级）。

---

## 11. [P0] SessionsView 仍走 legacy HTTP API（功能性失效）

`views/SessionsView.vue:343-378`：`fetch('/api/sessions')`、`fetch('/api/sessions/stats')`、`fetch('/api/sessions/reindex')`。本项目已迁移到 Tauri IPC（CLAUDE.md：「事件驱动…前端所有 API 调用通过 invoke()」），桌面运行时没有 `/api` HTTP 服务，**该页面 100% 加载失败**（用户点击侧边栏 Sessions → 永远显示「加载失败」）。这是用户可直接感知的缺陷；从性能审查角度它也意味着该页从未经受真实数据量（会话数可达数千）的渲染检验——修复时需同步考虑分页/虚拟化。

---

## 严重程度汇总

### P0（用户可感知失效/卡顿）
1. **SessionsView legacy fetch 全盘失败**（`views/SessionsView.vue:343-378`）——严格说是功能缺陷而非性能，但用户感知最强，且修复方案影响性能设计。

### P1（特定场景明显卡顿/资源浪费）
2. **CommandsView 长输出 O(n²) 路径**：全量快照 IPC/事件 + ledgerLines 全量重建 + 无行数上限（`CommandsView.vue:851-866、968-977`）。
3. **AnimatedBackground 无限 blur(88px)+scale 动画**（`components/common/AnimatedBackground.vue:118-203`）+ 68 处 backdrop-filter 叠加的持续 GPU 合成成本——低端核显/省电模式下整页滚动掉帧的首要嫌疑。
4. **keep-alive 9 视图常驻**：内存常驻 + 后台事件处理 + ApexCharts/表单 DOM 不释放（`MainLayout.vue:260-263`、`router/index.ts` cache 元数据）。

### P2（累积型/规模放大型）
5. usage/homeUsageOverview 双 store 对 `usage:snapshot-updated` 各自重查询（§6.2）。
6. 全仓库零 shallowRef，大 payload 深响应式（§5）。
7. `useBackendHealth` 模块级永久 30 s 轮询（§2.1）。
8. CheckinView/CodexAuthView 等巨石 chunk（§1.4）；CodexAuthView 134 KB 单 SFC。
9. apexcharts 467 KB vendor（懒加载已兜底，体积本身仍是首次进 usage 页的下载/解析成本）。
10. `index.css` gzip 贴预算线；bootMessages 双语内联。
11. useMonitoringFeed 每事件 O(n) 数组拷贝（500 上限内可控）。

### P3（卫生项）
12. `usePageTransition` 全局守卫不解注册；ConverterView 裸 setTimeout；本地搜索无防抖；index-as-key 小列表；`@types/*`、tailwindcss 依赖分类。

---

## 优化建议（按 收益/成本 排序）

1. **删除死代码组件群**（§10，成本：半天；收益：移除 marked+highlight.js 依赖链、消灭 rAF 反模式、加速 type-check/lint）。纯删除，零回归风险，建议第一批落地。
2. **修复或下线 SessionsView**（成本：1 天内——已有 `get_session_*` 类 invoke 可对接，或路由重定向到 /usage；收益：消除 P0 用户可见故障）。
3. **CommandsView 输出治理**（成本：前端 0.5 天 + 后端 delta 事件 1 天）：前端先加 `MAX_LEDGER_LINES`（如 2000，与 useStream 对齐）截断 + key 去掉整行文本；后端将 job-progress 改为增量行（参考 checkin 的 `checkin:job-delta` 已有先例，`checkinJobRuntime.ts:165`）。
4. **AnimatedBackground 动画降级**（成本：小时级）：移除 halo 的 `scale` 变换（保留 opacity 呼吸即可，blur 纹理可被合成器缓存）、或将 blur 半径降到 48px、或干脆与 AnimeBackground 一样静态化。同主题下视觉差异极小。
5. **大 payload 改 shallowRef**（成本：小时级，定点替换 + 确认整体替换语义）：`stores/usage.ts` 的 heatmap/trends/logs/modelStats/projectStats/snapshot、`homeUsageOverview.overview`、`CommandsView.currentSnapshot`。
6. **useBackendHealth 改按需**（成本：小时级）：去掉模块级自启动，由 Banner `onMounted` resume / `onUnmounted` pause；或失败后才进入密集轮询、健康时退避到 5 min。
7. **收敛 snapshot-updated 双查询**（成本：0.5-1 天）：让 home overview 从 usage store 的 dashboard payload 投影派生，或两 store 共享同一份聚合请求（统一走 `usePolledData` key）。
8. **keep-alive 白名单瘦身**（成本：小时级，需产品取舍）：仅保留切换频率最高的 3-4 个（dashboard/usage/commands），CodexAuthView/CodexProfilesView 这类表单页改为不缓存；并为缓存视图在 `onDeactivated` 暂停事件消费（如 CommandsView 的 job 监听）。
9. **巨石视图拆分**（成本：天级/视图）：CheckinView、CodexAuthView 按 tab/区块拆 defineAsyncComponent 子 chunk；同时把 CodexAuthView 拆出 composables 改善可维护性。
10. **图表库替换评估**（成本：周级，收益：-350 KB usage 页首次加载）：列为长期项，仅当 usage 页首开时间成为明确痛点时执行。

## Caveats / Not Found

- 未实际运行 app 做火焰图/内存剖析，所有结论基于静态代码证据 + dist 产物实测；GPU 合成成本（P1 #3）的量级需在低端设备上用 DevTools Performance 面板验证。
- `markdown-vendor` 21.9 KB 的具体成分（dompurify 之外是否含 marked 残片）未逐字节核对。
- Rust 侧（src-tauri）命令实现的查询成本未纳入本次范围，§6.2 的"聚合查询翻倍"基于命令名推断。
- Storybook 相关文件（`*.stories.*`）未纳入死代码判定。
