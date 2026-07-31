# 配色系统重构：中性高对比默认 + flavor/accent 精简 + 氛围层收敛

## Goal

重建 `tokens.css` 的配色几何与表面契约，把"泛白"的三条结构性根因（半透明叠层、半透明文本、粉彩 accent）在令牌层一次性修复；flavor 7→3、accent 8→4 精简并完成存储迁移；收敛全局氛围层。

## Background

- 父任务：`07-28-ccr-ui-visual-redesign`；根因证据见 `../07-28-ccr-ui-visual-redesign/research/diagnosis.md`。
- 用户决策：中性高对比现代风新默认；精简 flavor/accent；氛围层大幅收敛。
- 本任务是另外两个子任务的前置：新令牌值、新表面契约、新 flavor/accent 值域都由本任务定义。

## Requirements

### R1. 新配色几何（令牌值重建，令牌名不变）

- 暗色 elevation 逐级提亮（base < elevated < surface < overlay 明度递增），亮色保持"桌面压暗、卡片最亮"。
- `--surface-card-*` / `--surface-workspace-*` 提升到 100% 不透明；`--color-stage-surface-soft/medium/strong` 改不透明阶梯。
- 全部文本令牌（含 `--color-stage-text-*`）100% 不透明；对比度下限：primary ≥ 12:1、secondary ≥ 7:1、muted ≥ 4.5:1（对各自表面，WCAG 公式）。
- 边框可见性：暗色 subtle ≥ 14%、default ≥ 22%、strong ≥ 34%（对底色的相对对比）。
- 暗色移除全局 `antialiased/grayscale` 字体平滑（改 `auto`）。

### R2. flavor 精简 7 → 3 + 迁移

- 新值域：`neutral`（新默认）/ `clay` / `catppuccin`（自适应：light→latte、dark→mocha）。
- 迁移表：`paper|graphite → neutral`；`latte|frappe|macchiato|mocha → catppuccin`；非法值 → `neutral`。
- `themeBootstrap.ts` 与 `index.html` 首帧 IIFE 双实现同步；`FlavorMode` 类型、`FLAVOR_MODES`、`DEFAULT_FLAVOR`、Catppuccin 解析逻辑全部更新。
- `tokens.css` 删除 frappe/macchiato 调色板块与 paper/graphite flavor 块；保留 latte/mocha 调色板供 catppuccin 解析；Catppuccin 语义重映射修复 elevation 反转（elevated 不得暗于 base）。
- clay flavor 保留（暖色身份，作为普通 flavor），明暗两套按 R1 几何重修。

### R3. accent 精简 8 → 4 + 迁移

- 新值域：`clay`（默认）/ `sage` / `sky` / `mauve`。
- 迁移表：`sand|amber|rose → clay`；`slate → sky`；非法值 → `clay`。
- 暗色 accent 提高彩度（不再粉彩化）；Catppuccin flavor 下 accent 映射改为彩度更高的 ctp 对应色。
- 新增"实心按钮文字"语义：`--color-accent-primary-contrast`（暗色下为深色值），供子任务 B 把按钮白字改掉。

### R4. 氛围层收敛

- `StageBackground.vue`：删除 halo × 2、顶部洗色带、噪点层，只留不透明 `--color-bg-base` 基底（组件保留以免改动挂载点）。
- `AnimatedBackground.vue`：同样收敛为静态不透明基底（或在其 3 个挂载点直接移除组件，取改动更小者）。
- `backgrounds.css` 死代码（`.premium-background`/`.premium-bg-orb`/`.orb-*`/`.premium-bg-pattern`）与 `tokens.css` 的 `--stage-bg-mesh/aurora/orb/grid-*` 死令牌删除；`deferred-decorations.css` 同步清理。
- 玻璃契约：chrome 档（侧栏/顶栏）改不透明 elevated；inline 档改不透明 surface；floating 档保留 `blur ≤ 12px` 但 bg 不透明度 ≥ 88%、去掉 `saturate()`；`--surface-shell-*` / `--surface-status-*` / `--surface-modal-*` 语义别名重指。
- `prefers-reduced-transparency` 重置块同步覆盖新契约（含 mocha 作用域）。

### R5. 确定性 bug 修复

- `PlatformMcpView.vue:410,415`、`PlatformPluginsView.vue:371,376`：JS 内联白背景改令牌驱动 CSS（hover 态走类切换）。
- `OutputStylesView.vue:253,339`：`bg-white` 补暗色守卫或改语义类。

### R6. 新守卫测试 + 存量测试更新

- 新增 `tests/theme-contrast-contract.smoke.test.ts`：解析 tokens.css，按 theme × resolved-flavor 计算文本/表面/边框对比度并断言下限；断言表面令牌不透明、stage 文本令牌 100% 不透明。
- 更新 `theme-bootstrap.smoke.test.ts`（新值域、迁移表、IIFE 逐字锁定）、`apple-glass-surface-contract.smoke.test.ts`（新玻璃契约）、`app-settings.smoke.test.ts` 中与 flavor/accent 值域相关的断言（选项 UI 在子任务 C 改，本任务只保证值域逻辑）。

## Acceptance Criteria

- [ ] AC1: `theme-contrast-contract.smoke.test.ts` 新增并通过：light/dark × neutral/clay/latte/mocha 共 6 组组合的文本/表面/边框对比度全部达标；内容表面令牌 100% 不透明。
- [ ] AC2: flavor/accent 迁移单测通过：`paper→neutral`、`graphite→neutral`、`macchiato→catppuccin`、`mocha→catppuccin`、`sand→clay`、`slate→sky`、非法值回退；`data-resolved-flavor` 仅出现 `neutral|clay|latte|mocha`。
- [ ] AC3: `rg "stage-bg-(mesh|aurora|orb|grid)" ccr-ui/src` 无结果；`rg "premium-bg-orb|premium-background" ccr-ui/src` 无结果；StageBackground/AnimatedBackground 渲染输出无 halo/grain DOM。
- [ ] AC4: `PlatformMcpView`/`PlatformPluginsView`/`OutputStylesView` 在暗色下无白色内联背景/无守卫 `bg-white`（focused 测试或 rg 证据）。
- [ ] AC5: `bun run type-check && bun run lint` 通过；`theme-bootstrap` / `apple-glass-surface-contract` smoke 通过（`app-settings` 若因值域断言需要更新则一并改，UI 结构断言留给子任务 C）。
- [ ] AC6: 手动视觉核验：dark+neutral 下 Overview 页截图与诊断前对比，无背景雾、卡片边界清晰、文本实心（证据存 `../07-28-ccr-ui-visual-redesign/research/`）。

## Out Of Scope

- 组件层 247 处 alpha 表面、inset 高光、按钮白字迁移（子任务 B）。
- `/settings` UI 结构与文案重设计（子任务 C）；本任务仅保证 flavor/accent 值域逻辑与迁移。
- 字体轨道、字号、间距、圆角系统变更。
- i18n 文案重写（flavor/accent 显示名变更由子任务 C 统一处理；本任务新增键时保持最小新增）。
