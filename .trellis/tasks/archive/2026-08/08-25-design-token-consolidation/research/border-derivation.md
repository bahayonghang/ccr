# 边框实色推导

合成公式：`out = round(fg * A + bg * (1 - A))`，逐通道。
承载底色：该作用域 `--color-bg-elevated`（design.md §2）。

## clay 暗（设计稿给定，不推导）

| 令牌 | 实色 | `-rgb` |
|---|---|---|
| `--color-border-subtle` | `#322a25` | `50 42 37` |
| `--color-border-default` | `#3a302a` | `58 48 42` |
| `--color-border-strong` | `#4a3d35` | `74 61 53` |

`#3a302a` vs `--color-bg-surface` `#2a221e` 对比度约 1.216:1，满足既有 ≥1.2:1 门槛。`#322a25` 对 surface 约 1.11:1，测试只锁 default，subtle 按设计稿取值。

## 其余三作用域

| 作用域 | fg | subtle/default/strong | bg elevated |
|---|---|---|---|
| neutral 亮 | `25 27 32` | 12% / 19% / 30% | `#f2f3f5` = `242 243 245` |
| neutral 暗 | `235 238 245` | 14% / 22% / 34% | `#1a1b1f` = `26 27 31` |
| clay 亮 | `70 53 41` | 12% / 19% / 30% | `#f5eee1` = `245 238 225` |

| 作用域 | subtle | default | strong |
|---|---|---|---|
| neutral 亮 | `#d8d9db` `216 217 219` | `#c9cacd` `201 202 205` | `#b1b2b5` `177 178 181` |
| neutral 暗 | `#37393d` `55 57 61` | `#48494e` `72 73 78` | `#616368` `97 99 104` |
| clay 亮 | `#e0d8cb` `224 216 203` | `#d4cbbe` `212 203 190` | `#c1b7aa` `193 183 170` |

## B5 调用点

`rgb(var(--color-border-*-rgb) / x%)` 出现在 usage/mcp/sync/checkin/commands/tray/home.css 与 `ui/primitives.css`。`-rgb` 改为实色分量后，这些二次 alpha 会比旧的高对比前景更淡。本任务不改 feature CSS 选择器结构；语义边框走 `var(--color-border-*)` 的实色。回归走查记录淡化，不在本任务修调用点。
