# 外观设置页重排与预览一致性

父任务：`.trellis/tasks/08-25-react-home-style-redesign`
设计输入：父任务 `research/claude-design-source.html` 的「外观设置 · 亮色」区块
审阅裁定：父任务 `research/plan-review-adjudication.md` 的 TPR-07

## Goal

按设计稿重排外观设置页的分区与版式，并为 flavor 预览取值建立与 `tokens.css` 的一致性机制。
当前页面已具备大部分功能，本任务的实际增量是「版式重排」加「预览取值防漂移」两项，不是从零搭建。

## 前置

`08-25-design-token-consolidation` 先合入。

## Background and Confirmed Facts

`AppearanceSection.tsx` 现状（**已存在，不是本任务新增**）：

- 主题三选项 `light` / `dark` / `system`，经 `ThemeOption` 渲染。
- flavor 两张卡 `neutral` / `clay`，经 `FlavorCard` + `flavorPreviewStyle` 渲染带预览。
- UI 字体与代码字体两个下拉，`data-testid="settings-font-ui"` / `settings-font-code`，含自定义输入。
- 留空回退提示 callout。

真实缺口：

- `ccr-ui/src/features/configs/lib/flavorPreview.ts` 的 `FLAVOR_PREVIEW_TOKENS` 用 20 个十六进制字面量
  镜像 `tokens.css` 的 `--color-bg-base` / `-elevated` / `-surface` / `--color-text-primary` / `-muted`，
  覆盖 neutral/clay × light/dark 四组。**两处取值之间没有任何一致性机制**，`tokens.css` 改了预览不会跟着改。
- 该文件是 `.ts`，`hardcode-px-rgba.smoke.test.ts` 只查 px 与 `rgba()`，不查十六进制，因此现有门禁不覆盖它。
- 令牌子任务只改边框与圆角取值，不改 `--color-bg-*` 与 `--color-text-*`，所以这 20 个值在令牌子任务后**仍然正确**。
  本任务要解决的是「将来会漂移」，不是「现在是错的」。

## Requirements

- R1：产出前后差异表，逐项写明每块 UI 是「保持不变」「仅换版式」「行为改变」中的哪一类。差异表是本任务其余需求的范围依据。
- R2：主题三选项与 flavor 两选项合并进同一张卡，各自带简短说明；`theme === 'system'` 时显示当前解析结果。沿用既有 props 与回调，不改 `AppSettingsView` 接口。
- R3：字体卡给出界面字体与数据字体两个下拉，选择「自定义」时展开输入框；下方给出中文与 Latin 混排预览，数据样例用 mono 显示数字与金额。既有 `data-testid` 全部保留。
- R4：字体卡保留留空回退提示（回退到内置字体栈，缺字形逐级回退）。
- R5：为 `FLAVOR_PREVIEW_TOKENS` 与 `tokens.css` 建立一致性机制：新增测试解析 `tokens.css` 的四个作用域，
  断言预览映射表的每个取值与对应令牌一致。`tokens.css` 单方面改动时该测试必须失败。
- R6：不暴露失效组合。当前 flavor 值域仅 `neutral | clay`，accent 值域恒为 `clay`，界面不得呈现已退役的 mocha / latte 等选项。
- R7：选中态不只依赖颜色——需另有边框、图标或文字标记。
- R8：所有样式用语义令牌，不写硬编码十六进制颜色与 px 圆角字面量。`flavorPreview.ts` 的字面量是本任务显式接受的例外，由 R5 的测试守护。
- R9：本子任务必须包含测试改动（R5 的一致性测试），测试文件列入 change list。

## Acceptance Criteria

- [ ] AC1（R1）：`design.md` §1 的前后差异表每行都有分类结论，无「待定」。
- [ ] AC2（R2）：明暗三选项与底色族两选项在同一张卡内可见并可切换；选 `system` 时显示解析结果文案。
- [ ] AC3（R3,R4）：两个字体下拉可用，自定义输入可展开，混排预览与回退提示可见；数据样例为 mono；`settings-font-ui` 与 `settings-font-code` 仍存在。
- [ ] AC4（R5）：一致性测试存在并通过。人为把 `tokens.css` 的某个 `--color-bg-base` 改一位十六进制，该测试必须失败——此项在实施时验证一次后还原。
- [ ] AC5（R5）：切换 flavor 后预览色条与页面实际表面色一致（亮暗各验一次）。
- [ ] AC6（R6）：界面上不出现 `neutral` / `clay` 以外的 flavor 选项。
- [ ] AC7（R7）：灰度模拟下仍能判断当前选中的主题与 flavor。
- [ ] AC8（R8）：本子任务改动的 CSS 中无硬编码十六进制颜色与 px 圆角字面量。
- [ ] AC9（R9）：change list 与提交包含测试文件改动。
- [ ] AC10：`just frontend-check-quick` 通过；`app-settings-view.smoke.test.tsx` 与字体相关 smoke test 不回归。
- [ ] AC11：中英文文案键完整。

## Out of Scope

- 主题/flavor/字体的持久化与 bootstrap 逻辑（`themeBootstrap`、`fontPreferences`）。
- 设置页的其他分区（同步、更新、通用等）。
- 新增 flavor 或 accent 值域。
- 把 `FLAVOR_PREVIEW_TOKENS` 改为运行时从 CSS 读取。R5 选择用测试守护字面量，不做运行时求值——
  运行时求值需要探针元素与 `getComputedStyle`，改动面远大于收益。
</content>
