# 名称增量审计（阶段 A）

执行时间：2026-08-25。治理任务：`08-25-design-token-consolidation`。

## A1 基线名称集

命令等价实现：对 `ccr-ui/src/styles/**/*.css` 抽取 `--[a-z0-9-]+(?=:)` 唯一名。

| 口径 | 数量 | 说明 |
|---|---|---|
| `src/styles/**` 唯一名 | **426** | 写入 `token-names-before.txt` |
| `tokens.css` 定义点 | **448** | 与冻结叙事中的 448 对齐；含四作用域重复定义 |
| `tokens.css` 唯一名 | 278 | 与 `08-22-design-system` 归档记录一致 |

差异原因：448 是定义点计数，不是跨 `src/styles/**` 的唯一名并集。后续增量以唯一名为准。

## 设计稿 1c 逐项分类

| 设计稿需求 | 分类 | 依据 |
|---|---|---|
| 边框实色化 | 改既有令牌取值 | `--color-border-subtle\|default\|strong` 及 `-rgb` 已存在 |
| 圆角收敛到 4 档 | 改既有令牌取值 | `--radius-sm\|md\|lg\|xl\|2xl\|3xl\|full` 已映射；改 `--radius-sm` / `--radius-xl` / `--radius-3xl` |
| chrome 层实色 | 复用既有令牌 | `--surface-shell-bg` → `--material-glass-chrome-bg` → `--color-bg-elevated`；不新增 `--color-bg-chrome` |
| 语义 tint 底色（success/warning/danger/info） | 确需新增 | 见 A2 |
| accent tint 底色 | 复用既有令牌 | 选项 A，见 A4 |
| `--color-platform-opencode` 及 `-rgb` | 确需新增 | `ccr-ui/src/styles/` 无 `opencode` 令牌；`src/ui/agent-icons.tsx` 已引用该 var |
| 20px / 28px 数据字号 | 复用既有令牌 | 见 A3 |

## A2 tint 审计

`rg -n 'stage-chip|chip-neutral' ccr-ui/src` 消费点：

| 文件 | 用途 |
|---|---|
| `tokens.css` / `theme.css` | 定义与短名桥接 |
| `features/claude/home/homeModel.ts`、`HomeGrids.tsx` | 功能标签，中性 chip |
| `features/gemini/home/geminiHomeModel.ts`、`GeminiHomeGrids.tsx` | `neutral` 档标签 |
| `features/codex/home/CodexHomeCards.tsx` | 中性徽章 |
| `features/codex` `codex-auth-shared.css` | 中性 chip 底 |

结论：既有 `--color-stage-chip-neutral-*` 是单一中性档，**不能**承载设计稿五色语义底。现有消费点本身不需要五色分色，但父任务 1b / `08-25-home-side-rail` 需要 warning/danger 行底色，且本任务是唯一名称治理窗口。

因此新增 4 个实色 tint（不含 accent）：

- `--color-success-tint`
- `--color-warning-tint`
- `--color-danger-tint`
- `--color-info-tint`

## A3 字号审计

| 设计稿 | 既有档 | 差 | 结论 |
|---|---|---|---|
| 正文 14px | `--text-sm` 13px | 1px | 复用 |
| 标签 10px | `--text-xs` 12px | 2px | 复用；更小档会与 Profiles `0.75rem` 例外冲突 |
| 数据次级 20px | `--text-xl` 21px | 1px | 复用 |
| 数据 hero 28px | `--text-2xl` 26px | 2px | 复用 |

未在浏览器中观察到必须新增档位的明显问题；默认复用不新增 `--text-data-*`。

## A4 accent tint：选项 A

不新增 `--color-accent-tint`。accent 底色用 `rgb(var(--color-accent-primary-rgb) / 12%)`。

理由：该形式随 `applyCustomAccent()` 的 8 变量族自动跟随；选项 B 要改 `themeBootstrap.ts`（`.tsx`/`.ts` 不在本任务产品范围）。代价是 accent tint 不是实色，后续子任务按此消费。

## 确需新增名称的五项结论

共 **6** 个新名称，与最终增量相等。

### `--color-success-tint` / `--color-warning-tint` / `--color-danger-tint` / `--color-info-tint`

| 项 | 结论 |
|---|---|
| 分类 | 可切换语义变量（实色底） |
| `core.css` 归属层 | 应进 `@theme inline`（与 `--color-success` 同层）。本任务 AC10 不改 `core.css`；消费用 `var(--color-*-tint)`。映射可在后续不新增名称的前提下补进 `@theme inline` |
| 四作用域 | `:root`、`[data-theme='dark']`、`[data-flavor='clay']`、`[data-theme='dark'][data-flavor='clay']` 各定义一次。clay 暗用设计稿给定实色；其余三作用域按语义色 12% 叠在该作用域 `--color-bg-surface` 上 |
| 自定义强调色 | 不受影响（非 accent 轴） |
| 测试 | `ccr-ui/tests/token-consolidation.smoke.test.ts` |

### `--color-platform-opencode` / `--color-platform-opencode-rgb`

| 项 | 结论 |
|---|---|
| 分类 | 可切换语义变量；与既有 `--color-platform-*` 一样只在 `:root` 定义一次，四组合共用 |
| `core.css` 归属层 | 既有平台色在 `@theme inline`。本任务不改 `core.css`；`agent-icons.tsx` 已用 `var(--color-platform-opencode)`。短名 `--platform-opencode` 无消费点，不补 `theme.css` 桥 |
| 四作用域 | 单作用域一次（与 gemini/claude 一致） |
| 自定义强调色 | 不受影响 |
| 测试 | `ccr-ui/tests/token-consolidation.smoke.test.ts` |

不新增：`--color-bg-chrome`、`--radius-chip|control|card|pill`、`--color-accent-tint`、`--text-data-*`。
