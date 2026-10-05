# 路由未决项验证（批次 0）

> 验证日期：2026-08-24。依据：React Router 8.3.0 源码与文档、本仓 `vite.config.ts`、`index.html`、`src-tauri/src/desktop_shell.rs`。

## 1. `commands/:client?` 可选参数

结论：**语义等价，保留一条路由。**

React Router 8 的 `compilePath` 把 `/:param?` 编译为 `(?:/([^\\/]*))?`，并经 `explodeOptionalSegments` 展开为「有参数 / 无参数」两套匹配。因此：

| URL | 匹配同一路由 | `params.client` |
| --- | --- | --- |
| `/commands` | 是 | `undefined` |
| `/commands/claude` | 是 | `'claude'` |

不必拆成 `/commands` + `/commands/:client`。路径集合不变。

验证落点：`tests/router.smoke.test.ts` 用 `matchRoutes(appRoutes, url)` 断言。

## 2. `<ScrollRestoration />` 与内部滚动容器

结论：**对内部滚动容器不生效，改用 pathname → scrollTop 的 ref map。**

- React Router 的 `ScrollRestoration` 只恢复 **window / document** 滚动，并写入 `sessionStorage`。
- 本仓内容区是 `MainLayout` 内 `.content-scroll-area`（`overflow-y-auto`），window 本身不滚动。
- Vue 侧 `scrollBehavior` 对非缓存路由始终 `{ top: 0 }`；5 条 `meta.cache` 路由靠 keep-alive 保留 DOM 从而保留内部滚动。

落地：

- 不使用 `<ScrollRestoration />` 作为内容区方案。
- `src/shell/innerScroll.ts` 按 `location.pathname` 保存 / 恢复 `.content-scroll-area` 的 `scrollTop`。
- `handle.cache === true` 的路由恢复上次位置；其余路由滚到顶部（对齐 Vue 非缓存行为）。
- 离开时一律写入 map，供缓存路由返回时使用。

## 3. tray 窗口 HTML 入口

结论：**不需要独立 HTML 入口，不改 `vite.config.ts` 多入口。**

证据：

- `ensure_tray_panel_window` 使用 `WebviewUrl::App("index.html".into())`，与主窗口共用同一 SPA。
- `vite.config.ts` 只有 `index.html` → `src/main.tsx` 单入口。
- Tailwind v4 源文件检测已覆盖 `src/**`，无需为 tray 扩范围。

接线：托盘窗口 label 为 `codex-tray-panel`。外壳在挂载后若当前窗口 label 匹配且路径不是 `/tray/codex`，则 `replace` 导航到该路径。`shell:navigate` 仍只作用于主窗口。
