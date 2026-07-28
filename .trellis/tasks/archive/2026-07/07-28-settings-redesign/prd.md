# 设置系统重设计：AppSettingsView 重构与新视觉偏好体系接入

## Goal

重设计 `/settings` 页面（整体样式重设计的一部分）：信息架构与卡片布局重构、外观区接入子任务 A 的新 flavor/accent 值域、预览用真实 token 渲染；同步 i18n/bootMessages/MainLayout dock 与全部 settings 相关 smoke 测试。

## Background

- 父任务：`07-28-ccr-ui-visual-redesign`；依赖 `07-28-color-system-rebuild`（新 flavor 3 值、accent 4 值、对比度达标的新 palette）。
- 现状测绘（2026-07-27 审计）：`AppSettingsView.vue` 1365 行，Hero + 左侧 section 导航（appearance/language/shell/diagnostics）+ 卡片流；无 settings 专用子组件，全部内联实现；`settings` 命名空间约 100 键，在 `en-US.ts`/`zh-CN.ts` 与 `bootMessages.ts` 各有一份；testid 被 `app-settings.smoke.test.ts` 逐字锁定；`MainLayout.vue` settings dock 展示主题/语言摘要。
- 用户截图红框标注：外观设置区（主题模式 + flavor 选择）与侧栏底部 settings dock。

## Requirements

### R1. 信息架构与布局重构

- 保持四 section（appearance/language/shell/diagnostics）与路由 `/settings` 不变；Hero 摘要 pill 行保留但按新视觉语言重排。
- 卡片层级、间距、标题层级按新设计系统（不透明表面、实心文本、清晰边界）重排；左侧 section 导航样式与选中态重设计。
- 页面在新 palette 下明暗两套都需通过对比度自查。

### R2. 外观区重构（核心）

- **主题模式**：light/dark/system 三选项改分段控件（segmented control），带当前解析结果指示。
- **flavor 选择器**：选项更新为 `neutral`（默认）/ `clay` / `catppuccin` 三项；每项预览 swatch 用**真实 token 渲染**（mini 卡片：bg-base 底 + bg-surface 卡 + text-primary/muted 文本 + accent 条），不再是脱离实际的渐变块；catppuccin 项标注自适应解析（light→latte / dark→mocha）。
- **accent 选择器**：选项更新为 `clay/sage/sky/mauve` 四项；预览为实心按钮样例（accent 底 + contrast 文字）+ 文字色样例。
- 字体卡（UI/代码字体）行为不变，样式接入新视觉；预览区对比度达标。

### R3. 文案与 i18n 同步

- `settings` 命名空间文案按新体系重写（ flavor/accent 名称与描述、段落描述）；`en-US.ts` / `zh-CN.ts` / `bootMessages.ts` 三处同步（bootMessages 是首屏副本，遗漏会导致设置页注水前渲染旧文案）。
- 删除旧 flavor/accent 的 i18n 键（paper/graphite/latte/frappe/macchiato/mocha 独立键、sand/amber/rose/slate 键）。
- 文案不得含 `{` `}` `|`（vue-i18n 编译约束）。

### R4. dock 与测试同步

- `MainLayout.vue` settings dock 摘要（主题/语言 pill）按新值域更新显示逻辑与样式。
- 更新 `app-settings.smoke.test.ts`（新选项 testid、新值域断言）、`main-layout-theme-stage.smoke.test.ts`（dock）；保留现有 testid 命名（`settings-theme-*` 等）以最小化测试 churn，结构变化导致的必要新增/删除在测试中同步。
- `theme-bootstrap.smoke.test.ts` 与 `apple-glass-surface-contract.smoke.test.ts` 由子任务 A 负责，本任务只处理 UI 结构引起的断言变化。

## Acceptance Criteria

- [ ] AC1: `/settings` 在 light/dark × 3 flavor 下渲染正常：flavor 三项、accent 四项可选可切，切换后 `data-flavor`/`data-resolved-flavor`/`data-accent` 与 localStorage 正确更新；catppuccin 在 dark 下解析为 mocha、light 下解析为 latte。
- [ ] AC2: flavor/accent 预览 swatch 的计算样式确实来自当前 token（Playwright 或测试断言 preview 元素引用了 `--color-bg-*`/`--color-accent-*` 变量）。
- [ ] AC3: `bootMessages.ts` 与完整语言包的 `settings` 键集合一致（无旧 flavor/accent 键残留）；`bun run test:i18n` 通过。
- [ ] AC4: `app-settings.smoke.test.ts`、`main-layout-theme-stage.smoke.test.ts` 更新后通过；`bun run type-check && bun run lint` 通过。
- [ ] AC5: 视觉核验：settings 页 dark+neutral 截图，信息层级清晰、预览真实、无泛白（证据存父任务 research/）。

## Out Of Scope

- 新增设置项或偏好维度（密度/字号等）；`DesktopShellPreferences` 字段变更。
- 平台级 SettingsView（ClaudeCodeSettingsView 等）重设计。
- 设置页路由与懒加载策略变更。
