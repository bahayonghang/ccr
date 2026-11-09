# XC3 全页回归走查

日期：2026-08-25
基线：`a4d3e480`
预览：`http://127.0.0.1:5173/`（`bun run dev:web -- --host 127.0.0.1 --strictPort`，无 Tauri IPC）
方法：Playwright Chromium。每个组合先写 `localStorage` 的 `ccr-theme` / `ccr-flavor`，再导航，让 `themeBootstrap` 应用文档属性。走查结束时写回 `light` + `neutral`。

## 结论

- 横向溢出：32 个页面组合 overflowX=0
- 标题 contrast（相对 body/标题底）：最低 11.58；<4.5 的组合 0
- 标题/导航误用 mono：未发现 h1/h2/h3/nav 使用等宽字体
- 响应式（dark×clay 首页）：1440px overflowX=0 平台卡列=4； 1280px overflowX=0 平台卡列=4； 1024px overflowX=0 平台卡列=2
- reduced-motion：仍有 animationName 的节点 0 个
- Web 预览无 IPC：首页平台卡显示未追踪/空图，不伪造用量数字。
- 文档属性对齐：32 行 data-theme/data-flavor 均与组合一致
- 圆角取样优先 `.dashboard-platform`；非首页页若落到 `main` 可能报 0px，不代表平台卡圆角。

## 走查矩阵

| 页面 | 组合 | data-theme | data-flavor | dock | overflowX | 标题对比 | 取样节点 | 边框 | 圆角 |
|---|---|---|---|---|---:|---:|---|---|---|
| Dashboard | light×neutral | light | neutral | 浅色模式·中性·中文·CCR UI v7.2.0 | 0 | 14.19 | a.dashboard-platform | rgb(216, 217, 219) | 12px |
| Profiles | light×neutral | light | neutral | 浅色模式·中性·中文·CCR UI v7.2.0 | 0 | 14.19 | main.content-main | rgb(25, 27, 32) | 0px |
| MCP | light×neutral | light | neutral | 浅色模式·中性·中文·CCR UI v7.2.0 | 0 | 14.19 | main.content-main | rgb(25, 27, 32) | 0px |
| Commands | light×neutral | light | neutral | 浅色模式·中性·中文·CCR UI v7.2.0 | 0 | 14.19 | main.content-main | rgb(25, 27, 32) | 0px |
| Sync | light×neutral | light | neutral | 浅色模式·中性·中文·CCR UI v7.2.0 | 0 | 14.19 | main.content-main | rgb(25, 27, 32) | 0px |
| Check-ins | light×neutral | light | neutral | 浅色模式·中性·中文·CCR UI v7.2.0 | 0 | 14.19 | main.content-main | rgb(25, 27, 32) | 0px |
| Usage | light×neutral | light | neutral | 浅色模式·中性·中文·CCR UI v7.2.0 | 0 | 14.19 | main.content-main | rgb(25, 27, 32) | 0px |
| Settings | light×neutral | light | neutral | 浅色模式·中性·中文·CCR UI v7.2.0 | 0 | 14.19 | main.content-main | rgb(25, 27, 32) | 0px |
| Dashboard | light×clay | light | clay | 浅色模式·暖陶·中文·CCR UI v7.2.0 | 0 | 11.58 | a.dashboard-platform | rgb(224, 216, 203) | 12px |
| Profiles | light×clay | light | clay | 浅色模式·暖陶·中文·CCR UI v7.2.0 | 0 | 11.58 | main.content-main | rgb(49, 36, 28) | 0px |
| MCP | light×clay | light | clay | 浅色模式·暖陶·中文·CCR UI v7.2.0 | 0 | 11.58 | main.content-main | rgb(49, 36, 28) | 0px |
| Commands | light×clay | light | clay | 浅色模式·暖陶·中文·CCR UI v7.2.0 | 0 | 11.58 | main.content-main | rgb(49, 36, 28) | 0px |
| Sync | light×clay | light | clay | 浅色模式·暖陶·中文·CCR UI v7.2.0 | 0 | 11.58 | main.content-main | rgb(49, 36, 28) | 0px |
| Check-ins | light×clay | light | clay | 浅色模式·暖陶·中文·CCR UI v7.2.0 | 0 | 11.58 | main.content-main | rgb(49, 36, 28) | 0px |
| Usage | light×clay | light | clay | 浅色模式·暖陶·中文·CCR UI v7.2.0 | 0 | 11.58 | main.content-main | rgb(49, 36, 28) | 0px |
| Settings | light×clay | light | clay | 浅色模式·暖陶·中文·CCR UI v7.2.0 | 0 | 11.58 | main.content-main | rgb(49, 36, 28) | 0px |
| Dashboard | dark×neutral | dark | neutral | 深色模式·中性·中文·CCR UI v7.2.0 | 0 | 16.7 | a.dashboard-platform | rgb(55, 57, 61) | 12px |
| Profiles | dark×neutral | dark | neutral | 深色模式·中性·中文·CCR UI v7.2.0 | 0 | 16.7 | main.content-main | rgb(242, 243, 245) | 0px |
| MCP | dark×neutral | dark | neutral | 深色模式·中性·中文·CCR UI v7.2.0 | 0 | 16.7 | main.content-main | rgb(242, 243, 245) | 0px |
| Commands | dark×neutral | dark | neutral | 深色模式·中性·中文·CCR UI v7.2.0 | 0 | 16.7 | main.content-main | rgb(242, 243, 245) | 0px |
| Sync | dark×neutral | dark | neutral | 深色模式·中性·中文·CCR UI v7.2.0 | 0 | 16.7 | main.content-main | rgb(242, 243, 245) | 0px |
| Check-ins | dark×neutral | dark | neutral | 深色模式·中性·中文·CCR UI v7.2.0 | 0 | 16.7 | main.content-main | rgb(242, 243, 245) | 0px |
| Usage | dark×neutral | dark | neutral | 深色模式·中性·中文·CCR UI v7.2.0 | 0 | 16.7 | main.content-main | rgb(242, 243, 245) | 0px |
| Settings | dark×neutral | dark | neutral | 深色模式·中性·中文·CCR UI v7.2.0 | 0 | 16.7 | main.content-main | rgb(242, 243, 245) | 0px |
| Dashboard | dark×clay | dark | clay | 深色模式·暖陶·中文·CCR UI v7.2.0 | 0 | 15.62 | a.dashboard-platform | rgb(50, 42, 37) | 12px |
| Profiles | dark×clay | dark | clay | 深色模式·暖陶·中文·CCR UI v7.2.0 | 0 | 15.62 | main.content-main | rgb(243, 234, 223) | 0px |
| MCP | dark×clay | dark | clay | 深色模式·暖陶·中文·CCR UI v7.2.0 | 0 | 15.62 | main.content-main | rgb(243, 234, 223) | 0px |
| Commands | dark×clay | dark | clay | 深色模式·暖陶·中文·CCR UI v7.2.0 | 0 | 15.62 | main.content-main | rgb(243, 234, 223) | 0px |
| Sync | dark×clay | dark | clay | 深色模式·暖陶·中文·CCR UI v7.2.0 | 0 | 15.62 | main.content-main | rgb(243, 234, 223) | 0px |
| Check-ins | dark×clay | dark | clay | 深色模式·暖陶·中文·CCR UI v7.2.0 | 0 | 15.62 | main.content-main | rgb(243, 234, 223) | 0px |
| Usage | dark×clay | dark | clay | 深色模式·暖陶·中文·CCR UI v7.2.0 | 0 | 15.62 | main.content-main | rgb(243, 234, 223) | 0px |
| Settings | dark×clay | dark | clay | 深色模式·暖陶·中文·CCR UI v7.2.0 | 0 | 15.62 | main.content-main | rgb(243, 234, 223) | 0px |

## 响应式抽查（Dashboard dark×clay）

| 宽度 | overflowX | docOverflowX | 平台卡列数 | 截图 |
|---|---:|---:|---:|---|
| 1440 | 0 | 0 | 4 | shots/dashboard-dark-clay-1440.png |
| 1280 | 0 | 0 | 4 | shots/dashboard-dark-clay-1280.png |
| 1024 | 0 | 0 | 2 | shots/dashboard-dark-clay-1024.png |

## reduced-motion

首页抽样节点未检测到仍在播放的 animationName。

## 视觉发现（未按严重度过滤）

1. 侧栏与顶栏共用 `--surface-shell-bg`，暗色 clay 下与内容底、卡片底靠布局和边框区分，而不是四处不同填充色。
2. Web 预览环境切换器被 Tauri 门控，顶栏只有面包屑。
3. 首页用量图在无 series 时走空态；成本为 `—`。
4. Settings 外观卡把明暗与 flavor 放在同一张卡；仅 neutral/clay。
5. Profiles/MCP/Commands/Sync/Check-ins/Usage 为令牌层连带变样页面，本走查只验边框/圆角/溢出/对比，不重设计这些页。
6. 1024px dark×clay 首页平台卡为 2 列；1440/1280 为 4 列。设置 dock 文案与 `data-theme`/`data-flavor` 一致（例如 dark×clay 为「深色模式·暖陶」）。
7. 非首页取样落到 `main.content-main` 时圆角为 0px；首页 `.dashboard-platform` 为 12px（`--radius-2xl`）。

## 截图

Dashboard 与 Settings 的四组合 1440 截图，以及 dark×clay 三档宽度截图，见 `research/shots/`。
