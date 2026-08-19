# CCR UI 全站视觉与结构重构：去塑料感 · 编辑式收敛

## Goal

把 ccr-ui 全部 **51** 个 `*View.vue` 从「糖果塑料」实现收敛到 Editorial Control Room：清除拟物高光、饱和渐变、装饰性 glow 与系统外硬编码色；退役 catppuccin flavor 与多值 accent；把排版改回操作台尺度；并重排 dashboard、平台工作台与设置页的信息层级。

产品方向（calm / precise / editorial）不变。本任务同时**修订** `ccr-ui/DESIGN.md` 中与操作台尺度冲突的 frontmatter / 按钮 / 输入 / Glow 条款，使设计文档与实现配方同一套数字。

## Background

用户反馈：当前页面「有浓浓的塑料感，不符合现代审美和排版」。证据见 `research/visual-audit.md`（2026-08-18 初审 + 同日复核附录）。根因：

1. **拟物高光**：`tokens.css` 的 `--inner-glow` / `--glass-inner-glow` 经 `tailwind.config.ts:164-178` 打进全部 `.surface-card` / `.surface-modal`，再叠 1px 描边与宽投影。
2. **颜色失序**：字面 `rgb()` 约 180 处；`linear-gradient` 在 `.vue` 119 处、`.css` 另 20 处；`ConfigCard.vue:250-262`、`CodexAgentEditorModal.vue:324-370`、`ConfirmModal.vue:209-219` 各有一套系统外紫蓝外壳。
3. **Glow 家族**：按钮 hover、status-dot、text-glow、tag-glow、`Card.vue:159-171` 废弃覆盖层。
4. **Accent 过密**：dashboard 行动行实心橙（`DashboardNextActions.vue:211-215`）；导航左侧 accent 竖条（`MainLayout.vue:545-552`，DESIGN.md 禁止 side-stripe）。
5. **排版违和**：`DashboardView.vue:448` 的 `clamp(..., vw, ...)` 违反 Product Type Rule；eyebrow 复制多处并对 CJK 加字距；`--tracking-normal: -0.016em` 作用于正文。
6. **形状单一**：`rounded-full` 约 182 处；ReadinessLedger 嵌套卡。
7. **结构债**：51 个视图三种样式惯用法并存；`PageHeaderCard` 仅 6 引用；统计/切换组各页手造。

## Confirmed Facts

用户已确认（2026-08-18）：

- 退役 catppuccin（含 latte/mocha resolved flavor）与多值 accent；保留 `neutral | clay` flavor 与单一 `clay` accent。设置页去掉 accent 选择器，保留 flavor 双选。本 PRD 即 `theme-token-contracts` 要求的显式默认迁移声明（先例：`07-28-color-system-rebuild`）。
- 深度为视觉 + 结构重排。不改路由、不改功能流程、不动 Tauri / `llmusage`。
- 字体偏好轴（`ccr-font-ui` / `ccr-font-code`）不动。
- `data-theme` / `data-flavor` / `data-accent` 三层不合层，只收窄值域。

规划补完决议（2026-08-18，审阅后写入，待启动前确认）：

- **修订 DESIGN.md**：frontmatter 与 §3 / §5 的字重、正文字距、主按钮填充、输入圆角、Glow 词汇改为与 `design.md` 配方一致。修订完成后 DESIGN.md 仍是视觉世界权威。
- **视图按 51 个 `*View.vue` 勾销**。路由 redirect（`ccr-control` / `sessions` / `market` / `opencode-skills` 等）不产生视图。
- **原语按需接入**：有页头用 PageHeader，有统计用 StatTile，有单选切换用 PillToggleGroup。不要求每一页三件套。
- **单任务交付**，不拆子任务。工作在功能分支上进行；Wave 0–6 的中间提交不进入 `main` / 发版分支。可拆映射见 `implement.md`。
- **不强制**把存量超 300 行的 `<style>` 拆完。本任务验收视觉禁令；Checkin / Grok / Commands 的逻辑拆分不在范围内。
- `data-accent` 继续写入 DOM，值域只剩 `clay`。
- OpenCode 产品主题名 `catppuccin-mocha`（`opencode-view.smoke.test.ts`）属于 OpenCode TUI，不是 CCR flavor，不得当 flavor 清理。

## Requirements

### R0 Token 基座

- 删除 `--inner-glow` / `--glass-inner-glow` 及 surface 插件、`Card.vue`、`MainLayout.vue`、`PageHeaderCard.vue`、`base.css`、`AsyncStatePanel.vue`、`EmptyState.vue` 的消费点。静止卡：surface 实底 + `border-subtle` 1px，无投影或仅 `--shadow-sm`。
- 暗色静止卡不得使用 `--shadow-md` 及以上。阴影留给模态 / 浮层 / hover。
- `--glow-*` 只保留 focus 环。删除按钮 hover 光晕、status-dot 辉光、text-glow、tag-glow、pulse-glow。
- 删除死 token `premium.*`、`.text-gradient-*`、aurora/mesh 氛围 token。
- 消费点迁到 `--color-*`。phantom-token（`var(--x, #hex)` / `var(--x-rgb, 245 158 11)`）视同硬编码。
- 对比度阈值不降级：text-primary ≥12:1、secondary ≥7:1、muted ≥4.5:1（对 bg-surface）。只调 token，不改测试常量。

### R1 主题值域与迁移

- `FlavorMode = 'neutral' | 'clay'`。`AccentMode = 'clay'`。`ResolvedFlavor` 不再包含 `latte | mocha`。
- 删除 `tokens.css` 的 latte/mocha 段与多色 accent 段；删除 `isCatppuccinFlavor` / `CATPPUCCIN_FLAVORS` 及解析分支。
- **Flavor 迁移表**（读取时映射，空存储不播种）：

  | 存贮值 | 迁到 |
  |---|---|
  | `paper` / `graphite` / `catppuccin` / `latte` / `frappe` / `macchiato` / `mocha` | `neutral` |
  | `neutral` / `clay` | 原值 |
  | 未知 | `neutral` |

- **Accent 迁移表**（必须覆盖旧表里会指向即将删除值的条目）：

  | 存贮值 | 迁到 |
  |---|---|
  | `mauve` / `sage` / `sky` / `slate` / `sand` / `amber` / `rose` | `clay` |
  | `clay` | 原值 |
  | 未知 | `clay` |

  现行 `slate → sky` 必须删除。`sage` / `sky` 现为合法 `AccentMode`，收窄后必须进迁移表，不能只靠 whitelist 丢弃。

- `themeBootstrap.ts` 与 `index.html` 首绘 IIFE **行为字节等价**。
- 文案：`zh-CN.ts`、`en-US.ts`、`bootMessages.ts` 同步删除 catppuccin / mauve / sage / sky。Wave 0 跑 `bun run test:i18n`。
- 同日改写四个 theme smoke 的断言域（含 mocha 覆盖块必须存在的断言），并更新 `theme-token-contracts.md` 的值域段落。阈值常量不改。

### R2 排版

- `--tracking-normal` 归 0。负字距只留在 ≥1.5rem 的标题档。
- 字重 token 与字面量收敛为 400 / 500 / 600 / 700（去掉 560 / 620 / 640 / 800）。
- eyebrow 单一来源：`0.75rem` / 600 / `0.08em` / uppercase 仅拉丁短标签。`:lang(zh)` / `:lang(zh-CN)` 关闭 tracking 与 uppercase。拉丁短标签在 CJK 界面上标 `lang="en"`。
- 页头标题固定 rem（PageTitle 1.5rem，上限 2rem）。禁止 `clamp(...vw...)` 进入 app 表面。
- 数据数字使用 `tabular-nums`。禁止用语义色 / accent 装饰数字。

### R3 共享原语

- 修 `Card` / `Button` / `Input` / `Badge`（Badge 增加方角档）。
- 新增 **PageHeader**、**StatTile**、**PillToggleGroup**。`PageShell` 从按配方改过的 `OpenCodePageShell` 抽出，禁止把该文件里现有的 16 处字面 `rgb()` 复制进新壳。
- 组件处置：
  - 删除：`ui/StatCard`（由 StatTile 取代）、`ConfigItem.vue`（0 import；勿与类型 `ConfigItem` 混淆）。
  - 保留并按配方改样式：`ui/Sparkline`（`UsageMetricCard`）、`ListSearchHeader`（`McpListPanel`）、`ConfigCard`（`ConfigList`；色值清扫在 R7）。
- 形状：卡 12px；面板 12–16px；控件 8–10px；pill 仅 toggle / chip 与主按钮。

### R4 Shell 与设置

- 侧栏 / 顶栏去掉内高光。导航激活态：tonal 底 + 主文本色 + 细描边。删除左侧 accent 竖条与渐变底。
- 设置 dock 收敛为紧凑控件组。`AppSettingsView` 去掉 catppuccin / mauve / accent 选择器；色板预览 hex 白名单保留。

### R5 Dashboard

- display 标题改为 PageHeader；readiness 进入状态槽。
- 行动队列首行改为 tonal，不用实心 accent 填整行。
- ReadinessLedger 拆嵌套卡，改 StatTile 或数据行。
- UsageMovement / SignalStream 的 range 用 PillToggleGroup；图表柱用实心低彩度。
- 可按「状态 > 下一步 > 洞察 > 入口」重排网格。`dashboardPresentation.ts` 的 signal / readiness 语义不变。

### R6 平台族与功能族

- 51 个视图按 `views-inventory.md` 勾销。平台族先改干净 `OpenCodePageShell`，再铺 PageShell + PageHeader。
- profiles 三视图继续走 `profiles-page.css` + `components/profiles/*`，只修 token 与排版。
- 每页完成后：无硬编码色、无装饰渐变、无 glow；该用的原语已用。
- ConverterView 的 `--bg-primary` / `--text-primary` / `--border-color` / `--shadow-small` / `--accent-primary` 旧别名迁到 `--color-*`。
- Checkin 族受 `apple-glass-surface-contract` 的 `styleLockedPaths` 锁定。改样式时更新锁断言，禁止回退 raw rgb / Tailwind 调色板工具类。

### R7 模态与硬编码色

- 功能模态外壳迁到 `--surface-modal-*`。至少包括 `CodexAgentEditorModal`、`ConfirmModal`、`BulkDeleteDialog`、`AccountFormModal`、`OAuthWizardModal`。Confirm 只换外壳，不改 `requestConfirm` 门闩。
- `ConfigCard` cyan/violet 迁语义 token。
- 审计命令：
  - `rg "#[0-9a-fA-F]{3,8}\b|rgba?\(" ccr-ui/src --glob "*.vue"`
  - `rg "var\(--[a-z0-9-]+-rgb,\s*\d|var\(--[a-z0-9-]+,\s*#"`
- hex / 字面 rgb 白名单见下方 Acceptance。phantom-token 目标为零。MCP 族（`McpDetailPanel` / `McpListPanel` / `McpCreatePanel` / `McpManagerView`）与 `AgentIcons` / `CommandList` / `TokenDetailTab` 是已知残留点。

### R8 验证与文档

- web preview：`bun run dev:web -- --host 127.0.0.1 --strictPort` → `http://127.0.0.1:5173/`。预载 `ccr-theme` / `ccr-flavor` / `ccr-accent` 并断言 dataset 后再截图。
- 截图放在本任务 `evidence/`，命名 `{route}-{locale}-{theme}-{flavor}.png`。全量 51 × zh-CN × light/dark × neutral；en-US 与 clay 抽查 dashboard / 设置 / 两个平台主页 / 一个 profiles / 一个模态。
- Wave 0 当天更新 `theme-token-contracts.md` 值域。Wave 7 再跑一遍 `trellis-update-spec`。
- 修订 `ccr-ui/DESIGN.md`（见 `design.md` §0）。

## Acceptance Criteria

- [ ] `rg "inner-glow" ccr-ui/src` 无结果；卡片 computed style 无 inset 白色高光。
- [ ] 类型域、迁移表、`tokens.css`、设置 UI、`zh-CN` / `en-US` / `bootMessages` 中无 `catppuccin|mocha|latte|mauve|sage|sky` 作为 CCR flavor/accent。`slate → sky` 已删除。旧存储值按 R1 表回落。四个 theme smoke 与 `bun run test:i18n` 全绿。
- [ ] `.vue` 字面 hex/rgb 仅剩白名单：`AppSettingsView` 色板预览。`AgentIcons` 等平台色改为 `--color-platform-*` token，不进白名单。phantom-token 清零。`premium.*` 删除。
- [ ] `ccr-ui/src` 内 `linear-gradient` 仅白名单文件，合计 ≤10 处。允许：`usageChartOptions.ts` / dashboard 用量图在 ApexCharts 必须用渐变对象时的同色双停点；`branding/` 品牌资产。`.vue` 装饰性渐变为零。装饰性 glow 工具类为零。
- [ ] 全局负字距移除；eyebrow 单一来源；`:lang(zh)` 下无字距 / 无 uppercase；操作台无 vw 流体标题；数据数字 `tabular-nums`。
- [ ] Dashboard 首屏实心 accent 填充区域 ≤2 处（主按钮与小型 chip 选中点）。禁止：行动行整行实心、导航竖条、图表柱 accent、range pill 实心。用 `evidence/` 截图标注核对。导航无左侧 accent 竖条。
- [ ] `views-inventory.md` 51 行全部勾销。PageHeader / StatTile / PillToggleGroup 按该表「适用」列接入，未标适用的页不得为凑数而插入。
- [ ] 对比度阈值不降级；`prefers-reduced-transparency` 与 `prefers-reduced-motion` 重置完整（含原 flavor 作用域内的重置，迁到剩余 flavor）。
- [ ] `evidence/` 截图清单齐；en-US / clay 抽查通过。
- [ ] `just ui-check` 通过。
- [ ] `ccr-ui/DESIGN.md` 与 `theme-token-contracts.md` 已按本任务修订。

## Out of Scope

- 路由结构、功能流程、Tauri 后端、`llmusage` 数据层。
- 字体偏好轴与 MapleBright 品牌字体本身。
- 新增 flavor / accent；营销页。
- Checkin / Grok / Commands 等超长页的逻辑拆分与「单页 style ≤300 行」硬性拆档。
- OpenCode TUI 主题字符串（如 `catppuccin-mocha`）。
- VS Code 扩展与文档站换肤。

## Notes

- 视觉验证以 web preview 为准。纯浏览器里 Tauri-only `invoke()` 失败按运行时限制处理。
- 本任务只执行这一次默认主题迁移。之后改 flavor / accent 必须新开任务并再写迁移声明。
- DESIGN.md Don'ts 在修订后仍然有效：禁 side-stripe、禁渐变文字、禁 32px+ 卡片圆角、禁嵌套卡、禁紫蓝渐变。
