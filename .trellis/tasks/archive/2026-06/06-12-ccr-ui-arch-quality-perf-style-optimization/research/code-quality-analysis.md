# Research: ccr-ui 前端代码质量深度分析

- **Query**: 深入分析 `ccr-ui/`（Vue 3 + TypeScript + Tauri 2 + Pinia 前端，不含 src-tauri）的代码质量问题
- **Scope**: internal
- **Date**: 2026-06-12

## 代码规模基线

| 指标 | 数值 |
|---|---|
| `src/` 文件数 | 366（211 个 `.vue` + 155 个 `.ts`） |
| `src/` 总行数 | 130,585 |
| `tests/` 测试文件 | 81 个（80 个 smoke + 1 个 i18n cjs） |
| `tests/` 总行数 | 16,900（约为 src 的 13%） |
| i18n locale 体量 | zh-CN 4,457 行 / en-US 4,617 行，各 3,526 个 leaf key |

总体评价：这是一个**纪律性很强**的代码库（零 `any`、零 `@ts-ignore`、console 全收口、lint 抑制仅 2 处），主要问题集中在**死代码堆积、整组件级复制粘贴、i18n 硬编码、API 层类型逃逸、测试纵深不足**五个方面。未发现 P0 级（安全/正确性）问题。

---

## 1. 类型安全 — 总体优秀，API 层存在系统性类型逃逸

### 1.1 显式 any / lint 抑制（结论：干净）

| 模式 | 出现次数 | 说明 |
|---|---|---|
| `any` 类型（真实使用） | **0** | 全库仅 16 处字符串匹配，全部位于 i18n 文案/注释/CSS（如 `overflow-wrap: anywhere`） |
| `@ts-ignore` / `@ts-expect-error` / `@ts-nocheck` | **0** | — |
| `as unknown as` 双重断言 | **9 处 / 8 文件** | 见下表 |
| `eslint-disable` | **2 处** | `utils/logger.ts:1`（logger 必须用 console，有理由说明）、`views/UsageDashboardView.vue:1`（template-shadow） |

`as unknown as` 全部位置：
- `configs/providersCatalog.ts:161`（JSON catalog 转型）
- `i18n/index.ts:90`（vue-i18n locale 类型收窄）
- `components/claude-observer/{TokenDetailTab,CostAttributionTab,BehaviorAnalysisTab}.vue`（异步组件 module.default 转 Component，3 处同构重复）
- `views/ClaudeCodeProfilesView.vue:550`、`views/CodexProfilesView.vue:783`（functional component 手动挂 props 的 hack，两处复制，见 §3.6）
- `components/usage/LlmusageInstallDialog.vue:382,457`

### 1.2 API 层 `<T = Default>` 泛型逃逸（P1）

`src/api/domains/*.ts` 共 **312 处**函数采用如下模式（17 个文件，热点：`codex.ts` 72、`claude.ts` 48、`checkin.ts` 35、`stats.ts` 30、`opencode.ts` 27）：

```ts
// api/domains/claude.ts:22
export const getClaudeSettings = async <T = ClaudeSettingsData>(): Promise<T> => {
  return invoke('claude_get_settings')
}
```

问题：`invoke()` 返回 `Promise<any>`，被隐式转型为调用方任选的 `T`，**既无编译期跨端校验，也无运行时校验**。Rust 端改字段名/类型时前端完全静默。`invoke<T>` 显式泛型仅 11 处。`_shared.ts` 提供了 `isRecord/asRecord/pickArray` 等运行时收窄工具，但只覆盖少数入口。

- 严重度：**P1**（类型系统的"信任边界"形同虚设，是历史 bug 的温床）
- 缓解现状：`api/tauri.ts`（915 行）已标记为 compatibility-only facade，新 API 强制走 domains —— 架构方向正确，但 domains 自身仍是裸转型。

### 1.3 ESLint 类型规则配置滞后（P2，快赢）

`eslint.config.js`：

```js
'@typescript-eslint/no-explicit-any': 'warn', // TODO: 升级为 'error' - 需要先修复所有 any 类型
```

代码库实际已经是 0 `any`，这条 TODO 已过时 —— 可以**零成本升级为 `error`** 防止回归。同理 `no-console` 仅在 `NODE_ENV=production` 时为 error，本地/CI lint 实际执行 warn（当前 src 已 0 console，同样可直接收紧）。

---

## 2. 错误处理一致性 — 三种风格并存，工具函数推广不足

全库 `catch` 块共 **367 处 / 110 文件**。分布在三种范式：

| 范式 | 数量 | 说明 |
|---|---|---|
| `logger.error/warn(...)` | 157 处 / 60 文件 | 主流，logger 还带原生桥（批量上报 Rust 端） |
| `showError/showSuccess` 等 toast | 281 处 / 40 文件（含 success 类） | 视图层主流 |
| 带注释的静默 `catch {}` | 约 26 处 | **全部带中文注释说明吞错理由**（如 `themeBootstrap.ts:203 "忽略存储异常"`、`unifiedMcp.ts:90 "删除失败默认视为原本不存在"`），纪律良好 |

### 2.1 错误消息提取逻辑大量手写（P1，机械可修）

`utils/errorHandler.ts` 提供了 `getErrorMessage(error: unknown)`，但仅 **21 处 / 10 文件**使用；与此同时 `error instanceof Error ? error.message : String(error)` 三元式被手写了 **125 处 / 53 文件**（热点：`usePlatformPlugins.ts` 5、`useUnifiedMcp.ts` 5、`ConfigsView.vue` 7、`CodexAgentSourcesPanel.vue` 9）。

### 2.2 errorHandler 自身硬编码中文（P2）

```ts
// utils/errorHandler.ts:15
return '发生未知错误'   // fallback 文案绕过 i18n
```

### 2.3 浮动 Promise 无 lint 防护（P2）

- `@typescript-eslint/no-floating-promises` 未启用（需要 type-aware lint）。
- `void loadX()/void refresh()` fire-and-forget 约 41 处（显式 `void` 算自觉标记）；`.then(` 仅 17 处，链式风格已基本淘汰。
- 真空白 `catch(e) {}`（无注释）为 **0**。

---

## 3. 重复代码 — 最严重的问题域

### 3.1 home/* ↔ dashboard/* 整组件复制后原件未删（P1，与 §4 死代码联动）

外观重构（任务 06-11）把 home 组件复制为 dashboard 组件，**只改了 CSS 类名和 i18n key 前缀**，原件全部遗留：

| 原件（已死） | 行数 | 复制品（在用） | 行数 | diff 结果 |
|---|---|---|---|---|
| `components/home/HomeUsageSnapshot.vue` | 708 | `dashboard/DashboardUsageMovement.vue` | 670 | 逐行同构，仅类名/key 重命名 |
| `components/home/HomeActivityStream.vue` | 436 | `dashboard/DashboardSignalStream.vue` | 423 | 同上，外加少量排序逻辑差异 |

### 3.2 format* 工具函数 36 处分散定义（P1）

无共享的 `utils/format.ts`。同名函数在各文件各写一份：

| 函数 | 重复定义次数 | 代表位置 |
|---|---|---|
| `formatNumber` | 6 | `ActivityHeatmap.vue:170`、`TokenUsageChart.vue:454`、`UsageStatsDashboard.vue:494`、`HomeStatusBar.vue:60`、`StatsView.vue:548`、`platformUsagePresentation.ts:83` |
| `formatDate` | 6 | `DateRangePicker.vue:239`、`CheckinRecordsTab.vue:364`、`ClaudeAuthView.vue:401`、`SessionsView.vue:421`、`AccountsTable.vue:181`、`TokenUsageChart.vue:426` |
| `formatDateTime` | 5 | `DashboardUsageMovement.vue:186`、`HomeUsageSnapshot.vue:186`、`MonitoringView.vue:677`、`CheckinAccountDashboardView.vue:463`、`usageOverviewInsights.ts:41` |
| `formatTokens` | 4 | `UsageInsightPanel.vue:388`、`TokenDetailTab.vue:202`、`UsageStatsDashboard.vue:500`、`usageSummaryCards.ts:51` |
| `formatTime` | 3 | `DashboardSignalStream.vue:151`、`HomeActivityStream.vue:140`、`MonitoringView.vue:689` |
| 其余（formatCost/formatDuration/formatRelative/formatTimestamp 等） | 12+ | 散落 `codexHelpers.ts`、`StatsView.vue`、`CommandsView.vue`、`LlmusageInstallDialog.vue` 等 |

### 3.3 剪贴板逻辑 11 处直调（P2）

`navigator.clipboard.writeText` 直调 11 处（`CommandsView.vue:1166`、`GeminiCliView.vue:484`、`ConverterView.vue:731`、`AgentDetailView.vue:479`、`ClaudeCodeView.vue:436`、`OutputStylesView.vue:504`、`OAuthWizardModal.vue:537` 等）。已有两个包装（`codexHelpers.ts:204` 的 `copyToClipboard`、`opencode.ts:35`）但互不知晓、视图也不用。各处的"复制成功提示 + 还原"逻辑也是各写一份。

### 3.4 Claude/Codex Profiles composables 结构性重复（P2）

`useClaudeProfilesFilter.ts`(155) + `useClaudeProfilesInsights.ts`(232) 与 `useCodexProfilesFilter.ts`(129) + `useCodexProfilesInsights.ts`(255)，合计 771 行，diff 显示仅类型名（`ClaudeProfile`↔`CodexProfile`）与少量字段差异，过滤/排序/分组/健康审计骨架完全一致，可用泛型参数化合并。

### 3.5 重复常量（P2）

`const REFRESH_TTL_MS = 30_000` 在 `CodexProfilesView.vue:360`、`CodexMcpView.vue:717`、`CodexAuthView.vue:2072` 三处各定义一份；30s TTL 语义还散落在 `usage.ts:69`、`homeUsageOverview.ts:24`、`useCodexDashboard.ts:55`、`useBackendHealth.ts:60`、`balanceRefreshQueue.ts:5`。

### 3.6 functional component hack 复制（P2）

`ProfilesSection`（h() 手写 + `as unknown as { props }` 挂 props）在 `ClaudeCodeProfilesView.vue:536-550` 与 `CodexProfilesView.vue` 各复制一份。

### 3.7 claude-observer 异步组件加载样板（P2）

`module.default as unknown as Component` 模式在 `TokenDetailTab/CostAttributionTab/BehaviorAnalysisTab` 三个文件重复。

---

## 4. 死代码与遗留 — 约 8,000 行未引用代码（P1）

### 4.1 未被任何文件引用的 .vue 组件：30 个，约 7,900 行

引用扫描（全文匹配组件名 + import 路径，已逐个抽查验证）确认以下文件**零引用**：

**整个 home/ 组件家族（6 个，2,255 行）**——旧首页被 DashboardView 取代后遗留：
`HomeUsageSnapshot.vue`(708)、`HomePlatformRegistry.vue`(485)、`HomeActivityStream.vue`(436)、`HomeStatusBar.vue`(279)、`HomeQuickActions.vue`(242)、`HomeEditorialHero.vue`(105)

**不在 router 注册的死视图（3 个，1,200+ 行）**：
`views/ProviderHealthView.vue`(602)、`views/StatsView.vue`(608)、`views/mcp/UnifiedMcpView.vue`

**其余 21 个死组件**（节选）：
`UsageStatsDashboard.vue`、`TokenUsageChart.vue`、`ActivityHeatmap.vue`、`MarkdownEditor.vue`、`Navbar.vue`、`PageHeader.vue`、`StatusHeader.vue`、`Layout.vue`、`LanguageSwitcher.vue`、`DateRangePicker.vue`、`FolderSidebar.vue`、`DetailField.vue`、`BuiltinPromptsPanel.vue`、`CcrStatusWidget.vue`、`mcp/McpEditPanel.vue`、`claude/ClaudeProfileEditorSidebar.vue`、`common/BackgroundImage.vue`、`common/Skeleton.vue`、`configs/ProviderPresetSelector.vue`、`usage/AccountListTable.vue`、`usage/RingProgress.vue`

合计 **≈ 7,863 行（占 src 6%）**。注意：扫描对字符串名匹配偏保守（任何提及都算引用），真实死代码只多不少。

### 4.2 死 i18n keys 与死样式（P2）

- `home.*` 命名空间（zh-CN.ts:453-632，180 行；en-US 等量）只被死掉的 home/* 组件引用。
- `styles/home.css`(3.5K) 仍被 `core.css:9` import，服务对象已死。
- `styles/neko-decorations.css`(12K) **未被任何入口 import**，纯死文件。
- `animations.css` 中 `neko-press/neko-ear-wiggle/neko-tail-wag/neko-float/neko-breathe` 等 keyframes、`backgrounds.css` 的 `.bg-neko-grid` 仍在 live 加载链中 —— 按 `ccr-ui/CLAUDE.md` 设计规范，neko/anime 是**明确要求移除的历史分支**。
- `components/common/AnimeBackground.vue` 仍被 `App.vue:30` 实际使用、`backgroundCache.ts:3` CACHE_KEY 为 `'anime-background'` —— 与设计规范命名冲突（功能在用，命名遗留）。

### 4.3 死导出与遗留设施（P2）

- `utils/codexHelpers.ts:239,262` 的 `debounce`/`throttle` 导出后**全库零调用**（`stores/usage.ts` 另手写了一套 debounce timer）。
- Storybook：8 个 storybook devDependencies + 完整脚本，但全库仅 2 个 stories 文件（`ui/Button.stories.ts`、`ui/Card.stories.ts`），近乎弃用状态。
- `layouts/MainLayout.vue` 仅是 `components/MainLayout.vue` 的薄包装（双层同名，导航心智负担）。
- `stores/homeUsageOverview.ts`（`useHomeUsageOverviewStore`）被 DashboardView 使用 —— 命名漂移。

### 4.4 干净面（值得肯定）

- `TODO/FIXME/HACK` 全库仅 **1 处**（`configs/slashCommands.ts:77`）。
- 遗留 `console.log` 调试输出：**0**（console 全收口至 `utils/logger.ts`）。
- 大段注释掉的代码：未发现。

---

## 5. i18n 覆盖 — locale 对称完美，但硬编码文案规模大（P1）

### 5.1 locale 对称性（结论：优秀）

- zh-CN 与 en-US 各 **3,526 个 leaf key，双向差集为 0**（脚本递归比对验证）。
- 有专门的 `scripts/check-i18n.mjs` 接入 `just frontend-check` 链，CI 拦截单边加 key。
- `tests/i18n.test.cjs` 另做 namespace/占位符级校验。

### 5.2 硬编码 CJK 文案绕过 i18n（P1）

逐行扫描（区分注释与代码，注释中文符合项目规范不计）：

- 非注释 CJK 行 **1,384 行 / 85 文件**；扣除 `i18n/bootMessages.ts`（448 行，启动期 fallback 设计，属预期）后，**真实硬编码约 936 行 / 84 文件**。
- Top 文件：`ClaudeAuthView.vue`(56)、`CodexSessionsView.vue`(43)、`CheckinRecordsTab.vue`(44)、`CheckinProvidersTab.vue`(42)、`SshManagementView.vue`(42，含错误文案 `'加载 SSH 主机失败'`)、`BudgetView.vue`(41)、`OAuthWizardModal.vue`(41)、`CodexMcpView.vue`(41)、`CheckinAccountDashboardView.vue`(39)、`opencodeMeta.ts`(33)、`SkillsMigrationView.vue`(32)、`WslManagementView.vue`(26，含 `'${secs}秒前'` 相对时间)。
- OpenCode 系列视图（`OpenCodeMcpView/AgentsView/CommandsView/ProvidersView/PluginsView`）的 description props 全部中文裸串。

### 5.3 私有双语 helper 绕过 vue-i18n（P2）

```ts
// views/CodexMcpView.vue:722
const tt = (zh: string, en: string) => (isZh.value ? zh : en)
```

该文件 41 处 `tt('中文', 'English')` —— 文案游离在 locale 文件外，check-i18n 脚本无法覆盖。

### 5.4 `translateWithFallback` 内联中文 fallback（P2）

`HomeStatusBar.vue:110-118` 等处的 `translateWithFallback(t, 'home.systemMetricMemory', '已用 {used} / {total} GB', ...)` 模式把 zh fallback 内联进组件，与 locale 文件形成双源。

---

## 6. 测试覆盖 — 广度尚可、纵深不足，且 tests/ 在质量门外（P1）

### 6.1 现状

- 81 个测试文件全部为 **smoke 级**（vitest + jsdom，`vitest.smoke.config.ts`），16,900 行，约为 src 体量 13%。
- 覆盖面集中在：usage 仪表盘（20+ 文件）、codex（dashboard/profiles/auth/tray）、claude（profiles/settings）、主题/窗口 chrome、router、i18n 格式化、startup-recovery、sanitize。
- 另有 Playwright 路由截图（11 路由 × 3 主题）做视觉回归。

### 6.2 缺口

**Pinia stores（10 个中 5 个零测试）**：

| store | 测试引用 | 体量 |
|---|---|---|
| `claudeObserver.ts` | **0** | 7.9K |
| `commands.ts` / `commandsView.ts` | **0** | 2.4K / 1.7K |
| `configs.ts` | **0** | 2.0K |
| `shellPreferences.ts` | **0** | 7.9K（含 localStorage 持久化逻辑） |
| usage / homeUsageOverview / usageDashboardPayload / usageImportNormalization / ui | 有 | — |

**composables（26 个中 17 个零测试引用）**：`useAgents`、`useCachedFetch`、`usePolledData`、`useUnifiedMcp`、`usePlatformMcp`、`usePlatformPlugins`、`useMcpManager`、`useFuzzySearch`、`useClaudeProfilesFilter/Insights`、`useCodexProfilesFilter/Insights`、`useCodexAgentSources`、`useBackendHealth` 等。

**巨型视图零测试**：`CodexAuthView.vue`（3,937 行）、`CommandsView.vue`（1,742 行）、`ConverterView.vue`（1,101 行）等核心交互面无任何测试。

### 6.3 tests/ 不在 lint 与 type-check 范围内（P1）

- `eslint.config.js` ignores 含 `'**/tests/**'`；
- `tsconfig.json` include 仅 `src/**` —— `vue-tsc --noEmit` 不检查 tests/，vitest 运行时 esbuild 只剥类型不校验。16,900 行测试代码处于双重质量门之外。

---

## 7. lint 抑制与配置 — 行内纪律极好，配置层有松动点

- 行内 `eslint-disable`：仅 2 处且均有理由（§1.1）。
- 配置层关闭/降级的规则：

| 规则 | 状态 | 风险评估 |
|---|---|---|
| `vue/no-v-html` | **全局 off** | 注释称为 ANSI 渲染豁免，但全局关闭使新增 v-html 无告警。现有 5 处 v-html（`ClaudeProfileRow`、`MarkdownEditor`、`LlmusageInstallDialog`、`CommandsView`）经核查均有 `escapeHtml`/DOMPurify（`utils/sanitize.ts`）防护，**当前无 XSS 实害**；建议改为逐行豁免。P2 |
| `@typescript-eslint/no-explicit-any` | warn（TODO 过时） | 可零成本升 error。P2 快赢 |
| `no-console` | 仅 production env 为 error | 可直接常态 error。P2 快赢 |
| `vue/require-default-prop` | off | 与 type-based props + TS strict 组合下可接受 |
| `no-floating-promises` 等 type-aware 规则 | 未启用 | 见 §2.3。P2 |

- `tsconfig.json` 为 strict 全家桶（`strict` + `noUnusedLocals` + `noFallthroughCasesInSwitch`），良好。

---

## 8. 魔法数字/字符串 — 大体已命名化，少量散落（P2）

**好的一面**：定时/TTL 常量普遍用 `*_MS` 命名（`FILTER_DEBOUNCE_MS=300`、`IMPORT_PROGRESS_REFRESH_INTERVAL_MS=2_000`、`DASHBOARD_CACHE_TTL_MS=30_000`、`BALANCE_REFRESH_MIN_INTERVAL_MS=30_000` 等 16 处）。

**散落点**：
- `ConverterView.vue:683,733,758,781` —— 成功提示自动消隐 `3000/2000ms` 裸数字 4 处（同文件内还不一致）。
- 模态框聚焦延迟 `setTimeout(..., 100)` 在 `CommandFormModal.vue:265`、`AddConfigModal.vue:285`、`EditConfigModal.vue:386`、`UpdateModal.vue:448` 重复 4 处。
- `ConfigsView.vue:507` 高亮脉冲 `1500ms` 裸数字。
- `REFRESH_TTL_MS = 30_000` 三处重复定义（见 §3.5，属重复而非未命名）。

---

## 9. 组件 props/emits 规范 — 优秀

- `defineProps<{...}>()` 类型式声明 **136 处**；运行时对象式 `defineProps({...})` **0 处**。
- `defineEmits<{...}>()` 类型式 **74 处**；数组字符串式仅 **4 处**（`AddConfigModal.vue`、`EditConfigModal.vue`、`ui/Button.vue`、`ui/Input.vue`）。
- `vue/require-explicit-emits: 'error'` 已启用，模板内裸 `$emit` 字符串风险受控。

---

## 10. 额外发现：巨型文件（P1，可维护性）

| 文件 | 行数 |
|---|---|
| `views/CodexAuthView.vue` | **3,937** |
| `views/CommandsView.vue` | 1,742 |
| `views/CheckinView.vue` | 1,738 |
| `views/ClaudeCodeProfilesView.vue` | 1,638 |
| `views/ClaudeCodeSettingsView.vue` | 1,359 |
| `views/CodexMcpView.vue` | 1,339 |
| `views/AppSettingsView.vue` | 1,141 |
| `views/codex/CodexAgentsView.vue` | 1,131 |
| `views/ConverterView.vue` | 1,101 |
| 1,000+ 行视图合计 | **12 个** |
| `stores/usage.ts` | 928 |
| `views/usage/useUsageDashboardState.ts` | 998 |

checkin/usage 模块已示范了正确拆法（`views/checkin/tabs/* + composables/*`、`views/usage/*`），其余巨型视图未跟进。

---

## 严重程度汇总

| 级别 | 问题 | 规模 |
|---|---|---|
| P0 | （未发现） | — |
| P1 | 死代码堆积（home/* 家族、3 个死视图、30 个零引用组件、死 i18n/样式） | ≈ 7,900 行 + 360 行 i18n + 15.5K 样式 |
| P1 | home↔dashboard 整组件复制未删原件 | 2,237 行成对 |
| P1 | format*/错误消息提取等工具函数跨文件复制 | format* 36 处、`instanceof Error` 三元 125 处 |
| P1 | 硬编码中文绕过 i18n | ≈ 936 行 / 84 文件 + 私有 `tt()` |
| P1 | API 层 312 处 `<T = Default>` 无校验转型 | 17 文件 |
| P1 | 测试纵深不足 + tests/ 不受 lint/type-check 管控 | 5/10 store 零测试、17/26 composable 零测试 |
| P1 | 12 个 1,000+ 行巨型视图（峰值 3,937 行） | — |
| P2 | eslint 配置滞后（no-explicit-any=warn、vue/no-v-html 全局 off、no-console 条件 error、无 floating-promises） | — |
| P2 | 剪贴板 11 处直调、REFRESH_TTL_MS 三处重复、4 处裸 timeout、4 处数组式 emits | — |
| P2 | Claude/Codex Profiles composables 771 行结构性重复 | — |
| P2 | neko/anime 遗留命名与样式残留（违反设计规范方向） | AnimeBackground.vue 在用、neko keyframes 在 live CSS |
| P2 | Storybook 设施近弃用（8 个依赖、2 个 stories） | — |

---

## 优化建议（按收益/成本排序）

1. **删除死代码**（高收益/低成本，建议先行）：30 个零引用组件（先以 `knip` 或 `vue-tsc` 辅助二次确认）、3 个死视图、`home.*` i18n 命名空间、`home.css` import、`neko-decorations.css`、`debounce/throttle` 死导出。一次性减负约 6% src 体量，并消除 §3.1 的复制对。
2. **eslint/tsconfig 快赢**（高收益/极低成本）：`no-explicit-any` → error；`no-console` → 常态 error；`vue/no-v-html` 改回 error + 5 处逐行豁免；tests/ 纳入 eslint 与独立 tsconfig（`tsconfig.vitest.json`）。
3. **建立 `utils/format.ts` 与 `utils/clipboard.ts`**（高收益/低成本）：收口 36 处 format* 与 11 处剪贴板调用；同时把 `getErrorMessage` 推广替换 125 处手写三元（纯机械替换，可脚本辅助）。
4. **i18n 硬编码清理**（高收益/中成本，可分批）：按 Top 文件清单逐页迁移 936 行硬编码文案入 locale；删除 `CodexMcpView.vue` 的 `tt()`；`errorHandler.ts` fallback 走 i18n。可同时引入 `@intlify/eslint-plugin-vue-i18n` 的 `no-raw-text` 防回归。
5. **补 5 个零测试 store 的单测**（中收益/中成本）：`shellPreferences`（持久化）与 `claudeObserver` 优先（体量最大、逻辑最多）。
6. **合并 Claude/Codex Profiles composables**（中收益/中成本）：771 行 → 泛型化约 400 行，顺带消除 insights 逻辑分叉。
7. **API 层类型收敛**（高收益/高成本，长期）：以 Rust 端命令签名为源生成 typed client（或对关键 domain 引入 valibot/zod 运行时校验），替换 312 处 `<T = Default>` 逃逸。可先在 codex/claude 两个最大 domain 试点。
8. **拆分巨型视图**（中收益/高成本，随迭代渐进）：`CodexAuthView.vue`(3,937) 优先，参照 checkin/usage 模块的 tabs + composables 拆法；不建议专项大重构，随功能迭代逐步拆。
9. **遗留命名清理**（低收益/低成本）：`AnimeBackground` → 设计规范对齐命名、`anime-background` cache key 迁移、`useHomeUsageOverviewStore` 更名、`layouts/MainLayout.vue` 双层包装合并；Storybook 要么补 stories 要么移除 8 个依赖。

## Caveats / Not Found

- 死组件扫描基于全文名称匹配（保守策略：任何字符串提及都算引用），列表可信度高，但删除前仍建议用 `knip` / 构建产物分析二次确认。
- 未运行 `vue-tsc`/`eslint` 实测（重型命令），lint 现状基于源码静态统计。
- CJK 硬编码统计按行计数（非按字符串计数），且无法识别"故意保留中文"的设计场景（如 SkillsMigrationView 下线公告），实际迁移量可能略低于 936 行。
- 浮动 Promise 仅做了模式抽样（`void` 前缀 41 处、`.then` 17 处），精确清单需启用 type-aware lint 才能产出。
