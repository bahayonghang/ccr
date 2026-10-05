import { lazy, Suspense, useEffect, useRef } from 'react'
import type { ApexOptions } from 'apexcharts'
import { ChartErrorBoundary } from '@/features/claude/observer/ChartErrorBoundary'
import { ChartPreparingState } from '@/features/claude/observer/ChartPreparingState'
import { createThrottledResize } from '@/utils/chartResize'

const ReactApexChart = lazy(() => import('@/utils/apexChartsCore'))

interface ObserverChartProps {
  type: 'area' | 'bar' | 'heatmap'
  height: number
  options: object
  series: unknown[]
}

function ObserverChartInner({ type, height, options, series }: ObserverChartProps) {
  const chartRef = useRef<ApexCharts | null>(null)

  useEffect(() => {
    const handleResize = createThrottledResize(() => {
      const chart = chartRef.current as { resize?: () => void } | null
      chart?.resize?.()
    })
    window.addEventListener('resize', handleResize)
    return () => {
      handleResize.cancel()
      window.removeEventListener('resize', handleResize)
      const chart = chartRef.current as { destroy?: () => void } | null
      try {
        chart?.destroy?.()
      } catch {
        // react-apexcharts 卸载路径可能已经 destroy
      } finally {
        chartRef.current = null
      }
    }
  }, [])

  return (
    <ReactApexChart
      type={type}
      height={height}
      options={options as ApexOptions}
      series={series as ApexOptions['series']}
      width="100%"
      chartRef={chartRef}
    />
  )
}

/** 观测图：按需加载模块化 ApexCharts，卸载时销毁实例。 */
export function ObserverChart(props: ObserverChartProps) {
  return (
    <ChartErrorBoundary>
      <Suspense fallback={<ChartPreparingState />}>
        <ObserverChartInner {...props} />
      </Suspense>
    </ChartErrorBoundary>
  )
}
