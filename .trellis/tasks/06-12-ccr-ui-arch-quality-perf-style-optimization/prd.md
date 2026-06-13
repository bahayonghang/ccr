# ccr-ui 架构 / 质量 / 性能 / 样式综合修复计划

> 本文档是完整修复方案（plan）。每个问题给出：证据位置 → 修复方案 → 验收方式。
> 证据细节见本任务 `research/` 下四份报告：
> `architecture-analysis.md` / `code-quality-analysis.md` / `performance-analysis.md` / `style-analysis.md`。

## Goal

系统性修复 2026-06-12 四维度分析发现的全部问题：修复 2 个 P0 级用户可感知断裂（SessionsView 失效、签到模块主题失效）；删除约 7,900 行死代码并移除 `marked`/`highlight.js` 依赖链；合并双胞胎组件与重复工具（净删 ≥ 5,000 行）；收敛 API 边界与类型安全；治理性能热点（CommandsView O(n²)、GPU 装饰层、keep-alive 常驻）；将「深色工作台」设计语言从 Settings 推广到全局；最后用 lint/stylelint/合同测试/守卫测试把全部成果锁死。

## 基线（健康面，不动）

- 路由全量懒加载、9 路 vendor 分包、CI bundle 预算门禁（`scripts/check-bundle-budget.mjs`）。
- `any` / `@ts-ignore` 实际为 0；console 全收口 `utils/logger.ts`；TODO 全库仅 1 处。
- locale zh/en 各 3,526 key 完全对称，`scripts/check-i18n.mjs` 已入 CI。
- `usePolledData` 轮询基建（可见性暂停 + in-flight 去重）、Tauri listen/DOM listener 全部配对，无裸泄漏。
- `tokens.css` 三层主题模型（theme/flavor/accent）完整正交；`transition: all` 仅 14 处；全局 reduced-motion 兜底。
- props/emits 全类型化；tsconfig strict 全家桶。

---

## WS1 — P0 修复（用户可感知断裂）

### 1.1 SessionsView 整页失效

- **证据**：`views/SessionsView.vue:343-378` 用 legacy `fetch('/api/sessions')`、`fetch('/api/sessions/stats')`、`fetch('/api/sessions/reindex')`；Tauri 运行时无 HTTP 服务，页面 100% 加载失败。
- **方案**：
  1. 检查 src-tauri 是否已有 `get_session_*` / sessions 相关 IPC 命令；有则在 `api/domains/` 新建（或扩展）`sessions.ts` 域封装，SessionsView 全部改走 invoke。
  2. 若后端无对应命令且产品上该页已被 CodexSessionsView 等替代 → 直接下线：删视图 + 路由改 redirect，并删除其专属 i18n key。
  3. 若保留：同步加分页/上限（参考 `CodexSessionsView` 的 `SESSION_LIMIT=160` 模式），该页从未经受真实数据量检验。
- **验收**：Tauri 运行时打开 Sessions 页正常加载（或路由已 redirect 且无死链接入口）。

### 1.2 签到模块主题失效（外来调色板 + `.dark` 双轨）

- **证据**：`views/checkin/**` 约 450+ 处 Tailwind 默认调色板 raw rgb（CheckinView 200、CheckinProvidersTab 60、CheckinRecordsTab 56、OAuthWizardModal 56、AccountFormModal 39、AccountsTable 19、CheckinAccountsTab 15）；116 处 `.dark` 后代选择器（CheckinView 54、CheckinRecordsTab 29、CheckinProvidersTab 18、AccountFormModal 12、AccountsTable 2、MainLayout 1）构成第二套主题机制；`OAuthWizardModal.vue:657-689` 无任何暗色变体；高饱和 teal/green/blue/indigo 渐变按钮（`CheckinView.vue:837,856`）违反低饱和语义色规范。
- **方案**（机械替换为主，按文件分批）：
  1. 建立映射表：Tailwind 调色板值 → `tokens.css` 语义 token（blue-* → `--color-primary*`/`--color-info*`、green/teal-* → `--color-success*`、gray-* → `--color-surface*`/`--color-border*`/`--color-text*`、red-* → `--color-danger*`、amber-* → `--color-warning*`）。
  2. 逐文件替换 raw rgb → `rgb(var(--xxx-rgb) / α)` 或直接用语义变量；同一文件内顺手把 `.dark .xxx` 块合并进 token（token 本身已随 `data-theme` 切换，`.dark` 块整块删除）。
  3. 高饱和渐变按钮改扁平 surface + accent 边框/inset 激活态（遵循 theme-token-contracts）。
  4. OAuthWizardModal 全量 token 化后自动获得暗色支持。
  5. 改动期间不得破坏 `.trellis/spec/ccr-ui/frontend/checkin-ux-contracts.md` 的并发/4 态显示/toast 行为。
- **验收**：dark/light × mocha/latte/paper/graphite 全 flavor 下签到模块随主题正确换肤；checkin 子树 `.dark ` 选择器为 0、Tailwind 调色板 raw rgb 为 0（容许纯黑阴影类合理用例）；浅色主题下卡片边界可见。

### 1.3 `checkin-shared.css` 旧玻璃语言固化为全局样式

- **证据**：`styles/index.css:7` 将其与 core.css 并列首屏加载；`checkin-shared.css:8-23` `blur(20px)` + `border-radius: 1.5rem`(24px) + `rgb(255 255 255 / 20%)` 白玻璃边框（light 主题下卡片边界消失）+ `.checkin-badge-pill` 9999px。
- **方案**：按新语言重写整个文件——`--radius-lg`(8px) 卡片、不透明 `--color-surface*` 背景、`--color-border*` 边框、去 backdrop-filter、pill 仅保留真正的状态 chip；评估是否仍需全局加载（仅 checkin 使用则下沉为模块内样式）。
- **验收**：文件内无 blur/24px 圆角/白色玻璃边框/按钮级 pill；latte 主题下用 `bun run dev:web` 复核 Checkin 截图。

### 1.4 悬空 token 引用 + `styles/tokens.ts` 镜像漂移

- **证据**：`--shadow-glow-primary`（`tailwind.config.ts:112`、`LanguageSwitcher.vue:144`、`ui/EmptyState.vue:24`、`styles/tokens.ts:51`）、`--shadow-glow-success/danger`（`tailwind.config.ts:113-114`）、`--ease-default`（`tailwind.config.ts:122`、`tokens.ts:90`）、`--color-border-interactive-rgb`（`tailwind.config.ts:81`）均未在任何 CSS 中定义，引用静默失效。
- **方案**：glow 系引用直接删除（新设计语言去发光，不是补定义）；`--ease-default` → 指到已存在的 `--ease-out`（或在 tokens.css 补一行别名）；`--color-border-interactive-rgb` 删 utility 或补定义；`styles/tokens.ts` 与 tokens.css 全字段比对，同步或（若无运行时消费者）整体删除该镜像。
- **验收**：grep 四个变量名，引用与定义一致；type-check/lint 通过。

---

## WS2 — 死代码删除（净删 ~7,900 行 + 2 个依赖）

> 删除前统一用引用扫描二次确认（建议引入 `knip` 跑一次）；分 2-3 个 commit 落地便于回滚。

### 2.1 零引用 .vue 组件（30 个）

- **home/ 家族 6 个（2,255 行）**：`HomeUsageSnapshot`(708)、`HomePlatformRegistry`(485)、`HomeActivityStream`(436)、`HomeStatusBar`(279)、`HomeQuickActions`(242)、`HomeEditorialHero`(105)——外观重构复制为 `dashboard/*` 后原件未删（`DashboardUsageMovement` 等仅类名/i18n key 重命名）。
- **死视图 3 个**：`views/ProviderHealthView.vue`(602)、`views/StatsView.vue`(608)、`views/mcp/UnifiedMcpView.vue`（均不在 router）。
- **其余 21 个**：`UsageStatsDashboard` + `UsageStatsChart`（互引孤岛）、`TokenUsageChart`、`ActivityHeatmap`（+ 仅被其引用的 `activity/*` 4 个子组件）、`MarkdownEditor`、`Navbar`、`PageHeader`、`StatusHeader`、`Layout`、`Table`、`LanguageSwitcher`、`DateRangePicker`、`FolderSidebar`、`DetailField`、`BuiltinPromptsPanel`、`CcrStatusWidget`、`mcp/McpEditPanel`、`claude/ClaudeProfileEditorSidebar`、`common/BackgroundImage`、`common/Skeleton`、`configs/ProviderPresetSelector`、`usage/AccountListTable`、`usage/RingProgress`、`usage/StatCard`、`usage/AnimatedCounter`。
- **连带收益**：删 `MarkdownEditor` 后 `useMarkdownRender.ts` → `highlightLanguages.ts` 链零引用，**`marked@17` + `highlight.js@11` 从 package.json 移除**（dompurify 保留，CommandsView ansiRenderer 在用）；`AnimatedCounter.vue:68-105` / `RingProgress.vue:150-168` 的 rAF 不取消反模式随删除消失。
- **方案**：按「home 家族 → 死视图 → 其余」三批删除，每批后跑 `bun run type-check && bun run lint && bun run test:smoke` + `vite build` 确认 bundle 预算门禁通过。

### 2.2 死 i18n / 死样式 / 死导出

| 项 | 位置 | 方案 |
|---|---|---|
| `home.*` i18n 命名空间 | `zh-CN.ts:453-632`（180 行）+ en-US 等量 | 随 home 组件删除；同步检查 `bootMessages.ts` 是否内联了 home key |
| `styles/home.css`(3.5K) | 被 `core.css:9` import，但服务对象已死 | 确认 dashboard/* 不依赖后删 import + 文件 |
| `styles/neko-decorations.css`(12K/350 行) | 零 import | 直接删 |
| neko keyframes | `animations.css:288-543`、`tailwind.config.ts:141-165`（含 pink-400 发光，撞品牌红线）、`backgrounds.css:268-289` `.bg-neko-grid/.bg-cyber-grid` | 全部删除 |
| `debounce`/`throttle` 死导出 | `utils/codexHelpers.ts:239,262` 全库零调用 | 删除 |
| `codexConfig` 死配置 | `configs/slashCommands.ts:105` 零引用 | 删除（或在 4.6 中真正接入 Codex，二选一，默认删除） |
| `views/McpView.vue`(147B) 转发壳 | 与路由 redirect 重复 | 删文件，router 直接 redirect |

### 2.3 设施决断

- **Storybook**：8 个 devDependencies + 完整脚本，仅 2 个 stories。**决断：移除**（依赖 + 脚本 + 2 个 stories 文件），如未来需要再重建。
- **遗留命名**：`AnimeBackground.vue` 更名 `StageBackground.vue`（`App.vue:30` 同步），`backgroundCache.ts:3` 的 `'anime-background'` cache key 迁移（带旧 key 读取兜底）；与 `AnimatedBackground.vue`、已删的 `BackgroundImage.vue` 评估三合一（保留一个背景组件）；`layouts/MainLayout.vue` 薄包装与 `components/MainLayout.vue` 双层同名合并为一层；`stores/homeUsageOverview.ts` 更名 `dashboardUsageOverview`（消费者是 DashboardView）。

**验收**：删除清单全部落地；`marked`、`highlight.js`、storybook 系不在 package.json；`just frontend-check-quick` 全绿；bundle 预算通过。

---

## WS3 — 重复代码合并与工具收口

### 3.1 Claude/Codex Profiles 双胞胎合并（净删 ~2,500 行）

- **证据**：5 对组件归一化相似度 83–91%（`ProfilesContextRail` 946 vs `ClaudeProfilesContextRail` 886 行、89.3% 逐行相同；`ProfilesToolbar` 91.0%；`ProfileRow` 87.2%；`ProfilesHeader` 86.2%；`ProfilesStatStrip` 83.5%），约 3,800 行平行维护；composables `use{Claude,Codex}ProfilesFilter`（68.3% 相似）/ `Insights`（70.2%）合计 771 行且已漂移（Claude 版 filter 多了 provider 归一化，Codex 版未同步）。
- **方案**：
  1. 参照 `BaseSlashCommands.vue` + PlatformConfig 注入模式，建 `components/profiles/` 单套参数化组件（props/slot 注入平台差异：类型、API、字段映射）。
  2. composables 泛型化：`useProfilesFilter<T extends ProfileLike>` / `useProfilesInsights<T>`，平台差异（provider 归一化等）作为注入策略；合并时把 Claude 版独有修复同步给 Codex。
  3. 两侧视图（`ClaudeCodeProfilesView` / `CodexProfilesView`）切换到新组件，旧双胞胎删除。
  4. 顺带消除两处复制的 functional component hack（`ClaudeCodeProfilesView.vue:536-550` / `CodexProfilesView.vue:783` 的 `as unknown as` 挂 props）——改为正常 SFC 子组件。
- **安全网**：已有 claude-profiles-view / codex-profiles-view smoke 测试；合并前先跑通基线。
- **验收**：`components/{claude,codex}/profiles/` 删除；两平台 Profiles 页功能等价（smoke + 手工回归）；净删 ≥ 2,000 行。

### 3.2 工具函数收口

| 项 | 证据 | 方案 |
|---|---|---|
| `utils/format.ts` 新建 | format* 36 处分散（formatNumber×6、formatDate×6、formatDateTime×5、formatTokens×4、formatTime×3、其余 12+，详见 code-quality §3.2 清单） | 收口为单一模块（number/date/dateTime/tokens/cost/duration/relative），逐文件替换；死代码文件随 WS2 删除后实际迁移量约 25 处 |
| `utils/clipboard.ts` 新建 | `navigator.clipboard.writeText` 直调 11 处 + 2 个互不知晓的包装（`codexHelpers.ts:204`、`opencode.ts:35`） | 统一 `copyText(text, { toast? })`，含「复制成功提示 + 还原」逻辑；替换全部直调，删两个旧包装 |
| `getErrorMessage` 推广 | `instanceof Error ? e.message : String(e)` 手写 125 处/53 文件 vs 工具仅 21 处使用 | 脚本辅助机械替换；`utils/errorHandler.ts:15` 的硬编码中文 fallback `'发生未知错误'` 改走 i18n |
| 常量收口 | `REFRESH_TTL_MS = 30_000` 三处重复定义（`CodexProfilesView.vue:360`、`CodexMcpView.vue:717`、`CodexAuthView.vue:2072`）；模态聚焦延迟 `setTimeout(...,100)` 4 处；`ConverterView.vue:683/733/758/781` 裸 3000/2000ms | TTL 进 `config/constants.ts`（或各域常量文件）；聚焦延迟进 BaseModal/composable；ConverterView 统一命名常量并保存句柄清理 |
| `useTf()` | `translateWithFallback` 局部包装 `const tf = ...` 3 处重复、直用 14 文件 | 抽 `composables/useTf.ts`；`translateWithFallback(t, key, '中文 fallback')` 内联双源问题随 WS7.3 i18n 清理收敛 |
| claude-observer 异步组件样板 | `module.default as unknown as Component` 三个 Tab 文件重复 | 抽一个 `defineAsyncTabComponent` helper 或统一 `defineAsyncComponent(() => import(...))` 直写消除断言 |

**验收**：grep 各旧模式归零（容许白名单）；type-check/lint/smoke 全绿。

---

## WS4 — 架构边界与类型安全

### 4.1 invoke 守卫扩面 + 存量穿透修复

- **证据**：`tests/api-facade-boundary.smoke.test.ts:4` 只扫 `src/api/tauri.ts`；`composables/useMonitoringFeed.ts:302,311` 裸 `invoke('get_monitoring_feed')` / `invoke('get_recent_events')`（后者 `api/domains/events.ts:13` 已有封装）；`utils/logger.ts:134-135` 动态 import 后裸 invoke（基础设施，应显式豁免）。
- **方案**：守卫扫描范围扩到 `src/**`，豁免白名单 = `api/domains/*`、`api/runtime/*`、`api/tauri.ts`（既有冻结白名单）、`utils/logger.ts`；新建 `api/domains/monitoring.ts` 封装 `get_monitoring_feed`，`useMonitoringFeed` 改走 domains（events 复用既有封装）。
- **验收**：守卫测试在全 src 范围通过；故意加一处裸 invoke 能被测试拦截（验证后撤销）。

### 4.2 API 门面双轨决断

- **证据**：`api/index.ts:12-17` 六个 domain 命名空间（configApi/codexApi/syncApi/platformApi/usageApi/systemApi）应用代码 0 使用；62 个文件扁平导入 `from '@/api'`；2 个文件深挖 `from '@/api/tauri'`（`useCodexTrayPanel.ts:2`、`stores/claudeObserver.ts:4`）；3 个文件深挖 domains 路径。
- **方案**（决断，二选一，**默认取 B**）：
  - A. 推动迁移：codemod 把 62 个文件改为命名空间调用——改动面大、收益主要是美学。
  - B. **承认扁平导出为正式契约**：从 `api/index.ts` 删除零使用的命名空间聚合导出，更新 `.trellis/spec/ccr-ui/frontend/api-facade-boundary.md` 把「domain-first 命名空间」措辞改为「domain 文件组织 + 扁平再导出」；2 个深挖 `@/api/tauri` 的文件改回 `@/api` 入口。消除假性双轨。
- **验收**：spec 与实现一致；无 `@/api/tauri` 深挖导入。

### 4.3 类型去重与归位

- **证据**：8 组同名重复——`SyncStatusResponse`（`types/sync.ts:12` vs `api/tauri.ts:144`）、`UnifiedMcpServer`（`types/unifiedMcp.ts:10` vs `usePlatformMcp.ts:27`）、`TokenStats`（`types/stats.ts:5` vs `useMonitoringFeed.ts:21`）、`SlashCommand`（`types/mcp.ts:27` vs `types/platform.ts:1`）、`PlatformConfig`（4 处同名不同义）、`ImportResult`（checkin vs usage 同名异义）、`Platform`（usage 的平台枚举 vs install 的 OS 枚举，完全不同语义）、`UnknownRecord`（5 处局部定义）。`api/tauri.ts` 内还有 11 个导出 interface。
- **方案**：每组指定单一权威定义点（domain 类型进 `types/<domain>.ts`）；同名异义的改名消歧（如 `Platform` → `UsagePlatform` / `InstallOs`）；`UnknownRecord` 进 `types/common.ts`；`api/tauri.ts` 的 11 个 interface 迁至 `types/`，tauri.ts 只 re-export 保兼容。
- **验收**：8 组重复清零；type-check 全绿。

### 4.4 IPC 类型防漂移 + `<T = Default>` 逃逸收敛

- **证据**：无 ts-rs/specta，141+ 命令 TS 类型全手抄（已发现 `CodexMcpServer` 前后端字段不一致：`types/codex.ts:5` 有 `transport`/`name`，`src-tauri/src/commands/codex.rs:100` 无）；`api/domains/*` 312 处 `<T = Default>` 模式让 `invoke()` 的 any 被静默转型，无编译期/运行时校验。
- **方案**（分两步，第二步可独立 subtask）：
  1. 短期：收紧签名——把 `<T = Default>(...)：Promise<T>` 改为固定返回类型 `Promise<Default>`（泛型参数实际调用方几乎不用；逐文件机械改）；对 codex/claude/checkin 三个最大 domain 的关键入口用 `_shared.ts` 既有 `isRecord/asRecord/pickArray` 加运行时收窄。
  2. 长期：评估在 src-tauri 引入 `specta` + `tauri-specta` 自动导出 TS 类型，以 Rust 为源消除手抄；先在 codex domain 试点，验证 `SourceSyncStats` 等 llmusage 适配层兼容性。
- **验收**：短期——312 处泛型逃逸降为 0（或仅保留有真实多态需求的白名单）；长期——试点 domain 类型由生成物提供。

### 4.5 CodexAuthView god view 拆分（3,937 行 → ≤ 1,000）

- **证据**：template 1,928 行（含 718 行内联 Provider 编辑 Modal，L1088-1806；另有添加账号 Modal L931-1086、重命名 Modal L1818-1923）、script 1,353 行（136 个顶层声明、17 个 API 导入）、style 652 行；内聚 6 个子功能（双 Tab、4 种添加方式 + OAuth 状态机、Provider CRUD、配额、进程检测、导入）。
- **方案**：照 Checkin 模块既有拆法（tabs/ + components/ + composables/）建 `views/codex-auth/` 子目录：
  - `ProviderEditorModal.vue`（718 行模板直接成组件）、`AddAccountWizard.vue`（4 种方式）、`AccountsTab.vue` / `ProvidersTab.vue`、`RenameModal` 复用 BaseModal；
  - `useCodexOAuthFlow()` composable（端口占用/释放、回调提交、监听清理——现有 L3237-3282 逻辑平移）；
  - 已外提的 `codexAuthAccounts.ts` 保持。
- **安全网**：`codex-auth-view.smoke.test.ts`（16.5KB）已存在，重构前先跑基线。
- **验收**：单文件 ≤ 1,000 行；smoke 全绿；OAuth / 4 种添加 / Provider CRUD / 配额手工回归通过。

### 4.6 平台通用基建推广（4 套 MCP → 1 套）

- **证据**：`views/generic/PlatformMcpView.vue`(403) + `usePlatformMcp.ts` + `config/platformDescriptors.ts` 基建就绪但 `genericPlatformDescriptors` 只有 gemini（`platformDescriptors.ts:22-44`）；MCP 4 套实现（CodexMcpView 1,339、OpenCodeMcpView 424、generic 403、McpManagerView）；`CodexAgentsView`(1,131) 与 `generic/AgentsView`(718) 完全平行（相似度 8.1% = 各写各的）；`OpenCodeCommandsView`(11.2K) 绕开 BaseSlashCommands 自滚 CRUD。
- **方案**（按风险从小到大逐个迁移，每个独立可交付）：
  1. `OpenCodeMcpView` → descriptor 接入 generic（最小，424 行）；
  2. `OpenCodeCommandsView` → BaseSlashCommands + PlatformConfig（参考 gemini 薄壳 14 行）；
  3. `CodexAgentsView` → `generic/AgentsView`（差异点扩展 descriptor / slot）；
  4. `CodexMcpView` → generic（最大 1,339 行，留最后；其 `tt()` 私有 helper 与 41 处硬编码文案随迁移消除）。
  - 每迁一个平台，generic 基建缺的能力（如 codex 专属字段）以 descriptor 配置/slot 扩展，不写平台分支 if。
- **验收**：每页迁移后该平台 MCP/Agents/Commands 功能等价；旧实现删除；净删 400–1,300 行/页。

### 4.7 usage 状态域与目录整理

- **证据**：`stores/usage.ts` 928 行（数据获取 + 导入任务状态机 + 诊断 + 能力探测混合）+ `views/usage/useUsageDashboardState.ts` 998 行（feature composable 放在 views/ 下）+ 3 个外围 store；views/ 根平铺 44 个文件、目录与路由组不一致；11 条历史 redirect 无下线标注；`stores/commandsView.ts` / `shellPreferences.ts` 手写 localStorage 持久化；`stores/commands.ts` 与 `commandsView.ts` 命名混淆。
- **方案**（低优先，机会性执行）：
  1. 确立约定：feature-folder（`views/usage/` 内含本特性 composable）为合法布局，写入 spec；`useUsageDashboardState.ts` 保留原地但在 spec 标注。
  2. `stores/usage.ts` 拆出 `usageImportJob.ts`（导入状态机 L250-303 + listener 管理）。
  3. views/ 根按平台归子目录（codex 7 页、opencode 8 页分批，git mv + 路由 import 路径更新，无逻辑改动）。
  4. 11 条 redirect 加注释标注引入版本与计划下线版本。
  5. 持久化统一：引入 `pinia-plugin-persistedstate`（或自建 `definePersistedStore` 包装），`commandsView` / `shellPreferences` 迁移，删手写 `this.persist()`。
  6. `stores/commandsView.ts` 更名 `commandsViewPrefs.ts`。
- **验收**：路由全部可达（router smoke + Playwright 截图基线对比）；持久化行为等价（localStorage key 兼容迁移）。

### 4.8 main.ts 微整

- **证据**：`main.ts:21-88` 约 70 行 deferred stylesheet 注入工具内联；`:128-171` 6 个嵌套调度任务。
- **方案**：工具逻辑移 `utils/deferredStyles.ts`；调度任务表驱动（数组 + 循环）；main.ts 收敛到 ~180 行纯编排。注意 locale 预热（`:228-245`）与 router meta `deferLocaleHydration` 的耦合，平移不改逻辑。
- **验收**：启动行为等价（perfMark 序列不变）；startup-recovery smoke 通过。

---

## WS5 — 性能治理

### 5.1 CommandsView 长输出 O(n²)

- **证据**：后端 `commands:job-progress` 每事件携带全量快照（`CommandsView.vue:968-971`），IPC 序列化 O(n)/事件、累计 O(n²)；`ledgerLines` computed 全量重建（L851-866）且行 key 含整行文本（L531 `${channel}-${index}-${text}`）；无 maxLines（对比 `useStream.ts:54` 的 2000 上限）。
- **方案**：
  1. 前端先行（0.5 天）：加 `MAX_LEDGER_LINES = 2000` 环形截断（与 useStream 对齐）；key 改 `${channel}-${index}`（index 在截断窗口内稳定即可，必要时用单调递增行号）。
  2. 后端跟进（1 天）：`commands:job-progress` 改增量行 delta 事件，参考既有 `checkin:job-delta` 先例（`checkinJobRuntime.ts:165`）；前端累积 + 截断。
- **验收**：运行产出 ≥ 5,000 行输出的命令（如 `ccr doctor -v`），页面交互无可感知卡顿；输出完整性（截断提示）符合预期。

### 5.2 装饰层 GPU 成本

- **证据**：`AnimatedBackground.vue:118-203` 光晕 `blur(88px)` + `ambient-drift 20s infinite` 同时动 scale+opacity（scale 迫使模糊纹理反复重采样），34vw/30vw 双光晕，常驻 ClaudeCodeView / ConfigsView / OpenCodePageShell；全仓 68 处 backdrop-filter 叠加穿透采样。
- **方案**：移除光晕动画的 `scale` 变换（保留 opacity 呼吸，模糊纹理可被合成器缓存），或整体静态化（与 AnimeBackground 一致）——与新设计语言「去发光」方向一致，视觉差异极小；backdrop-filter 总量随 WS6 设计语言推广批量收敛（目标 < 20 处，仅保留 modal backdrop 等必要场景）。
- **验收**：DevTools Performance 录制 ClaudeCodeView 滚动，无持续 GPU 合成热点；视觉对比无明显劣化。

### 5.3 keep-alive 策略收紧

- **证据**：`meta.cache: true` 共 9 视图 + `MainLayout.vue:260-263` `<keep-alive :max="10">` 几乎不驱逐；缓存视图监听器生命周期 = 缓存期（CommandsView 3 个 job 监听、DashboardView monitoring feed 在后台持续处理事件）；UsageDashboardView 的 ApexCharts 实例 deactivated 不销毁；DashboardView `onBeforeUnmount` 的 `teardown()` 被架空。
- **方案**：
  1. cache 白名单收缩到高频切换的 3-4 个（DashboardView、UsageDashboardView、CommandsView、ConfigsView）；CodexAuthView / CodexProfilesView / CodexMcpView 等表单页移出缓存。
  2. 保留缓存的视图在 `onDeactivated` 暂停事件消费 / `onActivated` 恢复（CommandsView job 监听、DashboardView feed 改为 deactivated 时只累积不渲染或直接 unlisten+重放）。
- **验收**：切换 8 个页面后内存占用对比基线下降；缓存视图切走后事件处理停止（日志验证）；返回缓存页状态保留体验不回退。

### 5.4 响应式与重复请求

| 项 | 证据 | 方案 |
|---|---|---|
| 零 shallowRef | `stores/usage.ts:115-124` heatmap(365 天)/trends/logs/modelStats/projectStats/snapshot、`homeUsageOverview.ts:33` overview、`useMonitoringFeed.ts:255` logs、CommandsView currentSnapshot——全部深响应式、整体替换型只读数据 | 定点改 `shallowRef`（小时级；确认所有写入均为整体替换后落地） |
| snapshot 事件双查询 | `usage:snapshot-updated` 被 `stores/usage.ts:338-356`（→ getUsageDashboardV2）与 `homeUsageOverview.ts:85-98`（→ getHomeUsageOverviewV2）各自订阅各发聚合查询，导入期间（2s 节流）双倍 SQLite 负载 | home overview 从 usage store 的 dashboard payload 投影派生，或共享同一 `usePolledData` key 的聚合请求；二者数据本就 home ⊂ dashboard |
| useBackendHealth 永久轮询 | `useBackendHealth.ts:50-69` 模块加载即 30s `immediate:true` 轮询，无 stop 出口；`options.auto===false` 分支形同虚设 | 去模块级自启动：Banner `onMounted` resume / `onUnmounted` pause；健康时退避到 5min、失败后才回 30s |
| usePageTransition 守卫 | `usePageTransition.ts:34` `router.beforeEach` 注册后不解注册（当前消费者 MainLayout 常驻，无害但是地雷） | 保存解注册函数，`onUnmounted` 调用 |
| ConverterView 裸 setTimeout | L683/733/758/781 无句柄，卸载后回调写已卸载 ref | 保存句柄 + `onBeforeUnmount` 清理（随 3.2 常量收口一并做） |
| bootMessages 双语内联 | `bootMessages.ts` 52.8KB 同时含 zh+en 启动子集 | 按 `readStoredLocale()` 只内联一种，另一种走懒加载（省 ~20KB 入口；P3 可选） |

### 5.5 巨石 chunk（随其他 WS 解决，不单独动作）

CheckinView 100KB JS + 67KB CSS、CodexAuthView 75.6KB、CodexProfilesView 69.4KB——分别由 WS1.2/1.3（checkin CSS 减半）、WS4.5（CodexAuth 拆分 + defineAsyncComponent 子 chunk）、WS3.1（Profiles 合并）自然消解。apexcharts 467KB 替换（uPlot/chart.js）列为**长期项不入本任务**，仅当 usage 首开时间成为明确痛点时另立任务。

---

## WS6 — 设计语言推广（按批次迁移 + 每批锁定）

> 顺序即优先级；每完成一批，把该批文件加入合同测试 `migratedViewPaths` 并启用对应 stylelint 规则（见 WS7.2），防止「只重设计一面、其余继续漂移」复发。

### 批次 ①：Checkin 全模块 = WS1.2 + WS1.3（P0，最大违规簇）

### 批次 ②：Shell + 共享 primitive（杠杆最大，每屏可见）

- **MainLayout**（`components/MainLayout.vue`）：nav-item `rounded-2xl`→`var(--radius-lg)`、激活态去 `0 14px 28px` 发光改 border/inset + accent 标记（L511-545）；settings-dock 去 `blur(14px) saturate(116%)`、去 `0 20px 40px` hover 发光、去 radial accent mesh、`settings-dock-pill` 去 uppercase pill（L559-585）；`.dark` 残留 1 处清除。
- **`ui/Button.vue`**：primary/secondary/accent 三个 variant 去 `linear-gradient(180deg,…)` 与 `0 8px 16px` glow（L168-202），改扁平 surface + 边框层级。
- **`ui/Card.vue`**：glow / gradientBorder / pattern 装饰 prop 标记 deprecated，默认值关闭。
- **glass 别名**：`tailwind.config.ts:211-227` 的 `.glass-effect/.liquid-glass/.glass-modal/.glass-elevated` 标记废弃（注释 + stylelint 禁新增），存量随批次替换为 `.surface-*`。

### 批次 ③：图表色 + 高残留视图

- **图表接 `--chart-color-*`**（token 已存在于 `styles/chart-colors.css`）：`TokenUsageChart.vue:85-271` 12 处 SVG 硬编码（若该组件在 WS2 已删则跳过）、`usage/StatCard.vue:172-180` 7 色 colorMap（同上）、`HistoryList.vue:229-237` 8 个 hex（含禁用的 `#8b5cf6` 紫）、usage/ 系列存量。
- **旧语言孤岛重刷**：tray/ 子树（`CodexTrayPanelView.vue:140` 28px、`TrayOverview.vue:273` 22px、`TrayAccountSwitchScreen.vue:231` 22px）、`CommandPalette.vue:338` 18px、`ProviderTemplateSelector.vue:744,755` 18px、`PricingView.vue:683-775` pill 群、`PageHeaderCard.vue:144-145,249`（pill + blur(84px)）、`ScrollToTopButton.vue:54,61`、`ProviderStatsModal.vue:12-13` blur(24px)、`ClaudeCodeProfilesView.vue:1199` / `CodexProfileEditorModal.vue:915` / `McpListPanel.vue:344` / `MultiSelectFloatingBar.vue:77` blur(20px)。profiles 双胞胎样式随 WS3.1 合并一次性重写为新语言。

### 批次 ④：modal 家族 + 系统性收敛

- **modal 收敛 BaseModal**：8 个自滚 backdrop/panel 的迁移（`AddConfigModal`、`EditConfigModal`、`UpdateModal`、`CommandFormModal`、`UnifiedMcpFormModal`、`UnifiedMcpDeleteConfirmModal`、`ProviderStatsModal`、`GlobalConfirmDialog`；自定义 `modal-backdrop|overlay` 16 处/7 文件）；顺带消除 4 处数组式 emits（AddConfigModal/EditConfigModal 在列）。
- **z-index 全量 token 化**：32 处字面量 → `var(--layer-*)`（修正撞层：modal 用 50 高于 `--layer-modal:40`、`AccountActionsMenu.vue:170` 的 60 撞 `--layer-tooltip`）；模板 Tailwind `z-10/20/50` 同步对齐。
- **圆角收敛**：475 处字面量 → `--radius-*`，脚本半自动映射（≤4px→sm、6→md、8→lg、10→xl、12→2xl、>12 非 pill 降档 lg/xl；999/9999px 仅保留 chip/badge/toggle 语义，按钮/输入框 pill 改 `--radius-md`）；脚本产出 diff 后人工抽查每文件。
- **动效收尾**：raw 时长 77 处 → `var(--duration-*)`/`var(--motion-*)`（机会性，随各批次文件顺手改）。
- **断点约定**：书面化三档（720/960/1280）写入 frontend spec；新代码遵守，存量不强迁。
- **`:style=` 清理**（可选）：`ConverterView.vue`(70 处)、`generic/PlatformMcpView.vue`(42)、`UpdateModal.vue`(38) 等静态值移入 class；动态值统一 `style="--x: …"` 注入模式。

**验收（每批）**：目标文件 raw 调色板 rgb / >12px 非 pill 圆角 / 非白名单 backdrop-filter / `.dark ` 选择器归零；Playwright 11 路由 × 3 主题截图对比确认无视觉回归（预期内的语言更新除外）；`migratedViewPaths` 已扩入该批文件。

---

## WS7 — 质量门与防回归（贯穿，优先落地快赢）

### 7.1 eslint / tsconfig 快赢（半天，先行）

- `@typescript-eslint/no-explicit-any`: warn → **error**（现状已 0 any，零成本；删过时 TODO 注释）。
- `no-console`: 条件 error → **常态 error**（现状已 0 console）。
- `vue/no-v-html`: 全局 off → **error + 5 处逐行豁免**（`ClaudeProfileRow`、`MarkdownEditor`〔WS2 删除后剩 4 处〕、`LlmusageInstallDialog`、`CommandsView`，均已有 DOMPurify/escape 防护）。
- tests/ 纳入质量门：eslint ignores 移除 `'**/tests/**'`；新建 `tsconfig.vitest.json` include tests/，接入 `vue-tsc`。
- 评估启用 type-aware `@typescript-eslint/no-floating-promises`（现有 41 处显式 `void` 已是自觉标记，增量成本可控；若 lint 耗时上涨过多则只在 CI 跑）。
- 依赖卫生：`@types/dompurify`、`@types/marked`（随 WS2 删除）、`tailwindcss` 从 dependencies 移到 devDependencies。

### 7.2 样式防回归（随 WS6 批次启用）

- stylelint 增加规则（对已迁移文件强制）：禁 raw `backdrop-filter: blur(...)` 字面量、禁 `border-radius` px/rem 字面量（容许 token）、`color-no-hex`（vue 文件）、禁 `.dark ` 前缀后代选择器。
- 合同测试 `apple-glass-surface-contract.smoke.test.ts` 的 `migratedViewPaths` 随批次扩充，最终改为「全量 − 未迁移白名单」模式，黑名单条目增加 blur/大圆角/raw 调色板模式。

### 7.3 i18n 防回归 + 存量清理（清理部分可拆 subtask 分批）

- 引入 `@intlify/eslint-plugin-vue-i18n` 的 `no-raw-text`（warn 起步），锁死新增硬编码。
- 删除 `CodexMcpView.vue:722` 私有 `tt(zh,en)` helper（41 处文案随 WS4.6 迁移入 locale；若 generic 迁移延后，则先单独迁文案）。
- 存量 ~936 行/84 文件硬编码中文按 Top 文件清单分批迁移（`ClaudeAuthView` 56、`CodexSessionsView` 43、`CheckinRecordsTab` 44、`CheckinProvidersTab` 42、`SshManagementView` 42、`BudgetView` 41、`OAuthWizardModal` 41、`CheckinAccountDashboardView` 39、`opencodeMeta.ts` 33、OpenCode 系列 description props…）——**建议作为独立 subtask 滚动执行**，不阻塞本任务主线。

### 7.4 测试补口

- 5 个零测试 store 补单测，优先 `shellPreferences`（含 localStorage 持久化逻辑，且 WS4.7 要迁移持久化方案——先测后迁）与 `claudeObserver`（7.9K 体量最大）；其余 `commands` / `commandsView` / `configs` 随相关 WS 改动时补。
- 17 个零测试 composable 中，被本计划改动的优先补：`usePolledData`、`useBackendHealth`（WS5.4 改造对象）、合并后的 `useProfilesFilter/Insights`（WS3.1 产物）、`usePlatformMcp`（WS4.6 扩容对象）。

---

## Non-Goals

- apexcharts → uPlot/chart.js 替换（长期项，另立任务）。
- specta/tauri-specta 全量铺开（本任务只做 codex domain 试点评估，见 4.4）。
- i18n 存量 936 行全部清完（防回归 lint 必须落地；存量迁移拆 subtask 滚动执行）。
- src-tauri Rust 端改动（仅 5.1 的 delta 事件需要后端配合，范围锁定该命令）。
- 产品信息架构 / 路由结构变更（目录归整是 git mv，不改 URL）。

## 执行顺序与依赖

```
阶段 A（先行，~1 天）   ：WS7.1 eslint 快赢 + WS1.4 悬空 token + WS4.1 守卫扩面
阶段 B（删码，~1.5 天） ：WS2 全部（三批 commit）→ 跑全量验证
阶段 C（P0，~3 天）     ：WS1.1 SessionsView + WS1.2/1.3 签到迁移（= WS6 批次①）+ 7.2 首批 stylelint
阶段 D（合并，~3 天）   ：WS3.1 Profiles 合并 + WS3.2 工具收口 + WS4.3 类型去重
阶段 E（并行分批）      ：WS4.5 CodexAuth 拆分 ∥ WS4.6 平台基建推广（4 页逐个）∥ WS5 性能项 ∥ WS6 批次②③④
阶段 F（收尾）          ：WS4.2 门面决断 + WS4.4 短期收敛 + WS4.7/4.8 整理 + WS7.3/7.4 + 合同测试白名单反转
```

依赖关系：B 先于 D（死代码删除减少迁移面）；批次① 先于 7.2 stylelint checkin 规则；WS3.1 先于 WS6 批次③ 的 profiles 重刷（避免双倍重刷）；5.5 巨石 chunk 依赖 WS1/3/4 自然消解。每个阶段独立可交付、可拆 subtask（`task.py add-subtask`）。

## Acceptance Criteria（任务级）

1. Sessions 页在 Tauri 运行时可用或已下线，无死入口。
2. 签到模块全主题/flavor 正确换肤；checkin 子树 `.dark ` 选择器与 Tailwind 调色板 raw rgb 归零。
3. 4 个悬空 token 修复；`styles/tokens.ts` 与 tokens.css 一致（或已删除）。
4. WS2 清单全部删除；`marked`/`highlight.js`/storybook 依赖移除；src 总行数下降 ≥ 8,000。
5. Profiles 双胞胎合并，净删 ≥ 2,000 行，双平台功能等价。
6. invoke 守卫覆盖全 src，存量穿透归零；API 门面双轨消除（spec 同步）。
7. 8 组同名类型重复归零；`<T = Default>` 逃逸按 4.4 短期方案收敛。
8. CodexAuthView ≤ 1,000 行；至少 OpenCodeMcpView + OpenCodeCommandsView 完成 generic 迁移（CodexMcp/Agents 可顺延为 subtask）。
9. CommandsView 5,000 行输出场景无可感知卡顿；AnimatedBackground 无 scale 动画；keep-alive 白名单 ≤ 4。
10. WS6 批次①② 完成且 stylelint + 合同测试锁定；批次③④ 至少完成图表 token 接入与 z-index token 化。
11. eslint 三项收紧生效；tests/ 进 lint + type-check；`no-raw-text` 与 stylelint 规则在 CI 生效。
12. `just frontend-check-quick` 与 `just ui-check` 全绿；bundle 预算门禁通过。

## Verification

```bash
cd ccr-ui && bun run type-check && bun run lint
cd ccr-ui && bun run test:smoke -- tests/api-facade-boundary.smoke.test.ts
cd ccr-ui && bun run test:smoke -- tests/provider-templates.smoke.test.ts
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts \
  tests/apple-glass-surface-contract.smoke.test.ts \
  tests/theme-bootstrap.smoke.test.ts \
  tests/app-settings.smoke.test.ts
just frontend-check-quick   # 每阶段快路径
just ui-check               # 阶段收尾全量
# 视觉回归：Playwright 11 路由 × 3 主题截图对比（批次①②③④ 各跑一次）
```
