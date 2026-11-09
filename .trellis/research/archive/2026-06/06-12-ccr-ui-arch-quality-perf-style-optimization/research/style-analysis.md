# Research: ccr-ui 前端样式问题深度分析

- **Query**: 分析 ccr-ui（Vue 3 + Tauri）前端样式问题：token 遵从度、新旧设计语言残留、样式重复、Tailwind 混用、主题适配缺口、内联样式、z-index、动画、响应式
- **Scope**: internal（`ccr-ui/src/**`，对照 `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md` 与 2026-06-11 外观重设计 PRD）
- **Date**: 2026-06-12

---

## 0. 基准：既定设计语言

来源：

- `.trellis/tasks/archive/2026-06/06-11-ccr-ui-appearance-system-redesign/prd.md`
- `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md`
- `ccr-ui/CLAUDE.md`（Design Context：禁止 Neko/anime/purple-tech/guofeng 分支）

要点：扁平、锐利、卡片 8px 圆角、控件 6–8px 圆角、去厚重玻璃拟态、去发光（glow）、低饱和语义色、pill 仅保留在 chip/状态/开关语义场景；三层主题模型（`data-theme` / `data-flavor`+`data-resolved-flavor` / `data-accent`）必须正交；Catppuccin 通过语义 token 重映射而非第二套组件语言。

重设计第一期只落地了 **Settings 表面**（`AppSettingsView.vue`）+ 合同测试中标记的 4 个迁移视图（`tests/apple-glass-surface-contract.smoke.test.ts:7-12`：`MainLayout.vue`、`DashboardView.vue`、`UsageDashboardView.vue`、`CodexView.vue`，且只挡 `text-white`/`pink-`/`neko-` 等字面量，不挡玻璃/圆角/发光）。

---

## 1. 总体统计速览

| 指标 | 数值 | 说明 |
|---|---|---|
| `.vue` 文件总数 | 211 | `ccr-ui/src/**` |
| 含 `<style>` 块的文件 | 151 | |
| scoped 样式总行数 | **28,623 行** | 平均每文件 ~190 行 |
| `styles/*.css` 总行数 | 4,344 行 | tokens.css 1397 / utilities.css 887 / animations.css 754 |
| 硬编码 hex 颜色 | **72 处 / 19 文件** | 其中 AppSettingsView 15 处为 flavor/accent 预览 swatch（合理） |
| `rgb()/rgba()` 总量 | 2,051 处 / 131 文件 | 含合规的 `rgb(var(--xxx-rgb)/…)` |
| **数字字面量 rgb/rgba（绕过 token）** | **765 处 / 79 文件** | 真正的违规面 |
| `border-radius` px/rem 字面量 | **475 处 / 97 文件** | 对比 `var(--radius-*)` 仅 **40 处** |
| 其中 pill（999/9999px） | 151 处 | |
| 其中 >12px 非 pill 大圆角 | ≈100 处 | 1rem×45、1.1–1.75rem×~40、14–28px×~14 |
| `box-shadow` 总量 | 272 处 / 95 文件 | 其中 `var(--shadow/--elevation)` 117 处（43%） |
| 渐变（linear/radial/conic） | **291 处 / 77 文件** | |
| `backdrop-filter` | **72 处 / 42 文件** | 原始 `blur(Npx)` 30 处，4–88px 不等 |
| `:style="` 动态内联样式 | 441 处 / 65 文件 | |
| z-index 字面量 | 32 处 / 27 文件 | 与 `--layer-*` token 双轨 |
| `@media` 宽度断点 | ~150 处，≈22 种不同值 | 600–1440px 离散散落 |
| `.dark` 类后代选择器 | **116 处 / 6 文件** | 与 `[data-theme="dark"]` 双轨 |
| 组件内 `[data-theme=]` 选择器 | 21 处 / 9 文件 | |
| 组件内 `[data-resolved-flavor]` / `[data-accent]` | **0 处** | flavor 适配完全依赖 token，raw 值无挽救路径 |
| `@apply` | 732 处 / 26 文件 | 三种样式范式并存 |
| `transition: all` | 14 处 / 9 文件 | 总体克制（好） |
| `var(--motion-*)` 使用 | 259 处 | 动效 token 采用率较高（好） |
| `prefers-reduced-motion` | 37 个 vue 文件 + `base.css:301` 全局 kill switch | 覆盖良好（好） |

scoped 样式最大的文件（行数为 `<style>` 块行数）：

| 文件 | style 行数 |
|---|---|
| `src/views/CheckinView.vue` | 1,013 |
| `src/views/CodexAuthView.vue` | 653 |
| `src/views/checkin/CheckinAccountDashboardView.vue` | 619 |
| `src/views/ClaudeCodeProfilesView.vue` | 603 |
| `src/views/checkin/components/AccountFormModal.vue` | 576 |
| `src/components/provider-templates/ProviderTemplateSelector.vue` | 521 |
| `src/views/CommandsView.vue` | 518 |
| `src/views/checkin/tabs/CheckinProvidersTab.vue` | 496 |
| `src/views/GeminiCliView.vue` | 491 |
| `src/views/ClaudeCodeView.vue` | 488 |

---

## 2. P0 — 视觉断裂 / 主题失效

### P0-1 签到（Checkin）模块整体使用外来调色板 + `.dark` 双轨主题，flavor 体系在该模块完全失效

签到模块（`CheckinView.vue` 及 `views/checkin/**` 共 ~10 文件）是 raw 颜色重灾区，合计约 **450+ 处数字字面量 rgb/rgba**（CheckinView 200、CheckinProvidersTab 60、CheckinRecordsTab 56、OAuthWizardModal 56、AccountFormModal 39、AccountsTable 19、CheckinAccountsTab 15…），且颜色值直接取自 **Tailwind 默认调色板**而非项目 clay token：

```css
/* src/views/checkin/tabs/CheckinProvidersTab.vue:678-692 */
.checkin-providers__builtin-card {
  border: 1px solid rgb(191 219 254 / 100%);   /* tailwind blue-200 */
  background: linear-gradient(135deg, rgb(239 246 255 / 100%), rgb(238 242 255 / 100%)); /* blue-50/indigo-50 */
}
.dark .checkin-providers__builtin-card {
  border-color: rgb(75 85 99 / 100%);          /* gray-600 */
  background: linear-gradient(135deg, rgb(31 41 55 / 100%), rgb(55 65 81 / 100%)); /* gray-800/700 */
}
```

```css
/* src/views/CheckinView.vue:837,856 — 高饱和渐变按钮 */
linear-gradient(135deg, rgb(20 184 166 / 96%), rgb(22 163 74 / 98%));  /* teal-500 → green-600 */
linear-gradient(135deg, rgb(37 99 235 / 96%), rgb(79 70 229 / 96%));   /* blue-600 → indigo-600 */
```

后果（这就是「主题失效」级别的理由）：

1. **Catppuccin / paper / graphite flavor 切换对整个签到模块零效果** —— 模块只认 `.dark` 类（`themeBootstrap.ts:109` 同步维护），在 mocha 下渲染的仍是 Tailwind gray-800/blue 系，与 crust/surface0 基底直接撞色。
2. `OAuthWizardModal.vue:657-689` 使用 `rgb(34 197 94)`（green-500）、`rgb(59 130 246)`（blue-500）、`rgb(161 161 170)`（zinc-400）**且无任何 `.dark`/`data-theme` 变体** —— 单一调色板同时服务亮暗两套主题。
3. 高饱和 teal/green/blue/indigo 直接违反「低饱和语义色」要求。
4. `.dark` 后代选择器共 116 处集中在 6 个文件（CheckinView 54、CheckinRecordsTab 29、CheckinProvidersTab 18、AccountFormModal 12、AccountsTable 2、MainLayout 1），构成第二套主题机制。

### P0-2 `checkin-shared.css` 作为全局样式层固化旧玻璃语言

`src/styles/index.css:7` 将 `checkin-shared.css` 与 `core.css` 并列为**首屏全局样式**，其内容与新设计语言逐条对立：

```css
/* src/styles/checkin-shared.css:8-23 */
.checkin-surface-card {
  border: 1px solid rgb(255 255 255 / 20%);   /* 白色玻璃边框：light 主题下几乎不可见 */
  border-radius: 1.5rem;                      /* 24px，规范上限是 8px */
  background: var(--glass-bg, rgb(255 255 255 / 8%));
  box-shadow: 0 10px 30px rgb(15 23 42 / 10%);
  backdrop-filter: blur(20px);                /* 重度玻璃拟态 */
}
.checkin-badge-pill { border-radius: 9999px; }
```

`rgb(255 255 255 / 20%)` 边框在 light/latte 主题下与浅色底几乎无对比（卡片边界消失），属于可见的视觉断裂点。

---

## 3. P1 — 明显不一致

### P1-1 悬空 token 引用（静默失效）+ `tokens.ts` 镜像漂移

以下变量**在任何 CSS 中均未定义**（tokens.css 定义的是 `--glow-primary` / `--ease-out` 等），引用方静默失效：

| 悬空变量 | 引用位置 | 后果 |
|---|---|---|
| `--shadow-glow-primary` | `tailwind.config.ts:112`、`components/LanguageSwitcher.vue:144`、`components/ui/EmptyState.vue:24`（`hover:shadow-glow-primary`）、`styles/tokens.ts:51` | hover 阴影完全不渲染 |
| `--shadow-glow-success` / `--shadow-glow-danger` | `tailwind.config.ts:113-114` | 同上（生成的 utility 无效） |
| `--ease-default` | `tailwind.config.ts:122`、`styles/tokens.ts:90` | Tailwind 默认 timing function 失效，回退 `ease` |
| `--color-border-interactive-rgb` | `tailwind.config.ts:81` | `border-interactive` utility 无效（暂无人用，属地雷） |

`styles/tokens.ts`（TS 镜像）与 `tokens.css` 已经漂移，存在引用不存在变量的字段。

### P1-2 Shell（MainLayout）与绝大多数视图未迁移，旧玻璃/发光/大圆角语言仍是全局主调

PRD Phase 4（Shell 对齐）未执行。`src/components/MainLayout.vue` 现状：

- `MainLayout.vue:511-512` nav-item 仍是 `rounded-2xl`（16px）+ 渐变背景；
- `MainLayout.vue:538-545` 激活态用发光：`box-shadow: 0 14px 28px rgb(accent / 10%)`；
- `MainLayout.vue:559-585` settings-dock：`backdrop-filter: blur(14px) saturate(116%)`、hover `0 20px 40px` 发光、radial-gradient accent mesh、uppercase pill（`settings-dock-pill` `rounded-full` + `tracking-[0.12em] uppercase`）。

全局残留量化（即「重设计未覆盖面」）：

- **backdrop-filter 72 处 / 42 文件**；原始 blur 值热点：`blur(20px)`×5（ClaudeCodeProfilesView:1199、CodexProfileEditorModal:915、ScrollToTopButton:61、MultiSelectFloatingBar:77、McpListPanel:344）、`blur(84-88px)` 背景光晕×3（AnimeBackground:39、AnimatedBackground:119、PageHeaderCard:145）、`blur(24px)`×2（ProviderStatsModal:12-13）。
- **大圆角（>12px 非 pill）≈100 处**；极端值：`CodexTrayPanelView.vue:140` `border-radius: 28px`、`TrayOverview.vue:273` 22px、`TrayAccountSwitchScreen.vue:231` 22px、`CommandPalette.vue:338` 18px、`ProviderTemplateSelector.vue:744,755` 18px。tray/ 子树整体是「上一代」大圆角+玻璃语言。
- **pill 形 151 处**（999/9999px），其中大量用于按钮/输入框而非 chip 语义，如 `PageHeaderCard.vue:144,249`、`ScrollToTopButton.vue:54`、`PricingView.vue:683-775`。
- **渐变 291 处 / 77 文件**；连共享 primitive `ui/Button.vue` 的 primary/secondary/accent 三个 variant 仍是 `linear-gradient(180deg, …)` + `0 8px 16px accent/12%` 发光（`Button.vue:168-202`），与「flatter、active 态靠 border/inset 而非 glow」的要求相悖。
- glass utility 兼容别名（`.glass-effect`、`.liquid-glass`、`.glass-modal`、`.glass-elevated`）仍由 `tailwind.config.ts:211-227` 持续输出，旧写法零成本延续。
- 合同测试 `apple-glass-surface-contract.smoke.test.ts` 的 `migratedViewPaths` 只有 4 个文件，对其余 200+ 文件无约束。

### P1-3 图表/图标色硬编码 Tailwind 调色板，绕过已存在的 chart token

`styles/chart-colors.css` 已定义 `--chart-color-0..4`（接 token），但：

- `components/TokenUsageChart.vue:85-271`：SVG `stop-color`/`stroke`/`fill` 硬编码 `#3b82f6` / `#10b981` / `#f59e0b`（12 处）；
- `components/usage/StatCard.vue:172-180`：colorMap 硬编码 `#3B82F6` / `#10B981` / `#8B5CF6` / `#EF4444` 等 7 色；
- `components/HistoryList.vue:229-237`：操作类型色硬编码 8 个 hex（含 `#8b5cf6` 紫——品牌明确禁用的 purple-tech 轴）。

这些颜色在所有 theme/flavor/accent 下纹丝不动，且饱和度远超语义色规范。

### P1-4 圆角 token 采用率 8%，值域完全失控

`border-radius` 字面量 475 处 vs `var(--radius-*)` 40 处。一次性魔法值长尾：`0.82rem`、`0.88rem`、`0.95rem`、`1.1rem`、`1.15rem`、`1.18rem`、`1.28rem`、`1.35rem`、`1.45rem`、`1.55rem`、`1.6rem`、`1.75rem`……约 30 种互不相同的值。这意味着「把 token 圆角调小即可全局收紧」的 token-first 策略对 92% 的圆角声明不生效——重设计降半径的成果只体现在用 token/`rounded-lg` 的少数文件。

### P1-5 遗留品牌分支（neko/anime）未清除，违反设计基线

`ccr-ui/CLAUDE.md` 明确「现存相关风格视为待移除的历史遗留」，现状：

- `styles/neko-decorations.css`：**350 行，无任何 import 引用，纯死代码**；
- `styles/animations.css:288-543`：`neko-press` / `neko-ear-wiggle` / `neko-tail-wag` / `neko-float` / `neko-breathe` 等 keyframes 与 `.animate-neko-*` 类，经 `deferred-interactive.css` **对所有用户全局加载**；
- `tailwind.config.ts:141-165`：`neko-*` keyframes + `neko-breathe` 的粉色发光 `rgb(244 114 182 / 30%)`（pink-400，撞「禁止 pink-」红线，只是合同测试不覆盖 config 文件）；
- `styles/backgrounds.css:268-289`：`.bg-neko-grid` / `.bg-cyber-grid` 兼容类；
- `components/common/AnimeBackground.vue`：组件名遗留（内容已重写为 `claude-background` 光晕，但 `App.vue:30` 仍以 AnimeBackground 名义全局挂载）；同目录还有功能重叠的 `AnimatedBackground.vue`（27 处 rgba、21 处渐变、blur(88px)）与 `BackgroundImage.vue`（读 `.dark` 类）三件套。

### P1-6 模块级成对复制：claude/profiles 与 codex/profiles 几乎镜像

| claude 侧 | codex 侧 | style 行数对比 |
|---|---|---|
| `claude/profiles/ClaudeProfilesContextRail.vue` | `codex/profiles/ProfilesContextRail.vue` | 409 vs 413 |
| `ClaudeProfilesToolbar.vue` | `ProfilesToolbar.vue` | 同构（max-width:380px、z-index:5、pill 配方均相同） |
| `ClaudeProfilesStatStrip.vue` | `ProfilesStatStrip.vue` | 同构 |
| `ClaudeProfileListRow.vue` / `ClaudeProfileRow.vue` | `ProfileRow.vue` / `ProfileCard.vue` | 同构 |
| `ClaudeProfilesHeader.vue` | `ProfilesHeader.vue` | 同构 |

两套文件的 badge/pill/rail/toolbar 样式块基本是复制粘贴（小改色值），是「样式重复」最大的结构性热点；usage/ 模块 17 个组件（UsageOverviewTab 432 行等）同样各自重复 panel/chip/legend 配方。modal 家族 19 个文件中 `BaseModal.vue` 已被 24 处采用，但 `AddConfigModal` / `EditConfigModal` / `UpdateModal` / `CommandFormModal` / `UnifiedMcpFormModal` / `UnifiedMcpDeleteConfirmModal` / `ProviderStatsModal` / `GlobalConfirmDialog` 仍自滚 backdrop+panel（`modal-backdrop|overlay` 自定义实现 16 处 / 7 文件）。

### P1-7 三种样式范式并存，且与 `.dark` 双轨叠加

1. 模板 Tailwind utility（部分新文件 + EmptyState 等）；
2. scoped CSS + `@apply`（732 处，集中在 26 个文件：CommandsView 79、ClaudeCodeView 75、CodexView 69、GeminiCliView 63、OpenCodeView 61、AppSettingsView 52…）；
3. 纯手写 scoped CSS（约 125 个文件、占 28.6k 行的大头）。

同类页面随作者/时期在三种范式间漂移；`tailwind.config.ts:5` `darkMode: ['class', '[data-theme="dark"]']` 加上 `themeBootstrap.ts:109` 同步 `.dark` 类，使「哪种暗色选择器是正统」没有单一答案（合规写法应只剩 `[data-theme="dark"]` 与 token）。

---

## 4. P2 — 可打磨

### P2-1 z-index 双轨但量小

`--layer-*` token 体系定义完整（`tokens.css:574-594`），`MainLayout.vue:480-486` 等已采用；但仍有 32 处字面量 / 27 文件：modal 类组件用 `z-index: 50`（`UnifiedMcpFormModal.vue:359`、`ClaudeAuthView.vue:742`、`CheckinProvidersTab.vue:972`）高于 token 的 `--layer-modal: 40`，与 `--layer-popover: 50` 撞层；`AccountActionsMenu.vue:170` 用 60 撞 `--layer-tooltip`。模板里另有 35 处 Tailwind `z-10/20/50`。目前未观察到实际叠层 bug，但层级语义已经混乱。

### P2-2 动效时长/缓动散落（结构良好、细节欠收敛）

好的一面：`var(--motion-*)` 259 处、`transition: all` 仅 14 处、raw `cubic-bezier()` 仅 3 处、`base.css:301` 有全局 reduced-motion kill switch（`!important` 全杀）+37 文件局部降级。欠收敛：raw 时长（`0.2s`/`0.3s`/`2s` 等）约 77 处散落，`var(--duration-*)`/`var(--ease-*)` 直接使用仅 15/11 处——非 motion-token 路径的 transition 仍靠手写数值。

### P2-3 响应式断点无约定

~150 个 `@media` 宽度查询使用约 22 种不同断点（600/639/640/680/700/720/760/767/768/860/900/960/980/1024/1080/1100/1180/1240/1280/1440…）。CSS 媒体查询无法引用 var() 是客观限制，但当前连「桌面三档」之类的书面约定都没有，同一容器在相邻视图收缩时机不同。固定宽度风险面较小：`max-width: 380–540px` 共 7 处（toolbar/modal，合理），未发现明显的固定大宽度溢出源；主要风险是 `AppSettingsView` 式 `min-w-[220px]` nav 在窄窗时依赖横向滚动的模式未统一。

### P2-4 内联样式偏多但大部分必要性存疑参半

`:style="` 441 处 / 65 文件。Top：`ConverterView.vue` 70、`generic/PlatformMcpView.vue` 42、`UpdateModal.vue` 38、`generic/PlatformPluginsView.vue` 32、`BaseSlashCommands.vue` 26、`ProviderStatsModal.vue` 25、`Navbar.vue` 20、`StatusHeader.vue` 18。其中进度条宽度、图表几何等为合理动态值；但 generic 视图与 ConverterView 中大量为静态色/间距/字号（可移入 class 或 CSS 变量注入模式 `style="--x: …"`）。

### P2-5 缺少防回归机制

stylelint 仅用 `stylelint-config-standard`（无 token 强制规则）；合同测试只覆盖 4 文件的字面量黑名单。没有任何机制阻止新增 `blur(20px)`、`border-radius: 1.5rem`、raw Tailwind 调色板值。

---

## 5. 健康面（不需要动的部分）

- `tokens.css` 三层主题模型实现完整：theme/flavor/accent 正交，Catppuccin 走语义重映射，`html:root[data-resolved-flavor="mocha"]` 高优先级覆盖符合 spec 要求。
- `theme.css` 兼容桥干净（全部转发到新 token）。
- `tailwind.config.ts` 颜色/间距/圆角/时长全部映射到 CSS 变量（`<alpha-value>` 模式正确）。
- `ui/Card.vue` 已是新语言范本：surface token + elevation/motion 分级 + reduced-motion + focus-visible。
- `home.css` 与 dashboard/home 组件群已 token 化（局部 `--home-*` 语义层）。
- 动效纪律整体好于颜色纪律（`transition: all` 几乎绝迹、全局 reduced-motion 兜底）。

---

## 6. 优化建议（按收益/成本排序）

| # | 动作 | 严重度 | 收益 | 成本 | 是否属于「重设计推广」范围 |
|---|---|---|---|---|---|
| 1 | 修复 4 个悬空 token（`--shadow-glow-*`→`--glow-*` 或补定义、`--ease-default`、`--color-border-interactive-rgb`），同步或删除 `styles/tokens.ts` 镜像 | P1-1 | 中（消灭静默失效） | **极低**（半小时级） | 否，独立 bugfix |
| 2 | 删除 neko/anime 死代码：`neko-decorations.css`（整文件）、`animations.css` neko 块、`tailwind.config.ts` neko keyframes、`.bg-neko-grid/.bg-cyber-grid`；`AnimeBackground.vue` 更名（如 `StageBackground.vue`）并与 `AnimatedBackground.vue`/`BackgroundImage.vue` 三合一 | P1-5 | 中（-700+ 行死代码、品牌合规、减小 deferred CSS） | 低 | 是（清场） |
| 3 | **签到模块色彩迁移**：~450 处 raw Tailwind 色 → 语义 token；删除全部 `.dark` 后代选择器；重写 `checkin-shared.css` 为新语言（8px、不透明 surface、去 blur(20px)） | **P0-1/P0-2** | **最高**（修复 flavor 失效 + 最大违规簇 + light 主题断点） | 中高（6 文件 ~450 处，但机械替换为主） | **是（第一优先迁移面）** |
| 4 | 图表色接 `--chart-color-*`：TokenUsageChart / StatCard / HistoryList / usage 系列 | P1-3 | 高（所有主题下图表换肤生效） | 低中 | 是 |
| 5 | Shell 对齐（PRD Phase 4 补课）：MainLayout nav `rounded-2xl`→`--radius-lg`、去发光阴影、settings-dock 去 blur/mesh/uppercase pill | P1-2 | 高（全局视觉基调，每屏可见） | 中 | **是（第二优先迁移面）** |
| 6 | 共享 primitive 去旧语言：`Button.vue` primary/secondary 去渐变与 glow 阴影；收敛 `Card.vue` 的 glow/gradientBorder/pattern 装饰 prop（标记 deprecated）；glass 别名 utility（`.glass-effect` 等）标记废弃并逐步替换为 `.surface-*` | P1-2 | 高（一处改全局受益） | 中（需视觉回归检查） | 是 |
| 7 | 圆角收敛：475 处字面量 → `--radius-*`（脚本辅助映射：≤4→sm、6→md、8→lg、10→xl、12→2xl、>12 降档至 lg/xl；pill 仅保留 chip/badge/toggle 语义处） | P1-4 | 高（重设计「降半径」真正全局生效） | 中（量大但可半自动） | 是 |
| 8 | tray/、profiles×2、PricingView、ProviderTemplateSelector、CommandPalette 等大圆角+玻璃热点视图按新语言重刷；claude/codex profiles 成对组件抽公共样式或公共组件 | P1-2/P1-6 | 中高（消除最显眼的旧语言孤岛 + 去重 ~2k 行） | 高 | 是（第三批） |
| 9 | modal 家族收敛到 `BaseModal`（8 个自滚 backdrop 的迁移），z-index 全部改 `var(--layer-*)` | P1-6/P2-1 | 中 | 中 | 是 |
| 10 | 防回归：stylelint 增加规则（禁 `backdrop-filter` 字面量 blur、禁 `border-radius` 字面量、`color-no-hex` 于 vue、禁 `.dark ` 前缀选择器），并把 `apple-glass-surface-contract` 的 `migratedViewPaths` 改为「全量 - 白名单」模式随迁移逐步收紧 | P2-5 | 高（锁住一切上述成果） | 低 | 是（收尾必做） |
| 11 | 断点约定书面化（如 720/960/1280 三档）+ 新增代码遵守；存量不强迁 | P2-3 | 低中 | 低 | 可选 |
| 12 | `:style=` 静态值清理（ConverterView/generic 视图优先），动态值统一 `style="--var:"` 注入模式 | P2-4 | 低 | 中 | 可选 |

**「将重设计语言推广到全部视图」的建议圈定范围**（按批次）：
① Checkin 全模块（P0）→ ② MainLayout shell + Button/Card primitive（杠杆最大）→ ③ tray/、claude+codex profiles、Pricing、ProviderTemplateSelector、CommandPalette、usage/ 模块 → ④ modal 家族 + 防回归锁定。建议每批扩 `migratedViewPaths` 并增设 blur/radius/raw-color 的 stylelint 守卫，避免重蹈「只重设计 Settings、其余继续漂移」的覆辙。

---

## 7. Caveats / Not Found

- 本机 ripgrep 安装损坏（scoop shim 指向不存在的 `rg.exe`，实际回落到 GNU grep 3.0），统计改用 Claude Code 内置 Grep（真 ripgrep）+ GNU grep 校验，关键数字均经两道交叉确认；个别 bash 端 grep 调用受 rtk hook 改写影响被弃用。
- 「数字字面量 rgb/rgba」统计含少量合理用例（如纯黑阴影 `rgb(0 0 0 / x%)`、白色高光 inset），未逐条剔除；保守估计合理用例占 10–15%。
- 未运行浏览器视觉验证（本研究为静态代码分析）；P0-2 中 light 主题下白边框不可见的结论基于色值推断，建议实施前用 `bun run dev:web` 在 latte/light 下复核 Checkin 页面截图。
- `styles/tokens.ts` 的完整漂移面未逐字段比对，只确认了 2 个悬空引用字段。
- Storybook（已配置）与样式体系的关系未调查。
