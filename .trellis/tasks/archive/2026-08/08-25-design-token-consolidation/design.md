# 技术设计：令牌层收敛与名称治理

改动文件：`ccr-ui/src/styles/tokens.css`、按需的 `ccr-ui/src/styles/theme.css`、受影响的测试文件、`.trellis/spec/ccr-ui/frontend/theme-token-contracts.md`、本任务 `research/`。

## 0. 名称治理是本设计的第一约束

`theme-token-contracts.md:26` 冻结了 448 个令牌名称。任何新增名称都必须经过登记。
因此本设计的排序是：**先把能用取值修改达成的做完，再判断剩下的是否真的需要新名称。**

设计稿 `1c` 的六项需求按此分类：

| 设计稿需求 | 分类 | 依据 |
|---|---|---|
| 边框实色化 | 改取值 | `--color-border-subtle\|default\|strong` 及其 `-rgb` 已存在 |
| 圆角收敛到 4 档 | 改取值 | `core.css:153-159` 已映射 7 个既有圆角令牌，改值即可 |
| chrome 层实色 | 复用 | `--surface-shell-bg` → `--material-glass-chrome-bg` → `--color-bg-elevated` 已成立 |
| 语义 tint 底色 | 待审计 | 既有 `--color-stage-chip-*` 族可能承载 |
| `opencode` 平台色 | 确需新增 | `ccr-ui/src/styles/` 中无任何 `opencode` 命中 |
| 20px / 28px 数据字号 | 待审计 | 既有 `--text-xl`(21px) / `--text-2xl`(26px) 可能够用 |

「待审计」两项在实施第一步产出结论，写入 `research/token-name-delta.md`。

## 1. 作用域盘点

`tokens.css` 中与本任务相关的四个作用域：

| 作用域 | 选择器 | 起始行（改动前快照） |
|---|---|---|
| neutral 亮 | `:root` | 约 11 |
| neutral 暗 | `[data-theme='dark']` | 约 137 |
| clay 亮 | `[data-flavor='clay']` | 约 539 |
| clay 暗 | `[data-theme='dark'][data-flavor='clay']` | 约 569 |

每个可切换令牌的改动都要在这四处各落一次，缺一处会导致某个组合退回旧值。
行号是改动前快照，实施时先重新定位。

flavor 块不得设置 `--color-accent-primary` / `--color-border-accent`——三条轴保持正交（spec 第 17 行）。

## 2. 边框实色化（改取值，不新增名称）

clay 暗色直接取设计稿给定值：

| 令牌 | 新值 |
|---|---|
| `--color-border-subtle` | `#322A25` |
| `--color-border-default` | `#3A302A` |
| `--color-border-strong` | `#4A3D35` |

其余三个作用域没有设计稿给定值，按 **alpha 合成**推导，不自由发挥：

> 把现有的 `rgb(R G B / A%)` 边框与该作用域下最常见的承载底色做 alpha 合成，取合成结果的实色。
> 承载底色统一取该作用域的 `--color-bg-elevated`（卡片面，边框最常出现的位置）。
> 合成公式：`out = round(fg * A + bg * (1 - A))`，逐通道计算。

输入值（改动前）：

| 作用域 | 边框前景 | subtle/default/strong 透明度 | 合成底色 `--color-bg-elevated` |
|---|---|---|---|
| neutral 亮 | `rgb(25 27 32)` | 12% / 19% / 30% | `#F2F3F5` |
| neutral 暗 | `rgb(235 238 245)` | 14% / 22% / 34% | `#1A1B1F` |
| clay 亮 | `rgb(70 53 41)` | 12% / 19% / 30% | `#F5EEE1` |

推导后的实色写入令牌，`--color-border-*-rgb` 同步写成该实色的十进制 RGB 三元组。
计算过程写入 `research/border-derivation.md` 供复核。

**必须同步 `-rgb` 伴随令牌。** 仓库中存在 `rgb(var(--color-border-default-rgb) / x%)` 形式的调用点；若 `-rgb` 仍是旧的高对比前景值，这些调用点会得到与实色边框不一致的结果。改完后用
`rg -n 'color-border-[a-z]+-rgb' ccr-ui/src` 逐个调用点确认取色合理。

**已知连锁点**：`--surface-card-border: var(--color-border-subtle)` 被 `apple-glass-surface-contract.smoke.test.ts` 断言。该断言是对**引用关系**的断言，不是对取值的断言，改取值不会打断它。实施时确认这一点仍成立。

## 3. 圆角四档（改取值，不新增名称）

不引入 `--radius-chip|control|card|pill`。设计稿要的四档由既有 7 个令牌的取值承载：

```css
--radius-none: 0;        /* 不变 */
--radius-sm: 6px;        /* 原 4px  → chip 档 */
--radius-md: 6px;        /* 原 6px  → chip 档，不变 */
--radius-lg: 8px;        /* 原 8px  → control 档，不变 */
--radius-xl: 12px;       /* 原 10px → card 档 */
--radius-2xl: 12px;      /* 原 12px → card 档，不变 */
--radius-3xl: 12px;      /* 原 16px → card 档 */
--radius-full: 9999px;   /* 不变 */
```

这些是常量令牌，定义在 `:root`，与主题/flavor 无关，不需要四作用域各写一次；
`core.css` 侧它们已在 `@theme inline` 中，本次不动 `core.css`。

已知副作用：原 `--radius-sm` 的 4px 调用点变 6px，原 `--radius-3xl` 的 16px 调用点变 12px。
这是设计稿要求的收敛，回归走查按此判定，不判为缺陷。

后续子任务写新样式时，直接用 `--radius-md`（chip）、`--radius-lg`（control）、`--radius-2xl`（card）、`--radius-full`（pill）这四个作为规范入口。
这条约定写在本文件，不靠新令牌名承载。

## 4. chrome 层：零改动，复用既有链路

设计稿要求侧栏/顶栏是与卡片可区分的实色层。仓库现状已满足：

```
--surface-shell-bg  →  --material-glass-chrome-bg  →  --color-bg-elevated   （blur: none）
--surface-card-bg   →  --color-bg-surface
```

clay 暗色下：base `#17120f` / elevated `#221b18`（shell）/ surface `#2a221e`（card）/ overlay `#342b26`。
四层阶梯成立，chrome 与 card 取值不同。

因此：

- **不新增 `--color-bg-chrome`。**
- **不改 `--material-glass-chrome-bg` 的定义。**
- **不改其在 `prefers-reduced-transparency: reduce` 块中的回退目标。**

`apple-glass-surface-contract.smoke.test.ts` 对这两处的断言保持不变。

组件侧是否真的消费了 `--surface-shell-*`，由 `08-25-home-runtime-layout` 在其第一步核查（该子任务 design.md §3）。若组件绕过语义别名直接用了别的取值，那是组件侧问题，不是令牌层问题。

## 5. 语义 tint（待审计）

设计稿给定的 clay 暗色 tint：

```
accent  #33231B
success #25332A
warning #3A2A20
danger  #2B1F1C
info    #252D33
```

审计问题：既有 `--color-stage-chip-neutral-{bg,border,text}` 是单一中性档，无语义分色。
若现有组件的状态 chip 都走这一族，则设计稿的五色 tint 确需新增名称。

审计输出（写入 `research/token-name-delta.md`）：

1. `rg -n 'stage-chip|chip-neutral' ccr-ui/src` 列出全部消费点。
2. 判断这些消费点是否本来就需要按语义分色。
3. 若确需新增，新增名称为 `--color-{accent,success,warning,danger,info}-tint`，共 5 个，
   均为可切换值 → 归 `core.css` 的 `@theme inline`，四作用域各定义一次。
4. 亮色作用域的 tint 取同色相浅底，须与该作用域的前景语义色配对后满足既有对比度契约。
5. **自定义强调色影响**：`--color-accent-tint` 与 accent 同族，但不在
   `CUSTOM_ACCENT_VARIABLE_FAMILY` 的 8 个变量内。用户设自定义强调色时，
   accent 主色变了而 tint 不变，会出现色相不一致。
   两个处置选项，实施时二选一并记录理由：
   - **选项 A（推荐）**：`--color-accent-tint` 不新增，accent 底色统一用
     `rgb(var(--color-accent-primary-rgb) / 12%)` 表达。该形式随自定义强调色自动跟随，
     且不新增名称。代价是 accent tint 不是实色。
   - **选项 B**：新增 `--color-accent-tint` 并同步扩展 `CUSTOM_ACCENT_VARIABLE_FAMILY`
     与 `applyCustomAccent()` 的生成逻辑。改动面更大，需连带更新
     `theme-bootstrap.smoke.test.ts` 与 `hardcode-px-rgba.smoke.test.ts` 的
     `THEME_WRITER` 豁免条目。

   其余四个（success/warning/danger/info）与 accent 轴无关，不受此约束。

## 6. 平台色（确需新增）

```css
--color-platform-opencode: #735f52;
--color-platform-opencode-rgb: 115 95 82;
```

治理结论：

- 分类：可切换值？——既有 `--color-platform-*` 定义在 `:root` 单作用域（`tokens.css:104-111`），
  四个 flavor×theme 组合共用。新增项沿用同一处置，单作用域定义一次。
- `core.css` 归属：与既有 `--color-platform-*` 同层，实施时确认既有项在哪层并保持一致。
- 自定义强调色：不受影响。
- 桥接：`theme.css` 桥接了 `--platform-claude|codex|gemini` 的非 `-rgb` 短名，未桥接 `-rgb`。
  是否为 opencode 补短名桥接，取决于是否有短名消费点；无消费点则不补。

`--color-platform-gemini` 保持原名，不重命名。

## 7. 排版档位（待审计）

现有 `--text-sm` = 0.8125rem(13px)、`--text-lg` = 1.0625rem(17px) 与设计稿一致，不动。

| 设计稿用途 | 设计稿值 | 既有最近档 | 差 |
|---|---|---|---|
| 正文 | 14px | `--text-sm` 13px | 1px |
| 标签 | 10px | `--text-xs` 12px | 2px |
| 数据次级 | 20px | `--text-xl` 21px | 1px |
| 数据 hero | 28px | `--text-2xl` 26px | 2px |

审计判据：这四项都在 1–2px 内。**默认结论是复用既有档位，不新增名称。**
只有当实施者在真实浏览器中确认某一档的近似造成明显视觉问题时，才走新增流程，
且必须在 `research/token-name-delta.md` 中记录具体是哪一档、差在哪里。

注意：`theme-token-contracts.md:10` 已登记一条字号例外——Profiles 共享层的密集元信息可用 `0.75rem`，
这是唯一的次 Label 档位。新增更小的字号档位会与该例外冲突。

「哪个字号该配 mono」这件事不靠令牌名承载，写在本文件：数据类数字用 `--font-mono`，
标签用 `--font-mono` + `letter-spacing: 0.16em`，正文与标题一律 `--font-sans` / `--font-brand`。
后续子任务的检查项引用本节，不引用不存在的角色令牌名。

## 8. 测试迁移与新增

| 测试文件 | 本任务动作 |
|---|---|
| `tests/apple-glass-surface-contract.smoke.test.ts` | 不改。chrome 相关断言必须继续通过（AC4） |
| `tests/theme-contrast-contract.smoke.test.ts` | 不改阈值。边框实色化后重跑，若某组合失败则调令牌取值，不调阈值 |
| `tests/theme-switch.smoke.test.tsx` | 若锚点值涉及被改取值的令牌，同步锚点值，保持断言结构 |
| `tests/token-single-point.smoke.test.tsx` | 不改。单点生效语义不受本次影响 |
| `tests/theme-domain-extension.smoke.test.tsx` | 不改 |
| 新增断言 | 边框实色化、圆角四档收敛、每个新增名称四作用域可解析。落在既有 `apple-glass-surface-contract.smoke.test.ts` 还是新文件，实施时按该文件的组织方式决定并在 change list 中写明 |

## 9. 兼容性

`theme.css` 是旧变量名到新令牌的桥接层，本任务不改其结构，只在新增令牌需要旧名映射时补行。
改完后确认 `theme.css` 中每个 `var(--color-*)` / `var(--space-*)` / `var(--text-*)` 的目标仍有定义。

## 10. 回滚

`git checkout -- ccr-ui/src/styles/tokens.css ccr-ui/src/styles/theme.css ccr-ui/tests/` 即可完全回退，无组件侧依赖。
spec 文件与 `research/` 的改动可单独保留（它们是记录，不影响运行）。
</content>
