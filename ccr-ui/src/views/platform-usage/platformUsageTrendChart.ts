// 平台首页趋势图：把 DailyTrend.date（YYYY-MM-DD）转成 UTC 午夜时间戳。
// ApexCharts category + trim 会按全部点数分槽，30 天 ISO 日期会被裁成 2026-07...。
import type { DailyTrend } from '@/types/usage'
import type { PlatformUsageMetric } from '@/types/platformUsageInsight'
import { parseUtcDate } from '@/views/usage/usageChartOptions'

export type PlatformUsageTrendPoint = {
  x: number
  y: number
}

export type PlatformUsageTrendSeries = {
  name: string
  data: PlatformUsageTrendPoint[]
}

const toPoints = (
  trends: DailyTrend[],
  pick: (trend: DailyTrend) => number,
): PlatformUsageTrendPoint[] =>
  trends.map((trend) => ({
    x: parseUtcDate(trend.date).getTime(),
    y: pick(trend),
  }))

export const buildPlatformUsageTrendSeries = (
  trends: DailyTrend[],
  metric: PlatformUsageMetric,
): PlatformUsageTrendSeries[] => {
  if (metric === 'tokens') {
    return [
      { name: 'Input', data: toPoints(trends, (trend) => trend.input_tokens) },
      { name: 'Output', data: toPoints(trends, (trend) => trend.output_tokens) },
      { name: 'Cache read', data: toPoints(trends, (trend) => trend.cache_read_tokens) },
      { name: 'Cache write', data: toPoints(trends, (trend) => trend.cache_creation_tokens) },
    ]
  }

  if (metric === 'requests') {
    return [
      { name: 'Requests', data: toPoints(trends, (trend) => trend.request_count) },
    ]
  }

  return [
    { name: 'Cost', data: toPoints(trends, (trend) => trend.cost_usd) },
  ]
}

export const platformUsageTrendSeriesKey = (series: PlatformUsageTrendSeries[]) =>
  series
    .map((item) => `${item.name}:${item.data.map((point) => `${point.x}=${point.y}`).join(',')}`)
    .join('\n')
