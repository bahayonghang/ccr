# 语义 tint 推导

accent tint 不新增名称（选项 A）：`rgb(var(--color-accent-primary-rgb) / 12%)`。

clay 暗四色取设计稿 1c：

| 令牌 | 实色 |
|---|---|
| `--color-success-tint` | `#25332a` |
| `--color-warning-tint` | `#3a2a20` |
| `--color-danger-tint` | `#2b1f1c` |
| `--color-info-tint` | `#252d33` |

其余三作用域：该主题语义色 12% 叠在 `--color-bg-surface` 上。

| 作用域 | success | warning | danger | info |
|---|---|---|---|---|
| neutral 亮 | `#e8eeea` | `#f3eee6` | `#f5eae9` | `#ecf0f4` |
| neutral 暗 | `#2d3435` | `#383432` | `#383033` | `#30353d` |
| clay 亮 | `#eaede1` | `#f6ecdd` | `#f7e9df` | `#efeeeb` |
