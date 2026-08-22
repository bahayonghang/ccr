# 启动耗时与首屏渲染基线（迁移前，Vue 版）

采集时间：2026-08-22（feature/react-migration 分支，应用代码与 dev 一致）

## 测量方法

- 路由级冷启动：`bun ./scripts/measure-vite-route.mjs`（脚本自起 Vite :5173，记录 serverReadyAndWarmMs 与各资源 fetch 耗时），原始输出见 `route-timing-settings.json`。
- 首屏渲染：桌面运行时（tauri dev，WebView2）通过 CDP 在 `/`、`/settings`、`/usage`、`/configs` 四条路由读取 Navigation/Paint Timing。
- bundle 体积：`bun ./scripts/check-bundle-budget.mjs`，输出见 `bundle-budget.txt`。

## 数值

| 指标 | 路由 | 数值 |
| --- | --- | --- |
| server ready + warm | - | 12101 ms（含 Vite 冷启动与依赖预热，详见 route-timing-settings.json） |
| DOMContentLoaded | / | 57 ms |
| DOMContentLoaded | /settings | 53 ms |
| DOMContentLoaded | /usage | 52 ms |
| DOMContentLoaded | /configs | 52 ms |
| First Contentful Paint | / | 28 ms |
| First Contentful Paint | /settings | 32 ms |
| First Contentful Paint | /usage | 28 ms |
| First Contentful Paint | /configs | 36 ms |

LCP 未在本次采集中获得（页面以骨架屏为主，未触发 LCP 条目），以 FCP 与 DCL 作为首屏渲染对照口径；迁移后对比采用同一方法。

## 备注

- DCL/FCP 为 dev server 热缓存下的数值，用于同口径前后对比，不代表生产构建绝对值。
- 迁移后复测须使用相同命令与相同路由集合。
