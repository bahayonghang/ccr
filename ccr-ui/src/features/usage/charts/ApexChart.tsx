import { lazy, Suspense, useEffect, useRef } from 'react'
import type { ApexOptions } from 'apexcharts'
import { ChartErrorBoundary } from './ChartErrorBoundary'
import { createThrottledResize } from '@/utils/chartResize'

const ReactApexChart = lazy(() => import('@/utils/apexChartsCore'))

export type UsageApexChartType = NonNullable<ApexOptions['chart']>['type']

export interface UsageApexChartProps {
  type: UsageApexChartType
  series: ApexOptions['series']
  options: ApexOptions | Record<string, unknown>
  width?: string | number
  height?: string | number
  className?: string
}

function ApexChartInner({
  type,
  series,
  options,
  width = '100%',
  height,
  className,
}: UsageApexChartProps) {
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
      series={series}
      options={options as ApexOptions}
      width={width}
      height={height}
      className={className}
      chartRef={chartRef}
    />
  )
}

/** 懒加载 react-apexcharts/core，并包图表级错误边界。 */
export function ApexChart(props: UsageApexChartProps) {
  return (
    <ChartErrorBoundary>
      <Suspense fallback={null}>
        <ApexChartInner {...props} />
      </Suspense>
    </ChartErrorBoundary>
  )
}
