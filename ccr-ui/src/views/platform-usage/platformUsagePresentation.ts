import type {
  DailyTrend,
  ModelStat,
  ProjectStat,
  UsageSummary,
} from '@/types/usage'
import type {
  PlatformUsageDashboardData,
  PlatformUsageInsightLabels,
  PlatformUsageInsightPresentation,
  PlatformUsagePricingState,
  PlatformUsageRankRow,
  PlatformUsageTone,
} from '@/types/platformUsageInsight'
import { formatCost, formatTokens } from '@/views/usage/usageSummaryCards'

export const defaultPlatformUsageLabels: PlatformUsageInsightLabels = {
  costCard: 'Window cost',
  tokensCard: 'Token volume',
  requestsCard: 'Requests',
  cost: 'Cost',
  tokens: 'Tokens',
  requests: 'Requests',
  input: 'Input',
  output: 'Output',
  cacheRead: 'Cache read',
  cacheCreation: 'Cache write',
  models: 'Models',
  projects: 'Projects',
  topModel: 'Top model',
  topProject: 'Top project',
  noModel: 'Unknown model',
  noProject: 'Unknown project',
  pricingUnavailable: 'Pricing unavailable',
  tokenOnly: 'Token only',
  requestUnit: 'requests',
  modelUnit: 'models',
  projectUnit: 'projects',
  generatedAt: 'Generated',
}

export const buildPlatformUsageLabels = (
  overrides?: Partial<PlatformUsageInsightLabels>,
): PlatformUsageInsightLabels => ({
  ...defaultPlatformUsageLabels,
  ...overrides,
})

const toFiniteNumber = (value: number | null | undefined) =>
  Number.isFinite(value) ? Number(value) : 0

export const hasPlatformUsageData = (
  summary: UsageSummary | null | undefined,
  trends: DailyTrend[] = [],
  modelStats: ModelStat[] = [],
  projectStats: ProjectStat[] = [],
) => {
  if (!summary) return false

  const summaryHasUsage =
    toFiniteNumber(summary.total_requests) > 0 ||
    toFiniteNumber(summary.total_tokens) > 0 ||
    toFiniteNumber(summary.total_cost_usd) > 0
  const trendsHaveUsage = trends.some((trend) =>
    toFiniteNumber(trend.request_count) > 0 ||
    toFiniteNumber(trend.total_tokens) > 0 ||
    toFiniteNumber(trend.cost_usd) > 0,
  )

  return summaryHasUsage || trendsHaveUsage || modelStats.length > 0 || projectStats.length > 0
}

export const getPlatformUsagePricingState = (
  summary: UsageSummary | null | undefined,
): PlatformUsagePricingState => {
  if (!summary || !hasPlatformUsageData(summary)) return 'empty'
  if (toFiniteNumber(summary.total_cost_usd) <= 0 && toFiniteNumber(summary.total_tokens) > 0) {
    return 'token_only'
  }
  return 'available'
}

const formatNumber = (value: number) =>
  Math.round(toFiniteNumber(value)).toLocaleString()

const buildRankRows = <T extends ModelStat | ProjectStat>(
  items: T[],
  getLabel: (item: T) => string,
  labels: PlatformUsageInsightLabels,
): PlatformUsageRankRow[] => {
  const ranked = items
    .map((item, index) => {
      const cost = toFiniteNumber('total_cost' in item ? item.total_cost : 0)
      const tokens = toFiniteNumber(item.total_tokens)
      const requests = toFiniteNumber(item.request_count)
      const value = cost > 0 ? cost : tokens > 0 ? tokens : requests
      const displayValue = cost > 0 ? formatCost(cost) : tokens > 0 ? formatTokens(tokens) : formatNumber(requests)
      const label = getLabel(item).trim()

      return {
        id: `${label || index}-${index}`,
        label: label || ('model' in item ? labels.noModel : labels.noProject),
        title: label || ('model' in item ? labels.noModel : labels.noProject),
        detail: `${formatNumber(requests)} ${labels.requestUnit} · ${formatTokens(tokens)}`,
        value,
        displayValue,
        share: 0,
      }
    })
    .sort((a, b) => b.value - a.value)
    .slice(0, 8)

  const maxValue = Math.max(...ranked.map((item) => item.value), 0)
  return ranked.map((item) => ({
    ...item,
    share: maxValue > 0 ? Math.max(4, Math.round((item.value / maxValue) * 100)) : 0,
  }))
}

const buildCards = (
  summary: UsageSummary,
  labels: PlatformUsageInsightLabels,
  tone: PlatformUsageTone,
  modelRows: PlatformUsageRankRow[],
  projectRows: PlatformUsageRankRow[],
) => {
  const pricingState = getPlatformUsagePricingState(summary)
  const costValue =
    pricingState === 'token_only'
      ? labels.tokenOnly
      : formatCost(toFiniteNumber(summary.total_cost_usd))
  const averageCost =
    toFiniteNumber(summary.total_requests) > 0
      ? formatCost(toFiniteNumber(summary.total_cost_usd) / toFiniteNumber(summary.total_requests))
      : formatCost(0)

  return [
    {
      id: 'cost' as const,
      label: labels.costCard,
      value: costValue,
      detail:
        pricingState === 'token_only'
          ? labels.pricingUnavailable
          : `${averageCost} / ${labels.requestUnit}`,
      meta: `${formatNumber(summary.total_requests)} ${labels.requestUnit}`,
      icon: 'Wallet',
      tone,
      pricingState,
    },
    {
      id: 'tokens' as const,
      label: labels.tokensCard,
      value: formatTokens(toFiniteNumber(summary.total_tokens)),
      detail: `${labels.input} ${formatTokens(toFiniteNumber(summary.total_input_tokens))} · ${labels.output} ${formatTokens(toFiniteNumber(summary.total_output_tokens))}`,
      meta: `${labels.cacheRead} ${formatTokens(toFiniteNumber(summary.total_cache_read_tokens))}`,
      icon: 'Layers',
      tone,
      pricingState,
    },
    {
      id: 'requests' as const,
      label: labels.requestsCard,
      value: formatNumber(summary.total_requests),
      detail: `${modelRows.length} ${labels.modelUnit} · ${projectRows.length} ${labels.projectUnit}`,
      meta: `${labels.topModel}: ${modelRows[0]?.label ?? labels.noModel}`,
      icon: 'Activity',
      tone,
      pricingState,
    },
  ]
}

export interface BuildPlatformUsageInsightInput {
  data: PlatformUsageDashboardData | null | undefined
  labels?: Partial<PlatformUsageInsightLabels>
  tone?: PlatformUsageTone
}

export const buildPlatformUsageInsight = ({
  data,
  labels: labelOverrides,
  tone = 'neutral',
}: BuildPlatformUsageInsightInput): PlatformUsageInsightPresentation => {
  const labels = buildPlatformUsageLabels(labelOverrides)
  const summary = data?.summary ?? null
  const trends = data?.trends ?? []
  const modelStats = data?.model_stats ?? []
  const projectStats = data?.project_stats ?? []
  const modelRows = buildRankRows(modelStats, (item) => item.model, labels)
  const projectRows = buildRankRows(projectStats, (item) => item.project_path, labels)
  const empty = !hasPlatformUsageData(summary, trends, modelStats, projectStats)
  const pricingState = getPlatformUsagePricingState(summary)

  return {
    summary,
    trends,
    modelStats,
    projectStats,
    cards: summary && !empty ? buildCards(summary, labels, tone, modelRows, projectRows) : [],
    modelRows,
    projectRows,
    topModelLabel: modelRows[0]?.label ?? labels.noModel,
    topProjectLabel: projectRows[0]?.label ?? labels.noProject,
    pricingState,
    empty,
    generatedAt: data?.generated_at ?? null,
  }
}
