# 全局视觉类型 — 技术设计

## Boundaries

```
src/ui/button.tsx          Button + buttonClass
src/ui/badge.tsx           Badge
src/ui/field-label.tsx     FieldLabel
src/ui/url-text.tsx        UrlText
src/ui/primitives.css      .ui-btn / .ui-badge / .ui-field-label / .ui-url-text
src/ui/index.ts            导出
```

`ui-primitive` 只依赖 `types` / `utils` / `shared`。`UrlText` 可导入 `@/utils/text` 的 `formatBaseUrlDisplay`。禁止导入 `features/`、`api/`、store。

Profiles 与其它 features 只消费这些导出。`components/profiles/profiles-shared.css` 删除与 `.ui-btn` 重复的 `.cp-btn` 规则；QuickRail / Toolbar 的 `.cp-chip--switch` 保留，不并进 Badge。

## Contracts

### Button

```ts
type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'quiet' | 'warning' | 'danger' | 'accent-soft'
type ButtonSize = 'sm' | 'md'

buttonClass(opts: { variant?: ButtonVariant; size?: ButtonSize; className?: string }): string

function Button(props: ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: ButtonVariant
  size?: ButtonSize
}): JSX.Element
```

默认 `variant='secondary'`、`size='md'`、`type='button'`（避免表单里误 submit）。`<Link>` 使用 `buttonClass({ variant: 'primary' })`，不引入 Radix Slot。

### Badge

```ts
type BadgeMode = 'static' | 'interactive'
type BadgeTone = 'neutral' | 'accent' | 'warning' | 'success'

function Badge(props: HTMLAttributes<HTMLElement> & {
  mode?: BadgeMode
  tone?: BadgeTone
  as?: 'span' | 'button'
}): JSX.Element
```

`mode='static'` 时渲染 `span`（忽略 `as`）。`interactive` 默认 `button`。static 不得设置 `cursor: pointer`。

### FieldLabel / UrlText

`FieldLabel`：渲染 `span` 或 `dt`（`as` prop）。`UrlText`：`span`，`value` 必填；内部调用 `formatBaseUrlDisplay(value)`，`title={value}`。

### Profile fieldSlots

```ts
interface ProfileFieldSlot {
  labelKey: string
  columnWidth: string
  kind?: 'text' | 'url' | 'chip'  // 默认 text；删除 chip?: boolean
}
```

Claude：`url, text, chip, chip`。Codex / Grok / antigravity：`url, text, chip, text`。卡片按 `kind` 选择 UrlText / Badge / 纯文本。表格 `slots[0]` 用 UrlText，`slots[2]` 在 `kind==='chip'` 时用 Badge。不渲染 `slots[3]`。

## Data flow

无后端与存储变化。`formatBaseUrlDisplay` 只影响展示，搜索仍用原始 `base_url`（已在 `searchText`）。

## Compatibility

- `cp-btn` / `pe-btn` 在 profiles 子任务**删除**选择器与 className，禁止改成 `.ui-btn` 的 alias。`ProfilesHeader.tsx` 即使无生产消费方也必须迁到 `Button`，不得跳过。rollout 结束前 Profiles 不得残留独立按钮色板。
- `features/*/ui-classes.ts` 的按钮导出在 rollout 删除；`fieldInputClass` / `panelCardClass` / tone 映射保留。
- 不加 `@theme` 新名。unique-name union 保持 452。

## Trade-offs

- **组件 + `buttonClass()`，不要 asChild。** 仓库没有 `@radix-ui/react-slot`。Link 用 class 函数，避免新依赖。
- **不搬 QuickRail chip。** 它带 pin、编号、busy，和枚举 Badge 意图不同。Extract skill：意图不同就不要抽。
- **ConfirmModal / EmptyState 在 primitives 子任务就改。** 它们已在 `src/ui/`，是原语自洽，不是业务 rollout。

## Rollout / rollback

每个子任务一个提交。primitives 单独提交，便于只回滚原语。rollout 按域分批改文件，但同一次提交（或按平台拆 commit 若 diff 过大）。不改默认 flavor / accent。
