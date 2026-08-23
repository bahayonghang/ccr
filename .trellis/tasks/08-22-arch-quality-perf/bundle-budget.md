# Bundle 预算（React 基座）

> 所属任务：`08-22-arch-quality-perf` 批次 8（AC9）。
> 依据：`design.md` §8（bundle 预算）。
> 采集日期：2026-08-23，分支 `react-migration/react-foundation`，未提交。

## 1. Vue 基线（父任务 Phase 0）

来源：`.trellis/tasks/08-22-react-migration/baseline/bundle-budget.txt`（Vue 版本 `dev`，v7.2.0 等价，Tailwind v3.4.19 + vite 7）。

| 项 | 文件 | raw | gzip |
| --- | --- | --- | --- |
| UsageDashboardView | `UsageDashboardView-DW7llTPR.js` | 93.40 KiB | 26.51 KiB |
| index | `index-CW4QTwV-.js` | 243.69 KiB | 45.41 KiB |
| core.css | `index-DZpKQsqh.css` | 123.13 KiB | 19.35 KiB |
| shell-icons | `solarShellIconSubset.ts` | 24.19 KiB | 7.73 KiB |
| startup-font-css | `(none)` | 0.00 KiB | 0.02 KiB |

## 2. React 当前值（本批次实测）

构建命令：`cd ccr-ui && bun run build`（vite 8.2.2 / rolldown，Tailwind v4.3.3，`configLoader: native` 告警不阻塞）。

产物（`dist/assets/`，raw 与 gzip 均实测，gzip 为 `gzipSync(level:9)`）：

| 项 | 文件 | raw | gzip |
| --- | --- | --- | --- |
| index | `index-BDp2vaQ7.js` | 139.46 KiB | 9.70 KiB |
| react-vendor | `react-vendor-B4vv17UZ.js` | 264.51 KiB | 82.79 KiB |
| query-vendor | `query-vendor-Eleq6aX-.js` | 31.51 KiB | 9.67 KiB |
| largest-lazy | `(none)` | 0 | 0 |
| core.css | `index-TGTJC2zp.css` | 197.78 KiB | 28.19 KiB |
| shell-icons | `solarShellIconSubset.ts` | 24.19 KiB | 7.73 KiB |
| startup-font-css | `(none)` | 0.00 KiB | 0.02 KiB |
| rolldown-runtime | `rolldown-runtime-CbXtAM7H.js` | 0.58 KiB | 0.36 KiB |
| tauri-vendor | `tauri-vendor-Dowfqfn-.js` | 1.12 KiB | 0.54 KiB |

**构建时长**：第 1 次 `22.66s`（vite 内部计时，含 icons:ensure 预热）、第 2 次墙钟 `17.5s`、第 3 次墙钟 `15.6s`（均为 `bun run build`，wall 测量含 icons 生成）。详见 `implement.md` 批次 8 证据表。

**与 Vue 基线的对比**：

- index：243.69 → 139.46 KiB（**−42.8%**）。Vue 版本 index 内含 Vue 运行时与路由器；React 版本把 react/react-dom/react-router 拆进了 react-vendor，index 只含应用壳层。
- core.css：123.13 → 197.78 KiB（**+60.6%** raw）。增长原因与 preflight 无关（preflight 未开启，等价性见 `code-splitting.md` §3），主因是 Tailwind v4 自动内容扫描把未迁移 `.vue` 文件里 `@apply` 的工具类也产进了首屏 CSS（v4 无 `content` 配置，自动扫描工作区全部源文件）。该问题已记录，处置归 `08-22-design-system`（v4 落地时收敛内容扫描范围）。
- 字体：三层 CSS 语义下字体声明仍为惰性加载，`startup-font-css` 保持 `(none)` 0.00 KiB，与基线一致。

## 3. 预算表（脚本当前强制执行）

`ccr-ui/scripts/check-bundle-budget.mjs` 的 `BUDGETS` 常量。取值方法：与 Vue 基线一一对应的项以基线为参考上限（`index`），无基线对照的项以实测 × 余量（react-vendor ×1.21、query-vendor ×2、largest-lazy 以 Vue UsageDashboardView 为参考）。

| 项 | 预算 raw | 预算 gzip | 实测 raw/gzip | 依据 |
| --- | --- | --- | --- | --- |
| index | 256 KiB | 48 KiB | 139.46 / 9.70 | 与 Vue index 一一对应，以 243.69 / 45.41 为参考上限；实测为其 57% / 21% |
| react-vendor | 320 KiB | 96 KiB | 264.51 / 82.79 | 实测 ×1.21；react/react-dom/react-router 稳定，视图迁移期增长空间小 |
| query-vendor | 64 KiB | 20 KiB | 31.51 / 9.67 | 实测 ×2；`08-22-state-logic-port` 迁入后查询使用面会扩大 |
| largest-lazy | 128 KiB | 40 KiB | 0 | 以 Vue UsageDashboardView（93.40 / 26.51）为参考；React 壳层尚无懒加载则放行 |
| core.css | 240 KiB | 36 KiB | 197.78 / 28.19 | 实测 +21% 余量；v4 首屏 CSS 增长已记录（见上） |
| shell-icons | 40 KiB | 12 KiB | 24.19 / 7.73 | 文件未变，沿用旧预算 |
| startup-font-css | 150 KiB | — | 0.00 / 0.02 | 字体声明仍为惰性加载，沿用旧预算 |

脚本语义变更（对比 Vue 版本脚本）：`UsageDashboardView-` 前缀专用查找删除，改为通用「最大懒加载 chunk」查找（排除全部 `*-vendor-` 前缀与 rolldown-runtime 后取最大 `.js`）；`react-vendor` / `query-vendor` 缺失即失败（这两个分组是 React 基座必有的）。

## 4. motion 13.1.1 与 zod 4.4.3（R9.1 专用行）

`design.md` §8 第 3 步：为 motion 与 zod 单列两行，记录实际增量与预留值。预算超出不构成回退这两项选型的理由，但超出量需落盘。

### 4.1 实际增量（当前）

**两者均未被应用代码导入**，实际增量 = 0。

- **motion**：`grep -rn "from 'motion'\|from \"motion\"" src/ tests/` 零命中。构建产物检索 marker `AnimatePresence` / `framer-motion` / `motion-dom` / `motion-utils`，在全部 `dist/assets/*.js` chunk 中均为 0 命中。`dist/assets` 无 `motion-vendor-` 前缀 chunk。
- **zod**：`src/schemas/versionInfo.ts:5` 有 `import { z } from 'zod'`，但该文件仅被 `tests/zod-pilot.smoke.test.ts` 引用（`grep -rn "versionInfoSchema" src/` 无应用侧消费），故 zod 不在应用模块图中，tree-shaken 后实际 = 0。构建产物检索 marker `ZodError` / `zod v4` / `toJSONSchema`，在全部 `dist/assets/*.js` 中 0 命中。

**验证方法（已记录）**：marker 字符串检索 `dist/assets/*.js`。两个 marker 均通过 minification 存活验证（在 rolldown 构建的 scratch 产物中确认 `AnimatePresence` 与 `ZodError` 均以字面量形式存在于 minified bundle）。

### 4.2 预留值（derivation 完整记录）

预留值来源为**临时 scratch 入口 + 真实 rolldown 构建管线**的实测（非 bun build，后者 tree-shaking 不充分；非应用源码改动，scratch 文件在任务目录外创建、测后即删，`git status` 无残留）。

| 包 | scratch 入口 | 实测 raw | 实测 gzip | 预留值 raw/gzip |
| --- | --- | --- | --- | --- |
| motion 13.1.1 | `import { motion, AnimatePresence } from 'motion/react'`（globalThis 强制保留，等价真实渲染路径） | 121.89 KiB | 39.19 KiB | **128 / 44 KiB**（实测取整） |
| zod 4.4.3 | `import { z } from 'zod'; z.object({...})`（globalThis 强制保留） | 62.50 KiB | 16.62 KiB | **64 / 20 KiB**（实测取整） |

交叉验证：

- zod 与 `08-22-react-foundation` 的 `zod-pilot.md` 实测增量（index chunk 167,158 → 226,561 B，**+59,403 B raw / +16,008 B gzip**）在同一数量级且同源（同一 rolldown 管线），差异为测量口径（pilot 测 index-chunk delta，本任务测隔离 scratch 产物）。
- motion 的 `motion/react` 入口是 `framer-motion` 的重导出 shim（`node_modules/motion/dist/es/react.mjs` 仅 re-export `framer-motion`），真实体积在 framer-motion 及其依赖 `motion-dom` / `motion-utils`。scratch 实测已包含该依赖链。on-disk 参考：framer-motion dist/es 295.8 KiB（164 文件）、motion-dom 453.5 KiB（220 文件）、motion-utils 13.9 KiB（30 文件）、zod v4/classic 80.7 KiB（10 文件）——on-disk 总和远大于 minified gzip，因为后者经 rolldown minify + tree-shake。

### 4.3 更新约定

两行在 `08-22-design-system`（motion）与 `08-22-state-logic-port` / `08-22-test-contract-rebuild`（zod 推广）实际导入后**必须以真实消耗数更新**：

- 若包进入 `motion-vendor` / `form-vendor` 专用分组：实际增量 = 该 chunk 体积（脚本自动识别）。
- 若包被内联进 index 或其它 chunk：脚本对 marker 命中但无法归因的情况会**强制 FAIL**，要求更新预留行——这是脚本的显式防呆设计。

当前 `check-bundle-budget.mjs` 对两行输出：

```
[bundle-budget] motion: (none) actual=0.00 KiB / 0.00 KiB reserved=128.00 KiB / 44.00 KiB
[bundle-budget] zod: (none) actual=0.00 KiB / 0.00 KiB reserved=64.00 KiB / 20.00 KiB
```

## 5. manualChunks 判定记录（design §8 第 3 步）

见 `code-splitting.md` §4 与 `implement.md` 批次 8 证据。结论摘要：**新增 `query-vendor`（同时匹配 `@tanstack/react-query` 与 `@tanstack/query-core`）；`form-vendor` / `motion-vendor` 暂不加入**。

关键测量：

- 当前构建中 `@tanstack/react-query` 真实导入（`src/main.tsx` + `src/shell/queryClient.ts`），无手动分组时其查询引擎（`@tanstack/query-core` 独立包）被内联进 index chunk。
- 仅匹配 `@tanstack/react-query` 的正则**不生效**（query-core 是独立包），需同时匹配两者；修正后 query-vendor 单独成 chunk 32.26 kB，index 从 167.15 → 142.80 kB。
- `react-hook-form` / `motion` 当前无导入点：空分组不产出 chunk（无收益），且分组会在预算脚本里成为「应存在但缺失」的失败项，故不加入。待实际导入时补加（`vite.config.ts` 注释已写明）。

## 6. 红证（budget 检查确实失败）

临时把 `index` 预算从 256/48 收紧到 100/20 KiB，`bun ./scripts/check-bundle-budget.mjs` 输出：

```
[bundle-budget] FAIL index raw 139.46 KiB > 100.00 KiB
```

退出码 **1**。还原后复跑退出码 **0**（`PASS all budgets satisfied`）。`git diff` 确认脚本无残留改动。

## 7. 未决项 / 交接

- core.css 增长（v4 自动扫描 `.vue`）的收敛处置归 `08-22-design-system`。
- `motion` / `zod` 预留行的真实消耗数更新归 `08-22-design-system` / `08-22-state-logic-port`（§4.3）。
- 首屏 CSS 体积（197.78 KiB raw）作为 `08-22-regression-release` 的对比基线数据之一（协同点 L）。
