import { lazy, Suspense } from 'react'
import { ChartErrorBoundary } from '@/features/claude/observer/ChartErrorBoundary'
import { ChartPreparingState } from '@/features/claude/observer/ChartPreparingState'
import 'apexcharts/area'
import 'apexcharts/bar'
import 'apexcharts/heatmap'
import 'apexcharts/features/legend'
import 'apexcharts/dist/apexcharts.css'

const LazyApexChart = lazy(() => import('react-apexcharts'))

interface ObserverChartProps {
  type: 'area' | 'bar' | 'heatmap'
  height: number
  options: object
  series: unknown[]
}

/** 观测图：按需加载 react-apexcharts，错误边界自愈。 */
export function ObserverChart({ type, height, options, series }: ObserverChartProps) {
  return (
    <ChartErrorBoundary>
      <Suspense fallback={<ChartPreparingState />}>
        <LazyApexChart type={type} height={height} options={options} series={series} width="100%" />
      </Suspense>
    </ChartErrorBoundary>
  )
}
