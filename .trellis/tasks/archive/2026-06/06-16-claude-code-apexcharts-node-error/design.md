# 技术设计 · ApexCharts node 报错

## 决策摘要

在共享封装 `apexChart.ts` 外面套一层**自愈式错误边界** `ChartErrorBoundary.vue`：
用 `onErrorCaptured` 接住 ApexCharts 在异步 init/动画窗口里抛出的 `reading 'node'` 异常，
**有限次「干净重挂」**（换 `:key`）让图表在易变窗口过去后重新渲染；超过重试上限则降级为
`ChartPreparingState`。`onErrorCaptured` 返回 `false` 阻断异常向上冒泡，从而不再触达
`main.ts` 的全局 `errorHandler`，消除“应用错误”吐司。

三个 Tab（Cost/Token/Behavior）共用 `ApexChartAsync`，故修复只改封装层，三 Tab 文件不动。

## 为什么 `onErrorCaptured` 接得住

- 吐司文案 `应用错误:` 只在 `main.ts:121` 由 `app.config.errorHandler` 产出。
- 能走到 `app.config.errorHandler`，说明异常是在 **Vue 托管路径**（子组件的 watcher /
  生命周期钩子）里**同步抛出**的 —— 正是 `vue3-apexcharts` 封装在 `options/series` watcher 里
  调用 `updateOptions/updateSeries`、或 `onMounted` 里 `render()` 的同步段抛出。
- Vue 的错误传播顺序：子组件抛错 → 逐级 `errorCaptured`（就近到根）→ 全局 `errorHandler`。
  因此在子图表外层组件的 `onErrorCaptured` 必然先于全局 handler 接到；`return false` 即终止传播。
- 反证：若异常来自 ApexCharts 内部裸 `requestAnimationFrame` 回调（非 Vue 托管），则它根本
  不会进 `app.config.errorHandler`，而会是 `window.onerror`。但现状是走了全局 handler，
  故确认为 Vue 托管的同步抛出，`onErrorCaptured` 适用。

## 自愈机制（关键时序）

```
子图表 mount → ApexCharts 在易变窗口内抛 'node'
  → 边界 onErrorCaptured 命中
     ├─ retries < max:  degraded=true（先显示准备态，移除坏子树）
     │                  → 下一帧 rAF: reloadKey++ + degraded=false → 干净重挂
     │                  （易变窗口已过，重挂成功 → 正常出图）
     └─ retries >= max: degraded=true（永久降级为准备态，不再重试）
  → return false（阻断冒泡，无全局吐司）
```

- 重挂延迟用 `requestAnimationFrame`（等布局/动画稳定），无 rAF 环境退化到 `setTimeout(0)`。
- 重试上限默认 2（即最多 1 次首挂 + 2 次重挂）。现实中竞态非确定性，通常 1 次重挂即成功。
- 用 `:key=reloadKey` 强制 Vue 卸载旧实例、全新挂载，避免在半成品 ApexCharts 实例上原地 update。

## 组件契约

### `ChartErrorBoundary.vue`（新增）

- Props：`label?: string`（降级态文案，透传给 `ChartPreparingState`）、`maxRetries?: number = 2`。
- Slot：默认作用域插槽，暴露 `{ reloadKey: number }` 供外部当作 `:key`。
- 行为：`onErrorCaptured` 接住 → 自愈/降级 → `return false`。
- 无新增 i18n key（降级态复用 `ChartPreparingState`，其 label 缺省回落到 `observer.chart.preparing`）。

### `apexChart.ts`（改写）

- `ApexChartAsync` 由「直接的 `defineAsyncComponent`」改为「`defineComponent` 组合」：
  `inheritAttrs:false`，渲染 `ChartErrorBoundary` + 作用域插槽里的异步真图，
  `h(RealApexChart, { key: reloadKey, ...attrs })` 透传全部 `type/height/class/options/series`。
- 异步加载真图（`vue3-apexcharts`）+ `loadingComponent: ChartPreparingState` 行为保持不变。
- 三个 Tab 的 `:is="apexchart"` 用法与传参零改动。

## 备选方案与取舍

| 方案                                           | 取舍                                                                |
| ---------------------------------------------- | ------------------------------------------------------------------- |
| 仅加 `:key` 稳定重挂、不加边界                 | 不能保证消除吐司；无法覆盖首挂即抛的场景 → 否                       |
| 关 `animations` / `redrawOnParentResize`（R3） | 只降低概率、非根治；要改三处 options，面更大 → 暂不做，记为可选后续 |
| 升级 / 降级 apexcharts 版本                    | 上游修复状态不明，回归面大、不可控 → 否                             |
| **自愈错误边界（选定）**                       | 集中封装层、对竞态鲁棒、同时消吐司+保出图、改动面最小 → 采纳        |

## 影响面与回归

- 改动文件：`apexChart.ts`（改写）、新增 `ChartErrorBoundary.vue`、新增/扩展 smoke 测试。
- 三 Tab、`UsageInsightPanel`、`/usage` 完整看板均不改源码；`/usage` 同样使用 `vue3-apexcharts`
  但走自己的 `defineAsyncComponent`，不受影响（本次只动 claude-observer 的封装）。
- reduced-motion：`ChartPreparingState` 已自带 `prefers-reduced-motion` 降级，沿用。
