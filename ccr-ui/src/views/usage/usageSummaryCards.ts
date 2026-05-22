import type { UsageSummary } from '@/types/usage'
import type { UsageTrendBucket } from './usageDashboardPresentation'

export type UsageSummaryCardTone = 'rose' | 'sand' | 'sky' | 'amber'
export type UsageSummaryCardDeltaTone = 'up' | 'down' | 'flat'
export type UsageSummaryCardMetric = 'tokens' | 'cost' | 'activeDays' | 'requests'

export type UsageSparklinePoint = {
  label: string
  value: number
}

export type UsageSummarySparklinePoints = Record<UsageSummaryCardMetric, UsageSparklinePoint[]>

export type UsageSummaryCard = {
  id: UsageSummaryCardMetric
  label: string
  value: string
  detail: string
  icon: string
  tone: UsageSummaryCardTone
  sparkline: UsageSparklinePoint[]
  sparklineLabel: string
  averageLabel: string
  peakLabel: string
  deltaLabel: string
  deltaTone: UsageSummaryCardDeltaTone
}

type UsageSummaryCardDraft = Omit<
  UsageSummaryCard,
  'sparkline' | 'sparklineLabel' | 'averageLabel' | 'peakLabel' | 'deltaLabel' | 'deltaTone'
>

type UsageDashboardTranslator = (
  key: string,
  values: Record<string, number | string> | undefined,
  fallback: string,
) => string

export type BuildUsageSummaryCardsInput = {
  summary: UsageSummary
  activeDays: number
  modelCount: number
  projectCount: number
  sparklinePoints: UsageSummarySparklinePoints
  selectedWindowLabel: string
  translate: UsageDashboardTranslator
}

export const formatTokens = (value: number) =>
  value >= 1e6
    ? `${(value / 1e6).toFixed(1)}M`
    : value >= 1e3
      ? `${(value / 1e3).toFixed(1)}K`
      : value.toString()

export const formatCost = (value: number) =>
  value >= 1 ? `$${value.toFixed(2)}` : `$${value.toFixed(4)}`

export const formatPercent = (value: number) => `${(value * 100).toFixed(1)}%`

const formatCompactCount = (value: number) =>
  value >= 1e6
    ? `${(value / 1e6).toFixed(1)}M`
    : value >= 1e3
      ? `${(value / 1e3).toFixed(1)}K`
      : Number.isInteger(value)
        ? value.toLocaleString()
        : value.toFixed(1)

export const buildSummarySparklinePoints = (
  buckets: UsageTrendBucket[],
): UsageSummarySparklinePoints => ({
  requests: buckets.map((item) => ({ label: item.startDate, value: item.requestCount })),
  tokens: buckets.map((item) => ({ label: item.startDate, value: item.totalTokens })),
  cost: buckets.map((item) => ({ label: item.startDate, value: item.costUsd })),
  activeDays: buckets.map((item) => ({
    label: item.startDate,
    value: item.totalTokens > 0 ? 1 : 0,
  })),
})

export const buildMetricStats = (
  points: UsageSparklinePoint[],
  formatter: (value: number) => string,
) => {
  if (points.length === 0) {
    return {
      averageLabel: formatter(0),
      peakLabel: formatter(0),
      deltaLabel: '0%',
      deltaTone: 'flat' as UsageSummaryCardDeltaTone,
    }
  }

  const values = points.map((point) => point.value)
  const average = values.reduce((sum, value) => sum + value, 0) / values.length
  const peak = Math.max(...values)
  const first = values[0] ?? 0
  const last = values.at(-1) ?? 0
  const delta = first > 0 ? (last - first) / first : last > 0 ? 1 : 0
  const deltaTone: UsageSummaryCardDeltaTone =
    delta > 0.005 ? 'up' : delta < -0.005 ? 'down' : 'flat'
  const sign = delta > 0 ? '+' : ''

  return {
    averageLabel: formatter(average),
    peakLabel: formatter(peak),
    deltaLabel: `${sign}${(delta * 100).toFixed(0)}%`,
    deltaTone,
  }
}

const buildSummaryCard = (
  card: UsageSummaryCardDraft,
  sparklinePoints: UsageSummarySparklinePoints,
  selectedWindowLabel: string,
  translate: UsageDashboardTranslator,
  formatter: (value: number) => string,
): UsageSummaryCard => ({
  ...card,
  sparkline: sparklinePoints[card.id],
  sparklineLabel: translate(
    'usage.dashboard.cards.sparklineLabel',
    { metric: card.label, window: selectedWindowLabel },
    `${card.label} trend for ${selectedWindowLabel}`,
  ),
  ...buildMetricStats(sparklinePoints[card.id], formatter),
})

export const buildUsageSummaryCards = ({
  summary,
  activeDays,
  modelCount,
  projectCount,
  sparklinePoints,
  selectedWindowLabel,
  translate,
}: BuildUsageSummaryCardsInput): UsageSummaryCard[] => {
  const averageCostPerRequest =
    summary.total_requests > 0
      ? formatCost(summary.total_cost_usd / summary.total_requests)
      : formatCost(0)

  return [
    buildSummaryCard(
      {
        id: 'tokens',
        label: translate('usage.dashboard.cards.totalTokens', undefined, 'Total Tokens'),
        value: formatTokens(summary.total_tokens),
        detail: translate(
          'usage.dashboard.cards.tokensDetail',
          {
            input: formatTokens(summary.total_input_tokens),
            output: formatTokens(summary.total_output_tokens),
            cache: formatTokens(summary.total_cache_read_tokens),
          },
          `${formatTokens(summary.total_input_tokens)} in · ${formatTokens(summary.total_output_tokens)} out · ${formatTokens(summary.total_cache_read_tokens)} cache read`,
        ),
        icon: 'Layers',
        tone: 'rose',
      },
      sparklinePoints,
      selectedWindowLabel,
      translate,
      formatTokens,
    ),
    buildSummaryCard(
      {
        id: 'cost',
        label: translate('usage.dashboard.cards.totalCost', undefined, 'Total Cost'),
        value: formatCost(summary.total_cost_usd),
        detail: translate(
          'usage.dashboard.cards.costDetail',
          {
            average: averageCostPerRequest,
          },
          `${averageCostPerRequest} per request`,
        ),
        icon: 'Wallet',
        tone: 'sand',
      },
      sparklinePoints,
      selectedWindowLabel,
      translate,
      formatCost,
    ),
    buildSummaryCard(
      {
        id: 'activeDays',
        label: translate('usage.dashboard.cards.activeDays', undefined, 'Active Days'),
        value: activeDays.toLocaleString(),
        detail: translate(
          'usage.dashboard.cards.activeDaysDetail',
          {
            window: selectedWindowLabel,
          },
          `Days with token activity in ${selectedWindowLabel}`,
        ),
        icon: 'Calendar',
        tone: 'sky',
      },
      sparklinePoints,
      selectedWindowLabel,
      translate,
      formatCompactCount,
    ),
    buildSummaryCard(
      {
        id: 'requests',
        label: translate('usage.dashboard.cards.totalRequests', undefined, 'Total Requests'),
        value: summary.total_requests.toLocaleString(),
        detail: translate(
          'usage.dashboard.cards.requestsDetail',
          {
            models: modelCount,
            projects: projectCount,
          },
          `${modelCount} models · ${projectCount} projects`,
        ),
        icon: 'Activity',
        tone: 'amber',
      },
      sparklinePoints,
      selectedWindowLabel,
      translate,
      formatCompactCount,
    ),
  ]
}
