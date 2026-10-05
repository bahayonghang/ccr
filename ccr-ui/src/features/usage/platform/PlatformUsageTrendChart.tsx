import { useMemo } from 'react'
import type { ApexOptions } from 'apexcharts'
import type { DailyTrend } from '@/types/usage'
import type { PlatformUsageMetric } from '@/types/platformUsageInsight'
import {
  buildChartAnimations,
  buildChartTheme,
  formatTrendAxisLabel,
  getTrendTickAmount,
} from '@/views/usage/usageChartOptions'
import { buildPlatformUsageTrendSeries } from '@/views/platform-usage/platformUsageTrendChart'
import { formatCost, formatTokens } from '@/views/usage/usageSummaryCards'
import { ApexChart } from '../charts/ApexChart'
import '../styles/platform-usage-trend-chart.css'

interface PlatformUsageTrendChartProps {
  title: string
  eyebrow: string
  windowLabel: string
  emptyLabel: string
  metric: PlatformUsageMetric
  trends: DailyTrend[]
}

export function PlatformUsageTrendChart({
  title,
  eyebrow,
  windowLabel,
  emptyLabel,
  metric,
  trends,
}: PlatformUsageTrendChartProps) {
  const theme = buildChartTheme()
  const chartType = metric === 'tokens' ? 'bar' : metric === 'requests' ? 'line' : 'area'
  const canRenderApex = typeof navigator === 'undefined' || !/jsdom/i.test(navigator.userAgent)

  const series = useMemo(() => {
    const next = buildPlatformUsageTrendSeries(trends, metric)
    return next
  }, [metric, trends])

  const chartOptions = useMemo<ApexOptions>(() => {
    const formatAxisValue = (value: number) => {
      if (metric === 'cost') return formatCost(value)
      if (metric === 'tokens') return formatTokens(value)
      return Math.round(value).toLocaleString()
    }
    return {
      chart: {
        id: `platform-usage-${metric}`,
        toolbar: { show: false },
        animations: buildChartAnimations(),
        fontFamily: 'var(--font-sans)',
        background: 'transparent',
        stacked: metric === 'tokens',
        redrawOnParentResize: false,
        redrawOnWindowResize: false,
      },
      colors: [theme.primary, theme.secondary, theme.tertiary, theme.quaternary],
      dataLabels: { enabled: false },
      xaxis: {
        type: 'datetime',
        labels: {
          trim: false,
          datetimeUTC: true,
          formatter: (value: string, timestamp?: number) =>
            formatTrendAxisLabel(timestamp ?? Number(value), 'day', document.documentElement.lang || 'zh-CN'),
        },
        tickAmount: getTrendTickAmount(trends.length),
      },
      yaxis: {
        labels: { formatter: (value: number) => formatAxisValue(value) },
      },
    }
  }, [metric, theme, trends.length])

  return (
    <article className="platform-usage-chart">
      <div className="platform-usage-chart__head">
        <div>
          <p>{eyebrow}</p>
          <h3>{title}</h3>
        </div>
        <span>{windowLabel}</span>
      </div>
      {!trends.length ? (
        <div className="platform-usage-chart__empty">{emptyLabel}</div>
      ) : canRenderApex ? (
        <ApexChart
          className="platform-usage-chart__apex"
          type={chartType}
          height={286}
          options={chartOptions}
          series={series}
        />
      ) : (
        <div className="platform-usage-chart__fallback" aria-hidden="true" />
      )}
    </article>
  )
}
