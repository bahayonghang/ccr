# 回归缺陷清单（AC2）

> 任务：`08-22-regression-release`。视觉表见 `screen-comparison.md`（185 行，视觉判定缺陷 = 0）。

## 汇总

| ID | 面 | 状态 | 说明 |
| --- | --- | --- | --- |
| D1 | 构建 | 已修复 | `app-settings.css` 增加 `@reference`；主线程 `bun run build` 与 `just frontend-check` 均为 EXIT=0 |

视觉比对 185 行无「缺陷」判定。AC2 要求清单项全部修复并重验，本清单仍有未修复项，AC2 不勾选。

## D1 `gap-5` 导致生产构建失败

- 命令：`just frontend-check`（2026-08-24，本会话）
- 失败步骤：`frontend-build` → `cd ccr-ui && bun run build`
- 错误：`src/features/configs/styles/app-settings.css:1` PostCSS / Tailwind v4：`Cannot apply unknown utility class gap-5`（提示缺少 `@reference`）
- 源：`.app-settings-shell` 等规则使用 `@apply ... gap-5`
- 影响：无当前 `dist/` 生产产物，AC3 / AC10 / 打包后 CSP 控制台遍历均被挡住
- 修复：`ccr-ui/src/features/configs/styles/app-settings.css` 文件首行增加 `@reference '../../../styles/core.css';`
- 重验：主线程 `cd ccr-ui && bun run build` EXIT=0（Vite 13.67s）；随后 `just frontend-check` EXIT=0（含 frontend-build 与 docs-check）

## 非缺陷（已记录、不进修复队列）

| 项 | 归类 | 依据 |
| --- | --- | --- |
| Tauri 桌面 native chrome vs Vue 基线自定义 Titlebar | 可接受差异 | `window-chrome.smoke.test.ts`：Tauri 为 `native` |
| shadcn/Radix 焦点环 | 可接受差异 | design.md §1.4 示例 |
| OAuth 向导止于凭据步 | 政策边界 | 基线 README；本任务不要求付费凭据 |
| Web 预览 `invoke` 横幅 | 已知边界 | 基线 README；Vue 静帧同样出现 |
| `tests/artifacts/route-snapshots/` Neko v5.4.7 | 不用 | 2026-03-30，与 v7.2.0 基线无关 |
