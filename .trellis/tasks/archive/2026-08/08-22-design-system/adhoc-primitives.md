# 手写交互原语普查（batch 3）

> 依据 `.trellis/tasks/08-22-design-system/design.md` §6 的普查方法，对 `ccr-ui/src/`
> 中 Dropdown / Tooltip / Popover / Tabs / Accordion / Combobox 六类交互的**手写实现**
> 进行全量定位。这些实现全部位于 `.vue` 死代码中（迁移源真相），迁移到 React 时必须
> 由 `src/ui/` 的 shadcn 原语替换，而不是照搬成 React 版本。
>
> 检索方法：对六类特征做 `rg`——`role="menu"/"menuitem"/"listbox"/"option"/"tablist"/"tab"`、
> `aria-expanded`、`aria-haspopup`、`aria-activedescendant`、`aria-controls`、
> `@mouseenter`/`@mouseleave` + `position: absolute` 组合、`absolute top-full`、`activeTab` 状态、
> `<select`/`type="checkbox"`/`role="switch"`。**六类无未检视候选。**

## 1. 检索覆盖面说明

| 特征 | 检索命令 | 命中文件数 |
| --- | --- | --- |
| `role="menu"` / `role="menuitem"` | `rg 'role="menu"|role="menuitem"'` | 4 |
| `role="tablist"` / `role="tab"` / `aria-selected` | `rg 'role="tablist"|role="tab"'` | 9（另有 2 处无 ARIA 的手写 tab） |
| `role="listbox"` / `role="option"` | `rg 'role="listbox"|role="option"'` | 3 |
| `role="switch"` / `type="checkbox"` / `<select` | `rg` | switch 1、checkbox 20、select 20 |
| `aria-expanded`（+ `aria-haspopup` / `aria-controls`） | `rg 'aria-expanded'` | 10 |
| `aria-activedescendant` | `rg 'aria-activedescendant'` | 2 |
| `@mouseenter` / `@mouseleave` | `rg '@mouseenter|@mouseleave'` | 5（ui/Card 为原语自身，不计） |
| `absolute top-full`（下拉/弹层定位） | `rg 'absolute.*top-full'` | 7 |
| `fixed inset-0`（弹层，批次 4 范围） | `rg 'fixed inset-0'` | 13（见 §8 重叠注记） |
| `:title="..."`（原生 tooltip 属性） | `rg ':title='` | 117 |

> `fixed inset-0` 的 13 个文件是批次 4（弹层收口）的调用点，本批次只登记重叠，
> 不判定其归属。`src/components/ui/` 自身 16 个原语（含 `Card.vue` 的
> `@mouseenter`/`@mouseleave`）已由 `primitive-disposition.md` 覆盖，不在本表重复。

## 2. Dropdown / Menu（5 个实现 + 1 个定位退化）

| # | 文件 | 行数 | 手写内容 | 调用点 |
| --- | --- | --- | --- | --- |
| D1 | `src/components/claude/ClaudeProfileRow.vue` | 626 | `role="menu"` + `role="menuitem"` 下拉菜单；`aria-expanded`/`aria-haspopup="menu"`；`menuPopRef` 键盘导航（Arrow 移动、Esc 关闭）；`position: absolute` 面板 | `src/views/ClaudeCodeProfilesView.vue`；并被 D2 复用为样式/结构模板 |
| D2 | `src/components/codex/ProfileCard.vue` | 630 | 与 D1 同构的 `role="menu"`/`menuitem` 菜单 + 键盘导航 | `src/views/CodexProfilesView.vue`、`src/views/grok/GrokProfilesView.vue` |
| D3 | `src/components/grok/GrokProfileCard.vue` | 413 | `role="menu"`/`menuitem` 菜单（`selectAction` 分发） | `src/views/grok/GrokProfilesView.vue` |
| D4 | `src/components/profiles/ProfilesHeader.vue` | 424 | `role="menu"`/`menuitem` + `aria-haspopup="menu"` + 键盘导航 | `src/views/ClaudeCodeProfilesView.vue`、`src/views/CodexProfilesView.vue`、`src/views/grok/GrokProfilesView.vue` |
| D5 | `src/components/EnvironmentSwitcher.vue` | 208 | 环境切换下拉：`aria-expanded`/`aria-haspopup`；`absolute right-0 top-full` 面板；`role="listbox"`/`role="option"` | `src/components/MainLayout.vue` |
| D6 | `src/components/layout/Titlebar.vue` | 448 | 标题栏「菜单」下拉：`absolute top-full` + `@click` 切换；无 ARIA 角色（菜单项为普通 button） | `src/App.vue` |

**DropDown 迁移提示**：D1–D4 是同一菜单模式的四份复制，正是 `@radix-ui/react-dropdown-menu`
要消除的复制；D5/D6 是轻量下拉，可用 DropdownMenu（Trigger 自定义）或 Popover 承载。

## 3. Popover（3 个实现）

| # | 文件 | 行数 | 手写内容 | 调用点 |
| --- | --- | --- | --- | --- |
| P1 | `src/components/profiles/ProfilesToolbar.vue` | 604 | 筛选面板 popover：`role="dialog"` + `aria-haspopup="dialog"` + `aria-expanded`；`cp-filters__pop` 绝对定位；`onFiltersKeydown` 键盘导航；`filtersBtnRef`/`filtersPopRef` 定位锚 | `src/views/ClaudeCodeProfilesView.vue`、`src/views/CodexProfilesView.vue`、`src/views/grok/GrokProfilesView.vue` |
| P2 | `src/components/usage/UsageDashboardToolbar.vue` | 404 | `aria-expanded` 元信息折叠区（`metaOpen` 切换，`role="group"`）；面板绝对定位 | `src/views/UsageDashboardView.vue`、`src/components/usage/UsageCostTab.vue` |
| P3 | `src/components/McpPresetsPanel.vue` | 416 | 预设面板展开/收起（`showPresetsPanel` + ChevronUp/Down）；`fixed inset-0` 安装弹层（批次 4 重叠） | `src/views/mcp/McpManagerView.vue` |

## 4. Tabs（11 个实现）

| # | 文件 | 行数 | 手写内容 | 调用点 |
| --- | --- | --- | --- | --- |
| T1 | `src/components/usage/UsageTokensTab.vue` | 530 | `role="tablist"` + `role="tab"` + `aria-selected` 模式切换（breakdown/total） | `src/components/usage/UsageCostTab.vue`、`src/views/UsageDashboardView.vue` |
| T2 | `src/components/platform-usage/PlatformUsageInsightPanel.vue` | 557 | `role="tablist"`/`role="tab"` 维度切换 | `src/views/CodexView.vue`、`src/views/GeminiCliView.vue`、`src/views/OpenCodeView.vue` |
| T3 | `src/components/claude/ClaudeProfileEditorModal.vue` | 401 | `role="tablist"` 分区导航（`activeSectionId`） | `src/views/ClaudeCodeProfilesView.vue` |
| T4 | `src/components/codex/CodexProfileEditorModal.vue` | 956 | `role="tablist"` 分区导航 | `src/views/CodexProfilesView.vue` |
| T5 | `src/components/grok/GrokProfileEditorModal.vue` | 894 | `role="tablist"` 分区导航 | `src/views/grok/GrokProfilesView.vue` |
| T6 | `src/views/ClaudeCodeSettingsView.vue` | 1325 | `role="tablist"` 设置页 tab（`v-show` 内容切换） | 路由（`src/router/index.ts`） |
| T7 | `src/views/CodexSettingsView.vue` | 1023 | `role="tablist"` 设置页 tab | 路由 |
| T8 | `src/views/OpenCodeView.vue` | 783 | `role="tablist"` inspector tab（`activeInspector`） | 路由 |
| T9 | `src/views/grok/GrokSettingsView.vue` | 1245 | `role="tablist"` 设置页 tab | 路由 |
| T10 | `src/views/ConfigsView.vue` | 520 | **无 ARIA** 的手写 tab（`activeTab` + `@click` + `border-b-2` 激活下划线） | 路由 |
| T11 | `src/components/claude-observer/UsageInsightPanel.vue` | 623 | **无 ARIA** 的手写 tab（`activeTab` + `@click`） | `src/views/ClaudeCodeView.vue`、`src/views/CodexView.vue`、`src/views/GeminiCliView.vue`、`src/views/OpenCodeView.vue` |

> 注：`src/views/UsageDashboardView.vue` 与 `src/views/CheckinView.vue` 的 workspace
> 切换用的是 `PillToggleGroup` 原语（分段控件语义），不是 tablist，见 `primitive-disposition.md`
> 中 PillToggleGroup 的判定。

## 5. Accordion / Disclosure（3 个实现 + 1 处 shell 重叠）

| # | 文件 | 行数 | 手写内容 | 调用点 |
| --- | --- | --- | --- | --- |
| A1 | `src/components/claude/ClaudeProfileEditorSections.vue` | 842 | 高级配置区折叠：`aria-expanded` + `@click` 切换 | `src/components/claude/ClaudeProfileEditorModal.vue` |
| A2 | `src/views/checkin/tabs/CheckinRecordsTab.vue` | 889 | 失败记录行展开：`aria-expanded` + `toggleRecordExpanded` | `src/views/CheckinView.vue` |
| A3 | `src/components/BaseSlashCommands.vue` | 507 | 命令树文件夹 accordion：`expandedFolders` + ChevronDown `rotate-180` + `v-if` 内容 | `src/views/SlashCommandsView.vue`、`src/views/GeminiSlashCommandsView.vue` |
| — | `src/components/MainLayout.vue` | — | 侧边栏收起 `aria-expanded`（`isSidebarOpen`）——**shell 布局态，不是原语**，归 `08-22-shell-port`，不在本表 | `src/App.vue` |

> 说明：本次不实现 Accordion 的 shadcn 原语（design.md §6 的 9 类不含 Accordion）。
> A1–A3 三处手写折叠若迁移需要，可直接用 Radix 的 `@radix-ui/react-accordion`
> 原生组件或 `Collapsible`，无需本任务新增包装。A1–A3 已在四类外的备注中登记，
> 六类普查覆盖闭合。

## 6. Combobox / Autocomplete（2 个实现）

| # | 文件 | 行数 | 手写内容 | 调用点 |
| --- | --- | --- | --- | --- |
| C1 | `src/components/profiles/ProfilesCommandPalette.vue` | 625 | 命令面板：`role="listbox"`/`role="option"`、`aria-activedescendant`、`aria-controls`、`@mouseenter` 高亮、`ArrowDown/ArrowUp` 键盘导航、查询过滤 | `src/views/ClaudeCodeProfilesView.vue`、`src/views/CodexProfilesView.vue`、`src/views/grok/GrokProfilesView.vue` |
| C2 | `src/components/provider-templates/ProviderTemplateSelector.vue` | 1275 | Provider 模板选择器：`role="listbox"`/`role="option"`、`aria-activedescendant`、`aria-controls`、过滤 + 键盘导航 | `src/components/AddConfigModal.vue`、`src/components/claude/ClaudeProfileEditorModal.vue`、`src/components/codex/CodexProfileEditorModal.vue`、`src/views/codex/tabs/CodexAuthProvidersTab.vue`、`src/views/codex/components/AddCodexAccountModal.vue`、`src/views/OpenCodeProvidersView.vue` |

> C1 的语义是命令面板（command palette），对应 shadcn 的 Combobox（cmdk 底座）；
> C2 是传统搜索式下拉选择，同样落在 Combobox 原语。原生 `<select>`（20 个文件）在
> 迁移期由 Select 原语替换（见 `primitive-disposition.md` 的 Select 映射备注）。

## 7. Tooltip（2 个实现 + 117 处原生属性）

| # | 文件 | 行数 | 手写内容 | 调用点 |
| --- | --- | --- | --- | --- |
| F1 | `src/components/dashboard/DashboardUsageMovement.vue` | 565 | 图表 hover 读值浮层：`@mouseenter`/`@mouseleave` 驱动 `hoveredKey`，绝对定位 `.dashboard-usage__chart-readout`（`aria-live`） | `src/views/DashboardView.vue` |
| F2 | `src/views/checkin/components/AccountDashboardTrend.vue` | 322 | 图表 tooltip：`@mouseenter`/`@mouseleave` + `tooltipStyle` 计算的绝对定位 `.chart-tooltip` | `src/views/checkin/CheckinAccountDashboardView.vue` |
| — | 117 个 `.vue` 文件 | — | 原生 `:title="..."` 属性 tooltip（浏览器原生，无定位/无延迟） | 分布全 `views/` 与 `components/` |

> F1/F2 是图表自绘浮层，迁移期由 Tooltip 原语（Radix）承载或按图表 hover 语义重写；
> `:title=` 原生属性在 React 迁移时视场景替换为 Tooltip（截断名、状态说明等），
> 视原子任务按需处理，不在本任务强制。

## 8. 与批次 4（弹层收口）的重叠

13 个文件自行实现 `fixed inset-0` 弹层（不在本批次判定）：

`src/components/MainLayout.vue`、`src/components/McpPresetsPanel.vue`、`src/components/UpdateModal.vue`、
`src/components/common/AnimatedBackground.vue`、`src/components/common/BaseModal.vue`、
`src/components/configs/ProviderStatsModal.vue`、`src/components/layout/Titlebar.vue`、
`src/views/HooksView.vue`、`src/views/OutputStylesView.vue`、`src/views/generic/AgentDetailView.vue`、
`src/views/generic/AgentsView.vue`、`src/views/generic/PlatformMcpView.vue`、`src/views/generic/PlatformPluginsView.vue`

> 另：33 个文件引用 `BaseModal.vue`（`design.md` §7）。P3（McpPresetsPanel）的安装弹层
> 与 A2 的展开面板弹层分别与批次 4 有交叉，迁移时以 Dialog 底座为唯一入口。

## 9. 汇总

| 原语类 | 手写实现数 | 代表文件 |
| --- | --- | --- |
| Dropdown/Menu | 6 | D1–D6 |
| Popover | 3 | P1–P3 |
| Tabs | 11 | T1–T11 |
| Accordion | 3（+1 shell 布局态） | A1–A3 |
| Combobox | 2 | C1–C2 |
| Tooltip | 2（+117 处原生 `:title`） | F1–F2 |
| **合计** | **27 个手写交互实现** | — |

## 10. 替换映射（视图子任务查表依据）

| shadcn 原语（`src/ui/`） | 本清单中的替换目标 |
| --- | --- |
| `dropdown-menu.tsx` | D1、D2、D3、D4、D6 |
| `popover.tsx` | P1、P2、P3（面板部分）、D5（下拉可用 Popover） |
| `tabs.tsx` | T1–T11（含无 ARIA 的 T10/T11） |
| `combobox.tsx` | C1、C2 |
| `tooltip.tsx` | F1、F2 与 `:title=` 的交互场景 |
| `select.tsx` / `switch.tsx` / `checkbox.tsx` | 20 处原生 `<select>`、1 处 `role="switch"`（AppSettingsView）、20 处原生 `type="checkbox"` 的迁移替换 |
| `dialog.tsx` | 批次 4（弹层收口）的 33 + 13 个调用点 |
