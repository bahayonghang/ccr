# 组件表面迁移 — 执行计划

> 前置确认：`07-28-color-system-rebuild` 已归档或其令牌变更已合入当前工作区。
> 每批：改 → `bun run type-check && bun run lint` → 聚焦 smoke → 截图对比。

## Step 0 — 建立基线与白名单表

- `rg "bg-bg-(base|elevated|surface|overlay)/" ccr-ui/src -l` 全量清单落档到本文件下方"白名单与残留登记"。
- 对 Overview/Profiles/Monitoring/Usage 各截一张 before 图（dark+neutral）。

## Step 1 — B1 重灾区 9 文件

按 design.md §1 映射表迁移：`CodexAgentSourcesPanel.vue`、`MonitoringView.vue`、`OpenCodeAgentsView.vue`、`OpenCodeSettingsView.vue`、`OpenCodeMcpView.vue`、`CodexAgentsView.vue`、`OpenCodeProvidersView.vue`、`OpenCodePluginsView.vue`、`AgentsView.vue`。

## Step 2 — B2 其余 alpha 表面文件

`rg` 全量 − B1 − 白名单，逐文件迁移。完成后 AC1 扫描应仅剩白名单。

## Step 3 — B3 文本透明度 + inset 高光 + text-white

- 11 处直写透明文本 + 5 处 Tailwind 文本透明度类。
- ~30 处 inset 白高光（暗色删、亮色收敛进 `--inner-glow`）。
- `utilities.css` `.btn-*` 文字色 + 62 处 `text-white` 审计。

## Step 4 — B4 壳层/模态/图表

- `MainLayout.vue` 侧栏/顶栏不透明验证（计算样式证据）。
- `BaseModal.vue` → `--surface-modal-*`；`glass-panel` → card 语义；`ClaudeProfileRow.vue` 滚动区玻璃移除。
- `chart-colors.css` 五色校准。

## Step 5 — 全量回归

- `cd ccr-ui && bun run type-check && bun run lint && bun run test:smoke`。
- `just frontend-check-quick`。
- AC1-AC4 的全部 rg 扫描结果落档；after 截图 4 张存父任务 research/。

## 白名单与残留登记

### Step 0 基线扫描（2026-07-28）

`rg "bg-bg-(base|elevated|surface|overlay)/" ccr-ui/src`：**247 处 / 43 文件**。

| 文件 | 处数 |
|---|---|
| components/codex/CodexAgentSourcesPanel.vue | 19 |
| views/MonitoringView.vue | 18 |
| views/OpenCodeAgentsView.vue | 16 |
| views/OpenCodeSettingsView.vue | 15 |
| views/OpenCodeMcpView.vue | 13 |
| views/codex/CodexAgentsView.vue | 12 |
| views/OpenCodeProvidersView.vue | 12 |
| views/OpenCodePluginsView.vue | 11 |
| views/generic/AgentsView.vue | 11 |
| views/OpenCodeView.vue | 10 |
| views/OpenCodeCommandsView.vue | 10 |
| components/codex/CodexAccountCard.vue | 9 |
| views/ConfigsView.vue | 8 |
| views/generic/AgentDetailView.vue | 7 |
| views/SkillsMigrationView.vue | 7 |
| components/MainLayout.vue | 7 |
| views/CodexSessionsView.vue | 6 |
| views/PluginsView.vue | 5 |
| views/CodexSlashCommandsView.vue | 4 |
| views/OutputStylesView.vue | 4 |
| components/layout/Titlebar.vue | 4 |
| views/HooksView.vue | 3 |
| views/StatuslineView.vue | 3 |
| components/McpPresetsPanel.vue | 3 |
| components/EnvironmentSwitcher.vue | 3 |
| components/claude/ClaudeProfileRow.vue | 3 |
| views/codex/tabs/CodexAuthProvidersTab.vue | 2 |
| views/codex/components/SaveCodexSessionModal.vue | 2 |
| views/codex/components/AddCodexAccountModal.vue | 2 |
| components/McpSyncPanel.vue | 2 |
| components/HistoryList.vue | 2 |
| components/common/MarketplacePagination.vue | 2 |
| components/codex/CodexAgentEditorModal.vue | 2 |
| views/CommandsView.vue | 1 |
| views/codex/components/RenameCodexAccountModal.vue | 1 |
| components/EnvironmentBadge.vue | 1 |
| components/common/BaseModal.vue | 1 |
| components/codex/CodexProfileEditorModal.vue | 1 |
| components/opencode/OpenCodePageShell.vue | 1 |
| components/ui/AsyncStatePanel.vue | 1 |
| components/ui/Breadcrumb.vue | 1 |
| components/ui/Badge.vue | 1 |
| components/ui/NavItem.vue | 1 |

scrim 处理：`tokens.css` 原无 `--color-scrim`，已在 `:root`（`rgb(25 27 32 / 32%)`）与 `[data-theme='dark']`（`rgb(0 0 0 / 56%)`）新增，紧邻 surface 契约区。`theme-contrast-contract.smoke.test.ts` 的 alpha 扫描正则只匹配 `--color-(text|bg|stage-text|stage-surface)-*` 与 `--surface-(card|workspace)-bg`，不含 `--color-scrim`，无需加例外。

### 白名单（保留 alpha 的条目）

| 文件:行 | 写法 | 保留理由 |
|---|---|---|
| views/MonitoringView.vue:304 | `hover:bg-bg-elevated/60` | 交互态 alpha（日志行 hover 反馈） |
| views/codex/CodexAgentsView.vue:253 | `hover:bg-bg-surface/75` | 交互态 alpha（agent 卡片 hover 反馈） |
| views/generic/AgentsView.vue:121 | `hover:bg-bg-base/35` | 交互态 alpha（清除搜索按钮 hover 反馈） |
| views/generic/AgentsView.vue:314 | `bg-bg-base/40 backdrop-blur-[2px]` | 刻意透视设计：disabled 卡片遮罩层，需透出下方卡片内容 |

### Step 1 — B1 迁移记录

| 文件 | 迁移处数 | 目标 |
|---|---|---|
| components/codex/CodexAgentSourcesPanel.vue | 19 | surface/55,60,70→elevated(18)；elevated/70→base(1) |
| views/MonitoringView.vue | 17 | elevated/80→elevated(4)；surface/80→surface(3)；surface/55,60,68,70→elevated(9)；elevated/50→base(1)；hover 保留 1 |
| views/OpenCodeAgentsView.vue | 16 | base/35,45→base(16) |
| views/OpenCodeSettingsView.vue | 15 | base/35,45→base(15) |
| views/OpenCodeMcpView.vue | 13 | base/35,45→base(13) |
| views/codex/CodexAgentsView.vue | 11 | surface/35,55,60,70→elevated(8)；elevated/70→base(3)；hover 保留 1 |
| views/OpenCodeProvidersView.vue | 12 | base/35,45→base(12) |
| views/OpenCodePluginsView.vue | 11 | base/35,45→base(11) |
| views/generic/AgentsView.vue | 9 | surface/700(无效类 bug)→surface(4)；surface/50,70→elevated(3)；elevated/50→base(1)；overlay/20→`bg-[color:var(--color-scrim)]`(1，modal 真 scrim)；hover 保留 1、disabled 遮罩保留 1 |

合计迁移 123 处，白名单 4 处。

### Step 2 — B2 迁移记录（2026-07-28）

迁移后全仓扫描残留 **22 处 / 13 文件** = B1 白名单 4 处 + B2 交互态保留 18 处，全部对账通过。B2 共迁移 **102 处 / 31 文件**：

| 文件 | 迁移处数 | 目标 |
|---|---|---|
| components/claude/ClaudeProfileRow.vue | 3 | elevated/88,72→elevated(2)；surface/72→surface(1) |
| components/HistoryList.vue | 2 | surface/70,60→elevated(2) |
| components/common/BaseModal.vue | 1 | dark:elevated/90→dark:elevated(1)；bg-white/80 亮色侧留 Step 4 统一换 `--surface-modal-*` |
| components/EnvironmentBadge.vue | 1 | surface/700(无效类)→surface(1) |
| components/codex/CodexProfileEditorModal.vue | 1 | elevated/34→base(1) |
| components/codex/CodexAgentEditorModal.vue | 2 | surface/70,60→elevated(2) |
| components/codex/CodexAccountCard.vue | 8 | surface/70→elevated(6)；overlay/50→surface(1)；elevated/80→elevated(1)；hover 保留 1 |
| components/MainLayout.vue | 3 | surface/80→surface(2)；elevated/82→elevated(1)；hover/group-hover 保留 4 |
| components/McpSyncPanel.vue | 2 | surface/50→elevated(2) |
| components/McpPresetsPanel.vue | 3 | surface/50→elevated(2)；surface/70→elevated(1) |
| components/opencode/OpenCodePageShell.vue | 1 | surface/70→elevated(1) |
| components/layout/Titlebar.vue | 1 | surface/80→surface(1)；hover/active(isMenuOpen) 保留 3 |
| components/ui/Badge.vue | 1 | overlay/70→surface(1) |
| components/ui/AsyncStatePanel.vue | 1 | surface/90→surface(1) |
| components/ui/NavItem.vue | 1 | elevated/80→elevated(1)（hover 渐显层自带 opacity 过渡，底色改实心） |
| views/HooksView.vue | 3 | surface/60→elevated(2)；elevated/50→base(1) |
| views/OpenCodeCommandsView.vue | 10 | base/35,45→base(10) |
| views/SkillsMigrationView.vue | 6 | elevated/70→base(1)；surface/70→elevated(2)；overlay/70→surface(1)；base/60,50→base(2)；hover 保留 1 |
| views/ConfigsView.vue | 8 | elevated/70→base(1)；surface/70→elevated(1)；surface/60→elevated(5)；elevated/80→elevated(1) |
| views/CommandsView.vue | 1 | surface/70→elevated(1) |
| views/PluginsView.vue | 5 | surface/700(无效类)→surface(4)；surface/50→elevated(1) |
| views/OpenCodeView.vue | 10 | base/35,40,45,55→base(9)；elevated/60→base(1) |
| views/CodexSlashCommandsView.vue | 3 | surface/70→elevated(3)；hover 保留 1 |
| views/OutputStylesView.vue | 3 | surface/50→elevated(3)；hover 保留 1 |
| views/CodexSessionsView.vue | 5 | surface/70→elevated(4)；base/35→base(1)；hover 保留 1 |
| views/generic/AgentDetailView.vue | 7 | surface/70→elevated(2)；surface/50→elevated(2)；surface/700(无效类)→surface(2)；elevated/50→base(1) |
| views/StatuslineView.vue | 3 | surface/50→elevated(2)；surface/700(无效类)→surface(1) |
| views/codex/tabs/CodexAuthProvidersTab.vue | 2 | surface/70→elevated(1)；surface/40→elevated(1) |
| views/codex/components/SaveCodexSessionModal.vue | 2 | elevated/95→elevated(1)；surface/70→elevated(1) |
| views/codex/components/RenameCodexAccountModal.vue | 1 | surface/70→elevated(1) |
| views/codex/components/AddCodexAccountModal.vue | 2 | elevated/95→elevated(1)；surface/40→elevated(1) |

B2 交互态保留（hover/group-hover/active 反馈，按类统计不逐条登记）：**18 处** —— EnvironmentSwitcher(3)、MarketplacePagination(2)、CodexAccountCard(1)、MainLayout(4)、Titlebar(3，含 isMenuOpen active 态 1)、Breadcrumb(1)、SkillsMigrationView(1)、CodexSlashCommandsView(1)、OutputStylesView(1)、CodexSessionsView(1)。

B2 无新增逐条白名单（无"刻意透视"存疑项）。

#### 实心化后残留 backdrop-blur（本批次不删，归 Step 4 统一评估）

- components/claude/ClaudeProfileRow.vue:3 `backdrop-blur-xl`
- components/common/BaseModal.vue:231 `backdrop-blur-xl backdrop-saturate-150`（连同 `bg-white/80` 一起归 Step 4 换 `--surface-modal-*`）
- components/opencode/OpenCodePageShell.vue:168 `backdrop-blur-md`
- views/ConfigsView.vue:51 `backdrop-blur-md`
- views/codex/components/SaveCodexSessionModal.vue:13 `backdrop-blur`
- views/codex/components/AddCodexAccountModal.vue:18 `backdrop-blur`

### Step 3 — B3 迁移记录（2026-07-28）

#### 3.1 直写半透明文本（33 处 → 0）

- 直写 color → 实心令牌：Titlebar.vue ×4（secondary/88%→secondary、primary/98%→primary、muted/90%、muted/85%→muted）、McpCreatePanel.vue:416 placeholder→ghost、ListSearchHeader.vue:87 placeholder→ghost。
- 组件局部语义变量实心化（96-98%→primary、90-92%→secondary、82-86%→muted、72-74% placeholder→ghost）：ClaudeCodeProfilesView.vue `--editor-ink*` ×4、CodexProfileEditorModal.vue ×4、CodexAgentEditorModal.vue `--agent-*` ×4、ConfirmModal.vue `--confirm-text-*` ×4（亮暗两块各 2）。
- SyncView.vue:718,956 border → `var(--color-border-subtle)`。
- Titlebar.vue:398 hover 背景 → `rgb(var(--color-bg-overlay-rgb) / 55%)`（交互态 alpha，bg 令牌引用）、:399 ring → `var(--color-border-subtle)`。
- DashboardNextActions.vue:223,224,234,238 → `rgb(var(--color-accent-primary-contrast-rgb) / N%)`（accent 实心底上的次级文字/描边，见白名单）。
- base.css:156 滚动条 thumb → `var(--color-border-strong)`（与 Firefox fallback 对齐）。
- ClaudeCodeProfilesView.vue:1366 阴影 4% 项 → `rgb(0 0 0 / 4%)`。
- Tailwind 文本透明度类 ×4：SyncAccountDialog.vue:397 → `text-text-ghost`；Input.vue:33 placeholder → `text-text-ghost`；Breadcrumb.vue:27 → `text-text-ghost`；ClaudeCodeView.vue:728 水印图标 → `text-text-disabled`/`group-hover:text-text-ghost`。
- AC2 扫描 `rgb(var(--color-text-[a-z]+-rgb) /` 与 `text-text-*/N`：均为 0。

#### 3.2 inset 白高光（36 处消费者 → 0）

- 删除 31 处（暗色一律删；亮/双主题 ≤46% 视觉收敛删除）：UsageMetricCard ×2、ConfigFilters、UsageDashboardToolbar ×2、CodexProfileEditorModal ×3（797,833,834,954 计 4，其中 797 为亮色 42%）、ClaudeCodeProfilesView ×3（1387,1446,1596）、TrayOverview、CodexTrayPanelView、PlatformUsageInsightPanel、UsageDashboardView、Input、PricingView、UsageTokenBreakdownStrip、UsageCostConclusionCard、UsageStaleBanner、UsageOverviewTab、ThemeToggle、Button(glass 14%)、GeminiCliView ×2、utilities.css:241(nav-item-active-glow 20%)、ClaudeProfileRow:241(rgba 逗号式 6%)、ClaudeCodeView ×3（500/513 反向语法 28%/32% warning 按钮、635 暗色 5%）。
- 收敛 `var(--inner-glow)` 5 处（亮色 >46%）：PageHeaderCard.vue:157（66%）、EmptyState.vue:67（70%）、CodexProfileEditorModal.vue:796（64%）、ClaudeCodeProfilesView.vue:1366（surface 60% inset 同效写法）、ClaudeCodeView.vue:550（72% 反向语法）。
- AC2 扫描 `inset 0 1px 0 rgb(255`（含 `rgba(,255` 与 `0 1px 0 ... inset` 反向语法扩展扫描）：消费者 0 残留；tokens.css 内令牌定义（--inner-glow/--shadow-inner/--material-glass-*-highlight/--glass-inner-glow/--liquid-glass-highlight）按契约保留，不在消费者扫描范围。

#### 3.3 按钮白字与 text-white 审计

- tokens.css 增补对比文字令牌（4 区块：neutral light/dark、latte、mocha，各含 `-rgb` 变体，注释写明对比度）：
  - `--color-danger-contrast`：light `#fff8f2`（vs #c76953 = 3.58:1）；dark `#17181c`（vs #db8a73 = 6.74:1）；latte `#fff8f2`（vs ctp-red #d20f39 = 5.43:1）；mocha `var(--ctp-crust)`（vs ctp-red #f38ba8 ≈ 7.3:1）。
  - `--color-success-contrast`：light `#fff8f2`（vs #5b8a62 = 3.79:1）；dark `#17181c`（vs #7cab82 = 6.85:1）；latte `#17181c`（vs ctp-green #40a02b = 5.37:1，近白仅 3.34:1 不达标）；mocha `var(--ctp-crust)`（vs ctp-green #a6e3a1 ≈ 11.3:1）。
  - `--color-warning-contrast`（超出 brief 的增补，判断依据：ConfirmModal/CheckinView/CheckinProvidersTab 存在 warning 实心底白字，近白仅 ~3.2:1 不达标且无现成令牌）：light `#17181c`（vs #bc8540 = 5.61:1）；dark `#17181c`（vs #d6a76d = 8.21:1）；latte `#17181c`（vs ctp-yellow #df8e1d = 6.86:1）；mocha `var(--ctp-crust)`（vs ctp-yellow #f9e2af ≈ 15.5:1）。clay 无 danger/success/warning 覆盖，自动继承 neutral 值。
- theme.css 桥接：`--accent-success-contrast` / `--accent-warning-contrast` / `--accent-danger-contrast`。
- utilities.css：`.btn-primary/.btn-danger/.btn-success` `color: white` → 对应 contrast 令牌；AC3 扫描 `color: white` in styles = 0。
- `text-white` 61 处迁移（26 文件）：accent/功能色实心底 → `text-[color:var(--color-accent-primary-contrast)]`（primary/secondary/渐变底）、`text-[color:var(--color-success-contrast)]`（McpSyncPanel、UpdateModal:387）、`text-[color:var(--color-text-inverted)]`（AgentsView:316 muted badge）；非 accent 底（bug）→ `text-text-primary`（WslManagementView ×8、SshManagementView ×5、HistoryList ×4、StatCard、MarketplacePagination hover ×2、NavItem hover）；无效 hover:text-white（base 已是 text-primary）直接删除。
- 扩展扫描发现组件内 `color: white` / `color: '#fff'` 20 处，19 处同规则迁移（accent-primary→primary-contrast、warning→warning-contrast、danger→danger-contrast、success→success-contrast；CheckinView:804 基类白字拆入 --checkin/--balance 两个变体）；1 处白名单（BudgetView:557）。
- CodexSettingsView toast：基类 `text-white` 移除，`--success`/`--error` 变体各补对应 contrast 色；Titlebar 关闭按钮 hover `rgb(255 255 255 / 98%)` → `var(--color-danger-contrast)`。

### B3 白名单（保留项登记）

| 文件:行 | 写法 | 保留理由 |
|---|---|---|
| components/ConfigCard.vue:25 | `text-white` | 固定品牌渐变头像（cyan/violet/amber Tailwind 固定色，双主题同底，非主题令牌底），白字为固定最佳对比 |
| views/BudgetView.vue:557 | `color: white` | 固定紫色品牌渐变 `rgb(139 92 246)→rgb(147 51 234)`，双主题同底，非主题令牌底 |
| components/dashboard/DashboardNextActions.vue:223,224,234,238 | `rgb(var(--color-accent-primary-contrast-rgb) / 16-78%)` | accent 实心底上的次级文字/图标描边，刻意 alpha 保层次（非 text-*-rgb，不属 AC2 扫描） |
| components/layout/Titlebar.vue:398 | `rgb(var(--color-bg-overlay-rgb) / 55%)` | 交互态 alpha（窗口控制按钮 hover 反馈，bg 令牌引用） |
| components/layout/Titlebar.vue:416 | `inset 0 0 0 1px rgb(255 255 255 / 14%)` | 关闭按钮 danger 底 hover 装饰 ring（inset 0 0 0，非顶部高光，不属 AC2 扫描） |

### Step 4 — B4 壳层/模态/glass-panel/图表色（2026-07-28）

#### 4.1 壳层与模态

- `MainLayout.vue` `.sidebar-glass`/`.topbar-glass`：**验证通过，无改动**。两类已完全走 `--surface-shell-{bg,blur,border,shadow}` → chrome 档（`--material-glass-chrome-bg: var(--color-bg-elevated)` 实心、`chrome-blur: none`），类内无额外 blur/半透明背景。
- `BaseModal.vue`：面板 `bg-white/80 dark:bg-bg-elevated backdrop-blur-xl backdrop-saturate-150` + 散写 border/shadow → scoped `.base-modal-panel` 引用 `--surface-modal-{bg,blur,border,shadow}` 四件套（floating 档 92% 不透明 + blur(12px)，无 saturate）；`solid` 变体的 `!backdrop-saturate-100` 同步移除。scrim（:221 `backdrop-blur-md`）属 modal 遮罩保留，见白名单。
- 页面级/滚动区/实心底 blur 删除 10 处（blur 对不透明底无意义或滚动区禁玻璃）：
  - brief 内：`OpenCodePageShell.vue:168`（页面级图标块）、`ConfigsView.vue:51`（页面级背景层）、`ClaudeProfileRow.vue:3`（滚动列表行，同删残留的 4% 白 inset Tailwind 任意值 shadow）、`CodexAgentsView.vue:132`（实心底工具栏面板）、`SaveCodexSessionModal.vue:13` 与 `AddCodexAccountModal.vue:18`（均为 BaseModal `surface="glass"` 壳，sticky 头位于模态滚动内容区内，滚动区禁玻璃 → 删 `backdrop-blur`，壳体随 BaseModal 统一走 `--surface-modal-*`）。
  - AC4 对账扩展（同模式页面级残留，非 modal/浮层）：`BaseSlashCommands.vue:12`（glass-effect=workspace 不透明底）、`CodexSettingsView.vue:961`、`CodexMcpView.vue:14`、`GeminiCliView.vue:579`（均为实心底品牌图标块）。
- `AgentsView.vue:338` scrim `backdrop-blur-md`：**保留**（白名单）。理由：modal 遮罩属 floating 预算，与 BaseModal backdrop（:221）同模式；同路由 314/316 的 disabled 遮罩为 B1 已登记白名单且仅 disabled 卡片态出现，不与 modal scrim 同屏叠加。
- `utilities.css` `.nav-item-active-glow` / `.nav-item-inactive` 各删 `backdrop-filter: blur(10px)`：nav 项为侧栏内高频小元素，侧栏已不透明；两类当前无模板消费（仅 reduced-transparency 块引用），nav-item-inactive 为同理由顺带清理。

#### 4.2 usage 仪表盘 glass-panel

- `utilities.css` `.glass-panel` 统一定义：`--glass-bg-light`/`--glass-border-light`/`--glass-blur-sm`（deprecated 薄玻璃）→ `--surface-card-{bg,border,shadow,blur}` 四件套（实心 + blur none）；`:hover` 边框 `--glass-border-medium` → `--color-border-strong`。一次收敛全部 20 处消费（UsageOverviewTab/CostTab/TokensTab/ProvidersTab/ModelsTab/LogsTab/ProjectsTab 等）。
- `UsageDashboardView.vue` 原 364-369 的 `:deep(.glass-panel)` 覆盖块（90%→72% 渐变 + 14% 边框 + elevation-1）整段删除——与 card 契约重复且渐变违反不透明语义；inset 白高光 B3 已清零，确认无残留。

#### 4.3 chart-colors.css 五色校准

五色 ramp 改直引 tokens.css 规范令牌（不经 theme.css 短名桥接）：
- 0 `--color-accent-primary`（不变）、1 `--color-accent-secondary`→`--color-success`、2 `--color-warning`（不变）、3 `--color-info`（canonical，原经 `--accent-tertiary` 桥接同值）、4 `--color-danger`（不变）。
- 理由：暗色 neutral 下 ramp 为 橙 #e8835b / 绿 #7cab82 / 琥珀 #d6a76d / 蓝 #98afc9 / 珊瑚 #db8a73，色相分散可读（原 1 槽 tan #d0ae86 与 2 槽琥珀 #d6a76d 过于接近）；槽位 1 对齐 `TokenDetailTab.vue:141` 既定语义（注释"output 用 success 绿"，fallback 即旧 success #5b8a62）；语义锚点 validate→2(warning)、delete→4(danger) 不变。

### Step 5 — 全量回归（2026-07-28）

- `bun run type-check`：通过；`bun run lint`（eslint + stylelint）：0 error（1 个 pre-existing 无关 warning：DashboardSignalStream.vue raw text '×'）。
- `bun run test:smoke`：**106 文件 / 513 测试全绿**；`just frontend-check-quick`：通过。
- AC4 残留对账 `rg "backdrop-blur" ccr-ui/src`：9 条 = 1 条 `!backdrop-blur-none` 重置（BaseModal solid）+ 6 条 modal scrim 白名单 + 2 条 B1 已登记刻意透视（AgentsView:314,316）。
- AC4 计算样式证据（dev server + Playwright，dark+neutral+clay，dataset 已校验）：`.sidebar-glass` 与 `.topbar-glass` 均 `backdrop-filter: none`、`background: rgb(26, 27, 31)`（alpha=1）；令牌实测 `--surface-card-bg: #22242a`/`blur: none`、`--surface-shell-bg: #1a1b1f`/`blur: none`、`--surface-modal-bg: rgb(26 27 31 / 92%)`/`blur(12px)`、`--chart-color-1: #7cab82`；`.glass-panel` 加载规则确认为 card 四件套。
- AC1-AC3 无回退（本步仅删除/改引用，未新增 alpha 表面、透明文本、inset 高光或白字）。

### B4 白名单（backdrop-blur 残留登记）

| 文件:行 | 写法 | 保留理由 |
|---|---|---|
| components/common/BaseModal.vue:221 | `bg-black/30 dark:bg-black/60 backdrop-blur-md` | modal 遮罩 scrim（floating 预算内，同屏仅 1 个 modal） |
| components/McpPresetsPanel.vue:114 | `bg-black/40 backdrop-blur-md`（fixed inset-0 z-50） | modal 遮罩 scrim |
| views/generic/AgentsView.vue:338 | `bg-[color:var(--color-scrim)] backdrop-blur-md` | modal 遮罩 scrim（与 BaseModal 同模式） |
| views/generic/AgentDetailView.vue:213 | `bg-black/20 backdrop-blur-md` | modal 遮罩 scrim |
| views/HooksView.vue:874 | `bg-black/20 backdrop-blur-md`（fixed inset-0 z-50） | modal 遮罩 scrim |
| views/OutputStylesView.vue:190,268 | `bg-black/20 backdrop-blur-md` ×2 | modal 遮罩 scrim |
| views/generic/AgentsView.vue:314,316 | `backdrop-blur-[2px]` / `backdrop-blur-md` | B1 已登记：disabled 卡片遮罩层刻意透视设计 |

## 回滚点

每个 Step 结束即为一个回滚点；Step 间无共享文件冲突（MainLayout 仅 Step 4 触碰）。

## Step 6 — trellis-check 复核（2026-07-28）

- AC1 残留 22 处全部对上白名单（B1 逐条 4 + B2 交互态 18）；AC2 双扫描消费者 0 残留；AC3 `color: white` in styles = 0，`text-white` 仅剩 ConfigCard:25 登记项（BudgetView:557 登记项未变）；AC4 `backdrop-blur` 残留 10 行 = 1 重置 + 6 modal scrim + 1 scrim(scrim 令牌） + 2 B1 刻意透视，全部已登记。
- **发现并修复 1 处记录/代码不符**：Step 4 记录声称 `utilities.css` `.nav-item-active-glow` / `.nav-item-inactive` 已各删 `backdrop-filter: blur(10px)`，实际仍在（:237/:247）。两类确认零模板消费（仅 reduced-transparency 重置块引用），已实际删除这两行，记录自此属实。
- tokens.css diff 纯增量（6 hunk、0 删除行）：仅 scrim + 三组 contrast 增补，共享语义别名冻结未被触碰；theme.css 仅 3 行 contrast 桥接；contrast 对比度注释抽样验算 6 项全部属实（≥3.5:1）；chart-colors 槽位 1 语义锚点（TokenDetailTab.vue:141 注释）查证真实存在。
- 顺手修复确认：CodexSettingsView.vue phantom `var(--platform-codex-rgb, 245 158 11)` 已改为 canonical `--color-platform-codex-rgb`（无 fallback）。
- diff 新增行扫描：无新 `backdrop-filter` 面（仅 modal 契约引用与 card blur:none）、无 phantom 形式引入、无 `255 255 255` 新增、无调试残留、无 i18n 文件变更。
- 验证：`bun run type-check` ✅；`bun run lint` 0 error（1 个 pre-existing warning）✅；`bun run test:smoke` 106 文件 / 513 测试全绿 ✅；`git diff --check` ✅。
- 遗留（超范围，不改）：`HistoryList.vue` 的 init/import 事件色随 `--chart-color-1` 由 tan 变绿（槽位 1 校准的连带语义偏移，categorical 用色可接受，父任务视觉核验时留意）；`utilities.css:28` 玻璃预算注释仍写"≤3"，与 spec ≤1 契约不一致（pre-existing，建议后续任务顺手订正）。

