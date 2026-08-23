export interface ChartInstance {
  render: () => Promise<unknown> | unknown
  updateOptions: (options: unknown) => void
  updateSeries: (series: unknown) => void
  destroy: () => void
}

export type ChartCtor = new (el: Element, config: unknown) => ChartInstance

/** 图表实例控制器：构造一次，数据/主题走 update，卸载 destroy。 */
export function createChartController(Chart: ChartCtor) {
  let instance: ChartInstance | null = null

  return {
    async mount(el: Element, config: unknown) {
      instance = new Chart(el, config)
      await instance.render()
      return instance
    },
    updateSeries(series: unknown) {
      instance?.updateSeries(series)
    },
    updateOptions(options: unknown) {
      instance?.updateOptions(options)
    },
    destroy() {
      instance?.destroy()
      instance = null
    },
    get instance() {
      return instance
    },
  }
}
