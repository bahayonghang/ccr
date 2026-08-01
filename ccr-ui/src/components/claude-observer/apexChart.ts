import { defineAsyncComponent, defineComponent, h, type Component } from 'vue'
import ChartErrorBoundary from './ChartErrorBoundary.vue'
import ChartPreparingState from './ChartPreparingState.vue'

/**
 * 共享的 apexcharts 异步组件封装。
 *
 * 收口三个 claude-observer Tab 中逐字重复的 defineAsyncComponent 样板；
 * `as unknown as Component` 断言只在此处保留一次
 * （vue3-apexcharts 默认导出类型与 Vue Component 不严格匹配）。
 *
 * 外层再套一层 ChartErrorBoundary：ApexCharts 在「异步 init + 动画」窗口里可能解引用
 * 已移除的 SVG 元素抛 `reading 'node'`，边界负责接住、自愈重挂并阻断冒泡到全局 errorHandler。
 */
const RealApexChart = defineAsyncComponent({
  loader: async () => {
    const module = await import('@/utils/apexChartsCore')
    return module.default as unknown as Component
  },
  loadingComponent: {
    name: 'ClaudeObserverChartPreparingState',
    setup() {
      return () => h(ChartPreparingState)
    },
  },
  suspensible: false,
})

// 三个 Tab 以 `<component :is="apexchart" type=... :options=... :series=...>` 传任意属性，
// 故对外仍以宽松的 `Component` 暴露（与 vue3-apexcharts 原始导出一致），属性统一经 attrs 透传。
export const ApexChartAsync = defineComponent({
  name: 'ClaudeObserverApexChart',
  // 不把外部属性挂到边界根上，统一透传给真正的图表组件
  inheritAttrs: false,
  setup(_, { attrs }) {
    return () =>
      h(ChartErrorBoundary, null, {
        // reloadKey 自增即触发图表「干净重挂」，配合错误边界自愈
        default: ({ reloadKey }: { reloadKey: number }) =>
          h(RealApexChart, { key: reloadKey, ...attrs }),
      })
  },
}) as unknown as Component
