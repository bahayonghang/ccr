# Tailwind 生成 CSS 体积记录（段 2，AC3 数据）

> 日期：2026-08-23。测量对象：`bun run build` 产物中的主 CSS 文件（Tailwind 生成 + 项目自有样式合并单文件）。

## 前后对比

| 时点 | 产物 | raw | gzip | 来源 |
| --- | --- | --- | --- | --- |
| 阶段 0 基线（v3.4.19） | `core.css: index-DZpKQsqh.css` | 123.13 KiB (126,085 B) | 19.35 KiB (19,814 B) | `baseline/bundle-budget.txt` |
| 段 2 前（React 入口已活，仍 v3.4.19） | `index-*.css` | ≈121.44 kB | — | 主会话提供的参照值 |
| 段 2 后（Tailwind 4.3.3） | `dist/assets/index-Dz-Pw-Iy.css` | 202,436 B（197.7 KiB） | 29,310 B（28.6 KiB） | 本次实测 |

## 构成分析（为何 +81 kB）

`bun pm ls` 确认 tailwindcss@4.3.3 / @tailwindcss/postcss@4.3.3。对产物逐层测量：

- `@layer utilities` 占 148.9 kB，为增量主体。来源是引擎差异而非扫描面变化：
  - 透明度修饰符由 v3 的内联 `rgb(var() / alpha)` 改为 v4 的 `color-mix(...)` 包裹，533 处，每处约 +45 B；
  - v4 工具类普遍携带 `--tw-*` 自定义属性守卫（如 `.font-bold{--tw-font-weight:500;font-weight:500}`），单条规则字节数上升；
  - dark 变体规则以 `:where([data-theme=dark],[data-theme=dark] *)` 全量展开（77 条）。
- `@layer theme` 仅 7.5 kB：v4 按「用到的 token 才产出变量」，fontWeight 压缩语义生效后旧档位值不产出。
- 扫描面与 v3 content 等价并更完整：v4 自动检测覆盖 index.html 与全部源码（含 v3 content 列表未列出的新目录 `features/`、`shell/`、`ui/`）；`node_modules`、`ref/`、`storybook-static`、`dist`、`.omc`、`src-tauri/target` 均在 gitignore 内不参与检测。

## 后续归属

AC10 的 bundle 预算重设归 `08-22-arch-quality-perf`（R9.1）；token 层精调与 `@theme inline` 正式映射归 `08-22-design-system`。本表仅提供 AC3 要求的测量数据。
