import { defineAsyncComponent, h, type Component } from 'vue'
import ChartPreparingState from './ChartPreparingState.vue'

/**
 * 共享的 apexcharts 异步组件封装。
 *
 * 收口三个 claude-observer Tab 中逐字重复的 defineAsyncComponent 样板；
 * `as unknown as Component` 断言只在此处保留一次
 * （vue3-apexcharts 默认导出类型与 Vue Component 不严格匹配）。
 */
export const ApexChartAsync = defineAsyncComponent({
  loader: async () => {
    const module = await import('vue3-apexcharts')
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
