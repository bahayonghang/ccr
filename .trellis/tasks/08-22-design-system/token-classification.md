# token 分类表（batch 1）

> 依据 `.trellis/tasks/08-22-design-system/design.md` §2 分类方法对 `ccr-ui/src/styles/tokens.css`
> 的 448 个变量定义点逐条分类。**448 行，无未分类项。**
>
> 生成：`classify-tokens.mjs`（bun）。名称集合基线见 `token-names-before.txt`。

## 分类统计（按唯一名）

| 类 | 唯一名数 |
| --- | --- |
| 可切换语义变量 | 87 |
| 常量 token | 117 |
| 计算 token（跟随输入） | 74 |
| 计算 token（仅 @media 降级定义，跟随输入） | 0 |

## 分类方法（design.md §2）

1. 对每个变量名收集 `tokens.css` 全部顶层规则块中的定义点（`@media` 内定义为降级覆盖，不参与计数）。
2. 归一化选择器：剥掉 `:where()` / `html` / `:root` 链，取 `[data-*]` 属性值对集合为上下文。
3. 出现在 **2 个以上不同上下文**（`:root` + `[data-theme=...]` / `[data-flavor=...]` / `[data-accent=...]`）→ **可切换语义变量（第 1 层）**。
4. 单上下文且值为字面量（间距、圆角、字号、字重、时长、z-index…）→ **常量 token（进 @theme）**。
5. 值引用其他变量（`var()` / `calc()` / `color-mix()`）→ **计算/别名 token，跟随其输入变量的类别**。

> 批次 1 落位说明：第 1 层变量**物理上仍留在 `tokens.css`**（批次 1 与批次 2 之间的
> 兼容约束，见 `implement.md` 批次 1 证据块——`theme-contrast-contract` / `apple-glass-surface-contract` /
> `theme-bootstrap` 三个 smoke 测试直接解析 `tokens.css` 文本）。「目标落位」列记录的是
> design.md §3 的目标位置，批次 2 目录分层时随测试契约重建（批次 8）落地。

## 明细（448 行，按源码顺序）

| # | 变量名 | 定义点值（节选） | 类 | 目标落位 |
| --- | --- | --- | --- | --- |
| 1 | `--color-bg-base` | `#e8e9ec` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 2 | `--color-bg-base-rgb` | `232 233 236` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 3 | `--color-bg-elevated` | `#f2f3f5` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 4 | `--color-bg-elevated-rgb` | `242 243 245` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 5 | `--color-bg-surface` | `#fbfcfd` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 6 | `--color-bg-surface-rgb` | `251 252 253` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 7 | `--color-bg-overlay` | `#dcdee3` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 8 | `--color-bg-overlay-rgb` | `220 222 227` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 9 | `--color-scrim` | `rgb(25 27 32 / 32%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 10 | `--color-scrim-rgb` | `25 27 32` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 11 | `--color-border-subtle` | `rgb(25 27 32 / 12%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 12 | `--color-border-subtle-rgb` | `25 27 32` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 13 | `--color-border-default` | `rgb(25 27 32 / 19%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 14 | `--color-border-default-rgb` | `25 27 32` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 15 | `--color-border-strong` | `rgb(25 27 32 / 30%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 16 | `--color-border-strong-rgb` | `25 27 32` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 17 | `--color-border-accent` | `rgb(var(--color-accent-primary-rgb) / 18%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 18 | `--color-border-interactive` | `var(--color-accent-primary)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 19 | `--color-border-interactive-rgb` | `var(--color-accent-primary-rgb)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 20 | `--color-text-primary` | `#191b20` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 21 | `--color-text-primary-rgb` | `25 27 32` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 22 | `--color-text-secondary` | `#3f434c` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 23 | `--color-text-secondary-rgb` | `63 67 76` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 24 | `--color-text-muted` | `#5f646e` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 25 | `--color-text-muted-rgb` | `95 100 110` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 26 | `--color-text-ghost` | `#878d98` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 27 | `--color-text-ghost-rgb` | `135 141 152` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 28 | `--color-text-disabled` | `#b3b8c0` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 29 | `--color-text-disabled-rgb` | `179 184 192` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 30 | `--color-text-inverted` | `#f7f8fa` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 31 | `--color-text-inverted-rgb` | `247 248 250` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 32 | `--color-accent-primary` | `#cf6239` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 33 | `--color-accent-primary-hover` | `#d9714a` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 34 | `--color-accent-primary-active` | `#b8542f` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 35 | `--color-accent-primary-rgb` | `207 98 57` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 36 | `--color-accent-primary-glow` | `rgb(207 98 57 / 10%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 37 | `--color-accent-primary-contrast` | `#fff8f2` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 38 | `--color-accent-primary-contrast-rgb` | `255 248 242` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 39 | `--color-accent-secondary` | `#b99666` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 40 | `--color-accent-secondary-hover` | `#c6a777` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 41 | `--color-accent-secondary-active` | `#a98658` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 42 | `--color-accent-secondary-rgb` | `185 150 102` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 43 | `--color-accent-secondary-glow` | `rgb(185 150 102 / 10%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 44 | `--color-success` | `#5b8a62` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 45 | `--color-success-hover` | `#67996f` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 46 | `--color-success-rgb` | `91 138 98` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 47 | `--color-success-glow` | `rgb(91 138 98 / 10%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 48 | `--color-success-contrast` | `#fff8f2` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 49 | `--color-success-contrast-rgb` | `255 248 242` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 50 | `--color-warning` | `#bc8540` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 51 | `--color-warning-hover` | `#ca9453` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 52 | `--color-warning-rgb` | `188 133 64` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 53 | `--color-warning-glow` | `rgb(188 133 64 / 10%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 54 | `--color-warning-contrast` | `#17181c` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 55 | `--color-warning-contrast-rgb` | `23 24 28` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 56 | `--color-danger` | `#c76953` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 57 | `--color-danger-hover` | `#d57a65` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 58 | `--color-danger-rgb` | `199 105 83` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 59 | `--color-danger-glow` | `rgb(199 105 83 / 10%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 60 | `--color-danger-contrast` | `#fff8f2` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 61 | `--color-danger-contrast-rgb` | `255 248 242` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 62 | `--color-info` | `#7d97b6` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 63 | `--color-info-hover` | `#8ea8c6` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 64 | `--color-info-rgb` | `125 151 182` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 65 | `--color-info-glow` | `rgb(125 151 182 / 10%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 66 | `--color-cyan` | `#8fa5bd` | 常量 token | @theme（非 inline） |
| 67 | `--color-cyan-rgb` | `143 165 189` | 常量 token | @theme（非 inline） |
| 68 | `--color-teal` | `#7d948b` | 常量 token | @theme（非 inline） |
| 69 | `--color-teal-rgb` | `125 148 139` | 常量 token | @theme（非 inline） |
| 70 | `--color-gray` | `#857367` | 常量 token | @theme（非 inline） |
| 71 | `--color-gray-rgb` | `133 115 103` | 常量 token | @theme（非 inline） |
| 72 | `--color-platform-claude` | `#d97757` | 常量 token | @theme（非 inline） |
| 73 | `--color-platform-claude-rgb` | `217 119 87` | 常量 token | @theme（非 inline） |
| 74 | `--color-platform-codex` | `#5b8a62` | 常量 token | @theme（非 inline） |
| 75 | `--color-platform-codex-rgb` | `91 138 98` | 常量 token | @theme（非 inline） |
| 76 | `--color-platform-grok` | `#716b80` | 常量 token | @theme（非 inline） |
| 77 | `--color-platform-grok-rgb` | `113 107 128` | 常量 token | @theme（非 inline） |
| 78 | `--color-platform-gemini` | `#7d97b6` | 常量 token | @theme（非 inline） |
| 79 | `--color-platform-gemini-rgb` | `125 151 182` | 常量 token | @theme（非 inline） |
| 80 | `--state-loading` | `var(--color-info)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 81 | `--state-error` | `var(--color-danger)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 82 | `--animation-state` | `running` | 常量 token | @theme（非 inline） |
| 83 | `--color-stage-text-primary` | `var(--color-text-primary)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 84 | `--color-stage-text-secondary` | `var(--color-text-secondary)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 85 | `--color-stage-text-muted` | `var(--color-text-muted)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 86 | `--color-stage-text-quiet` | `var(--color-text-ghost)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 87 | `--color-stage-surface-soft` | `var(--color-bg-surface)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 88 | `--color-stage-surface-medium` | `var(--color-bg-elevated)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 89 | `--color-stage-surface-strong` | `var(--color-bg-overlay)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 90 | `--color-stage-border-soft` | `var(--color-border-subtle)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 91 | `--color-stage-border-medium` | `var(--color-border-default)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 92 | `--color-stage-chip-neutral-bg` | `var(--color-bg-overlay)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 93 | `--color-stage-chip-neutral-border` | `var(--color-border-default)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 94 | `--color-stage-chip-neutral-text` | `var(--color-text-secondary)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 95 | `--color-bg-base` | `#131316` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 96 | `--color-bg-base-rgb` | `19 19 22` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 97 | `--color-bg-elevated` | `#1a1b1f` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 98 | `--color-bg-elevated-rgb` | `26 27 31` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 99 | `--color-bg-surface` | `#22242a` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 100 | `--color-bg-surface-rgb` | `34 36 42` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 101 | `--color-bg-overlay` | `#2c2f37` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 102 | `--color-bg-overlay-rgb` | `44 47 55` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 103 | `--color-scrim` | `rgb(0 0 0 / 56%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 104 | `--color-scrim-rgb` | `0 0 0` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 105 | `--color-border-subtle` | `rgb(235 238 245 / 14%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 106 | `--color-border-subtle-rgb` | `235 238 245` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 107 | `--color-border-default` | `rgb(235 238 245 / 22%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 108 | `--color-border-default-rgb` | `235 238 245` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 109 | `--color-border-strong` | `rgb(235 238 245 / 34%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 110 | `--color-border-strong-rgb` | `235 238 245` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 111 | `--color-border-accent` | `rgb(var(--color-accent-primary-rgb) / 24%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 112 | `--color-text-primary` | `#f2f3f5` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 113 | `--color-text-primary-rgb` | `242 243 245` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 114 | `--color-text-secondary` | `#c9ccd3` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 115 | `--color-text-secondary-rgb` | `201 204 211` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 116 | `--color-text-muted` | `#9ba1ab` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 117 | `--color-text-muted-rgb` | `155 161 171` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 118 | `--color-text-ghost` | `#6d727c` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 119 | `--color-text-ghost-rgb` | `109 114 124` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 120 | `--color-text-disabled` | `#4f545d` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 121 | `--color-text-disabled-rgb` | `79 84 93` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 122 | `--color-text-inverted` | `#17181c` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 123 | `--color-text-inverted-rgb` | `23 24 28` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 124 | `--color-accent-primary` | `#e8835b` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 125 | `--color-accent-primary-hover` | `#f0926c` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 126 | `--color-accent-primary-active` | `#d4744a` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 127 | `--color-accent-primary-rgb` | `232 131 91` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 128 | `--color-accent-primary-glow` | `rgb(232 131 91 / 16%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 129 | `--color-accent-primary-contrast` | `#1d1207` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 130 | `--color-accent-primary-contrast-rgb` | `29 18 7` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 131 | `--color-accent-secondary` | `#d0ae86` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 132 | `--color-accent-secondary-hover` | `#d9bb96` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 133 | `--color-accent-secondary-active` | `#c3a177` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 134 | `--color-accent-secondary-rgb` | `208 174 134` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 135 | `--color-accent-secondary-glow` | `rgb(208 174 134 / 16%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 136 | `--color-success` | `#7cab82` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 137 | `--color-success-hover` | `#8bbb91` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 138 | `--color-success-rgb` | `124 171 130` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 139 | `--color-success-glow` | `rgb(124 171 130 / 16%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 140 | `--color-success-contrast` | `#17181c` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 141 | `--color-success-contrast-rgb` | `23 24 28` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 142 | `--color-warning` | `#d6a76d` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 143 | `--color-warning-hover` | `#dfb47d` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 144 | `--color-warning-rgb` | `214 167 109` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 145 | `--color-warning-glow` | `rgb(214 167 109 / 16%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 146 | `--color-warning-contrast` | `#17181c` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 147 | `--color-warning-contrast-rgb` | `23 24 28` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 148 | `--color-danger` | `#db8a73` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 149 | `--color-danger-hover` | `#e59b86` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 150 | `--color-danger-rgb` | `219 138 115` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 151 | `--color-danger-glow` | `rgb(219 138 115 / 16%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 152 | `--color-danger-contrast` | `#17181c` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 153 | `--color-danger-contrast-rgb` | `23 24 28` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 154 | `--color-info` | `#98afc9` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 155 | `--color-info-hover` | `#a7bdd4` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 156 | `--color-info-rgb` | `152 175 201` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 157 | `--color-info-glow` | `rgb(152 175 201 / 16%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 158 | `--space-0` | `0` | 常量 token | @theme（非 inline） |
| 159 | `--space-px` | `1px` | 常量 token | @theme（非 inline） |
| 160 | `--space-0-5` | `0.125rem` | 常量 token | @theme（非 inline） |
| 161 | `--space-1` | `0.25rem` | 常量 token | @theme（非 inline） |
| 162 | `--space-1-5` | `0.375rem` | 常量 token | @theme（非 inline） |
| 163 | `--space-2` | `0.5rem` | 常量 token | @theme（非 inline） |
| 164 | `--space-2-5` | `0.625rem` | 常量 token | @theme（非 inline） |
| 165 | `--space-3` | `0.75rem` | 常量 token | @theme（非 inline） |
| 166 | `--space-3-5` | `0.875rem` | 常量 token | @theme（非 inline） |
| 167 | `--space-4` | `1rem` | 常量 token | @theme（非 inline） |
| 168 | `--space-5` | `1.25rem` | 常量 token | @theme（非 inline） |
| 169 | `--space-6` | `1.5rem` | 常量 token | @theme（非 inline） |
| 170 | `--space-7` | `1.75rem` | 常量 token | @theme（非 inline） |
| 171 | `--space-8` | `2rem` | 常量 token | @theme（非 inline） |
| 172 | `--space-9` | `2.25rem` | 常量 token | @theme（非 inline） |
| 173 | `--space-10` | `2.5rem` | 常量 token | @theme（非 inline） |
| 174 | `--space-11` | `2.75rem` | 常量 token | @theme（非 inline） |
| 175 | `--space-12` | `3rem` | 常量 token | @theme（非 inline） |
| 176 | `--space-14` | `3.5rem` | 常量 token | @theme（非 inline） |
| 177 | `--space-16` | `4rem` | 常量 token | @theme（非 inline） |
| 178 | `--space-20` | `5rem` | 常量 token | @theme（非 inline） |
| 179 | `--space-24` | `6rem` | 常量 token | @theme（非 inline） |
| 180 | `--space-28` | `7rem` | 常量 token | @theme（非 inline） |
| 181 | `--space-32` | `8rem` | 常量 token | @theme（非 inline） |
| 182 | `--font-sans-base` | `'MapleBright', 'SF Pro Text', 'PingFang SC', 'Microsoft YaHei UI', 'Microsoft YaHei', sans-serif` | 常量 token | @theme（非 inline） |
| 183 | `--font-brand-base` | `'SF Pro Display', 'Segoe UI Variable Display', 'PingFang SC', 'Microsoft YaHei UI',
    'Microsoft YaHei', sans-serif` | 常量 token | @theme（非 inline） |
| 184 | `--font-mono-base` | `'Cascadia Code', 'Cascadia Mono', 'SFMono-Regular', ui-monospace, 'Consolas', 'MapleBright',
    monospace` | 常量 token | @theme（非 inline） |
| 185 | `--font-sans` | `var(--font-sans-base)` | 计算 token（跟随输入） | @theme（非 inline） |
| 186 | `--font-brand` | `var(--font-brand-base)` | 计算 token（跟随输入） | @theme（非 inline） |
| 187 | `--font-mono` | `var(--font-mono-base)` | 计算 token（跟随输入） | @theme（非 inline） |
| 188 | `--text-xs` | `0.75rem` | 常量 token | @theme（非 inline） |
| 189 | `--text-sm` | `0.8125rem` | 常量 token | @theme（非 inline） |
| 190 | `--text-base` | `1rem` | 常量 token | @theme（非 inline） |
| 191 | `--text-lg` | `1.0625rem` | 常量 token | @theme（非 inline） |
| 192 | `--text-xl` | `1.3125rem` | 常量 token | @theme（非 inline） |
| 193 | `--text-2xl` | `1.625rem` | 常量 token | @theme（非 inline） |
| 194 | `--text-3xl` | `2rem` | 常量 token | @theme（非 inline） |
| 195 | `--text-4xl` | `2.5rem` | 常量 token | @theme（非 inline） |
| 196 | `--text-5xl` | `3.5rem` | 常量 token | @theme（非 inline） |
| 197 | `--text-6xl` | `4.25rem` | 常量 token | @theme（非 inline） |
| 198 | `--font-normal` | `400` | 常量 token | @theme（非 inline） |
| 199 | `--font-medium` | `500` | 常量 token | @theme（非 inline） |
| 200 | `--font-semibold` | `600` | 常量 token | @theme（非 inline） |
| 201 | `--font-bold` | `700` | 常量 token | @theme（非 inline） |
| 202 | `--font-extrabold` | `700` | 常量 token | @theme（非 inline） |
| 203 | `--leading-none` | `1` | 常量 token | @theme（非 inline） |
| 204 | `--leading-tight` | `1.12` | 常量 token | @theme（非 inline） |
| 205 | `--leading-snug` | `1.24` | 常量 token | @theme（非 inline） |
| 206 | `--leading-normal` | `1.56` | 常量 token | @theme（非 inline） |
| 207 | `--leading-relaxed` | `1.68` | 常量 token | @theme（非 inline） |
| 208 | `--leading-loose` | `2` | 常量 token | @theme（非 inline） |
| 209 | `--tracking-tighter` | `-0.05em` | 常量 token | @theme（非 inline） |
| 210 | `--tracking-tight` | `-0.028em` | 常量 token | @theme（非 inline） |
| 211 | `--tracking-normal` | `0` | 常量 token | @theme（非 inline） |
| 212 | `--tracking-wide` | `0.018em` | 常量 token | @theme（非 inline） |
| 213 | `--tracking-wider` | `0.05em` | 常量 token | @theme（非 inline） |
| 214 | `--tracking-widest` | `0.1em` | 常量 token | @theme（非 inline） |
| 215 | `--radius-none` | `0` | 常量 token | @theme（非 inline） |
| 216 | `--radius-sm` | `4px` | 常量 token | @theme（非 inline） |
| 217 | `--radius-md` | `6px` | 常量 token | @theme（非 inline） |
| 218 | `--radius-lg` | `8px` | 常量 token | @theme（非 inline） |
| 219 | `--radius-xl` | `10px` | 常量 token | @theme（非 inline） |
| 220 | `--radius-2xl` | `12px` | 常量 token | @theme（非 inline） |
| 221 | `--radius-3xl` | `16px` | 常量 token | @theme（非 inline） |
| 222 | `--radius-full` | `9999px` | 常量 token | @theme（非 inline） |
| 223 | `--shadow-xs` | `0 1px 2px rgb(25 27 32 / 6%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 224 | `--shadow-sm` | `0 2px 6px rgb(25 27 32 / 9%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 225 | `--shadow-md` | `0 10px 24px rgb(25 27 32 / 13%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 226 | `--shadow-lg` | `0 18px 38px rgb(25 27 32 / 16%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 227 | `--shadow-xl` | `0 26px 54px rgb(25 27 32 / 19%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 228 | `--shadow-2xl` | `0 34px 72px rgb(25 27 32 / 22%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 229 | `--shadow-inner` | `inset 0 1px 0 rgb(255 255 255 / 46%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 230 | `--glow-primary` | `0 0 0 2px rgb(var(--color-accent-primary-rgb) / 40%)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 231 | `--glow-secondary` | `0 0 0 2px rgb(var(--color-accent-secondary-rgb) / 40%)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 232 | `--glow-success` | `0 0 0 2px rgb(var(--color-success-rgb) / 40%)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 233 | `--glow-warning` | `0 0 0 2px rgb(var(--color-warning-rgb) / 40%)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 234 | `--glow-danger` | `0 0 0 2px rgb(var(--color-danger-rgb) / 40%)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 235 | `--glow-info` | `0 0 0 2px rgb(var(--color-info-rgb) / 40%)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 236 | `--gradient-border` | `var(--color-border-subtle)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 237 | `--gradient-brand` | `var(--color-accent-primary)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 238 | `--elevation-0` | `none` | 常量 token | @theme（非 inline） |
| 239 | `--elevation-1` | `var(--shadow-sm)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 240 | `--elevation-2` | `var(--shadow-md)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 241 | `--elevation-3` | `var(--shadow-lg)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 242 | `--elevation-4` | `var(--shadow-xl)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 243 | `--shadow-xs` | `0 1px 2px rgb(0 0 0 / 22%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 244 | `--shadow-sm` | `0 2px 6px rgb(0 0 0 / 26%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 245 | `--shadow-md` | `0 12px 28px rgb(0 0 0 / 30%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 246 | `--shadow-lg` | `0 20px 44px rgb(0 0 0 / 36%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 247 | `--shadow-xl` | `0 28px 60px rgb(0 0 0 / 44%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 248 | `--shadow-2xl` | `0 36px 78px rgb(0 0 0 / 50%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 249 | `--shadow-inner` | `inset 0 1px 0 rgb(235 238 245 / 3%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 250 | `--material-glass-floating-bg` | `rgb(var(--color-bg-elevated-rgb) / 92%)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 251 | `--material-glass-floating-blur` | `blur(12px)` | 常量 token | @theme（非 inline） |
| 252 | `--material-glass-floating-border` | `var(--color-border-strong)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 253 | `--material-glass-floating-highlight` | `inset 0 1px 0 rgb(255 255 255 / 40%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 254 | `--material-glass-floating-shadow` | `0 24px 64px rgb(25 27 32 / 20%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 255 | `--material-glass-chrome-bg` | `var(--color-bg-elevated)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 256 | `--material-glass-chrome-blur` | `none` | 常量 token | @theme（非 inline） |
| 257 | `--material-glass-chrome-border` | `var(--color-border-default)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 258 | `--material-glass-chrome-highlight` | `inset 0 1px 0 rgb(255 255 255 / 30%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 259 | `--material-glass-chrome-shadow` | `var(--shadow-sm)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 260 | `--material-glass-inline-bg` | `var(--color-bg-surface)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 261 | `--material-glass-inline-blur` | `none` | 常量 token | @theme（非 inline） |
| 262 | `--material-glass-inline-border` | `var(--color-border-default)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 263 | `--material-glass-inline-highlight` | `inset 0 1px 0 rgb(255 255 255 / 30%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 264 | `--material-glass-inline-shadow` | `var(--shadow-sm)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 265 | `--glass-bg-light` | `rgb(var(--color-bg-surface-rgb) / 86%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 266 | `--glass-bg-medium` | `rgb(var(--color-bg-elevated-rgb) / 92%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 267 | `--glass-bg-strong` | `rgb(var(--color-bg-surface-rgb) / 96%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 268 | `--glass-border-light` | `rgb(var(--color-border-default-rgb) / 12%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 269 | `--glass-border-medium` | `rgb(var(--color-border-default-rgb) / 18%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 270 | `--glass-border-strong` | `rgb(var(--color-border-default-rgb) / 26%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 271 | `--glass-blur-sm` | `blur(2px) saturate(104%)` | 常量 token | @theme（非 inline） |
| 272 | `--glass-blur-md` | `blur(3px) saturate(106%)` | 常量 token | @theme（非 inline） |
| 273 | `--glass-blur-lg` | `blur(4px) saturate(108%)` | 常量 token | @theme（非 inline） |
| 274 | `--glass-blur-xl` | `blur(6px) saturate(110%)` | 常量 token | @theme（非 inline） |
| 275 | `--glass-shadow` | `0 10px 24px rgb(73 54 40 / 13%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 276 | `--glass-shadow-elevated` | `0 18px 40px rgb(73 54 40 / 16%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 277 | `--liquid-glass-bg` | `rgb(var(--color-bg-elevated-rgb) / 92%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 278 | `--liquid-glass-blur` | `blur(4px) saturate(106%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 279 | `--liquid-glass-border` | `rgb(var(--color-border-default-rgb) / 16%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 280 | `--liquid-glass-highlight` | `inset 0 1px 0 rgb(255 251 245 / 38%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 281 | `--liquid-glass-shadow` | `0 12px 30px rgb(73 54 40 / 10%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 282 | `--surface-shell-bg` | `var(--material-glass-chrome-bg)` | 计算 token（跟随输入） | 第 1 层（跟随计算输入） |
| 283 | `--surface-shell-blur` | `var(--material-glass-chrome-blur)` | 计算 token（跟随输入） | @theme（非 inline） |
| 284 | `--surface-shell-border` | `var(--material-glass-chrome-border)` | 计算 token（跟随输入） | 第 1 层（跟随计算输入） |
| 285 | `--surface-shell-shadow` | `var(--material-glass-chrome-shadow)` | 计算 token（跟随输入） | 第 1 层（跟随计算输入） |
| 286 | `--surface-workspace-bg` | `var(--color-bg-elevated)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 287 | `--surface-workspace-blur` | `none` | 常量 token | @theme（非 inline） |
| 288 | `--surface-workspace-border` | `var(--color-border-default)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 289 | `--surface-workspace-shadow` | `var(--shadow-sm)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 290 | `--surface-card-bg` | `var(--color-bg-surface)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 291 | `--surface-card-blur` | `none` | 常量 token | @theme（非 inline） |
| 292 | `--surface-card-border` | `var(--color-border-subtle)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 293 | `--surface-card-shadow` | `none` | 常量 token | @theme（非 inline） |
| 294 | `--surface-modal-bg` | `var(--material-glass-floating-bg)` | 计算 token（跟随输入） | 第 1 层（跟随计算输入） |
| 295 | `--surface-modal-blur` | `var(--material-glass-floating-blur)` | 计算 token（跟随输入） | @theme（非 inline） |
| 296 | `--surface-modal-border` | `var(--material-glass-floating-border)` | 计算 token（跟随输入） | 第 1 层（跟随计算输入） |
| 297 | `--surface-modal-shadow` | `var(--material-glass-floating-shadow)` | 计算 token（跟随输入） | 第 1 层（跟随可切换输入） |
| 298 | `--surface-status-bg` | `var(--material-glass-inline-bg)` | 计算 token（跟随输入） | 第 1 层（跟随计算输入） |
| 299 | `--surface-status-blur` | `var(--material-glass-inline-blur)` | 计算 token（跟随输入） | @theme（非 inline） |
| 300 | `--surface-status-border` | `var(--material-glass-inline-border)` | 计算 token（跟随输入） | 第 1 层（跟随计算输入） |
| 301 | `--surface-status-shadow` | `var(--material-glass-inline-shadow)` | 计算 token（跟随输入） | 第 1 层（跟随计算输入） |
| 302 | `--material-glass-floating-highlight` | `inset 0 1px 0 rgb(235 238 245 / 3%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 303 | `--material-glass-floating-shadow` | `0 24px 64px rgb(0 0 0 / 52%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 304 | `--material-glass-chrome-highlight` | `inset 0 1px 0 rgb(235 238 245 / 3%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 305 | `--material-glass-inline-highlight` | `inset 0 1px 0 rgb(235 238 245 / 3%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 306 | `--glass-bg-light` | `rgb(34 27 24 / 92%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 307 | `--glass-bg-medium` | `rgb(34 27 24 / 96%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 308 | `--glass-bg-strong` | `rgb(42 34 30 / 98%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 309 | `--glass-border-light` | `rgb(243 234 223 / 12%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 310 | `--glass-border-medium` | `rgb(243 234 223 / 16%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 311 | `--glass-border-strong` | `rgb(243 234 223 / 22%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 312 | `--glass-shadow` | `0 12px 28px rgb(0 0 0 / 34%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 313 | `--glass-shadow-elevated` | `0 20px 46px rgb(0 0 0 / 42%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 314 | `--liquid-glass-bg` | `rgb(34 27 24 / 96%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 315 | `--liquid-glass-blur` | `blur(4px) saturate(106%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 316 | `--liquid-glass-border` | `rgb(243 234 223 / 16%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 317 | `--liquid-glass-highlight` | `inset 0 1px 0 rgb(255 248 240 / 5%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 318 | `--liquid-glass-shadow` | `0 16px 36px rgb(0 0 0 / 38%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 319 | `--duration-instant` | `50ms` | 常量 token | @theme（非 inline） |
| 320 | `--duration-fast` | `100ms` | 常量 token | @theme（非 inline） |
| 321 | `--duration-normal` | `200ms` | 常量 token | @theme（非 inline） |
| 322 | `--duration-slow` | `300ms` | 常量 token | @theme（非 inline） |
| 323 | `--duration-slower` | `500ms` | 常量 token | @theme（非 inline） |
| 324 | `--ease-linear` | `linear` | 常量 token | @theme（非 inline） |
| 325 | `--ease-in` | `cubic-bezier(0.4, 0, 1, 1)` | 常量 token | @theme（非 inline） |
| 326 | `--ease-out` | `cubic-bezier(0, 0, 0.2, 1)` | 常量 token | @theme（非 inline） |
| 327 | `--ease-in-out` | `cubic-bezier(0.4, 0, 0.2, 1)` | 常量 token | @theme（非 inline） |
| 328 | `--ease-out-back` | `cubic-bezier(0.34, 1.56, 0.64, 1)` | 常量 token | @theme（非 inline） |
| 329 | `--ease-spring` | `cubic-bezier(0.175, 0.885, 0.32, 1.275)` | 常量 token | @theme（非 inline） |
| 330 | `--motion-none-duration` | `0ms` | 常量 token | @theme（非 inline） |
| 331 | `--motion-subtle-duration` | `var(--duration-fast)` | 计算 token（跟随输入） | @theme（非 inline） |
| 332 | `--motion-standard-duration` | `var(--duration-normal)` | 计算 token（跟随输入） | @theme（非 inline） |
| 333 | `--motion-enter-duration` | `var(--duration-normal)` | 计算 token（跟随输入） | @theme（非 inline） |
| 334 | `--motion-exit-duration` | `var(--duration-fast)` | 计算 token（跟随输入） | @theme（非 inline） |
| 335 | `--motion-feedback-duration` | `var(--duration-fast)` | 计算 token（跟随输入） | @theme（非 inline） |
| 336 | `--motion-subtle-ease` | `var(--ease-out)` | 计算 token（跟随输入） | @theme（非 inline） |
| 337 | `--motion-standard-ease` | `var(--ease-in-out)` | 计算 token（跟随输入） | @theme（非 inline） |
| 338 | `--motion-enter-ease` | `var(--ease-out)` | 计算 token（跟随输入） | @theme（非 inline） |
| 339 | `--motion-exit-ease` | `var(--ease-in)` | 计算 token（跟随输入） | @theme（非 inline） |
| 340 | `--motion-feedback-ease` | `var(--ease-out-back)` | 计算 token（跟随输入） | @theme（非 inline） |
| 341 | `--layer-background` | `-50` | 常量 token | @theme（非 inline） |
| 342 | `--layer-base` | `0` | 常量 token | @theme（非 inline） |
| 343 | `--layer-raised` | `5` | 常量 token | @theme（非 inline） |
| 344 | `--layer-sticky` | `10` | 常量 token | @theme（非 inline） |
| 345 | `--layer-dropdown` | `20` | 常量 token | @theme（非 inline） |
| 346 | `--layer-modal-backdrop` | `30` | 常量 token | @theme（非 inline） |
| 347 | `--layer-modal` | `40` | 常量 token | @theme（非 inline） |
| 348 | `--layer-popover` | `50` | 常量 token | @theme（非 inline） |
| 349 | `--layer-tooltip` | `60` | 常量 token | @theme（非 inline） |
| 350 | `--layer-toast` | `70` | 常量 token | @theme（非 inline） |
| 351 | `--z-behind` | `-1` | 常量 token | @theme（非 inline） |
| 352 | `--z-base` | `var(--layer-base)` | 计算 token（跟随输入） | @theme（非 inline） |
| 353 | `--z-dropdown` | `var(--layer-dropdown)` | 计算 token（跟随输入） | @theme（非 inline） |
| 354 | `--z-sticky` | `var(--layer-sticky)` | 计算 token（跟随输入） | @theme（非 inline） |
| 355 | `--z-fixed` | `30` | 常量 token | @theme（非 inline） |
| 356 | `--z-modal-backdrop` | `var(--layer-modal-backdrop)` | 计算 token（跟随输入） | @theme（非 inline） |
| 357 | `--z-modal` | `var(--layer-modal)` | 计算 token（跟随输入） | @theme（非 inline） |
| 358 | `--z-popover` | `var(--layer-popover)` | 计算 token（跟随输入） | @theme（非 inline） |
| 359 | `--z-tooltip` | `var(--layer-tooltip)` | 计算 token（跟随输入） | @theme（非 inline） |
| 360 | `--z-toast` | `var(--layer-toast)` | 计算 token（跟随输入） | @theme（非 inline） |
| 361 | `--breakpoint-sm` | `640px` | 常量 token | @theme（非 inline） |
| 362 | `--breakpoint-md` | `768px` | 常量 token | @theme（非 inline） |
| 363 | `--breakpoint-lg` | `1024px` | 常量 token | @theme（非 inline） |
| 364 | `--breakpoint-xl` | `1280px` | 常量 token | @theme（非 inline） |
| 365 | `--breakpoint-2xl` | `1536px` | 常量 token | @theme（非 inline） |
| 366 | `--color-bg-base` | `#ebe1d0` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 367 | `--color-bg-base-rgb` | `235 225 208` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 368 | `--color-bg-elevated` | `#f5eee1` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 369 | `--color-bg-elevated-rgb` | `245 238 225` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 370 | `--color-bg-surface` | `#fefaf2` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 371 | `--color-bg-surface-rgb` | `254 250 242` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 372 | `--color-bg-overlay` | `#e2d6c3` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 373 | `--color-bg-overlay-rgb` | `226 214 195` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 374 | `--color-border-subtle` | `rgb(70 53 41 / 12%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 375 | `--color-border-subtle-rgb` | `70 53 41` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 376 | `--color-border-default` | `rgb(70 53 41 / 19%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 377 | `--color-border-default-rgb` | `70 53 41` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 378 | `--color-border-strong` | `rgb(70 53 41 / 30%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 379 | `--color-border-strong-rgb` | `70 53 41` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 380 | `--color-text-primary` | `#31241c` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 381 | `--color-text-primary-rgb` | `49 36 28` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 382 | `--color-text-secondary` | `#5f4d3f` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 383 | `--color-text-secondary-rgb` | `95 77 63` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 384 | `--color-text-muted` | `#715d4c` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 385 | `--color-text-muted-rgb` | `113 93 76` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 386 | `--color-text-ghost` | `#9a8373` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 387 | `--color-text-ghost-rgb` | `154 131 115` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 388 | `--color-text-disabled` | `#b4a08f` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 389 | `--color-text-disabled-rgb` | `180 160 143` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 390 | `--color-text-inverted` | `#fff8f0` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 391 | `--color-text-inverted-rgb` | `255 248 240` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 392 | `--color-bg-base` | `#17120f` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 393 | `--color-bg-base-rgb` | `23 18 15` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 394 | `--color-bg-elevated` | `#221b18` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 395 | `--color-bg-elevated-rgb` | `34 27 24` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 396 | `--color-bg-surface` | `#2a221e` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 397 | `--color-bg-surface-rgb` | `42 34 30` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 398 | `--color-bg-overlay` | `#342b26` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 399 | `--color-bg-overlay-rgb` | `52 43 38` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 400 | `--color-border-subtle` | `rgb(243 234 223 / 14%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 401 | `--color-border-subtle-rgb` | `243 234 223` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 402 | `--color-border-default` | `rgb(243 234 223 / 22%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 403 | `--color-border-default-rgb` | `243 234 223` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 404 | `--color-border-strong` | `rgb(243 234 223 / 34%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 405 | `--color-border-strong-rgb` | `243 234 223` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 406 | `--color-text-primary` | `#f3eadf` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 407 | `--color-text-primary-rgb` | `243 234 223` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 408 | `--color-text-secondary` | `#dacbbc` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 409 | `--color-text-secondary-rgb` | `218 203 188` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 410 | `--color-text-muted` | `#b9a695` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 411 | `--color-text-muted-rgb` | `185 166 149` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 412 | `--color-text-ghost` | `#977f6d` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 413 | `--color-text-ghost-rgb` | `151 127 109` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 414 | `--color-text-disabled` | `#735f52` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 415 | `--color-text-disabled-rgb` | `115 95 82` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 416 | `--color-text-inverted` | `#211915` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 417 | `--color-text-inverted-rgb` | `33 25 21` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 418 | `--material-glass-floating-bg` | `var(--color-bg-elevated)` | 计算 token（跟随输入）（@media 降级） | 第 1 层（跟随可切换输入） |
| 419 | `--material-glass-floating-blur` | `none` | 常量 token（@media 降级） | @theme（非 inline） |
| 420 | `--material-glass-chrome-bg` | `var(--color-bg-elevated)` | 计算 token（跟随输入）（@media 降级） | 第 1 层（跟随可切换输入） |
| 421 | `--material-glass-chrome-blur` | `none` | 常量 token（@media 降级） | @theme（非 inline） |
| 422 | `--material-glass-inline-bg` | `var(--color-bg-elevated)` | 计算 token（跟随输入）（@media 降级） | 第 1 层（跟随可切换输入） |
| 423 | `--material-glass-inline-blur` | `none` | 常量 token（@media 降级） | @theme（非 inline） |
| 424 | `--glass-bg-light` | `var(--color-bg-elevated)` | 可切换语义变量（@media 降级） | 第 1 层（themes/ 普通 CSS 变量） |
| 425 | `--glass-bg-medium` | `var(--color-bg-elevated)` | 可切换语义变量（@media 降级） | 第 1 层（themes/ 普通 CSS 变量） |
| 426 | `--glass-bg-strong` | `var(--color-bg-surface)` | 可切换语义变量（@media 降级） | 第 1 层（themes/ 普通 CSS 变量） |
| 427 | `--glass-blur-sm` | `none` | 常量 token（@media 降级） | @theme（非 inline） |
| 428 | `--glass-blur-md` | `none` | 常量 token（@media 降级） | @theme（非 inline） |
| 429 | `--glass-blur-lg` | `none` | 常量 token（@media 降级） | @theme（非 inline） |
| 430 | `--glass-blur-xl` | `none` | 常量 token（@media 降级） | @theme（非 inline） |
| 431 | `--liquid-glass-bg` | `var(--color-bg-elevated)` | 可切换语义变量（@media 降级） | 第 1 层（themes/ 普通 CSS 变量） |
| 432 | `--liquid-glass-blur` | `none` | 可切换语义变量（@media 降级） | 第 1 层（themes/ 普通 CSS 变量） |
| 433 | `--color-accent-primary` | `#cf6239` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 434 | `--color-accent-primary-hover` | `#d9714a` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 435 | `--color-accent-primary-active` | `#b8542f` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 436 | `--color-accent-primary-rgb` | `207 98 57` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 437 | `--color-accent-primary-glow` | `rgb(207 98 57 / 10%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 438 | `--color-accent-primary-contrast` | `#fff8f2` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 439 | `--color-accent-primary-contrast-rgb` | `255 248 242` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 440 | `--color-border-accent` | `rgb(207 98 57 / 18%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 441 | `--color-accent-primary` | `#e8835b` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 442 | `--color-accent-primary-hover` | `#f0926c` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 443 | `--color-accent-primary-active` | `#d4744a` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 444 | `--color-accent-primary-rgb` | `232 131 91` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 445 | `--color-accent-primary-glow` | `rgb(232 131 91 / 16%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 446 | `--color-accent-primary-contrast` | `#1d1207` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 447 | `--color-accent-primary-contrast-rgb` | `29 18 7` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |
| 448 | `--color-border-accent` | `rgb(232 131 91 / 24%)` | 可切换语义变量 | 第 1 层（themes/ 普通 CSS 变量） |

---
共 448 行（tokens.css 内全部自定义属性定义点）。未分类项：0。
