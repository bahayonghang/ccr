# Tailwind v4 与 shadcn/ui 设计体系重建

> 父任务：`08-22-react-migration`

## Goal

在 React 侧建立单一样式决策平面：Tailwind v4 `@theme` 承载 token，shadcn/ui 原语承载交互组件，组件内不再出现硬编码样式值。本任务是「降低开发者改样式成本」这一原始目标的直接交付物。

## Scope

### token 层

| 项目 | 现状 | 目标 |
|---|---|---|
| `src/styles/tokens.css` | 448 个 CSS 变量，26.7 KB | 分两集合落位（见下）。工具类可直接引用，且运行时主题切换生效 |
| `src/styles/*.css` 总量 | 4,026 行，18 个文件 | 按 base / tokens / themes / components / utilities 分层落位 |
| `src/styles/{base,components,themes,utilities}/` | 4 个空目录（历史未完成的分层重构） | 填充或删除，不保留空目录 |
| 组件内 `<style>` | 24,434 行，覆盖 139 / 185 个组件 | 迁移后组件级样式只表达布局与该组件独有规则 |

**两集合模型**（Tailwind v4 的 `@theme` 与 `@theme inline` 语义不同，细节见 `design.md` §1）：

| 集合 | 内容 | 落位 |
|---|---|---|
| 稳定语义变量集合 | 在 `[data-theme]` / `[data-flavor]` / `[data-accent]` 下有不同取值的变量 | `src/styles/themes/` 下的普通 CSS 变量，**不在** `@theme` 内 |
| Tailwind namespace 映射集合 | 指向上一集合的映射（如 `--color-surface-1: var(--surface-1)`） | `@theme inline` |
| 常量 token | 全主题同值（间距、圆角、字号、字重、时长、z-index） | `@theme`（非 inline） |

可切换的值不能直接写进 `@theme inline` 的字面量位置——那会让工具类内联死值，主题切换失效。

两集合的**变量名**并集等于迁移前 `tokens.css` 的 448 个名字，无新增无删除（4,097 处 `var(--)` 引用与 `theme-token-contracts.md` 的断言依赖这些名字）。

### 硬编码值收口

| 类型 | 数量 | 处理 |
|---|---|---|
| `.vue` 内 px 字面量 | 1,639 | 映射到间距 / 字号 / 圆角 token |
| `.vue` 内 `rgba()` / `rgb()` | 932 | 映射到颜色与材质 token |
| `.vue` 内 hex 颜色 | 20 | 同上 |
| `.css` 内 px 字面量 | 290 | 同上 |
| `.css` 内 hex 颜色 | 102 | 同上 |

`theme-token-contracts.md` 已禁止 px 字面量字号，并登记了一处例外：Profiles 共享层的密集元信息可用 `0.75rem`（低于 Label 下限 `0.8125rem` 一档）。该例外需在新体系中保留。

图表与画布等确需字面量的场景逐个登记豁免，不强制收口。

### 原语层

现状：`src/components/ui/` 有 16 个手写原语（2,201 行）：`AsyncStatePanel`、`Badge`、`Breadcrumb`、`Button`、`Card`、`EmptyState`、`IconWrapper`、`Input`、`NavItem`、`PageHeader`、`PageShell`、`PillToggleGroup`、`SIcon`、`Sparkline`、`Spinner`、`StatTile`。

迁移后位置为 `src/ui/`（父任务 `design.md` §2 的目标目录结构，`08-22-arch-quality-perf` 的 boundary element 亦按 `ui/` 建模）。本任务落地后 `src/components/ui/` 不再存在。

缺口：`Dropdown`、`Tooltip`、`Popover`、`Tabs`、`Accordion`、`Combobox` 命名文件各 0 个，对应交互由手写 div 承担，需先普查定位。

目标：接入 shadcn/ui，至少覆盖 Dialog、Popover、DropdownMenu、Tooltip、Tabs、Combobox、Select、Switch、Checkbox。现有 16 个原语逐个判定：由 shadcn/ui 替换，或保留并改为消费新 token。

### 弹层收口

- 33 个文件引用 `BaseModal`（`src/components/common/BaseModal.vue`），该文件是现有收口点。
- 13 个文件自行实现 `fixed inset-0` 弹层，未走收口点。
- 18 个 `*Modal.vue` + 5 个 `*Dialog.vue`。

目标：焦点陷阱、Esc 关闭、滚动锁定、层级管理只有一处实现。

### 主题配置域

- 保留 `data-theme` / `data-flavor` / `data-accent` 三层模型语义（见 `theme-token-contracts.md`）。
- 当前值域：`FlavorMode = 'neutral' | 'clay'`，`AccentMode = 'clay'`，`DEFAULT_FLAVOR = 'neutral'`，`DEFAULT_ACCENT = 'clay'`。
- 目标：值域可扩展，`themeBootstrap` 支持自定义 accent 输入。
- 存储键 `ccr-theme` 等视觉偏好键的读写兼容保留，旧值可正常解析。

## Requirements

- R1 448 个变量按两集合模型落位（Scope 表）：稳定语义变量集合为普通 CSS 变量，Tailwind namespace 映射集合进 `@theme inline`，常量 token 进 `@theme`。工具类可直接引用，运行时切换 `data-theme` / `data-flavor` / `data-accent` 后生效。
- R1.2 两集合的变量名并集等于迁移前 `tokens.css` 的 448 个名字，无新增无删除。
- R1.1 组件内样式承载方式为「Tailwind 工具类为主，残余进 CSS Modules」（父任务 `design.md` §6）。24,434 行组件内样式尽量压为工具类，复杂选择器与关键帧动画留 `.module.css`。
- R2 组件内 px 字面量降到 0，`rgba()` / `rgb()` 降到 0，hex 降到 0。豁免项逐个登记并说明原因。
- R3 `src/styles/` 下无空目录。
- R4 shadcn/ui 原语接入，覆盖 R3.2 列出的 9 类交互组件。
- R5 33 个 `BaseModal` 调用点与 13 个自实现弹层统一到单一 Dialog 原语。
- R6 现有 16 个手写原语逐个给出「替换 / 保留」判定。
- R7 `data-flavor` 与 `data-accent` 值域可扩展，`themeBootstrap` 支持自定义 accent。
- R8 明暗两套主题的对比度不低于迁移前，reduced motion 降级路径保留。
- R9 视觉方向遵循 `ccr-ui/CLAUDE.md` 的 Design Context。禁止引入 `Neko` / `anime` / `purple-tech` / `guofeng` 分支。
- R10 `theme-token-contracts.md`（31.5 KB）重写，含 `0.75rem` 字号例外的保留说明。

## Acceptance Criteria

- [ ] AC1 `src/styles/**` 下 px 字面量数等于登记豁免数，超出项为 0。`rg -o '[0-9]+px' -g '*.css' src/styles | wc -l`。
- [ ] AC2 `src/styles/**` 下 `rgba()` / `rgb()` 数为 0（豁免同上）。
- [ ] AC3 `ls src/styles/{base,components,themes,utilities}` 无空目录。
- [ ] AC4 9 类 shadcn/ui 原语在 `src/ui/` 下可用，各有一个消费示例。`src/components/ui/` 不再存在。
- [ ] AC5 弹层行为（焦点陷阱、Esc、滚动锁定、层级）在代码中只有一处实现，由 smoke 测试断言。
- [ ] AC6 16 个原有原语判定表落盘，无未判定项。
- [ ] AC7 切换 `data-flavor` 与 `data-accent` 到新增值后界面正确响应，由 smoke 测试断言。
- [ ] AC8 明暗主题对比度检查通过，`prefers-reduced-motion` 下动效降级生效。
- [ ] AC9 `theme-token-contracts.md` 重写完成，无残留 Vue 文件路径与 SFC 模式引用。
- [ ] AC10 `bun run lint:style` 退出码 0。
- [ ] AC11 改一处 token 值可同时影响所有消费点，由一个跨组件的验证用例证明。
- [ ] AC12 `src/**/*.tsx` 内 px 字面量与 `rgba()` / `rgb()` 数为 0（豁免逐条登记）。**本项在阶段 5 结束时由父任务视图门核对**，其归零动作由七个视图子任务随各自迁移完成（本任务只提供 `hardcode-mapping.md` 查表依据）。列在此处以保证该检查有明确归属，不作为本任务交付门的准出条件。
- [ ] AC13 两集合的变量名并集等于迁移前 `tokens.css` 的 448 个名字，由脚本比对前后名字集合证明（R1.2）。

## 前置与后续

- 前置：`08-22-dep-upgrade`（Tailwind 已在 v4）。
- 后续：`08-22-state-logic-port`。本任务必须在 `08-22-views-*` 全部子任务之前完成原语与 token 层，避免视图迁移时重复决策样式。

## Out of Scope

- 视图与业务组件的迁移。
- 信息架构与页面布局重做。
- 新增视觉风格分支。
- 图表配色以外的 ApexCharts 桥接（属 `08-22-views-usage`）。

## Notes

- `src/styles/chart-colors.css`（5 个变量）与 `usage-chart-stability-contracts.md`、`apexcharts-style-contract.smoke.test.ts` 存在耦合，token 迁移时需同步。
- **与动画选型的协同**：`animations.css`（580 行）与 `src/styles/animations/`（空目录）的处理需与 `motion` 13.1.1 的引入协同（父任务 `design.md` §9）。580 行逐段判定：进出场类交给 motion，装饰类与关键帧保留 CSS。禁止同一元素同一属性由 CSS 动画与 motion 两套机制同时驱动。判定结果逐段落盘。
- **reduced motion 收敛到一处**：现状散在多个组件的 `@media (prefers-reduced-motion)` 与 `useAnimationVisibility.ts` 两处逻辑，需与 motion 的 reduced motion API 合并为单一实现，避免双轨。
- `codex-auth-shared.css`（14.8 KB）、`home.css`（41 变量）、`profiles-page.css`（28 变量）是三处按页面聚合的样式文件，迁移时判定归入 components 层或保留页面级。
- 硬编码收口建议按 `views/` 域分批推进，与 `08-22-views-*` 的批次对齐，而非一次性完成。
