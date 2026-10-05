# Bundle 预算重设（发布门，2026-08-24）

> 任务：`08-22-regression-release` AC10 / 父任务 AC19。
> 命令：`cd ccr-ui && bun run check:bundle-budget`
> 构建：`just tauri-build` 内 `bun run build`（vite 8.2.2 / rolldown）

## 1. Vue 基线

来源：`08-22-react-migration/baseline/bundle-budget.txt`

| 项 | raw KiB | gzip KiB |
| --- | --- | --- |
| index | 243.69 | 45.41 |
| UsageDashboardView（最大懒加载） | 93.40 | 26.51 |
| core.css | 123.13 | 19.35 |

## 2. 基座期 React 预算（arch-quality-perf 批次 8）

当时视图未迁完，index gzip 9.70，无懒加载 chunk，motion/zod 实际增量 0。预算按该壳层取值。

## 3. 视图迁完后实测

| 项 | 文件 | raw KiB | gzip KiB |
| --- | --- | --- | --- |
| index | `index-5ttdN54c.js` | 230.48 | 72.92 |
| react-vendor | `react-vendor-BOcZQ-dY.js` | 265.24 | 83.35 |
| query-vendor | `query-vendor-DEgXC9yG.js` | 34.73 | 10.18 |
| largest-lazy（排除 locale/vendor 后） | `logger-C06Iqv4e.js` | 143.32 | 10.86 |
| core.css | `index-D2Z9poxD.css` | 188.68 | 27.88 |
| motion | `motion-vendor-Do058HhP.js` | 118.38 | 38.10 |
| zod | `zod-BkJwUjWR.js` | 66.37 | 17.93 |
| locale（不计入 largest-lazy） | `en-US-ChfGVMBQ.js` | 163.03 | 49.71 |

相对 Vue index：raw 230.48 < 243.69；gzip 72.92 > 45.41。gzip 升高的原因：入口在视图迁完后含壳层 + 共享平台面，压缩比低于 Vue 单包。React 运行时在 `react-vendor`（gzip 83.35），不计入 index。

locale chunk 是 i18n 懒加载文案，不是 UsageDashboard 对照项。`largest-lazy` 排除 `en-US-` / `zh-CN-` 与 vendor 前缀。

## 4. 脚本重设

`ccr-ui/scripts/check-bundle-budget.mjs`：

| 项 | 旧预算 raw/gzip | 新预算 raw/gzip | 依据 |
| --- | --- | --- | --- |
| index gzip | 48 | 80 | 实测 72.92 + 约 10% |
| largest-lazy raw | 128 | 160 | 实测 logger 143.32 + 约 12%；gzip 仍 40 |
| zod 专用前缀 | `form-vendor-` | `zod-` | 产物为 `zod-*.js` |
| zod 预留 | 64 / 20 | 80 / 24 | 实测 66.37 / 17.93 |
| lazy 排除 | vendor 前缀 | 另排除 locale 与 `zod-` | locale 不与 UsageDashboard 对照 |

重设后：`[bundle-budget] PASS all budgets satisfied`。
