# 按钮/标签/URL 原语落地

父任务：`08-26-profile-visual-types`
规格：`../08-26-profile-visual-types/research/visual-language.md`

## Goal

在 `src/ui/` 落地可复用的 `Button`、`Badge`、`FieldLabel`、`UrlText` 与 `buttonClass()`，并让同层的 `EmptyState`、`ConfirmModal` 改用 `Button`。本任务不改 Profile 页或其它业务页。

## Requirements

- R1：新增 `button.tsx` / `badge.tsx` / `field-label.tsx` / `url-text.tsx`，从 `src/ui/index.ts` 导出。样式只写 `primitives.css` 的 `.ui-*` 类，消费现有 token，无 hex、无 px 字面量。
- R2：`buttonClass` 与 `Button` 共用同一 class 生成函数。变体与尺寸按父任务封闭集。默认 `type="button"`。
- R3：`Badge` 区分 `static` / `interactive`；static 的 computed `cursor` 不是 `pointer`。
- R4：`UrlText` 调用 `formatBaseUrlDisplay`，`title` 为原始 `value`。非法 URL 原文展示。
- R5：`FieldLabel` 字号 `0.75rem`、muted、tracking `0.08em`。
- R6：`EmptyState` 动作按钮为 `Button variant="primary"`。`ConfirmModal` 确认/取消改为 `Button`，映射为：`type=danger` → 确认 `danger`；`type=warning` → 确认 `warning`；`type=info` → 确认 `primary`；三种 type 的取消均为 `ghost`。不得把三种确认都做成 `primary`。
- R7：测试放 `ccr-ui/tests/ui/`。覆盖变体 CSS 契约、同尺寸同高、focus/active/disabled、static cursor、UrlText 截断与非法回退、FieldLabel 三项、分层不导入 features。
- R8：`prefers-reduced-motion: reduce` 下 `.ui-btn` 无 transform 过渡。

## Acceptance Criteria

- [ ] AC1（R1）：`@/ui` 可 import 四个组件与 `buttonClass`。
- [ ] AC2（R2）：`.ui-btn--{variant}` 的 CSS 声明使用 token `var()`，且满足：`primary` 背景 `--color-accent-primary`、`secondary` 背景 `--color-bg-surface`、`ghost` 背景 `transparent`、`quiet` 无边框、`warning` 背景 `--color-warning-tint` 且边 `--color-warning`、`danger` 背景 `--color-danger`、`accent-soft` 背景为 accent 的 alpha。同 `size` 的七变体渲染高度相等（`getBoundingClientRect().height`）。`focus-visible` 使用 accent ring；`:active` 为 `scale(0.96)`；`:disabled` 为 opacity `0.55` 且 `cursor: not-allowed`。
- [ ] AC3（R3）：static Badge `cursor !== 'pointer'`；interactive 为 `pointer`。
- [ ] AC4（R4）：`https://api.example.com/abcdefghijklmnopqrstuvw`（pathname 长于 18）的 textContent 含 host、含 `…`、短于原文；`title` 等于原文。非法输入 `not-a-url` 的 textContent 与 `title` 均等于原文。
- [ ] AC5（R6）：EmptyState 动作为 `.ui-btn--primary`。分别渲染 ConfirmModal `type=danger|warning|info`：确认按钮 class 为 `.ui-btn--danger` / `.ui-btn--warning` / `.ui-btn--primary`；三种的取消均为 `.ui-btn--ghost`。
- [ ] AC6（R7）：`cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/ui/action-visual-types.smoke.test.tsx` 通过；`bun run check:arch-boundaries` 通过。
- [ ] AC7（R1）：对 `primitives.css` 中 `.ui-btn` / `.ui-badge` / `.ui-field-label` / `.ui-url-text` 规则，`rg` 无 hex、无 `[0-9]+px` 字面量。
- [ ] AC8（R8）：reduced-motion 媒体块取消 `.ui-btn` 的 transform。
- [ ] AC9（R7）：`just frontend-check-quick` 通过。
- [ ] AC10（R5）：`.ui-field-label` 的 CSS 为 `font-size: 0.75rem`、`letter-spacing: 0.08em`、`color: var(--color-text-muted)`；渲染后 `getComputedStyle` 的 `fontSize` 等于根字号 × 0.75。

## Out of scope

- Profile 列表、编辑器、业务页调用点
- 新 token 名
- asChild / Radix Slot

## Notes

- 实现形态对齐现有 `page-header.tsx`：`cn()` + BEM，不引入 cva。
- `bulk-delete-dialog` 按钮可顺手改，非 AC 必须。
