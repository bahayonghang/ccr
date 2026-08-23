# `src/ui/` — 原语层约定（batch 3, 08-22-design-system）

> 原语层落位目标（父任务 design.md §2）。9 类 shadcn/ui 原语（Radix 底座）位于本目录。

## 边界（eslint `boundaries/elements`）

- 元素类型：`ui-primitive`，pattern `['src/ui', 'src/components/ui']`。
- 只允许依赖 `types` / `utils` / `shared`；**不得导入 features、api、store**。
- 本目录代码不包含业务逻辑；交互行为全部由 Radix 底座承担。

## 样式约定

1. **只消费 CCR token 命名空间 + Tailwind 工具类**。禁止硬编码 `px` / `hex` / `rgba` 字面量
   （AC6 精神，lint 会执行）。示例映射：
   - `bg-*` / `text-*` / `border-*` → `--color-*` 语义色
   - `bg-scrim` → `--color-scrim` 遮罩压暗层（Dialog overlay）
   - `surface-modal`（`src/styles/components/surfaces.css` 组件类）→ 悬浮玻璃材质
   - `shadow-{xs,sm,md,lg,xl,2xl}` → `--shadow-*`（@theme inline 已映射）
   - `z-{dropdown,popover,modal,modal-backdrop,tooltip,...}` → `--layer-*` 层级
   - `rounded-*` → `--radius-*`（@theme inline 已补齐 2xl/3xl/full）
2. 进入动画用 `animate-fade-in` / `animate-scale-in`（keyframes 在 `core.css` @theme 内）。
   退出动画由 Radix 的 `data-[state=closed]` 类承接；reduced-motion 降级沿用全局约定。
3. 类名合并统一走 `cn.ts`（clsx + tailwind-merge）。

## 文件清单

| 文件 | 底座 | 语义备注 |
| --- | --- | --- |
| `dialog.tsx` | @radix-ui/react-dialog | batch 4 弹层收口的唯一底座 |
| `popover.tsx` | @radix-ui/react-popover | |
| `dropdown-menu.tsx` | @radix-ui/react-dropdown-menu | 替代手写 role="menu"（adhoc D1–D6） |
| `tooltip.tsx` | @radix-ui/react-tooltip | |
| `tabs.tsx` | @radix-ui/react-tabs | 替代手写 tablist（adhoc T1–T11） |
| `combobox.tsx` | cmdk | 替代手写 listbox（adhoc C1/C2） |
| `select.tsx` | @radix-ui/react-select | 替代原生 `<select>` |
| `switch.tsx` | @radix-ui/react-switch | 对齐 AppSettingsView 手写 role="switch" |
| `checkbox.tsx` | @radix-ui/react-checkbox | 替代原生 type="checkbox" |
| `cn.ts` | clsx + tailwind-merge | 类名合并 |
