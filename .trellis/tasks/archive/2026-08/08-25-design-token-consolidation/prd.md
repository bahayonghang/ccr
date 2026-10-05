# 设计令牌层收敛与名称治理：实色边框、四档圆角、mono 语义分层

父任务：`.trellis/tasks/08-25-react-home-style-redesign`（决策 D2：令牌改动落在全局层；决策 D7：本子任务同时承担名称治理）
设计输入：父任务 `research/claude-design-source.html` 的 `1c` 令牌提案
审阅裁定：父任务 `research/plan-review-adjudication.md` 的 TPR-02

## Goal

把设计稿 `1c` 的令牌提案落到 `ccr-ui/src/styles/tokens.css`，让全站在明暗两主题 × neutral/clay 两 flavor 下共用一套收敛后的边框、圆角、语义色与排版令牌。
优先用取值修改与复用既有令牌达成目标；只有确无既有承载者时才新增名称，并按治理流程登记。
本子任务只改令牌层、必要的兼容桥接与受影响的既有测试断言，不改任何页面的 DOM 结构。

## 前置

无。本子任务是其余四个 UI 子任务的前置。

## Background and Confirmed Facts

- `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md:26` 冻结了 448 个令牌名称：`src/styles/**` 的变量名并集必须等于迁移前集合，新增名称需要专门的 token 治理任务。父任务 D7 指定本子任务为该治理任务。
- 令牌分两层：第 1 层是 `tokens.css` 中 `:root` / `[data-theme='dark']` / `[data-flavor='clay']` / `[data-accent='clay']` 下的普通自定义属性；第 2 层是 `core.css` 的 `@theme inline`（可切换值）与 `@theme`（常量值）命名空间映射。可切换值不得写进 `@theme`，常量值不得写进 `@theme inline`。
- `core.css:153-159` 已把 `--radius-sm|md|lg|xl|2xl|3xl|full` 映射进 `@theme inline`。因此把这 7 个既有令牌的**取值**收敛到 4 个不同值，即可达成设计稿的四档圆角，不需要 `--radius-chip|control|card|pill` 这组新名称。
- chrome 层已存在：`--surface-shell-bg` → `--material-glass-chrome-bg` → `--color-bg-elevated`，且 chrome 档 `blur: none`。四层表面阶梯 base / elevated（shell）/ surface（card）/ overlay 已成立，不需要 `--color-bg-chrome`。
- `ccr-ui/tests/apple-glass-surface-contract.smoke.test.ts` 断言 `--material-glass-chrome-bg: var(--color-bg-elevated)`，并在 `prefers-reduced-transparency` 块中重复断言同一取值。
- `ccr-ui/src/utils/themeBootstrap.ts:343` 的 `CUSTOM_ACCENT_VARIABLE_FAMILY` 恰为 8 个变量：`--color-accent-primary`、`-hover`、`-active`、`-rgb`、`-glow`、`-contrast`、`-contrast-rgb`、`--color-border-accent`。`applyCustomAccent()` 只重写这 8 个。
- `--color-platform-claude|codex|grok|gemini` 及其 `-rgb` 存在；`--color-platform-opencode` 不存在，`ccr-ui/src/styles/` 中无任何 `opencode` 命中。
- `--color-stage-chip-neutral-{bg,border,text}` 已存在，指向 `--color-bg-overlay` / `--color-border-default` / `--color-text-secondary`；语义 tint 是否需要新名称，取决于既有 stage/chip 族能否承载。
- `--text-xs..6xl` 为 10 档完整比例，`--text-sm: 0.8125rem`（13px）与 `--text-lg: 1.0625rem`（17px）已与设计稿一致；设计稿要的 20px 与 28px 在现有档位中无精确对应（`--text-xl` 为 21px，`--text-2xl` 为 26px）。
- 对比度阈值由 `ccr-ui/tests/theme-contrast-contract.smoke.test.ts` 锁定：相对 `bg-surface`，text-primary ≥ 12:1、secondary ≥ 7:1、muted ≥ 4.5:1、accent 对 accent-contrast ≥ 3.5:1。阈值是契约，不得为让新配色通过而下调。
- 暗色下表面必须单调变亮（`bg-base < bg-elevated < bg-surface < bg-overlay`），全部表面与文本令牌必须 100% 不透明。

## Requirements

- R1：边框令牌从 alpha 组合改为实色。`--color-border-subtle` / `--color-border-default` / `--color-border-strong` 在四个 flavor×theme 作用域中各给实色值，配套的 `--color-border-*-rgb` 同步为该实色的 RGB 分量。这是取值修改，不新增名称。
- R2：圆角收敛为 4 档（chip 6px / control 8px / card 12px / pill 9999px）。实现方式是修改 `--radius-sm|md|lg|xl|2xl|3xl` 的取值使其只落在这 4 个值上，保留 `--radius-none` 与 `--radius-full`。不新增角色令牌名称，不做调用点批量重命名。
- R3：chrome 层复用既有 `--surface-shell-*` → `--material-glass-chrome-bg` → `--color-bg-elevated` 链路。不新增 `--color-bg-chrome`，不改动 `--material-glass-chrome-bg` 的定义与其 `prefers-reduced-transparency` 回退目标。
- R4：先产出**名称增量审计**，把设计稿 `1c` 的每一项需求分类为「改既有令牌取值」「复用既有令牌」「确需新增名称」三类之一，逐项给出结论与依据。审计结论决定后续步骤范围。
- R5：对分类为「确需新增名称」的每一项，完成治理动作：登记到本任务 `research/token-name-delta.md`；判定属可切换值还是常量值并给出 `core.css` 的对应层；在四个作用域中各定义一次或说明为何单作用域足够；判定自定义强调色是否需要覆盖该名称。
- R6：语义色 tint 底色。先判断既有 `--color-stage-chip-*` 族能否承载；不能承载时按 R5 走新增流程。
- R7：平台色补齐 `--color-platform-opencode` 及其 `-rgb`。这是确需新增的名称，按 R5 走治理流程。不重命名 `--color-platform-gemini`。
- R8：排版档位。设计稿的 20px 数据字号与 28px hero 字号在现有 `--text-*` 中无精确对应。先判断能否用既有档位近似（21px / 26px）而不新增名称；若视觉上不可接受，按 R5 走新增流程。17px 页标题与 13px 次要文本沿用既有 `--text-lg` / `--text-sm`，取值不得改动。
- R9：`ccr-ui/src/styles/theme.css` 的兼容桥接不得断链——所有旧变量名在改动后仍解析到有效值。
- R10：受本次改动影响的既有测试断言必须迁移而非删除，并保持原有的保护意图。对比度阈值不得下调。
- R11：新增或修改的令牌必须有对应的测试断言。至少覆盖：边框实色化、圆角四档收敛、每个新增名称在四作用域可解析。
- R12：本子任务不改任何 `.tsx` 文件，也不改 feature 局部 CSS 的选择器结构。

## Acceptance Criteria

- [x] AC1（R4）：`research/token-name-delta.md` 存在，逐项列出设计稿 `1c` 的每条需求与其分类结论；分类为「确需新增」的条目数等于最终新增的名称数。
- [x] AC2（R1）：`tokens.css` 中 `--color-border-subtle|default|strong` 的全部定义为实色十六进制值，无 `rgb(... / ...%)` 形式；对应 `-rgb` 令牌的三元组与该实色一致。四个作用域各有一套。
- [x] AC3（R2）：`--radius-sm|md|lg|xl|2xl|3xl` 解析后的取值集合恰为 `{6px, 8px, 12px}`；`--radius-none` 为 `0`，`--radius-full` 为 `9999px`。`tokens.css` 中不再出现 `4px`、`10px`、`16px` 作为圆角取值。
- [x] AC4（R3）：`apple-glass-surface-contract.smoke.test.ts` 中关于 `--material-glass-chrome-bg` 的两处断言未被修改且仍通过。`tokens.css` 中不存在 `--color-bg-chrome`。
- [x] AC5（R5,R6,R7,R8）：每个新增名称在 `research/token-name-delta.md` 中有五项结论：分类、`core.css` 归属层、四作用域定义结论、自定义强调色影响结论、对应测试断言位置。
- [x] AC6（R5）：`.trellis/spec/ccr-ui/frontend/theme-token-contracts.md` 的名称冻结段落已更新为「448 + 本次登记增量」，并列出增量清单。
- [x] AC7（R9）：`theme.css` 中每个 `var()` 目标令牌仍有定义；`just frontend-check-quick` 通过。
- [x] AC8（R10,R11）：`bunx vitest run --config vitest.smoke.config.ts tests/apple-glass-surface-contract.smoke.test.ts tests/theme-contrast-contract.smoke.test.ts tests/theme-switch.smoke.test.tsx tests/token-single-point.smoke.test.tsx tests/theme-domain-extension.smoke.test.tsx` 全部通过；`theme-contrast-contract.smoke.test.ts` 的阈值常量未被修改。
- [x] AC9（R11）：新增的令牌断言已落在测试文件中，且该文件出现在本任务的 change list 与提交里。
- [x] AC10（R12）：本任务提交的改动文件集合仅含 `ccr-ui/src/styles/tokens.css`、按需的 `ccr-ui/src/styles/theme.css`、测试文件、spec 文件与本任务 `research/`。无 `.tsx` 改动。
- [x] AC11：全页视觉回归走查完成，确认边框在深底上可见、圆角同屏一致；结果记录到 `research/token-regression.md`。原 4px chip 变 6px、原 16px 容器变 12px 属预期收敛，不判为缺陷。

## Out of Scope

- 重命名 `--color-platform-gemini`。
- 批量替换调用点的 `--radius-sm` 等旧令牌名。
- 删除玻璃材质令牌（`--glass-*`、`--material-glass-*`）。
- 修改 `--material-glass-chrome-bg` 的定义或其 reduced-transparency 回退目标。
- 修改 `theme-contrast-contract.smoke.test.ts` 的阈值常量。
- 任何页面布局或组件结构改动。

## Open Questions

以下三项由 R4 的名称增量审计在实施第一步内解决，不阻断启动：

- 语义 tint 能否由既有 `--color-stage-chip-*` 族承载。
- 20px / 28px 数据字号能否用既有 `--text-xl` / `--text-2xl` 近似。
- 新增名称是否需要进入 `CUSTOM_ACCENT_VARIABLE_FAMILY`。
</content>
