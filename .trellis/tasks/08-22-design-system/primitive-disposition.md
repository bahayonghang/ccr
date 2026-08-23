# 原语判定表（batch 3）

> 依据 `design.md` §6 的初判与判定标准，对 `ccr-ui/src/components/ui/` 下 16 个手写原语
> （合计约 2,201 行，Vue 死代码、阶段 5 迁移源真相）逐个核对现有用法后确认判定。
> 回答 `08-22-views-usage/implement.md` 批次 3 登记的待确认项：「Sparkline 是否消费
> `chart-colors.css`」——结论见第 4 节：不消费。

## 1. 用法统计方法

对每个原语名统计 `ccr-ui/src/` 内（排除 `src/components/ui/` 自身）的消费者文件数。
命中标准（满足其一即计该文件一次）：模板标签 `<Name` / `<Name …>`，或精确导入路径
`components/ui/Name.vue`。统计时排除了子串误报（如 `ScrollToTopButton` 之于 `Button`、
`EnvironmentBadge` 之于 `Badge`、`mainLayoutShell.ts` 的 `MainLayoutNavItem` 接口之于
`NavItem`）。等价检索命令形态：

```bash
rg -l '<Button[\s/>]|components[/\\]Button\.vue' ccr-ui/src \
  -g '*.vue' -g '*.ts' -g '*.tsx' -g '!src/components/ui/**'
```

汇总：AsyncStatePanel 4、Badge 3、Breadcrumb 0、Button 38、Card 31、EmptyState 5、
IconWrapper 0、Input 5、NavItem 0、PageHeader 44、PageShell 44、PillToggleGroup 10、
SIcon 128、Sparkline 1、Spinner 3、StatTile 10（单位均为消费者文件数）。

## 2. 判定表

| 原语 | 现有用法核对（消费方文件数与代表调用点） | 关键 API 面 | 判定 | 理由 | 替换映射或迁移归属 |
|---|---|---|---|---|---|
| Button | 38 个文件。代表：`views/CodexView.vue`(4 处)、`views/GeminiCliView.vue`(4)、`components/sync/SyncAccountDialog.vue`、`components/AddConfigModal.vue`；分布覆盖全部七个 views 域 | `variant`(primary/secondary/accent/outline/ghost/glass/danger/success)、`size`(sm/md/lg/icon)、`surface`/`elevation`/`motion`/`density` 四个修饰轴、`loading`/`block`/`disabled`；emit `click` | shadcn/ui 替换 | shadcn Button 的 variant/size 覆盖主轴；surface/elevation/motion/density 是 token 驱动的样式组合，由 `cva` variant + 工具类表达，无需保留 313 行 scoped CSS | React 版落 `ccr-ui/src/ui/button.tsx`（shadcn 底座 + 本仓 variant 集），由最先执行迁移的视图子任务落地，38 个调用点随各自所属 views 子任务改调用 |
| Badge | 3 个文件：`components/codex/CodexAccountCard.vue`、`components/opencode/OpenCodePageShell.vue`、`components/usage/UsageDiagnosticsDrawer.vue` | `label`、`variant`(8 种)、`size`(4 档)、`dot`、`removable`、`pill`/`shape`、`platform`(claude/codex/gemini 平台色)；slots `leading`/`trailing`；emit `remove` | shadcn/ui 替换 | shadcn Badge 覆盖 variant/size 主语义；dot/removable/platform 是薄扩展（圆点 span、关闭按钮、className 注入），可在 shadcn Badge 上以组合实现 | `ccr-ui/src/ui/badge.tsx`；三个调用点分别归 `08-22-views-codex`、`08-22-views-secondary-platforms`、`08-22-views-usage` 改调用 |
| Card | 31 个文件。代表：`views/ClaudeCodeSettingsView.vue`、`views/codex/tabs/CodexAuthProvidersTab.vue`、`components/codex/CodexAccountCard.vue`；分布覆盖全部七个 views 域 | `variant`(default/base/elevated/glass/outline)、`padding`(none/sm/md/lg)、`hover`/`interactive`/`disabled`；glow/glowEffect/glowColor/gradientBorder/pattern 已 deprecated 无效果；同款 surface/elevation/motion/density 修饰轴 | shadcn/ui 替换 | shadcn Card 覆盖容器语义；deprecated props 已是 no-op，迁移即清除；interactive/hover 态用工具类表达 | `ccr-ui/src/ui/card.tsx`；31 个调用点随各自 views 子任务改调用 |
| Input | 5 个文件：`components/AddConfigModal.vue`、`components/EditConfigModal.vue`、`components/sync/SyncAccountDialog.vue`、`components/sync/SyncPassphraseModal.vue`、`views/CodexSessionsView.vue` | v-model `modelValue`、`label`、`type`、`placeholder`、`error`、`hint`、`fullWidth`、surface/elevation/motion/density 修饰轴；expose `focus()` | shadcn/ui 替换 | shadcn Input 覆盖输入语义；label/error/hint 组成表单行包装（配合 react-hook-form 注册，父任务已定 react-hook-form 方向）；修饰轴并入 variant | `ccr-ui/src/ui/input.tsx`；调用点归 `08-22-views-profiles-config`（3 个 modal/configs 文件）、`08-22-views-sync-tools`（2 个 sync 弹层）、`08-22-views-codex`（CodexSessionsView）改调用 |
| Breadcrumb | **0 个文件**（仅 `components/ui/index.ts` 的 re-export；`MainLayout.vue` L177 的 "Breadcrumbs" 只是注释） | `items: {label, path?, icon?}[]`、`moduleColor?`；内置 RouterLink 分段 + ChevronRight 分隔 + 末项玻璃面高亮 | shadcn/ui 替换 | 维持 §6 初判类别；但实测零调用点——现 `.vue` 实现不移植，React 侧仅在出现面包屑需求时按需接 `ccr-ui/src/ui/breadcrumb.tsx` | 无存量调用点需迁移；如需接入由对应视图子任务直接用 `src/ui/breadcrumb.tsx` |
| EmptyState | 5 个文件：`views/CodexView.vue`、`views/CodexSessionsView.vue`、`views/ClaudeAuthView.vue`、`views/grok/GrokView.vue`、`components/mcp/McpListPanel.vue` | `icon`、`title`、`description`、`actionText`/`actionIcon`/`onAction`（内联 action 按钮）；`role="status"` + `aria-live="polite"`；卡片化容器消费 `--surface-card-*` token | 保留并改消费新 token | 本仓特有的空态组合（图标圆盘 + 标题 + 描述 + 单动作），shadcn 无对应原语；ARIA status 语义须保留 | 移植为 `ccr-ui/src/ui/empty-state.tsx` 并改消费 `themes/` 新 token。跨四域共享，由消费方中先执行的子任务落地（Codex 两处调用点最多，建议 `08-22-views-codex`），其余三域改引用 |
| IconWrapper | **0 个文件**（仅 index.ts re-export） | `name`/`icon` 双别名、`size`(xs–xl)、`variant`(7 色)、`color`、`background`+`shape`(circle/rounded/square)，内部包 SIcon | 保留不变 | 维持 §6 初判类别；纯渲染封装与样式体系无关，但实测零调用点——不移植死代码。未来如需图标背景封装，尺寸/颜色映射用工具类直接表达 | 无迁移动作；不进入 `src/ui/` |
| AsyncStatePanel | 4 个文件：`views/UsageDashboardView.vue`、`components/usage/UsageProvidersTab.vue`、`views/SyncView.vue`、`components/claude-observer/UsageInsightPanel.vue` | `state`(loading/error/empty/runtime-unavailable 四态驱动图标与配色)、`title`/`description`/`icon`、`actionLabel`/`actionIcon`、`compact`；emit `action`；内部组合 Button+SIcon+Spinner | 保留并改消费新 token | 四态状态面板是本仓语义（含 runtime-unavailable 这种 IPC 运行态），shadcn 无对应物；配色已走 accent/danger token，改写只在移植时对齐 `themes/` 新变量名 | 移植为 `ccr-ui/src/ui/async-state-panel.tsx`。横跨 usage/sync-tools/claude 三域，由先执行者落地并登记（建议 `08-22-views-usage`，占 2 个调用点），其余改引用；依赖的 button/icon/spinner 需先在 `src/ui/` 就位 |
| NavItem | **0 个文件**。侧边导航由 `MainLayout.vue` 手写条目（配置在 `config/mainLayoutShell.ts` 的 `MainLayoutNavItem`，未经过本原语） | `to`(RouteLocationRaw)、`icon`、`label`、`isActive`、`showActiveIndicator`、`badge`/`badgeVariant`、`disabled`；活跃态渐变指示条 | 保留并改消费新 token | 判定类别维持 §6 初判（本仓特有的导航项语义），但初判依据（存在 shell 调用点）不成立——实测零调用。React 侧是否保留独立 NavItem 原语由 `08-22-shell-port` 重写 MainLayout 导航时决定；现 `.vue` 不移植 | 归属 `08-22-shell-port`：其重写侧边栏时要么落地 `src/ui/nav-item.tsx`（消费新 token），要么以工具类直接表达导航条目并废弃该原语名。两个方向均不在批次 3 执行 |
| PageHeader | 44 个文件（全站页头）。代表：`views/DashboardView.vue`、`views/mcp/McpManagerView.vue`、`components/profiles/ProfilesHeader.vue`、`components/opencode/OpenCodePageShell.vue` | `title`、`eyebrow`(+自动 `lang` 判定的 `eyebrowLang`)、`description`；slots `leading`/`status`/`actions`；eyebrow 小字号排版为编辑式视觉的关键件 | 保留并改消费新 token | 本仓特有的页头版式（eyebrow/title/description 三层 + 双 aside 槽），shadcn 无对应物；scoped CSS 全部消费语义变量，改写即对齐 `themes/` 新变量名 | 归属 `08-22-shell-port` 移植为 `ccr-ui/src/ui/page-header.tsx`（shell 级版式原语，44 个调用点全部是其下游）；各 views 子任务改消费 |
| PageShell | 44 个文件（每个路由根组件的页壳，各 2 处标签）。代表：`views/CheckinView.vue`、`views/ClaudeCodeProfilesView.vue`、`components/opencode/OpenCodePageShell.vue`(复用为平台壳) | slots `header`/`subnav`/default(content)；`page-shell__inner` 纵向栅格 + content 网格间距；响应式 padding 断点 | 保留并改消费新 token | 页壳槽结构是全站布局契约（platform-unify 的薄壳形态即 `PageShell + PageHeader + BaseX`），shadcn 无对应物 | 归属 `08-22-shell-port` 移植为 `ccr-ui/src/ui/page-shell.tsx`；44 个调用点随各自 views 子任务改消费 |
| PillToggleGroup | 10 个文件。代表：`components/dashboard/DashboardSignalStream.vue`、`components/dashboard/DashboardUsageMovement.vue`、`components/usage/UsageDashboardToolbar.vue`、`views/CheckinView.vue`、`views/CommandsView.vue`、`views/mcp/McpManagerView.vue` | 泛型 `T extends string\|number`；`options: {value,label,disabled}[]`、v-model、`ariaLabel`；`role="radiogroup"` + `role="radio"` + `aria-checked`（分段控件语义，非 tablist，见 adhoc-primitives.md §4 注） | 保留并改消费新 token | 本仓特有的分段切换控件；9 类 shadcn 接入清单不含 ToggleGroup，Radix Toggle Group 未列入本批范围；radiogroup ARIA 语义必须保留，不得降级为无 ARIA 手写 tab | 移植为 `ccr-ui/src/ui/pill-toggle-group.tsx` 并改消费新 token。跨 dashboard/usage/checkin/codex/sync-tools 五域，由先执行者落地（dashboard 两文件属 `08-22-views-usage`，建议该任务），其余改引用 |
| SIcon | 128 个文件（全仓图标唯一入口）。代表：`components/MainLayout.vue`、`components/common/ToastContainer.vue`、`views/tray/components/TrayOverview.vue` 等遍布所有域 | `name`(语义名或 solar:* 原始 ID，经 `@/config/icons` 的 iconMap 解析)、`size`(Tailwind 尺寸类)；薄包 `@iconify/vue` 的 `<Icon>` | 保留不变 | 与样式体系无关的纯渲染封装；25 行无 scoped CSS。React 侧等价物为 `@iconify/react` 同构薄包装，iconMap 配置原样复用 | 归属 `08-22-shell-port` 随外壳落地（MainLayout 即其调用点）为 `ccr-ui/src/ui/` 下图标包装（或直接导出 Iconify Icon + iconMap）；128 个调用点随各域迁移改引用 |
| Sparkline | 1 个文件：`components/usage/UsageMetricCard.vue`（L31，width=120 趋势档） | `values: number[]`、`width`/`height`、`stroke`(默认 currentColor)、`strokeWidth`、`fill`(传入即渐变面积+端点档)、`label`(role=img 无障碍)；纯 SVG 计算，无图表库 | 保留不变 | 纯渲染封装；**不消费 `chart-colors.css`**（详见第 4 节）：折线取 `currentColor`，端点描边取 `--color-bg-elevated-rgb`，颜色全部由消费方注入 | 归属 `08-22-views-usage`（唯一消费方在其范围内，其 implement.md 批次 3 已登记此项），移植为 `ccr-ui/src/ui/sparkline.tsx`，行为不变 |
| Spinner | 3 个文件：`components/EditConfigModal.vue`、`components/HistoryList.vue`、`components/configs/ConfigList.vue` | `size`(sm/md/lg/xl → w-4 至 w-12)；Tailwind `animate-spin` + 双 opacity 圈，currentColor | 保留不变 | 纯渲染封装，与主题体系无关（currentColor 自适应）。三个调用点同属 profiles-config 域 | 归属 `08-22-views-profiles-config` 移植为 `ccr-ui/src/ui/spinner.tsx`（十几行，随其首批 modal 迁移顺带落地），三个调用点同步改引用 |
| StatTile | 10 个文件。代表：`components/dashboard/DashboardReadinessLedger.vue`、`components/dashboard/DashboardUsageMovement.vue`、`views/BudgetView.vue`、`views/checkin/CheckinAccountDashboardView.vue`、`views/tray/components/TrayOverview.vue` | `label`/`value`/`hint`、`tone`(neutral/success/warning/danger/accent，data-tone 驱动数值着色)；slot `label`；tone 联合类型定义在原语上避免反向依赖 dashboard 视图模块 | 保留并改消费新 token | 本仓特有的指标瓦片（标签/大数值/hint + tone 语义色），shadcn 无对应物；179 行 scoped CSS 全部消费语义变量，改写在移植时完成 | 归属 `08-22-views-usage` 移植为 `ccr-ui/src/ui/stat-tile.tsx` 并改消费新 token（其 implement.md 批次 3 R10 已登记 StatTile 适配）；checkin/codex/secondary-platforms/sync-tools(tray) 各自改引用 |

## 3. 与 design.md §6 初判的差异说明

16 行的判定类别全部与 §6 初判一致，无一行改判。三行补充了初判未掌握的用法事实：

| 原语 | §6 初判隐含假设 | 实测事实 | 对迁移的影响 |
|---|---|---|---|
| Breadcrumb | 「shadcn/ui 有对应且行为覆盖现有用法」，暗示存在存量调用点 | 0 个调用点，仅 index.ts re-export | 现 `.vue` 不移植；`breadcrumb.tsx` 仅按需接入 |
| NavItem | 「保留并改消费新 token」，归 shell 布局 | 0 个调用点；MainLayout 手写导航条目（`config/mainLayoutShell.ts`） | 是否保留该原语名由 `08-22-shell-port` 重写导航时决定，现 `.vue` 不移植 |
| IconWrapper | 「保留不变」的纯渲染封装 | 0 个调用点，仅 index.ts re-export | 不进入 `src/ui/` |

另两处初判口径修正（不改判定）：SIcon 的 React 归属明确为 `08-22-shell-port` 首落地
（初判未指派）；Spinner 的三个调用点经核对全部落在 profiles-config 域，故归属唯一化为
`08-22-views-profiles-config`。

## 4. Sparkline 与 chart-colors.css 核对

结论：**Sparkline 不消费 `chart-colors.css`**。证据：

- `Sparkline.vue` 自身（77 行实现 + 20 行 scoped style）只引用两类颜色：
  折线默认 `stroke: 'currentColor'`（props 默认值），端点圆点描边
  `rgb(var(--color-bg-elevated-rgb) / 92%)`（L134）。文件内无任何 `--chart-color-*`
  引用。
- 唯一消费方 `UsageMetricCard.vue` 也未传 `--chart-color-*`：它给 Sparkline 只传
  `values`/`width`，折线颜色继承卡片的 `--usage-metric-rgb`（L198），该变量按卡片 tone
  取 `--color-accent-primary-rgb` / `--color-accent-secondary-rgb` / `--color-info-rgb` /
  `--color-warning-rgb`（L73–101），全部是 tokens 体系变量而非 chart-colors 五色 ramp。

因此 chart-colors.css（5 个 `--chart-color-*` 变量）的消费方仍只有 ApexCharts 图表桥与
柱状图 `bar-color-*` 类，与 Sparkline 无耦合；token 迁移时二者互不牵连
（prd.md Notes 所述 chart-colors 与 apexcharts 契约的同步义务不受本判定影响）。

## 5. 偏差注记

「保留并改消费新 token」的原语改写发生在各自 React 移植时（shell-port / views 阶段），
不在批次 3 执行——16 个原语当前仍为 `.vue` 死代码；本表即为视图子任务的替换映射查表依据。
