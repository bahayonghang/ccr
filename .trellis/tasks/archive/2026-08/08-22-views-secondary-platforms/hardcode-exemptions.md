# 硬编码豁免登记（AC4）

本批次组件使用 token 类与 `var(--stage-*)` / `var(--color-*)` / `var(--platform-*)`。

未使用 `px` 字面量或 `rgba()`。

`color-mix(in srgb, var(--platform-gemini) …)` 与 `color-mix(in srgb, var(--color-platform-grok) …)` 是 token 混色，不是硬编码色值。
