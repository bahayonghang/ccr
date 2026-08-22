# vite 7.3.5 → 8.2.2 breaking change 核对记录（design.md §6）

> 任务：`08-22-dep-upgrade` 段 1。核对日期：2026-08-23。
> 本文件补录段 1 的 §6 核对结论。段 1 提交：d11ef85c；配置现状见 `ccr-ui/vite.config.ts`（React 基座改写由 `08-22-react-foundation` 批次 2 完成）。

## a. §6 核对项现状确认表

| 核对项 | v7 行为 | v8 行为 | 本仓配置是否受影响 | 证据 |
| --- | --- | --- | --- | --- |
| `build.rollupOptions.output.manualChunks` 接受形态 | 对象与函数两种形态均接受（底层 rollup） | 底层换为 rolldown，对象形式不再支持，仅函数形式 | 受影响，已改为函数形式（9 个 vendor 分组逐一正则匹配，语义不变） | `vite.config.ts:22-46` 函数形式及注释；`08-22-react-foundation/implement.md:30`「vite 8/rolldown 下 manualChunks 需函数形态」；`overrides-review.md` rollup 行「vite 8.2.2 底层为 rolldown，依赖树中已不存在 rollup」 |
| `optimizeDeps.noDiscovery` 语义 | 关闭扫描发现，仅预构建 `include` 清单 | 语义不变；`noDiscovery: true` 走 explicit deps optimizer，跳过 crawl 发现，只处理 `include` 列表 | 不受影响；沿用 noDiscovery + 显式清单模型 | vite 8.2.2 源码 `dist/node/chunks/node.js`：`noDiscovery ? createExplicitDepsOptimizer : createDepsOptimizer`、`noDiscovery … return`（跳过发现）；`vite.config.ts:68-90` |
| `optimizeDeps.include` 子路径要求 | 无此坑（有扫描兜底） | noDiscovery 下无扫描兜底，子路径必须逐条列入；缺 `react-dom/client` 时该入口以裸 CJS 直出，`createRoot` 具名导入失败 | 已按子路径补全并验证 | `vite.config.ts:70-89` 含 `react-dom/client` 及 `@tauri-apps/api/*` 子路径；`08-22-react-foundation/implement.md:33` 记录 2026-08-23 修复 |
| `server.fs.allow` 配置键 | 放行 dev server 文件访问目录列表 | 键保留，行为一致 | 不受影响；放行仓库根 `crates/ccr-checkin/data` 的 catalog 数据目录 | `vite.config.ts:56-60`；2026-08-23 实测 `bun run dev` 启动成功（ready 1230 ms），键被正常接受 |
| `server.warmup` 配置键 | 预热 client 文件列表 | 键保留，行为一致 | 不受影响；`clientFiles` 继续消费手写清单 `scripts/dev-warm-targets.json` | `vite.config.ts:61-63`；同上实测启动成功；清单来源见 `plugin-selection.md`「dev-warm-targets 生成方式」（手写，无生成脚本） |
| `server.watch.ignored` | chokidar 忽略模式 | 键保留，行为一致 | 不受影响；继续忽略 `src-tauri/target`、`ref/`、`logs/` | `vite.config.ts:53-55`；同上实测启动成功 |
| CSS 处理与 PostCSS 集成（叠加 Tailwind v4） | PostCSS 管线 + autoprefixer | PostCSS 管线保留，`postcss.config.*` 自动加载方式不变；与 Tailwind v4（`@tailwindcss/postcss`）兼容 | 不受影响；段 2 已完成 v4 切换并全量验证 | `ccr-ui/postcss.config.js` 仅挂 `@tailwindcss/postcss`（autoprefixer 由 v4 内置取代）；`implement.md` 段 2 验证行：`bun run build` exit 0、`lint:style` exit 0、audit 0 advisories；体积数据见 `css-size.md` |
| Vitest 4.1.10 对 vite 8 的支持 | — | 兼容，无需随 vite 升级 | 不受影响；vitest.smoke 配置以 `@vitejs/plugin-react` 跑通 | `vitest.smoke.config.ts`（plugin-react + jsdom）；`08-22-react-foundation/implement.md:53`：`bun run test:smoke` exit 0，59 文件 / 293 用例全绿 |

覆盖计数：§6 共 7 个核对面（manualChunks 形态、noDiscovery 语义、fs.allow/warmup 键、watch.ignored、CSS/PostCSS、Vitest），上表 8 行（include 子路径作为 noDiscovery 的补充项单列），全部有证据，无未核实项。

## b. 发现的 breaking change

### b1. rolldown：manualChunks 仅接受函数形式

vite 8 底层构建引擎由 rollup 换为 rolldown。对象形式的 `manualChunks`（`{ 'vendor-name': ['dep1', 'dep2'] }`）不再被接受，必须写成 `(id) => string | undefined` 的函数形式。

本仓处理：`vite.config.ts` 以函数形式维护 9 个分组（react-vendor / ui-vendor / charts-vendor / i18n-vendor / markdown-vendor / search-vendor / tauri-vendor / virtual-vendor / term-vendor），分组语义与原设计清单一一对应。连带影响：rollup / esbuild 两项 override 在 vite 8 依赖树中失去 pin 对象，已在 `overrides-review.md` 判定为移除。

### b2. configLoader 原生加载器的 advisory warning（当前 warning-only）

vite 8 引入三档 `configLoader`（`bundle` / `runner` / `native`），默认仍为 `bundle`。`native` 计划在未来大版本成为默认；对 native 加载器不支持的写法，启动时输出 advisory warning（不影响本次运行）。

本仓命中的两条（2026-08-23 实测 `bun run dev` 输出原文）：

```text
(!) Your Vite config uses features that are unsupported by `configLoader: 'native'`, which is planned to become the default in a future major version of Vite:
  - `__dirname` (vite.config.ts:8:10). Use `import.meta.dirname` instead
  - JSON import "./scripts/dev-warm-targets.json" without import attributes (vite.config.ts:5:28). Add `with { type: 'json' }`
Set `VITE_CONFIG_NATIVE_IGNORE_WARNING=true` to suppress this warning.
```

对应源码位置：`vite.config.ts:5` 的 JSON 导入与 `:7-8` 的 `__dirname` 回退定义。修复方向明确（`import.meta.dirname` + `with { type: 'json' }`），当前无功能影响，dev server 正常 ready。

**未来风险登记**：当 vite 后续大版本把默认 `configLoader` 切到 `native` 时，上述两条会从 warning 变为硬错误。届时需在本仓完成两处改写；若提前想消除告警，可设 `VITE_CONFIG_NATIVE_IGNORE_WARNING=true` 或直接现在改写。该项不阻塞本任务，建议随下一次 vite 小版本升级一并处理。

## c. vue-i18n 双入口 alias 移除

v7 配置中的 `vue-i18n` dev / build 双入口 alias（原 `vite.config.ts:22`）随 `vue-i18n` 依赖移除而删除。该 alias 存在原因是桌面壳 CSP 与 vue-i18n runtime compiler 冲突；i18next 无 runtime compiler，CSP 下无等价问题。现状确认：当前 `vite.config.ts` 的 `resolve.alias` 仅剩 `'@'` 一项（`:14-18`）。i18n 侧后续义务（静态检查缺口 R9 补齐）归 `08-22-i18n-port`，交叉引用见 design.md §8。
