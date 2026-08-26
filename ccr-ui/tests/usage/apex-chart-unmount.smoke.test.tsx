import { render, waitFor } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

const apexHost = vi.hoisted(() => ({
  destroy: vi.fn(),
  resize: vi.fn(),
}))

vi.mock('@/utils/apexChartsCore', async () => {
  const React = await import('react')
  function MockReactApexChart({
    chartRef,
  }: {
    chartRef?: { current: { destroy: () => void; resize: () => void } | null }
  }) {
    React.useLayoutEffect(() => {
      if (!chartRef) return
      chartRef.current = { destroy: apexHost.destroy, resize: apexHost.resize }
    }, [chartRef])
    return React.createElement('div', { 'data-testid': 'apex-chart-host' })
  }
  return { default: MockReactApexChart }
})

import { ApexChart } from '@/features/usage/charts/ApexChart'

describe('ApexChart unmount cleanup', () => {
  it('destroys the chart instance and removes the window resize listener', async () => {
    apexHost.destroy.mockClear()
    apexHost.resize.mockClear()
    const added = new Set<EventListenerOrEventListenerObject>()
    const addSpy = vi.spyOn(window, 'addEventListener')
    const removeSpy = vi.spyOn(window, 'removeEventListener')

    const view = render(
      <ApexChart type="area" height={200} series={[{ data: [1, 2] }]} options={{ chart: { type: 'area' } }} />,
    )
    await waitFor(() => {
      expect(view.getByTestId('apex-chart-host')).toBeTruthy()
    })

    for (const call of addSpy.mock.calls) {
      if (call[0] === 'resize' && typeof call[1] === 'function') added.add(call[1])
    }
    expect(added.size).toBeGreaterThan(0)

    view.unmount()
    expect(apexHost.destroy).toHaveBeenCalled()
    for (const handler of added) {
      expect(removeSpy).toHaveBeenCalledWith('resize', handler)
    }

    addSpy.mockRestore()
    removeSpy.mockRestore()
  })
})
