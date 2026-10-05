# 硬编码豁免（批次 2–6）

目标：本批次 CSS/`className` 不含 `px` 字面量、不含 `rgba()`。

| 位置 | 形态 | 处理 |
| --- | --- | --- |
| Provider template 选择器 label | `font-size: 0.75rem` | 与 Profiles 密集元信息同档，保留。 |
| AppSettings 侧栏宽度展示 | `{sidebarWidth}px` | 用户可见数值标签，不是 CSS 字面量。 |
| range input min/max | `min={200} max={480}` | 控件属性，不是 CSS px。 |
| flavor 预览 hex | `#e8e9ec` 等 | 静态复制自 tokens.css 的预览白名单，仅用于 `--fp-*`。 |
| `rgb(0 0 0 / 24%)` 模板弹层阴影 | 纯黑阴影 | apple-glass 允许的纯黑例外。 |

无未登记的 `px` / `rgba()` 字面量。
