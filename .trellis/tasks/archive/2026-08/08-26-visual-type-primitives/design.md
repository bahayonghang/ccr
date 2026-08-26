# 原语落地 — 设计

## 文件

| 文件 | 职责 |
| --- | --- |
| `ccr-ui/src/ui/button.tsx` | `buttonClass` + `Button` |
| `ccr-ui/src/ui/badge.tsx` | `Badge` |
| `ccr-ui/src/ui/field-label.tsx` | `FieldLabel` |
| `ccr-ui/src/ui/url-text.tsx` | `UrlText` |
| `ccr-ui/src/ui/primitives.css` | `.ui-btn*` `.ui-badge*` `.ui-field-label` `.ui-url-text` |
| `ccr-ui/src/ui/index.ts` | 导出 |
| `ccr-ui/src/ui/empty-state.tsx` | 动作改 `Button variant="primary"` |
| `ccr-ui/src/ui/confirm-modal.tsx` | footer 改 `Button`：确认按 `type` 映射 `danger`/`warning`/`primary`，取消 `ghost` |

## class 约定

`.ui-btn` + `.ui-btn--{variant}` + `.ui-btn--{size}`。不要同时保留一套 Tailwind 工具类和一套 CSS 变体；视觉值进 CSS，组件只拼 class。

焦点：`focus-visible` + accent ring（`--color-accent-primary` glow）。`:active`：`transform: scale(0.96)`。disabled：opacity 0.55 + `not-allowed`。同一 `size` 的七变体 `min-height` 相同（`sm` / `md` 各一档）。

变体 CSS 必须写 token，禁止 hex/px：

| variant | 背景 | 边框 | 字色 |
| --- | --- | --- | --- |
| `primary` | `var(--color-accent-primary)` | 同背景 | `var(--color-text-inverted)` |
| `secondary` | `var(--color-bg-surface)` | `var(--color-border-default)` | `var(--color-text-primary)` |
| `ghost` | `transparent` | `var(--color-border-subtle)` | `var(--color-text-muted)` |
| `quiet` | `transparent` | `none` | `var(--color-text-muted)` |
| `warning` | `var(--color-warning-tint)` | `var(--color-warning)` | `var(--color-warning)` |
| `danger` | `var(--color-danger)` | 同背景 | `var(--color-text-inverted)` |
| `accent-soft` | `rgb(var(--color-accent-primary-rgb) / 14%)` | `rgb(var(--color-accent-primary-rgb) / 35%)` | `var(--color-accent-primary)` |

## UrlText

```ts
export function UrlText({ value, className }: { value: string; className?: string })
```

空字符串由调用方决定不渲染。组件内不对空值画 `-`。

截断向量：`https://api.example.com/abcdefghijklmnopqrstuvw`（pathname `/abcdefghijklmnopqrstuvw` 长度 23 > 18），展示必须含 host 与 `…`。非法向量：`not-a-url`，展示与 `title` 均为原文。

## 测试

`ccr-ui/tests/ui/action-visual-types.smoke.test.tsx`：

- 读取 `.ui-btn--*` / `.ui-field-label` 的 stylesheet 文本或 `getComputedStyle`，断言上表 token 与 FieldLabel 三项。
- 同 `size` 七变体高度相等。
- ConfirmModal 三种 `type` 分别断言确认/取消 variant class。
- UrlText 使用上面两条向量，不只检查「短于原文」。
- `rg` 扫描新 `.ui-*` 规则的 hex/`px`。

分层：现有 arch 边界测试应覆盖新文件路径（若 allowlist 要加，只加测试夹具，不加生产豁免）。

## 风险

`primitives.css` 被全站加载。选择器必须带 `.ui-` 前缀，避免撞 `.cp-btn`。本任务还不删 `.cp-btn`。
