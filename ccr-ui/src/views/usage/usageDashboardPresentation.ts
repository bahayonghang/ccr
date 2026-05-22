import type { DailyTrend, ModelStat, ProjectStat, UsageSummary } from '@/types/usage'
import {
  buildOverviewHighlights,
  buildTopModelRankings,
  buildTopProjectRankings,
  type OverviewHighlightItem,
  type OverviewRankItem,
  type UsageDashboardTranslate,
} from './usageOverviewInsights'
import {
  buildSummarySparklinePoints,
  buildUsageSummaryCards,
  type UsageSummaryCard,
  type UsageSummarySparklinePoints,
} from './usageSummaryCards'

export type TrendGranularity = 'day' | 'week' | 'month'

export interface UsageTrendBucket {
  id: string
  startDate: string
  endDate: string
  requestCount: number
  inputTokens: number
  outputTokens: number
  reasoningOutputTokens: number
  totalTokens: number
  cacheReadTokens: number
  cacheCreationTokens: number
  costUsd: number
}

export interface ModelDistributionSlice {
  id: string
  label: string
  totalCost: number
  totalTokens: number
  requestCount: number
  share: number
  childCount: number
  isOther: boolean
}

const DAY_MS = 86_400_000
const modelCost = (model: ModelStat) => model.cost_with_cache ?? 0

type ModelDistributionMetric = 'cost' | 'tokens'

const parseUtcDate = (value: string) => {
  const [year, month, day] = value.split('-').map(Number)
  return new Date(Date.UTC(year, (month || 1) - 1, day || 1))
}

const formatUtcDate = (date: Date) => date.toISOString().slice(0, 10)

const startOfWeekUtc = (date: Date) => {
  const normalized = new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate()))
  const day = normalized.getUTCDay()
  const diff = day === 0 ? -6 : 1 - day
  normalized.setUTCDate(normalized.getUTCDate() + diff)
  return normalized
}

const startOfMonthUtc = (date: Date) =>
  new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), 1))

const toBucketKey = (date: string, granularity: TrendGranularity) => {
  const parsed = parseUtcDate(date)

  if (granularity === 'month') {
    return formatUtcDate(startOfMonthUtc(parsed))
  }

  if (granularity === 'week') {
    return formatUtcDate(startOfWeekUtc(parsed))
  }

  return formatUtcDate(parsed)
}

export const selectTrendGranularity = (spanDays: number): TrendGranularity => {
  if (spanDays >= 365) return 'month'
  if (spanDays >= 90) return 'week'
  return 'day'
}

export const aggregateDailyTrends = (
  trends: DailyTrend[],
  granularity: TrendGranularity,
): UsageTrendBucket[] => {
  const buckets = new Map<string, UsageTrendBucket>()

  for (const item of trends) {
    const key = toBucketKey(item.date, granularity)
    const existing = buckets.get(key)

    if (existing) {
      existing.endDate = item.date > existing.endDate ? item.date : existing.endDate
      existing.requestCount += item.request_count
      existing.totalTokens += item.total_tokens
      existing.inputTokens += item.input_tokens
      existing.outputTokens += item.output_tokens
      existing.reasoningOutputTokens += item.reasoning_output_tokens
      existing.cacheReadTokens += item.cache_read_tokens
      existing.cacheCreationTokens += item.cache_creation_tokens
      existing.costUsd += item.cost_usd
      continue
    }

    buckets.set(key, {
      id: key,
      startDate: key,
      endDate: item.date,
      requestCount: item.request_count,
      totalTokens: item.total_tokens,
      inputTokens: item.input_tokens,
      outputTokens: item.output_tokens,
      reasoningOutputTokens: item.reasoning_output_tokens,
      cacheReadTokens: item.cache_read_tokens,
      cacheCreationTokens: item.cache_creation_tokens,
      costUsd: item.cost_usd,
    })
  }

  return Array.from(buckets.values()).sort((left, right) => left.startDate.localeCompare(right.startDate))
}

export const expandTrendBucketEnd = (bucket: UsageTrendBucket, granularity: TrendGranularity) => {
  if (bucket.endDate !== bucket.startDate) return bucket.endDate

  const start = parseUtcDate(bucket.startDate)

  if (granularity === 'day') {
    return bucket.startDate
  }

  if (granularity === 'week') {
    return formatUtcDate(new Date(start.getTime() + DAY_MS * 6))
  }

  return formatUtcDate(new Date(Date.UTC(start.getUTCFullYear(), start.getUTCMonth() + 1, 0)))
}

const groupModelsByMetric = (
  modelStats: ModelStat[],
  maxVisible = 6,
  metric: ModelDistributionMetric,
): ModelDistributionSlice[] => {
  const sorted = [...modelStats].sort((left, right) => {
    if (metric === 'tokens' && right.total_tokens !== left.total_tokens) {
      return right.total_tokens - left.total_tokens
    }

    if (metric === 'cost' && modelCost(right) !== modelCost(left)) {
      return modelCost(right) - modelCost(left)
    }

    if (right.total_tokens !== left.total_tokens) {
      return right.total_tokens - left.total_tokens
    }

    if (modelCost(right) !== modelCost(left)) {
      return modelCost(right) - modelCost(left)
    }

    return right.request_count - left.request_count
  })

  const totalCost = sorted.reduce((sum, item) => sum + modelCost(item), 0)
  const totalTokens = sorted.reduce((sum, item) => sum + item.total_tokens, 0)
  const shareDenominator = metric === 'tokens' ? totalTokens : totalCost
  const visible = sorted.slice(0, maxVisible)
  const hidden = sorted.slice(maxVisible)

  const slices = visible.map((item) => ({
    id: item.model,
    label: item.model,
    totalCost: modelCost(item),
    totalTokens: item.total_tokens,
    requestCount: item.request_count,
    share: shareDenominator > 0
      ? (metric === 'tokens' ? item.total_tokens : modelCost(item)) / shareDenominator
      : 0,
    childCount: 1,
    isOther: false,
  }))

  if (hidden.length > 0) {
    const otherCost = hidden.reduce((sum, item) => sum + modelCost(item), 0)
    const otherTokens = hidden.reduce((sum, item) => sum + item.total_tokens, 0)
    const otherRequests = hidden.reduce((sum, item) => sum + item.request_count, 0)

    slices.push({
      id: 'others',
      label: 'Others',
      totalCost: otherCost,
      totalTokens: otherTokens,
      requestCount: otherRequests,
      share: shareDenominator > 0
        ? (metric === 'tokens' ? otherTokens : otherCost) / shareDenominator
        : 0,
      childCount: hidden.length,
      isOther: true,
    })
  }

  return slices
}

export interface DisplayUsageTrendBucket extends UsageTrendBucket {
  displayEndDate: string
}

export interface UsageTrendSeriesItem {
  name: string
  data: Array<{ x: string; y: number }>
}

export interface UsageDashboardPresentation {
  trendBuckets: DisplayUsageTrendBucket[]
  summarySparklinePoints: UsageSummarySparklinePoints
  summaryCards: UsageSummaryCard[]
  trendSeries: UsageTrendSeriesItem[]
  modelDistribution: ModelDistributionSlice[]
  modelTokenDistribution: ModelDistributionSlice[]
  pieSeries: number[]
  modelTokenPieSeries: number[]
  overviewHighlights: OverviewHighlightItem[]
  topModelRankings: OverviewRankItem[]
  topProjectRankings: OverviewRankItem[]
}

export const groupModelDistribution = (
  modelStats: ModelStat[],
  maxVisible = 6,
): ModelDistributionSlice[] => groupModelsByMetric(modelStats, maxVisible, 'cost')

export const groupModelTokenDistribution = (
  modelStats: ModelStat[],
  maxVisible = 6,
): ModelDistributionSlice[] => groupModelsByMetric(modelStats, maxVisible, 'tokens')

export const buildUsageDashboardPresentation = ({
  modelStats,
  projectStats,
  selectedWindowLabel,
  summary,
  translate,
  trendGranularity,
  trendGranularityLabel,
  trends,
}: {
  modelStats: ModelStat[]
  projectStats: ProjectStat[]
  selectedWindowLabel: string
  summary: UsageSummary | null
  translate: UsageDashboardTranslate
  trendGranularity: TrendGranularity
  trendGranularityLabel: string
  trends: DailyTrend[]
}): UsageDashboardPresentation => {
  const trendBuckets = aggregateDailyTrends(trends, trendGranularity).map((bucket) => ({
    ...bucket,
    displayEndDate: expandTrendBucketEnd(bucket, trendGranularity),
  }))
  const activeDays = trends.filter((item) => item.total_tokens > 0).length
  const summarySparklinePoints = buildSummarySparklinePoints(trendBuckets)
  const inputName = translate('usage.dashboard.chart.input', undefined, 'Input')
  const outputName = translate('usage.dashboard.chart.output', undefined, 'Output')
  const cacheName = translate('usage.dashboard.chart.cache', undefined, 'Cache Read')
  const localizeOther = (item: ModelDistributionSlice): ModelDistributionSlice => ({
    ...item,
    label: item.isOther
      ? translate('usage.dashboard.chart.others', undefined, 'Others')
      : item.label,
  })
  const modelDistribution = groupModelDistribution(modelStats, 6).map(localizeOther)
  const modelTokenDistribution = groupModelTokenDistribution(modelStats, 6).map(localizeOther)

  return {
    trendBuckets,
    summarySparklinePoints,
    summaryCards: summary
      ? buildUsageSummaryCards({
          summary,
          activeDays,
          modelCount: modelStats.length,
          projectCount: projectStats.length,
          sparklinePoints: summarySparklinePoints,
          selectedWindowLabel,
          translate,
        })
      : [],
    trendSeries: [
      {
        name: inputName,
        data: trendBuckets.map((item) => ({
          x: `${item.startDate}T00:00:00Z`,
          y: item.inputTokens,
        })),
      },
      {
        name: outputName,
        data: trendBuckets.map((item) => ({
          x: `${item.startDate}T00:00:00Z`,
          y: item.outputTokens,
        })),
      },
      {
        name: cacheName,
        data: trendBuckets.map((item) => ({
          x: `${item.startDate}T00:00:00Z`,
          y: item.cacheReadTokens,
        })),
      },
    ],
    modelDistribution,
    modelTokenDistribution,
    pieSeries: modelDistribution.map((item) => item.totalCost),
    modelTokenPieSeries: modelTokenDistribution.map((item) => item.totalTokens),
    overviewHighlights: buildOverviewHighlights({
      modelStats,
      projectStats,
      summary,
      trendBuckets,
      trendGranularityLabel,
      selectedWindowLabel,
      translate,
    }),
    topModelRankings: buildTopModelRankings(modelStats, translate),
    topProjectRankings: buildTopProjectRankings(projectStats, translate),
  }
}
