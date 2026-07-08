import type { ModelStat, ProjectStat, UsageArchiveDiagnostics, UsageSummary } from '@/types/usage'
import type { UsageTrendBucket } from './usageDashboardPresentation'
import type { UsageRangePreset } from './dateWindow'
import { formatCost, formatPercent, formatTokens } from './usageSummaryCards'

export type UsageDashboardTranslate = (
  key: string,
  values: Record<string, number | string> | undefined,
  fallback: string
) => string

export type DashboardMetaItem = {
  id: string
  label: string
  value: string
}

export type OverviewHighlightItem = {
  id: string
  label: string
  value: string
  detail: string
}

export type OverviewRankItem = {
  id: string
  label: string
  title: string
  detail: string
  value: string
  share: number
}

export const modelCost = (model: ModelStat) => model.cost_with_cache ?? 0

export const shortenPath = (path: string) => {
  const parts = path.replace(/\\/g, '/').split('/')
  return parts.length > 2 ? `.../${parts.slice(-2).join('/')}` : path
}

export const formatDateTime = (value: string, locale: string) =>
  new Date(value).toLocaleString(locale)

export const formatArchiveTimestamp = (value: string | null | undefined, locale: string) => {
  if (!value) return '—'
  return formatDateTime(value, locale)
}

// 人话化来源计数：替代开发者速记 "L 2,053 · M 2,829 · D 0"，主层与抽屉统一走这个格式。
export const formatSourceCounts = (
  live: number,
  missing: number,
  deleted: number,
  translate: UsageDashboardTranslate
) =>
  [
    `${translate('usage.dashboard.ops.health.liveLabel', undefined, 'Live')} ${live.toLocaleString()}`,
    `${translate('usage.dashboard.ops.health.missingLabel', undefined, 'Missing')} ${missing.toLocaleString()}`,
    `${translate('usage.dashboard.ops.health.deletedLabel', undefined, 'Deleted')} ${deleted.toLocaleString()}`,
  ].join(' · ')

export const buildSelectedPlatformLabel = (
  selectedPlatform: string,
  translate: UsageDashboardTranslate
) => {
  if (!selectedPlatform) {
    return translate('usage.dashboard.allPlatforms', undefined, 'All Platforms')
  }

  const fallbackLabels: Record<string, string> = {
    claude: 'Claude',
    codex: 'Codex',
    gemini: 'Gemini',
    opencode: 'OpenCode',
  }

  return translate(
    `usage.platforms.${selectedPlatform}`,
    undefined,
    fallbackLabels[selectedPlatform] ?? selectedPlatform
  )
}

export const buildSelectedWindowLabel = (
  selectedRange: UsageRangePreset,
  translate: UsageDashboardTranslate
) => {
  const labels: Record<UsageRangePreset, { key: string; fallback: string }> = {
    today: { key: 'usage.dashboard.range.today', fallback: 'Today' },
    this_week: { key: 'usage.dashboard.range.thisWeek', fallback: 'This Week' },
    this_month: { key: 'usage.dashboard.range.thisMonth', fallback: 'This Month' },
    last_30d: { key: 'usage.dashboard.range.last30', fallback: 'Last 30 Days' },
    all_time: { key: 'usage.dashboard.range.allTime', fallback: 'All Time' },
  }

  const selected = labels[selectedRange]
  if (!selected) return selectedRange

  return translate(selected.key, undefined, selected.fallback)
}

export const buildDashboardMetaItems = ({
  archive,
  locale,
  modelCount,
  projectCount,
  selectedPlatformLabel,
  selectedWindowLabel,
  translate,
}: {
  archive: UsageArchiveDiagnostics | null
  locale: string
  modelCount: number
  projectCount: number
  selectedPlatformLabel: string
  selectedWindowLabel: string
  translate: UsageDashboardTranslate
}): DashboardMetaItem[] => [
  {
    id: 'scope',
    label: translate('usage.dashboard.meta.scope', undefined, 'Scope'),
    value: selectedPlatformLabel,
  },
  {
    id: 'window',
    label: translate('usage.dashboard.meta.window', undefined, 'Window'),
    value: selectedWindowLabel,
  },
  {
    id: 'models',
    label: translate('usage.dashboard.meta.models', undefined, 'Models'),
    value: modelCount.toLocaleString(),
  },
  {
    id: 'projects',
    label: translate('usage.dashboard.meta.projects', undefined, 'Projects'),
    value: projectCount.toLocaleString(),
  },
  {
    id: 'archive',
    label: 'Archive',
    value: archive
      ? formatSourceCounts(
          archive.live_sources,
          archive.missing_sources,
          archive.deleted_sources,
          translate
        )
      : '—',
  },
  {
    id: 'archive-root',
    label: 'Archive Root',
    value: archive ? shortenPath(archive.archive_root) : '—',
  },
  {
    id: 'archive-time',
    label: 'Last Sync',
    value: archive
      ? formatArchiveTimestamp(archive.history_completed_at ?? archive.recent_completed_at, locale)
      : '—',
  },
]

export const buildOverviewHighlights = ({
  modelStats,
  projectStats,
  summary,
  trendBuckets,
  trendGranularityLabel,
  selectedWindowLabel,
  translate,
}: {
  modelStats: ModelStat[]
  projectStats: ProjectStat[]
  summary: UsageSummary | null
  trendBuckets: UsageTrendBucket[]
  trendGranularityLabel: string
  selectedWindowLabel: string
  translate: UsageDashboardTranslate
}): OverviewHighlightItem[] => {
  const topModel = modelStats[0]
  const topProject = projectStats[0]

  return [
    {
      id: 'density',
      label: translate('usage.dashboard.highlights.density', undefined, 'Trend Density'),
      value: trendGranularityLabel,
      detail: translate(
        'usage.dashboard.highlights.densityDetail',
        {
          points: trendBuckets.length,
          window: selectedWindowLabel,
        },
        `${trendBuckets.length} points across ${selectedWindowLabel}`
      ),
    },
    {
      id: 'top-model',
      label: translate('usage.dashboard.highlights.topModel', undefined, 'Top Model'),
      value: topModel?.model ?? translate('usage.dashboard.table.noData', undefined, 'No data'),
      detail: topModel
        ? `${formatCost(modelCost(topModel))} · ${formatTokens(topModel.total_tokens)}`
        : translate('usage.dashboard.table.noData', undefined, 'No data'),
    },
    {
      id: 'top-project',
      label: translate('usage.dashboard.highlights.topProject', undefined, 'Top Project'),
      value: topProject
        ? shortenPath(topProject.project_path)
        : translate('usage.dashboard.table.noData', undefined, 'No data'),
      detail: topProject
        ? `${formatCost(topProject.total_cost)} · ${topProject.request_count.toLocaleString()} ${translate('usage.dashboard.table.requests', undefined, 'requests')}`
        : translate('usage.dashboard.table.noData', undefined, 'No data'),
    },
    {
      id: 'cache',
      label: translate('usage.dashboard.highlights.cacheRead', undefined, 'Cache Read'),
      value: summary
        ? formatTokens(summary.total_cache_read_tokens)
        : translate('usage.dashboard.table.noData', undefined, 'No data'),
      detail: summary
        ? translate(
            'usage.dashboard.highlights.cacheReadDetail',
            {
              percent: formatPercent(summary.cache_efficiency),
            },
            `Cache reuse ${formatPercent(summary.cache_efficiency)}`
          )
        : translate('usage.dashboard.table.noData', undefined, 'No data'),
    },
  ]
}

export const buildTopModelRankings = (
  modelStats: ModelStat[],
  translate: UsageDashboardTranslate
): OverviewRankItem[] => {
  const totalCost = modelStats.reduce((sum, item) => sum + modelCost(item), 0)

  return [...modelStats]
    .sort(
      (left, right) =>
        modelCost(right) - modelCost(left) ||
        right.total_tokens - left.total_tokens ||
        right.request_count - left.request_count
    )
    .slice(0, 5)
    .map((item) => ({
      id: item.model,
      label: item.model,
      title: item.model,
      detail: `${item.request_count.toLocaleString()} ${translate('usage.dashboard.table.requests', undefined, 'requests')} · ${formatTokens(item.total_tokens)}`,
      value: formatCost(modelCost(item)),
      share: totalCost > 0 ? modelCost(item) / totalCost : 0,
    }))
}

export const buildTopProjectRankings = (
  projectStats: ProjectStat[],
  translate: UsageDashboardTranslate
): OverviewRankItem[] => {
  const totalCost = projectStats.reduce((sum, item) => sum + item.total_cost, 0)

  return [...projectStats]
    .sort(
      (left, right) =>
        right.total_cost - left.total_cost ||
        right.total_tokens - left.total_tokens ||
        right.request_count - left.request_count
    )
    .slice(0, 5)
    .map((item) => ({
      id: item.project_path,
      label: shortenPath(item.project_path),
      title: item.project_path,
      detail: `${formatTokens(item.total_tokens)} · ${item.request_count.toLocaleString()} ${translate('usage.dashboard.table.requests', undefined, 'requests')}`,
      value: formatCost(item.total_cost),
      share: totalCost > 0 ? item.total_cost / totalCost : 0,
    }))
}
