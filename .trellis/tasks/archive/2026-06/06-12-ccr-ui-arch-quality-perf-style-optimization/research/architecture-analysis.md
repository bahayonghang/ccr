# Research: ccr-ui 前端代码架构分析

- **Query**: 深入分析 ccr-ui/（Vue 3 + TypeScript + Tauri 2 + Pinia 前端）的代码架构问题：巨型视图、跨平台重复、API 边界、状态管理、composables 复用、路由懒加载、类型组织、main.ts
- **Scope**: internal
- **Date**: 2026-06-12

---

## 总览数据

| 指标                          | 数值                                                 |
| ----------------------------- | ---------------------------------------------------- |
| `src/` 总代码量（.ts + .vue） | 130,585 行                                           |
| 视图文件数 / 总行数           | 64 个 `.vue` / 46,701 行                             |
| 组件文件数                    | 145 个 `.vue`                                        |
| composables 数量              | 26 个                                                |
| Pinia store 数量              | 10 个 / 2,394 行                                     |
| >25KB 的视图/组件             | 28 个（见下表）                                      |
| `api/tauri.ts`                | 915 行 / 30.8KB / 33 个直连 `invoke()`（冻结白名单） |
| i18n 单语言包                 | en-US.ts 172.7KB、zh-CN.ts 169.3KB                   |

### >25KB 视图/组件清单（按体积降序）

| 文件                                                                    | 字节    | 行数  |
| ----------------------------------------------------------------------- | ------- | ----- |
| `ccr-ui/src/views/CodexAuthView.vue`                                    | 137,164 | 3,937 |
| `ccr-ui/src/views/CommandsView.vue`                                     | 54,554  | 1,742 |
| `ccr-ui/src/views/ClaudeCodeProfilesView.vue`                           | 53,465  | 1,638 |
| `ccr-ui/src/views/CheckinView.vue`                                      | 50,381  | 1,738 |
| `ccr-ui/src/views/ClaudeCodeSettingsView.vue`                           | 45,190  | 1,359 |
| `ccr-ui/src/views/CodexMcpView.vue`                                     | 44,717  | 1,339 |
| `ccr-ui/src/components/codex/CodexProfileEditorModal.vue`               | 39,516  | 1,014 |
| `ccr-ui/src/views/AppSettingsView.vue`                                  | 38,825  | 1,141 |
| `ccr-ui/src/views/codex/CodexAgentsView.vue`                            | 38,144  | 1,131 |
| `ccr-ui/src/components/provider-templates/ProviderTemplateSelector.vue` | 36,587  | —     |
| `ccr-ui/src/views/checkin/components/AccountFormModal.vue`              | 35,778  | —     |
| `ccr-ui/src/views/HooksView.vue`                                        | 35,404  | 909   |
| `ccr-ui/src/views/ConverterView.vue`                                    | 34,612  | 1,101 |
| `ccr-ui/src/views/CodexProfilesView.vue`                                | 32,945  | 1,023 |
| `ccr-ui/src/views/SyncView.vue`                                         | 32,195  | 979   |
| `ccr-ui/src/views/CodexSettingsView.vue`                                | 31,591  | 966   |
| `ccr-ui/src/views/checkin/tabs/CheckinProvidersTab.vue`                 | 30,554  | —     |
| `ccr-ui/src/views/generic/AgentsView.vue`                               | 30,278  | 718   |
| `ccr-ui/src/views/checkin/tabs/CheckinRecordsTab.vue`                   | 28,453  | —     |
| `ccr-ui/src/views/GeminiCliView.vue`                                    | 28,247  | 995   |
| `ccr-ui/src/views/PricingView.vue`                                      | 28,180  | —     |
| `ccr-ui/src/views/ClaudeCodeView.vue`                                   | 27,921  | 927   |
| `ccr-ui/src/views/MonitoringView.vue`                                   | 27,711  | —     |
| `ccr-ui/src/views/CodexSessionsView.vue`                                | 27,390  | —     |
| `ccr-ui/src/views/checkin/CheckinAccountDashboardView.vue`              | 26,990  | —     |
| `ccr-ui/src/views/checkin/components/OAuthWizardModal.vue`              | 26,530  | —     |
| `ccr-ui/src/views/OpenCodeView.vue`                                     | 26,122  | 811   |
| `ccr-ui/src/views/CodexView.vue`                                        | 25,938  | 934   |

另有 TS 大文件：`ccr-ui/src/views/usage/useUsageDashboardState.ts`（998 行）、`ccr-ui/src/stores/usage.ts`（928 行）、`ccr-ui/src/api/tauri.ts`（915 行）、`ccr-ui/src/composables/useCodexDashboard.ts`（20.9KB）、`ccr-ui/src/views/dashboard/dashboardPresentation.ts`（20.3KB）。

---

## 1. 巨型视图文件 【P0】

### 1.1 CodexAuthView.vue —— 单文件 3,937 行的「god view」（P0）

`ccr-ui/src/views/CodexAuthView.vue` 结构拆解：

| 区块             | 行范围      | 行数   |
| ---------------- | ----------- | ------ |
| `<template>`     | 1–1,928     | ≈1,928 |
| `<script setup>` | 1,930–3,283 | ≈1,353 |
| `<style scoped>` | 3,285–3,937 | ≈652   |

内聚了至少 6 个独立子功能，全部内联在同一文件：

- 双 Tab 管理面（`type ManagerTab = 'accounts' | 'providers'`，`ccr-ui/src/views/CodexAuthView.vue:2010` 附近）；
- 4 种添加账号方式（`type AddMethod = 'oauth' | 'token' | 'api' | 'local'`）含 OAuth 端口占用/释放、回调 URL 提交等完整流程；
- 4 个内联 Modal：添加账号 `BaseModal`（931–1,086）、**一个跨 718 行的 Provider 编辑 Modal（1,088–1,806）**、`ConfirmModal`（1,808）、重命名 `BaseModal`（1,818–1,923）；
- 模型 Provider CRUD（`codexListModelProviders` / `codexSaveModelProvider` / `codexDeleteModelProvider`）；
- 配额查询（`getCodexAllQuotas`）、进程检测（`detectCodexProcess`）、导入（payload/local）。

脚本区有 **136 个顶层 const/function 声明**，从 `@/api` 一次性导入 **17 个 API 函数**（`ccr-ui/src/views/CodexAuthView.vue:1943` 起）。该文件已有部分逻辑外提（`ccr-ui/src/views/codex/codexAuthAccounts.ts`，5.8KB），证明拆分可行，但只完成了纯函数部分；模板、模态框、状态机全部仍在单文件内。

**拆分可行性：高。** Provider 编辑 Modal（718 行模板）、添加账号向导（4 种方式）、账号列表/卡片区、配额面板均为天然组件边界；OAuth 流程状态机可抽 `useCodexOAuthFlow()` composable。

### 1.2 其它巨型视图（P1）

| 文件                                           | template | script | style      | 主要问题                                                                                   |
| ---------------------------------------------- | -------- | ------ | ---------- | ------------------------------------------------------------------------------------------ |
| `views/CommandsView.vue`（1,742 行）           | ≈561     | ≈660   | ≈517       | CCR 命令执行 + 任务轮询 + 历史 + 收藏混在一处                                              |
| `views/ClaudeCodeProfilesView.vue`（1,638 行） | ≈385     | ≈647   | ≈602       | 列表/筛选/编辑器编排 + 602 行 CSS                                                          |
| `views/CheckinView.vue`（1,738 行）            | ≈642     | ≈80    | **≈1,012** | 脚本已抽干净（`useCheckinState` + 4 个 Tab 组件），但 `:726` 起是 1,012 行 scoped CSS 单体 |
| `views/ClaudeCodeSettingsView.vue`（1,359 行） | ≈769     | ≈218   | ≈368       | 769 行表单模板未按 Settings 分区拆分                                                       |
| `views/CodexMcpView.vue`（1,339 行）           | ≈658     | ≈423   | ≈254       | 自带完整 MCP CRUD（见 §2）                                                                 |
| `views/codex/CodexAgentsView.vue`（1,131 行）  | ≈585     | ≈432   | ≈109       | 未复用 `views/generic/AgentsView.vue`                                                      |

注意 CheckinView 的形态说明本仓库**已有成功的拆分先例**（tabs/ + composables/ + components/ 子目录），可直接照搬到其它巨型视图。

---

## 2. 跨平台视图重复 【P0】

### 2.1 Claude/Codex Profiles 组件套件：83–91% 同构的复制粘贴（P0）

`components/codex/profiles/` 与 `components/claude/profiles/` 是两套平行组件库。把平台 token（claude/codex）归一化后做行级 diff：

| Codex 版                            | Claude 版                                 | 相似度                      |
| ----------------------------------- | ----------------------------------------- | --------------------------- |
| `ProfilesContextRail.vue`（946 行） | `ClaudeProfilesContextRail.vue`（886 行） | **89.3%**（818 行逐行相同） |
| `ProfilesToolbar.vue`（335 行）     | `ClaudeProfilesToolbar.vue`（364 行）     | **91.0%**                   |
| `ProfileRow.vue`（234 行）          | `ClaudeProfileListRow.vue`（243 行）      | **87.2%**                   |
| `ProfilesHeader.vue`（254 行）      | `ClaudeProfilesHeader.vue`（210 行）      | **86.2%**                   |
| `ProfilesStatStrip.vue`（165 行）   | `ClaudeProfilesStatStrip.vue`（144 行）   | **83.5%**                   |

仅这 5 对就有 **约 3,800 行平行维护代码**。配套 composables 同样成对存在且已开始漂移：

- `composables/useClaudeProfilesFilter.ts`（155 行）vs `useCodexProfilesFilter.ts`（129 行）：归一化后相似度 **68.3%**；
- `composables/useClaudeProfilesInsights.ts`（232 行）vs `useCodexProfilesInsights.ts`（255 行）：相似度 **70.2%**。

「先复制、再各自演化」是最危险的形态：修 bug 必须改两处，而两处签名已不完全一致（Claude 版 filter 多了 `@/utils/claudeProfiles` 的 provider 归一化逻辑，Codex 版没有同步）。

### 2.2 通用平台基建已存在，但只有 Gemini 一个平台接入（P0）

仓库已经搭好了三层「平台无关」基建：

- `views/generic/PlatformMcpView.vue`（403 行）+ `composables/usePlatformMcp.ts`（11.7KB，内置 per-platform 配置表 `:108`）；
- `views/generic/AgentsView.vue`（718 行）/ `AgentDetailView.vue` + `composables/useAgents.ts`；
- `views/generic/PlatformPluginsView.vue` + `composables/usePlatformPlugins.ts`；
- 路由侧 `config/platformDescriptors.ts` 通过描述符批量注册路由（`router/index.ts:33-64`）。

但 `config/platformDescriptors.ts:22-44` 的 `genericPlatformDescriptors` **只有 `gemini` 一个条目**。结果是同一个「MCP 服务器管理」业务在代码库里有 4 套独立实现：

| 实现                                | 行数  | 复用基建？                                                                            |
| ----------------------------------- | ----- | ------------------------------------------------------------------------------------- |
| `views/CodexMcpView.vue`            | 1,339 | 否，自带 list/add/update/delete + 模态 + 254 行 CSS（`:664` 直接导入 codex 专属 API） |
| `views/OpenCodeMcpView.vue`         | 424   | 否，自带一套（`:312`）                                                                |
| `views/generic/PlatformMcpView.vue` | 403   | 是（仅 gemini 路由使用）                                                              |
| `views/mcp/McpManagerView.vue`      | —     | 走 `useMcpManager.ts`（8.2KB），另有 `useUnifiedMcp.ts`（17.1KB）                     |

Agents 同理：`views/codex/CodexAgentsView.vue`（1,131 行）与 `views/generic/AgentsView.vue`（718 行）功能同构、实现互不复用（归一化相似度仅 8.1%，即完全平行编写）。

### 2.3 SlashCommands：配置驱动模式做了一半（P1）

正确模式已存在：`components/BaseSlashCommands.vue`（646 行）+ `configs/slashCommands.ts`（按平台注入 `PlatformConfig`），`views/SlashCommandsView.vue`（12 行）和 `views/GeminiSlashCommandsView.vue`（14 行）都是薄壳。但是：

- `configs/slashCommands.ts:105` 导出的 `codexConfig` **没有任何消费者**（全仓 grep 0 引用）——为 Codex 准备的配置写完后被遗弃；
- `views/OpenCodeCommandsView.vue`（11.2KB）绕开 BaseSlashCommands 重新实现了一套 CRUD + Modal（`:233-240`）。

### 2.4 平台落地页

`ClaudeCodeView.vue`（927 行）/ `CodexView.vue`（934 行）/ `GeminiCliView.vue`（995 行）/ `OpenCodeView.vue`（811 行）四个平台 dashboard 归一化相似度仅 10–14%，属于「结构同构但视觉各自手写」——重复主要体现在每个文件 450–490 行的 scoped CSS（见 §9.1），而非脚本逻辑。

---

## 3. API 层边界 【P1】

既定契约见 `.trellis/spec/ccr-ui/frontend/api-facade-boundary.md`：新 API 必须进 `src/api/domains/*`，`tauri.ts` 仅作兼容门面、直连 `invoke()` 被白名单冻结。

### 3.1 守卫范围太窄：门面之外的裸 invoke 无人看管（P1）

`ccr-ui/tests/api-facade-boundary.smoke.test.ts:4` 只扫描 `src/api/tauri.ts` 一个文件。已发生穿透：

- `ccr-ui/src/composables/useMonitoringFeed.ts:302` `invoke<unknown[]>('get_monitoring_feed', { query })`、`:311` `invoke('get_recent_events', ...)` —— 而 `api/domains/events.ts:13` 明明已封装 `get_recent_events`，属于绕过 domains 的真实违例；
- `ccr-ui/src/utils/logger.ts:134-135` 动态 `import('@tauri-apps/api/core')` 后裸 invoke `append_frontend_logs`（基础设施层，可豁免但应入白名单显式化）。

### 3.2 「domain-first」命名空间零采用（P1）

`api/index.ts:12-17` 导出了 `configApi` / `codexApi` / `syncApi` / `platformApi` / `usageApi` / `systemApi` 六个命名空间，但在 views/components/stores/composables 中 **使用次数为 0**。实际消费方式：

- 62 个文件仍用扁平命名导入 `from '@/api'`（其中 40 个在 views/）；
- 2 个文件深挖旧门面 `from '@/api/tauri'`：`composables/useCodexTrayPanel.ts:2`、`stores/claudeObserver.ts:4`；
- 3 个文件深挖 domains：`views/CommandsView.vue:579`（`@/api/domains/uiState`）、`views/SkillsMigrationView.vue:151`、`components/usage/LlmusageInstallDialog.vue:299`。

机械层面边界成立（invoke 都收在 api/ 内、tauri.ts 33 个直连命令与白名单一致），但「迁移到 domain 命名空间」的演进实际停滞——门面文件成了永久转发层而非过渡层。

### 3.3 tauri.ts 仍承载类型定义（P2）

`api/tauri.ts` 内定义了 11 个导出接口（`:59` HeatmapData、`:75` ClaudeSettingsData、`:144` SyncStatusResponse 等），其中 `SyncStatusResponse` 与 `types/sync.ts:12` **重名重复定义**（两份独立维护，见 §7）。

---

## 4. 状态管理（Pinia） 【P1/P2】

10 个 store，无相互依赖（grep `from '@/stores` 于 stores/ 内为 0），这点健康。问题：

### 4.1 usage 域状态被切成三块（P1）

- `stores/usage.ts`：**928 行**，混合了数据获取编排、导入任务（`UsageImportJobSnapshot` 状态机，`:250-303`）、诊断、能力探测；
- `views/usage/useUsageDashboardState.ts`：**998 行**的特性 composable（放在 views/ 下而非 composables/），再包一层 store；
- `stores/usageDashboardPayload.ts`（154 行）、`stores/usageImportNormalization.ts`（83 行）、`stores/homeUsageOverview.ts`（346 行）作为外围拆片。

单一业务域 2,500+ 行状态代码分散在两种目录约定下，新人无法判断「该改哪一层」。

### 4.2 视图 UI 偏好做成全局 store 且手写持久化（P2）

`stores/commandsView.ts`（72 行）存的是 CommandsView 的排序/视图模式/折叠状态，每个 action 手动调 `this.persist()` 写 localStorage；`stores/shellPreferences.ts`（254 行）同模式。两者都该用统一的 persist 插件或下沉为组件局部状态 + `useLocalStorage` 式工具。

### 4.3 命名混淆（P2）

`stores/commands.ts`（86 行，缓存命令列表）与 `stores/commandsView.ts`（视图偏好）职责接近、命名难区分。

---

## 5. composables 复用度 【P1】

26 个 composables，但「公共能力」采用率极低：

- `usePolledData.ts`（5.7KB，唯一的 `setInterval` 封装，含可见性暂停）：仅 3 个消费者（`components/StatusHeader.vue`、`useBackendHealth.ts`、`stores/usage.ts`）；
- `useCachedFetch.ts`：仅 1 个消费者（`stores/commands.ts`）；
- 与此同时 **27 个视图**手写 `const loading = ref(...)`、**42 个视图**在 `onMounted` 手写 load 函数 + try/catch + `useUIStore().showError`。`views/CodexMcpView.vue`、`OpenCodeMcpView.vue`、`OpenCodeCommandsView.vue` 等都是同一份「loading/error/load/save/delete + modal 开关」八股，每处 150–400 行。

缺失的明显抽象：`useAsyncAction`（loading + error + toast 包装）、`useModalForm`（开关 + 表单 reset + 提交）、`useCrudResource`（list/add/update/delete 编排）。好的一面：全局只有 `usePolledData.ts` 一处 `setInterval`，轮询纪律是好的；toast 统一走 `stores/ui.ts`（`showToast/showError/confirm`），无散装实现。

另有 3 处局部重复定义 i18n 回退助手 `const tf = ...`（`translateWithFallback` 包装），14 个文件直接用 `translateWithFallback` —— 可统一为一个 `useTf()`。

---

## 6. 路由与懒加载 【P2，整体健康】

`router/index.ts`（537 行）：

- **全部路由组件均为 `() => import(...)` 懒加载**，包括 `MainLayout.vue`（`:75`），无静态导入泄漏；
- keep-alive 以 `meta.cache + cacheKey` 为单一事实源（`collectCachedComponentNames`，`:520-536`），设计良好；
- 问题 1（P2）：**目录结构与路由组织不一致**——Codex 的 7 个子页中 6 个在 `views/` 根（`CodexMcpView`、`CodexAuthView`…），唯独 `views/codex/CodexAgentsView.vue` 在子目录；Checkin 一半在 `views/checkin/`、入口却是根级 `views/CheckinView.vue`；OpenCode 8 个视图全部平铺在根。views/ 根目录已积累 44 个文件；
- 问题 2（P2）：历史 redirect 堆积 11 条（skills 系 6 条 `:248-281`、mcp 系 2 条、gemini-cli 系 4 条 `:371-390`），无下线计划标注。

---

## 7. 类型定义组织 【P1】

### 7.1 同名类型重复定义（P1）

| 类型名               | 定义点 1                                     | 定义点 2                                                                                                  | 风险                   |
| -------------------- | -------------------------------------------- | --------------------------------------------------------------------------------------------------------- | ---------------------- |
| `SyncStatusResponse` | `types/sync.ts:12`                           | `api/tauri.ts:144`                                                                                        | 同名异义，两份独立演化 |
| `UnifiedMcpServer`   | `types/unifiedMcp.ts:10`                     | `composables/usePlatformMcp.ts:27`                                                                        | MCP 统一模型分叉       |
| `TokenStats`         | `types/stats.ts:5`                           | `composables/useMonitoringFeed.ts:21`                                                                     | 监控/统计字段漂移      |
| `SlashCommand`       | `types/mcp.ts:27`                            | `types/platform.ts:1`                                                                                     | types/ 内部自重复      |
| `PlatformConfig`     | `types/platform.ts:30`                       | `composables/usePlatformMcp.ts:55`（另 `configs/slashCommands.ts`、`usePlatformPlugins.ts` 各有局部同名） | 4 处同名不同义         |
| `ImportResult`       | `types/checkin.ts:446`                       | `types/usage.ts:294`                                                                                      | 同名异义               |
| `Platform`           | `types/usage.ts:390`（'claude'\|'codex'\|…） | `api/domains/install.ts:14`（'macos'\|'linux'\|…）                                                        | 同名完全不同语义       |
| `UnknownRecord`      | 5 处局部定义                                 | —                                                                                                         | 应进 types/common      |

### 7.2 与 Rust IPC 结构无防漂移机制（P1）

`src-tauri/Cargo.toml` 无 `specta` / `ts-rs` 等类型导出工具，全部 TS 类型为手抄。实例：`types/codex.ts:5` 的 `CodexMcpServer` 带 `transport`、`name` 字段，而 Rust 侧 `src-tauri/src/commands/codex.rs:100` 的同名 struct 无这两个字段（name 来自 map key、transport 为前端推导）——目前靠约定工作，但 141+ 命令 × 手抄类型没有任何编译期/测试期 drift 检查，唯一守卫是 `api-facade-boundary.smoke.test.ts`（只管 invoke 白名单，不管 payload 形状）。

---

## 8. main.ts 【P2，基本健康】

`src/main.ts` 271 行 / 8.9KB。优点：启动顺序有清晰的 perfMark 埋点、字体/装饰 CSS/全量 locale/图标全部延迟到首帧后（`scheduleAfterPaint` / `scheduleWhenIdle`）、有启动失败兜底（`startupRecovery`）。

可改进：

- `:21-88` 约 70 行「deferred stylesheet link 注入」纯工具逻辑（`ensureDeferredStyleLink` / `applyDeferredStyle*`）应移入 `utils/`（与既有 `utils/scheduling.ts` 同级），main.ts 回到纯编排；
- `:128-171` `scheduleDeferredStartupTasks` 内 6 个嵌套调度任务可表驱动；
- locale 预热判断逻辑（`:228-245`）与 router meta（`deferLocaleHydration`）耦合较深，迁移时注意。

非问题但相关：单语言包 170KB+（`i18n/locales/en-US.ts` 172.7KB），靠 `bootMessages.ts`（52.8KB）+ 延迟 hydration 缓解，长期应按路由分包。

---

## 9. 其它发现

### 9.1 巨型 scoped CSS 与视觉代码重复（P1）

style 块行数 Top：`CheckinView.vue` **1,013 行**、`CodexAuthView.vue` 653、`CheckinAccountDashboardView.vue` 619、`ClaudeCodeProfilesView.vue` 603、`AccountFormModal.vue` 576、`ProviderTemplateSelector.vue` 521、`CommandsView.vue` 518、`GeminiCliView.vue` 491、`ClaudeCodeView.vue` 488、`PricingView.vue` 482、`CodexView.vue` 460。views/ 下共出现 **91 处 `linear-gradient`** 手写渐变。每个平台页都重新手写 hero/卡片/网格皮肤（如 `codex-auth-view__*`、`codex-slash-hero__*`），这是「平台落地页相似度只有 10%」的根因——逻辑可复用但皮肤层从未沉淀为公共 surface 组件/工具类。

### 9.2 与设计方向冲突的遗留（P2）

`components/common/AnimeBackground.vue` 仍被 `App.vue` 与 `components/MainLayout.vue` 引用。`ccr-ui/CLAUDE.md` 设计上下文已明确 `Neko / anime` 分支为待移除遗留。

### 9.3 死代码 / 弃用半成品（P2）

- `configs/slashCommands.ts:105` `codexConfig` 导出后 0 引用；
- `views/McpView.vue`（147B）只是 `McpManagerView` 的转发壳，与路由 redirect 功能重复。

---

## 优化建议（按收益/成本排序）

1. **统一 Profiles 组件套件（收益最高，成本中）** —— 将 `components/{claude,codex}/profiles/` 5 对 83–91% 同构组件合并为单套 props/slot 参数化组件（参考 BaseSlashCommands 模式），同时合并 `use{Claude,Codex}Profiles{Filter,Insights}` 为泛型 composable。预计净删 ~2,500 行，且消除双倍修 bug 成本。已有 smoke 测试（claude-profiles-view / codex-profiles-view）兜底，重构风险可控。

2. **拆解 CodexAuthView.vue（收益高，成本中高）** —— 按既有 Checkin 先例（tabs/ + components/ + composables/）切分：`codex-auth/` 子目录 + AccountsTab / ProvidersTab / ProviderEditorModal（718 行模板直接成组件）/ AddAccountWizard / `useCodexOAuthFlow()`。目标单文件 <400 行。`codex-auth-view.smoke.test.ts`（16.5KB）已存在，可作为重构安全网。

3. **扩大 invoke 守卫范围（收益高，成本极低）** —— 把 `tests/api-facade-boundary.smoke.test.ts` 的扫描范围从 `src/api/tauri.ts` 扩到全 `src/**`（白名单豁免 `api/domains/*`、`api/runtime/*`、`utils/logger.ts`），立刻锁死 `useMonitoringFeed.ts:302,311` 这类穿透并防再犯；顺手把这两处改走 `api/domains/events.ts` / 新增 `domains/monitoring.ts`。半天工作量。

4. **平台描述符扩容（收益高，成本中）** —— 将 Codex / OpenCode / Claude 的 MCP、Agents、Commands 页逐个迁入 `views/generic/*` + `config/platformDescriptors.ts` 体系（基建已就绪，gemini 已验证）。每迁一页净删 400–1,300 行。建议顺序：OpenCodeMcpView（424 行，最小）→ OpenCodeCommandsView → CodexAgentsView → CodexMcpView（最大，留最后）。

5. **类型去重 + 漂移防护（收益中高，成本低/中）** —— 低成本部分：消除 §7.1 的 8 组同名重复（半天）；中成本部分：引入 `ts-rs` 或 `specta` 给 141+ Tauri 命令的 payload 生成 TS 类型，或至少为高频域（codex/checkin/usage）加快照测试。

6. **沉淀 useAsyncAction / useCrudResource composable（收益中，成本低）** —— 把 27 个视图重复的 loading/error/toast/modal 八股收敛；新代码强制使用，旧代码随重构逐步迁移。

7. **CSS 体系收口（收益中，成本中，可与外观任务合并）** —— 把 CheckinView 的 1,013 行及各平台页 450–650 行 scoped CSS 收敛为公共 surface/hero/grid 组件与 token 化工具类，消除 91 处手写渐变；同时移除 AnimeBackground 遗留分支。

8. **API 门面收尾（收益低中，成本低）** —— 决断「domain 命名空间」去留：要么推动消费方迁移（codemod 可自动化 62 个文件），要么承认扁平导出为正式契约、从 index.ts 删除零使用的命名空间导出并更新 spec，避免假性双轨。tauri.ts 内 11 个 interface 迁至 `types/`。

9. **目录与路由整理（收益低，成本低）** —— views/ 根 44 个文件按平台归入子目录（`views/codex/`、`views/opencode/`…），与路由 group 对齐；给 11 条历史 redirect 标注下线版本；删除 `McpView.vue` 壳与 `codexConfig` 死代码；usage 域统一目录约定（`useUsageDashboardState.ts` 移入 composables/ 或确立 feature-folder 规范）。

10. **main.ts 微整（收益低，成本极低）** —— deferred-style 注入工具移入 `utils/deferredStyles.ts`，main.ts 控制在 ~180 行。

## Caveats / Not Found

- 相似度数据基于「平台 token 归一化后的行级 diff」（difflib SequenceMatcher），用于量化复制粘贴程度，不等于可直接合并的比例；合并时需处理两侧已漂移的 8–17% 差异。
- 未运行 bundle 分析（vite build --report），首屏体积/分包效果未量化；路由懒加载结论仅基于源码静态检查。
- Rust IPC 漂移仅抽查 `CodexMcpServer` 一例，未对 141+ 命令逐一比对；结论是「无防护机制」而非「已存在大量漂移」。
- `views/.omc/state/` 目录用途未深究（疑似工具产物）。
- 本机 shell 的 `rg`/`diff` 被 rtk 代理改写输出格式，部分统计改用 grep/python 完成，计数口径已交叉验证。
