# 硬编码豁免登记（08-22-views-claude）

本批次目标：组件内 CSS/`className` 不含 `px` 字面量、不含 `rgba()`。

| 位置 | 形态 | 处理 |
| --- | --- | --- |
| Observer ApexCharts `height={260\|280}` | 图表 API 数字高度 | 非 CSS px 字面量；保留。 |
| Heatmap `colorScale.ranges[].color` | 原 `rgba(125,151,182,…)` | 改为 `rgb(var(--color-info-rgb) / α)`。 |
| Home / Auth / Hooks 交互热区 | 原 `min-height: 44px` | 改为 `min-h-11`（2.75rem）。 |
| Console 圆角 | 原 `border-radius: 12px` | 改为 `rounded-xl`。 |

无登记豁免的 `px` / `rgba()` 字面量。
