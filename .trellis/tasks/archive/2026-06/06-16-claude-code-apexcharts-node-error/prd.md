# 修复 Claude Code 页面 ApexCharts node 报错

## Goal

打开 ccr-ui 的 **Claude Code** 页面（`/claude-code`）时，右上角不再弹出
`应用错误: TypeError: Cannot read properties of undefined (reading 'node')`，
且“近 30 天每日费用”面积图能正常渲染（不再留下空白容器）。

## 现象（Symptom）

- 进入 Claude Code 页面，右上角出现全局错误吐司：
  `应用错误: TypeError: Cannot read properties of undefined (reading 'node')`。
- 同时“费用归因”Tab 下的「近 30 天每日费用」面积图区域为空白（既没有图、也没有“暂无数据”空态），
  但「按模型拆分」横条有数据 —— 说明数据已到位，是**图表渲染本身崩了**。

## 根因分析（Root Cause）

### 1. 错误如何冒泡到全局吐司

`ccr-ui/src/main.ts:100` 注册了全局兜底 `app.config.errorHandler`，把任意未捕获的 Vue
组件异常格式化为 `应用错误: ${name}: ${message}` 并通过 `useUIStore().showError()` 弹出。
所以这条吐司本质是**某个组件在渲染/生命周期里抛了未捕获异常**，被全局兜底接住。

### 2. `'node'` 来自 ApexCharts / SVG.js

- 全量搜索 `ccr-ui/src` 没有任何业务代码访问 `.node`（`rg '\.node\b' ccr-ui/src` 为空）。
- 整个依赖里只有 **apexcharts**（内部 SVG.js）大量使用 `element.node` 访问真实 DOM 节点。
- Claude Code 页面上唯一的 ApexCharts 实例就是 `CostAttributionTab.vue` 的每日面积图
  （`ccr-ui/src/components/claude-observer/CostAttributionTab.vue:14-23`）。截图里空白的正是它。

> 结论：异常来自 ApexCharts 在**首次渲染/动画过程中**对一个已不存在的 SVG 元素做了 `el.node`
> 解引用，抛出 `Cannot read properties of undefined (reading 'node')`。属于 ApexCharts 已知问题族
> （react-apexcharts #602 `[ChartID] ... reading 'node'`、#598 `... reading 'group'`，共享核心库 bug）。

### 3. 触发窗口：vue3-apexcharts 异步 init + 响应式 watcher + 动画

`node_modules/vue3-apexcharts/dist/vue3-apexcharts.js` 的封装逻辑：

```js
// onMounted 里调用 init()，init 是异步的：先 await nextTick() 再 new ApexCharts().render()
q(() => { g.value = $el; r() })
const r = async () => { if (await H(), t.value) return; ...; t.value = new w(g.value, s); return t.value.render() }
// 非 immediate 的 options/series watcher，t.value 为空时会再次调用 init()
c(u.options, () => { !t.value && a.options ? r() : t.value.updateOptions(a.options) })
c(u.series,  () => { !t.value && a.series ? r() : t.value.updateSeries(a.series) }, { deep: true })
```

叠加本仓库的具体场景，形成生命周期竞态：

1. `UsageInsightPanel.vue` 本身是 `defineAsyncComponent` 异步挂载
   （`ClaudeCodeView.vue:342`），`onMounted` 里才 `store.fetchAll()` 拉数据。
2. 图表用 `v-if="hasDaily && shouldRenderChart"` 驱动：**数据到位的那一刻**才挂载
   `ApexChartAsync`（又是一个异步组件，先显示 `ChartPreparingState`，import 完成后才换成真图）。
3. 真图在「`fetchAll` 的 Promise.all 多个切片陆续 flush、面板正在密集重渲染」的窗口内挂载，
   而 ApexCharts `render()` 是异步的、且 `animations.enabled` 开启
   （`CostAttributionTab.vue:183-187`）会用 `requestAnimationFrame` 做 morph 动画。
4. 在这个窗口里，ApexCharts 实例被销毁/重建或动画回调晚于 DOM 清理执行，morph 回调拿到的
   SVG 元素已被移除 → `el.node` 为 `undefined` → 抛错。图表 SVG 被清掉 → 留下空白容器。

> 补充：`dailyOptions` 里显式开了 `redrawOnParentResize/redrawOnWindowResize`
> （`CostAttributionTab.vue:184-185`，其实是 ApexCharts 默认值），会注册 ResizeObserver；
> 虽然 `_parentResizeCallback` 有 `animationEnded` 守卫，但它进一步放大了“渲染/动画期间被外力打断”的面。

### 4. 影响范围

三个 Tab（费用归因 / Token 详情 / 行为分析）都通过**同一个**共享封装
`ccr-ui/src/components/claude-observer/apexChart.ts` 的 `ApexChartAsync` 渲染 ApexCharts
（`CostAttributionTab` 面积图、`TokenDetailTab` 堆叠面积、`BehaviorAnalysisTab` 热力图）。
因此修复应收口在共享封装层，而不是只补 `CostAttributionTab`。

## Requirements（修复方向 · 防御纵深）

> 先复现确认（Tauri dev 打开 `/claude-code`，观察吐司 + 空白图），再按下列优先级修。

- **R1（必须）抑制冒泡 / 兜底降级**：在共享 `apexChart.ts`（或包一层
  `ChartErrorBoundary`）用 `onErrorCaptured` 接住 ApexCharts 内部抛出的异常，降级为
  空态 / `ChartPreparingState`，**不再冒泡到 `main.ts` 全局 errorHandler**。这直接消除三个 Tab 的“应用错误”吐司。
- **R2（应当）稳定图表挂载，保证真渲染**（解决空白，不只是吞错）：
  - 给 `<component :is="apexchart">` 加**稳定 `:key`**（按数据指纹/Tab 维度），让 options/series
    变化走「干净重挂」而非在半成品实例上原地 `updateOptions/updateSeries`。
  - 把图表挂载**延后到容器有非零尺寸且首轮响应式风暴结束之后**（`nextTick`/`requestAnimationFrame`
    后再置 `shouldRenderChart`，或在容器宽度 > 0 时才挂），缩小竞态窗口。
- **R3（可选）收敛竞态面**：首屏渲染阶段关闭入场动画或
  `chart.redrawOnParentResize:false / redrawOnWindowResize:false`（尺寸已由 CSS 固定，无需 ApexCharts 自己 redraw）。
- **约束**：改动收口在 `claude-observer/` 共享封装层，避免三处重复；不破坏现有
  `ChartPreparingState` 空态/准备态逻辑；遵守 reduced-motion 降级。

## Acceptance Criteria

- [x] 新增回归测试 `ccr-ui/tests/chart-error-boundary.smoke.test.ts`：
      断言图表封装内部抛错时被错误边界接住（`app.config.errorHandler` 不被调用）、瞬时错误自愈重挂渲染出真内容。
- [x] `bun run type-check` ✓、`bun run lint` ✓（0 error，仅历史 no-raw-text 警告）、
      `bunx vitest run --config vitest.smoke.config.ts` ✓（80 文件 / 357 例全过，含新增 2 例）。
- [ ]（人工目检，需 Tauri dev）打开 `/claude-code`，**不再出现** `应用错误: TypeError ... (reading 'node')` 吐司。
- [ ]（人工目检）「近 30 天每日费用」面积图正常渲染出数据（不再空白容器）。
- [ ]（人工目检）依次切换 费用归因 / Token 详情 / 行为分析 三个 Tab，各自图表都能渲染且不抛错。
- [ ]（人工目检）`/usage` 完整看板无回归（同样用 `vue3-apexcharts`）。

## Notes / 关键文件

- 触发组件：`ccr-ui/src/components/claude-observer/CostAttributionTab.vue:14-23,166-229`
- 共享封装（修复收口点）：`ccr-ui/src/components/claude-observer/apexChart.ts`
- 同封装的另两个 Tab：`TokenDetailTab.vue:67`、`BehaviorAnalysisTab.vue:22`
- 数据/挂载时序：`UsageInsightPanel.vue:169-193,323-362`（`shouldRenderChart` / `renderedTabs` / `fetchAll`）
- 全局错误吐司来源：`ccr-ui/src/main.ts:100-125`
- 依赖版本：`apexcharts@5.6.0`、`vue3-apexcharts@1.10.0`
- 上游同类问题：react-apexcharts
  [#602](https://github.com/apexcharts/react-apexcharts/issues/602)、
  [#598](https://github.com/apexcharts/react-apexcharts/issues/598)（共享核心库）
- 备注：精确的内部触发点与时序相关、难以静态 100% 锁定，因此采用「错误边界兜底（R1）+ 稳定挂载（R2）」防御纵深，
  R1 保证吐司消失，R2 保证图真渲染。
