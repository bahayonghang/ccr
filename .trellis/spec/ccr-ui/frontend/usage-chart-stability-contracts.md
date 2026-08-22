# Usage Chart Stability Contracts

> ApexCharts(vue3-apexcharts)在 usage 仪表盘中的引用纪律与 KeepAlive 交互契约。
> 违反任意一条的直接症状:tab 再进入或窗口切换时图表 canvas 被销毁重建(闪烁、
> 入场动画重放、耗时数倍),KeepAlive 缓存收益被整体抵消。
> 提炼自 `07-07-ui-usage-dashboard` 任务的性能复测(归档任务内
> `research/after/perf-comparison.md` 有完整根因链与前后数据)。

---

## 1. vue3-apexcharts 的 prop 监听语义(事实基础)

`vue3-apexcharts` 组件(v1.x,`node_modules/vue3-apexcharts/dist/vue3-apexcharts.js`)
对 props 的处理决定了一切上层纪律:

| prop | 监听方式 | 触发后果 |
| --- | --- | --- |
| `options` | **引用监听**(非 deep) | `chart.updateOptions()` → ApexCharts 内部全量 update,**销毁重建 `.apexcharts-canvas` 节点** |
| `series` | **deep watch** | `chart.updateSeries()` → 同样走 ApexCharts 全量 update 重建 canvas |
| `type` / `width` / `height` | 值监听 | `destroy()` + 重新 `init()`(最重路径) |

推论:

- "updateSeries 是快路径"仅相对于 Vue 层组件重挂而言;series **引用一变**(即使值相同)
  canvas 节点照样重建。因此引用稳定性必须在数据源头保证,而不是指望 apexcharts 去重。
- `type/width/height` 不允许绑定会变化的表达式(高度用固定值或 CSS 控制)。

## 2. options 构建纪律

- 所有 usage 图表 options 统一走 `src/views/usage/usageChartOptions.ts` 的工厂
  (`buildTrendChartOptions` / `buildDistributionPieOptions`)或遵循同样的结构:
  **静态骨架用模块级 `Object.freeze` 常量**,工厂只注入主题色、locale、坐标轴标量。
- options 的 computed 依赖只允许:theme、locale、坐标轴形状(tickAmount/granularity)、
  按 join key 记忆化后的 labels/seriesNames。**数据经取值器闭包(如 `getBuckets`)
  在渲染时读取**,不得成为 options 的构建期依赖。
- **每个 chart options 必须包含**:

  ```ts
  redrawOnParentResize: false,
  redrawOnWindowResize: false,
  ```

  ApexCharts 默认 `redrawOnParentResize: true`,而 KeepAlive 重挂(DOM 摘下再插回)
  必然触发 parentResize → 全量重建。TREND/PIE 冻结基座已内置;**tab 组件内自建的
  局部 options(如 UsageTokensTab/UsageCostTab 的柱状图)最容易漏**,新增图表时逐项核对。
- 动画统一走 `buildChartAnimations()`(已导出;默认开启、`prefers-reduced-motion` 降级),
  不要新增硬编码 `animations: { enabled: false }`。

## 3. series 引用纪律

- `useUsageCharts.ts` 中所有喂给图表的 series(`trendSeries` / `pieSeries` /
  `modelTokenPieSeries`)**按值记忆化**:`computed(previous)` + join key,值未变时
  返回上一引用。Vue 的 computed 以 `Object.is` 判等,旧引用不向下游传播,
  vue3-apexcharts 的 deep watch 便不会被无效触发。
- 为什么必须做:`dashboardPresentation` 的输入含 `selectedWindowLabel` 等纯文案,
  窗口切换、locale 变化、30s auto-refresh(同值新数组)都会让 presentation 整体重算,
  产出"值相同、引用全新"的 series——不记忆化则每次都白白重建 canvas
  (实测 37ms 内发生、早于 300ms 筛选防抖的 refetch,数据根本没变)。
- 新增图表时同样处理;join key 必须覆盖 series 的全部渲染字段(name + 每个数据点)。

## 4. KeepAlive 交互契约(UsageDashboardView)

- tab 组件引用必须是**模块级稳定引用**(静态 import 或模块级 `defineAsyncComponent`
  包装),不得在渲染期重建,否则 KeepAlive 判为新组件丢缓存。
- 图表水合门控(`shouldRenderTrendChart` 等)只跟随 `*Ready` 单调标志
  (false→true 一次),**不得耦合 `activeTab`**:切走的 tab 实例仍活在缓存里,
  门控翻假会卸载已挂的图表,返回时重建,抵消 KeepAlive。
- 刷新期间已有内容不拆树:loading 面板只在"尚无可渲染数据"时接管
  (`hasDashboardData` 门控),数据刷新交给 series 记忆化 + updateSeries。

## 5. ApexCharts 完整 CSS 双路径交付

`src/utils/apexChartsCore.ts` 是 ApexCharts 的唯一模块化装配入口。该入口必须同时满足：

```ts
import VueApexCharts from 'vue3-apexcharts/core'
import 'apexcharts/dist/apexcharts.css'

import 'apexcharts/area'
// 继续按实际使用注册 chart type / feature
```

- 构建路径：静态导入上游发布的完整 `apexcharts.css`，由 Vite 随
  `apexChartsCore` 异步 chunk 交付。
- 运行时路径：保留 ApexCharts 默认的 `chart.injectStyleSheet: true`，不得为了消除重复
  规则而关闭 `#apexcharts-css` 注入。两条路径内容相同，任一路径可用时都应满足完整布局
  契约。
- 懒加载边界：CSS import 必须与模块化装配入口共置，不得提升到 `main.ts`。生产构建需
  确认 CSS 是图表调用方的 preload 依赖，且 `index.html` 没有首屏直链它。

这不是单一 marker 的视觉补丁。完整样式同时负责 tooltip 的绝对定位、series group 初始
隐藏、marker host 的 `12x12` 尺寸及内部 SVG 缩放。不得复制一组 ApexCharts 私有 selector、
修改全局 SVG reset，或只给 marker 补宽高；这些做法会留下静态占位、错误 tooltip 布局或
升级漂移。

依赖升级或装配入口调整时，`tests/apexcharts-style-contract.smoke.test.ts` 必须继续断言：

- `vue3-apexcharts/core` wrapper 与所有实际使用的模块注册保持唯一；
- 完整 CSS import 保持唯一；
- `.apexcharts-tooltip` 为绝对定位；
- `.apexcharts-tooltip-series-group` 初始隐藏；
- `.apexcharts-tooltip-marker` 为 `12x12`，其 SVG 为 `100% x 100%`。

故障复测应在首次图表挂载前阻止运行时 `#apexcharts-css` append，而不是挂载后临时删除。
此时构建管理的 CSS 仍须让上述契约全部成立，tooltip hover 不得改变卡片高度。该故障注入
只证明双路径容错，不等于找到了现场使运行时样式缺失或失效的自然触发源。

## 6. 验证方法(回归复测口径)

- 浏览器桩 + 测量脚本在归档任务 `07-07-ui-usage-dashboard/research/perf-harness/`:
  `tauri-shim.js`(伪造 Tauri IPC 的 fixture 桩)、`measure-after.mjs`(基线同口径
  复测:tab 切换 ×3 轮、窗口切换 ×2、20 次往返内存)、`diagnose-after.mjs`
  (canvas/组件根节点身份 probe)。
- 判定标准:**节点身份**(`data-perf-id` 标记法)——tab 再进入 rebuilt=false、
  窗口切换旧 canvas 全部存活;耗时只是辅助指标。
- 注意 store 有 `DASHBOARD_CACHE_TTL_MS = 30s` 快照缓存:30s 内切回同窗口不发 IPC,
  测窗口切换要区分 refetch 路径与缓存路径。

## 7. 横轴日期标签

平台首页趋势图（`PlatformUsageTrendChart`）与 Usage 仪表盘日趋势必须用 `xaxis.type: 'datetime'`，
标签走 `formatTrendAxisLabel` + `parseUtcDate`（`YYYY-MM-DD` → UTC 午夜）。禁止：

- category 轴 + `labels.trim: true` + ISO `YYYY-MM-DD` 字符串。ApexCharts 按全部
  category 槽宽判断溢出，30 天窗口会把 `2026-07-22` 裁成 `2026-07...`，即使
  `tickAmount` 只显示 6 个刻度。
- 再写一套月日格式化。locale short 已由 `formatTrendAxisLabel` 覆盖（en-US `Jul 22`，
  zh-CN `7月22日`）。

`tests/platform-usage-trend-chart.smoke.test.ts` 锁 datetime、`trim: false`、
`redrawOnParentResize: false`。`tests/usage-chart-diagnostics.smoke.test.ts` 锁
`parseUtcDate` 与日标签文案。

## 已知偏差(接受现状,改动时顺手收敛)

- ~~`UsageTokensTab.vue` / `UsageCostTab.vue` 局部 options 硬编码
  `animations: { enabled: false }`~~——已收敛(07-07-ui-consistency-sweep R2-6,
  两处均改走导出的 `buildChartAnimations()`)。
- cost tab 的 options 直接依赖 `ctx.trends`(数据刷新会对离屏缓存图表触发
  updateOptions 重建,用户不可见)。将图表 options 收编进工厂时一并处理。
