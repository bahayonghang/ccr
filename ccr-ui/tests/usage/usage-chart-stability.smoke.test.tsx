import { describe, expect, it, vi } from 'vitest'
import { createChartController } from '@/features/usage/charts/chartController'
import { createThrottledResize } from '@/utils/chartResize'

describe('usage chart stability', () => {
  it('updates series without reconstructing the chart', async () => {
    const destroy = vi.fn()
    const updateOptions = vi.fn()
    const updateSeries = vi.fn()
    const renderChart = vi.fn().mockResolvedValue(undefined)
    const ApexCtor = vi.fn().mockImplementation(function ApexMock(this: {
      render: typeof renderChart
      updateOptions: typeof updateOptions
      updateSeries: typeof updateSeries
      destroy: typeof destroy
    }) {
      this.render = renderChart
      this.updateOptions = updateOptions
      this.updateSeries = updateSeries
      this.destroy = destroy
    })
    const controller = createChartController(ApexCtor)
    const el = document.createElement('div')
    await controller.mount(el, { series: [{ data: [1] }] })
    expect(ApexCtor).toHaveBeenCalledTimes(1)
    controller.updateSeries([{ data: [2] }])
    expect(ApexCtor).toHaveBeenCalledTimes(1)
    expect(updateSeries).toHaveBeenCalledTimes(1)
    controller.destroy()
    expect(destroy).toHaveBeenCalledTimes(1)
  })

  it('theme option updates go through updateOptions', async () => {
    const updateOptions = vi.fn()
    const ApexCtor = vi.fn().mockImplementation(function ApexMock(this: {
      render: () => Promise<void>
      updateOptions: typeof updateOptions
      updateSeries: () => void
      destroy: () => void
    }) {
      this.render = vi.fn().mockResolvedValue(undefined)
      this.updateOptions = updateOptions
      this.updateSeries = vi.fn()
      this.destroy = vi.fn()
    })
    const controller = createChartController(ApexCtor)
    await controller.mount(document.createElement('div'), { theme: { mode: 'light' } })
    controller.updateOptions({ theme: { mode: 'dark' } })
    expect(ApexCtor).toHaveBeenCalledTimes(1)
    expect(updateOptions).toHaveBeenCalledTimes(1)
  })

  it('throttles resize callbacks', () => {
    vi.useFakeTimers()
    const onResize = vi.fn()
    const throttled = createThrottledResize(onResize, 150)
    throttled()
    throttled()
    throttled()
    expect(onResize).toHaveBeenCalledTimes(1)
    vi.advanceTimersByTime(150)
    expect(onResize.mock.calls.length).toBeLessThanOrEqual(2)
    vi.useRealTimers()
  })

  it('cancels a pending resize timer so onResize does not run after cancel', () => {
    vi.useFakeTimers()
    const onResize = vi.fn()
    const throttled = createThrottledResize(onResize, 150)
    throttled()
    const callsAfterStart = onResize.mock.calls.length
    expect(callsAfterStart).toBeGreaterThanOrEqual(1)
    throttled()
    throttled.cancel()
    vi.advanceTimersByTime(150)
    expect(onResize).toHaveBeenCalledTimes(callsAfterStart)
    vi.useRealTimers()
  })
})
