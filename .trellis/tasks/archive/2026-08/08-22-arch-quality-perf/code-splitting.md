# 代码分割与 CSS 分层的等价方案

> 所属任务：`08-22-arch-quality-perf` 批次 8（AC10）。
> 依据：`design.md` §9（代码分割与 CSS 分层的等价方案）。
> 采集日期：2026-08-23，分支 `react-migration/react-foundation`，未提交。

## 1. 约定表（现状 → React 侧等价）

| 现状 | React 侧等价 | 状态 |
| --- | --- | --- |
| 22 处 `defineAsyncComponent` 路由级懒加载 | `React.lazy` + `Suspense`，或 React Router 路由级 `lazy`。二选一由 `08-22-shell-port` 落地 | 约定（本条） |
| 三层 CSS 加载（`shell-critical` / `deferred-decorations` / `deferred-interactive`） | 三层语义保留。Tailwind v4 下的落地方式由 `08-22-design-system` 决定 | 约定（本条） |
| `corePlugins.preflight: false` + 自带 reset | 已由 `08-22-dep-upgrade` 段 2 完成等价处理 | 已核对（§3） |

## 2. 路由级懒加载（约定）

**约定：懒加载边界与路由边界一致，不在路由内部再分割。**

- 落地方式（`React.lazy` + `Suspense` 或 React Router 路由 `lazy`）由 `08-22-shell-port` 决定，本任务不定死。
- 约束：每个懒加载点对应一条路由（或其整棵子树），禁止在同一路由内部把子组件再拆成独立懒加载点。
- 等价性判据：React 侧每个路由懒加载点对应 Vue 侧一个 `defineAsyncComponent` 路由级懒加载点；22 个 Vue 懒加载点的路由清单见父任务 `08-22-react-migration`（75 条路由按域划分，每域 2–3 条）。

当前 React 壳层（阶段 1 最小页面）**尚无任何懒加载**：`src/shell/router.tsx` 只有 1 条路由且直接同步渲染 `<App />`。因此本批次记录：

```
[bundle-budget] largest-lazy: (none) 当前 React 壳层无懒加载 chunk，放行
```

预算脚本已实现通用「最大懒加载 chunk」查找（排除 `*-vendor-` 与 rolldown-runtime），`08-22-shell-port` 引入 `React.lazy` 后自动生效，无需再改脚本。

## 3. CSS 分层与 preflight 等价性核对

### 3.1 三层 CSS 语义

现状（Vue）：`src/main.ts` 同步加载 `styles/index.css`（含 `shell-critical` 首屏关键样式），`deferred-decorations.css` / `deferred-interactive.css` 由 `import('...?url')` 在 `main.ts:32-33` 惰性加载。

React 现状（`src/main.tsx:7`）：同步加载 `styles/index.css`，内容不变（`core.css` + `checkin-shared.css` + `codex-auth-shared.css`）。`deferred-*.css` 尚无 React 侧等价加载点——**三层语义的 React 落地归 `08-22-design-system`**。

**约定：首屏 CSS 只含 `shell-critical` 层，其余两层延迟加载。** 该语义在 v4 迁移后必须保留（`codex-auth-shared.css` 于 2026-06 抽出，属全局样式层，仍应进入首屏；`deferred-*` 两层仍应惰性）。

### 3.2 preflight 等价性核对（design §9 第三行）

- **preflight 未开启**：`src/styles/core.css:7-8` 注释明确「只引入 theme 与 utilities 两层，不引入 preflight；reset 仍由 base.css 提供（等价于 v3 `corePlugins.preflight: false`）」。构建产物核对：Tailwind v4 preflight 特征字符串（`border-style:solid` 的 preflight 上下文、`img,video{max-width:100%`、`button{background` 等）在 `dist/assets/index-TGTJC2zp.css` 中**不构成 preflight 块**——`border-width:0` / `border-style:solid` 的命中来自 `.sr-only` 与 `.border-0` 工具类和 `--tw-border-style` 属性，非 preflight 重置。自定义 reset 由 `src/styles/base.css` 的 `@layer base { *{box-sizing:border-box;margin:0;padding:0} }` 提供，已确认存在于产物。
- **首屏 CSS 体积对比**：

| 指标 | Vue 基线 | React 当前 | 差值 |
| --- | --- | --- | --- |
| core.css raw | 123.13 KiB | 197.78 KiB | +74.65 KiB |
| core.css gzip | 19.35 KiB | 28.19 KiB | +8.84 KiB |

**结论：首屏 CSS 体积较 Vue 基线上升，但上升与 v4 无关的 preflight 无关——preflight 未开启，等价性成立（preflight 层面首屏 CSS 未因 v4 变差）。** 上升主因：Tailwind v4 的自动内容扫描（无 `content` 配置）覆盖全部源文件，未迁移 `.vue` 文件里 `@apply` 的工具类（如 `grid-cols-2`、`backdrop-blur-md`）被产进首屏 CSS；已确认 `backdrop-blur-md` 仅在 `.vue` 文件中出现、却存在于构建产物。该问题的收敛（v4 显式 content 配置或分层裁剪）归 `08-22-design-system`，本任务只核对与记录。

layer 构成实测（`index-TGTJC2zp.css`，202,528 B 总 raw）：theme 7.5 KiB、base 7.7 KiB、components 视分布、utilities 约 187.5 KiB（含 `@layer properties` 2.0 KiB 的 v4 默认属性层）。utilities 占绝对主体，进一步佐证「工具类被过量扫入」的判断。

## 4. manualChunks 判定（design §8 第 3 步，通知 react-foundation）

`08-22-react-foundation` 已交付归档（`.trellis/tasks/archive/2026-08/`），按 `design.md` §8 直接改 `vite.config.ts`。判定基于实测 chunk 图：

### 4.1 现状 chunk 图（改前）

`@tanstack/react-query` 在 `src/main.tsx` 与 `src/shell/queryClient.ts` **真实导入**，但不在任何 manualChunks 分组中。实测其查询引擎（`@tanstack/query-core`，独立包）被内联进 **index chunk**（`index-*.js` 检出 `queryFn`、`cancelQueries`、`setQueryData` 等 marker）。index = 167.15 kB。

`react-hook-form`（依赖但未导入）、`motion`（依赖但未导入）、`zustand`（依赖但未导入）：marker 检索全部 0 命中，不在任何 chunk 中。

### 4.2 判定

| 分组 | 判定 | 依据 |
| --- | --- | --- |
| `query-vendor` | **加入**（同时匹配 `@tanstack/react-query` 与 `@tanstack/query-core`） | react-query 已真实导入且引擎在独立包中，不分组则内联进 index。加入后 index 167.15 → 142.80 kB（−24.35 kB），query-vendor 单独成 chunk 32.26 kB，利于缓存与并行加载 |
| `form-vendor` | **暂不加入** | react-hook-form 当前无导入点，空分组不产出 chunk（零收益）；且会在预算脚本中成为「应存在但缺失」的失败项 |
| `motion-vendor` | **暂不加入** | motion 当前无导入点，同上 |

**技术要点（踩坑记录）**：`query-vendor` 的正则必须同时匹配 `@tanstack/query-core`。仅匹配 `@tanstack/react-query` 时，rolldown 把 query-core 留在 index（分组不生效），实测为「分组存在但 react-query 未拆出」——这会让预算脚本误判通过。当前 `vite.config.ts` 正则：`/[\\/]node_modules[\\/]@tanstack[\\/](react-query|query-core)[\\/]/`。

### 4.3 改后效果（before/after）

| 指标 | 改前 | 改后 | 差值 |
| --- | --- | --- | --- |
| index raw | 167,158 B | 142,807 B | **−24,351 B** |
| index gzip | 16.77 KiB | 9.70 KiB | −7.07 KiB |
| react-vendor raw | 278,608 B | 270,861 B | −7,747 B（rolldown 重新平衡） |
| query-vendor raw | — | 32,264 B | +32,264 B |
| core.css | 202,528 B | 202,528 B | 0（手动分组不影响 CSS） |
| 构建 wall | 17.5 s | 15.6 s | 同量级 |

合计 chunk 数：改前 5 个 `.js`，改后 6 个 `.js`。

## 5. 阶段 2 快照（当前构建的实际 chunk/lazy 结构）

`bun run build`（vite 8.2.2 / rolldown）产物 `dist/`：

```
dist/
  index.html
  assets/
    index-BDp2vaQ7.js        139.46 KiB  # 应用壳层（含 App.tsx、queryClient、router 同步代码）
    react-vendor-B4vv17UZ.js 264.51 KiB  # react + react-dom + react-router
    query-vendor-Eleq6aX-.js  31.51 KiB  # @tanstack/react-query + @tanstack/query-core
    tauri-vendor-Dowfqfn-.js   1.12 KiB  # @tauri-apps/api（当前壳层用到的最小面）
    rolldown-runtime-CbXtAM7H.js 0.58 KiB
    index-TGTJC2zp.css       197.78 KiB  # 首屏 CSS
    favicon-Duj6Comt.svg
  fonts/    # 惰性字体资源（MapleBright 子集 + Inter/JetBrainsMono）
```

- **懒加载**：0 处（`src/shell/router.tsx` 单路由同步渲染）。`src/shell/` 无 `React.lazy` / `Suspense` / 动态 `import()`。
- **index.html modulepreload**：`rolldown-runtime`、`query-vendor`、`react-vendor`、`tauri-vendor` 四个 `.js`（无 `.css` 单独 preload——CSS 由 `<link rel="stylesheet">` 加载）。
- 这是 `08-22-shell-port` 引入懒加载与 `08-22-design-system` 落地三层 CSS 之前的阶段 2 基线快照。

## 6. 交接

- `React.lazy` / 路由 `lazy` 的落地：`08-22-shell-port`。
- 三层 CSS 的 v4 落地与 `.vue` 工具类扫描收敛：`08-22-design-system`。
- 预算脚本的懒加载 chunk 上限与手动分组（含 `form-vendor` / `motion-vendor` 待补分组）：本任务 `bundle-budget.md` §3/§5。
