# 首屏模块集合对比（AC3）

> 测量：2026-08-24，`cd ccr-ui && bun run build`。基线：`.trellis/tasks/08-22-react-migration/baseline/bundle-budget.txt`（Vue 迁移前）。

## 产物分块（本次）

| chunk | raw |
| --- | --- |
| `index-*.js`（入口） | 425.24 kB |
| `react-vendor` | 271.59 kB |
| `motion-vendor` | 128.67 kB |
| `query-vendor` | 33.43 kB |
| `ui-vendor`（iconify） | 17.85 kB |
| `tauri-vendor` | 15.32 kB |
| `zh-CN` / `en-US` locale | 160 / 167 kB，**懒加载** |
| `index-*.css`（shell-critical + shell.css） | 222.83 kB |
| `deferred-interactive.css`（含 ui primitives） | 15.40 kB，首帧后 |
| `deferred-decorations.css` | 1.61 kB，空闲 |

基线入口 JS `index` gzip 前 243.69 KiB。本次入口变大的主因是 React 运行时与外壳（Titlebar / MainLayout / Radix Dialog）进入同步图；**locale 大包与 motion 已拆出独立 chunk**，不进首屏同步图以外的 locale 路径。

## 模块集合判定

未扩大的项：

- 完整 locale（`i18n/locales/*`）不进入口，由 `hydrateShellLocale` / 路由 loader 按需 import。
- 业务视图仍是路由级 `lazy` 占位，不把 22 个原异步视图打进入口。
- ApexCharts / 虚拟列表 / 编辑器不进入口。
- `primitives.css` 挂在 deferred-interactive，不进 shell-critical。

入口新增（外壳门必需）：

- `src/shell/*` 布局与接线
- Radix Dialog（全局确认 / Titlebar About / UpdateModal）
- `@iconify/react`（侧栏图标）

`motion` 进 `motion-vendor`，不与业务视图耦合。

结论：首屏**业务模块集合**未扩大；入口体积增加来自 React 外壳替换 Vue 运行时，属阶段 3 预期，发布门再对照 gzip 预算。
