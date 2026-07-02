import type {
  DailyTrend,
  HeatmapResponse,
  ModelStat,
  PaginatedLogs,
  UsagePlatform,
  ProjectStat,
  ProviderBreakdown,
  SourceBreakdown,
  UsageArchiveDiagnostics,
  UsageDashboardResponse,
  UsageFeatureCapability,
  UsageLogsQuery,
  UsageSnapshotProjection,
  UsageSummary,
} from '@/types/usage'

export type UsageDashboardPayload = Omit<
  UsageDashboardResponse,
  'summary' | 'heatmap' | 'generated_at' | 'archive' | 'provider_stats' | 'source_stats' | 'snapshot'
> & {
  summary?: UsageSummary | null
  archive?: UsageArchiveDiagnostics | null
  snapshot?: UsageSnapshotProjection | null
  heatmap?: HeatmapResponse
  by_model?: ModelStat[]
  by_project?: ProjectStat[]
  provider_stats?: ProviderBreakdown[]
  source_stats?: SourceBreakdown[]
}

export type UsageDashboardCacheEntry = {
  payload: UsageDashboardPayload
  ts: number
}

export type NormalizedUsageDashboardPayload = {
  summary: UsageSummary | null
  trends: DailyTrend[]
  modelStats: ModelStat[]
  projectStats: ProjectStat[]
  providerStats: ProviderBreakdown[]
  sourceStats: SourceBreakdown[]
  archive: UsageArchiveDiagnostics | null
  snapshot: UsageSnapshotProjection | null
  heatmap: HeatmapResponse | null | undefined
}

export const parseEnvFlag = (value: string | undefined, defaultValue: boolean): boolean => {
  if (value == null || value === '') return defaultValue
  return ['1', 'true', 'yes', 'on'].includes(value.toLowerCase())
}

export const buildDashboardFetchKey = ({
  platform,
  provider,
  start,
  end,
  includeHeatmap,
}: {
  platform?: UsagePlatform
  provider?: string
  start?: string
  end?: string
  includeHeatmap: boolean
}) =>
  [
    platform ?? 'all',
    provider ?? 'all',
    start ?? '',
    end ?? '',
    includeHeatmap ? 'heatmap' : 'core',
  ].join('|')

export const normalizeDashboardPayload = (
  data: UsageDashboardPayload,
  includeHeatmap: boolean,
): NormalizedUsageDashboardPayload => ({
  summary: data.summary ?? null,
  trends: data.trends ?? [],
  // 兼容后端 "by_model" / "model_stats" 两种字段名。
  modelStats: data.model_stats ?? data.by_model ?? [],
  projectStats: data.project_stats ?? data.by_project ?? [],
  providerStats: data.provider_stats ?? [],
  sourceStats: data.source_stats ?? [],
  archive: data.archive ?? null,
  snapshot: data.snapshot ?? null,
  heatmap: includeHeatmap ? data.heatmap ?? null : undefined,
})

export const buildDashboardCachePayload = ({
  summary,
  trends,
  modelStats,
  projectStats,
  providerStats = [],
  sourceStats = [],
  archive,
  snapshot,
  heatmap,
  includeHeatmap,
}: {
  summary: UsageSummary | null
  trends: DailyTrend[]
  modelStats: ModelStat[]
  projectStats: ProjectStat[]
  providerStats?: ProviderBreakdown[]
  sourceStats?: SourceBreakdown[]
  archive: UsageArchiveDiagnostics | null
  snapshot?: UsageSnapshotProjection | null
  heatmap: HeatmapResponse | null
  includeHeatmap: boolean
}): UsageDashboardPayload => ({
  summary,
  trends,
  model_stats: modelStats,
  project_stats: projectStats,
  provider_stats: providerStats,
  source_stats: sourceStats,
  archive: archive ?? undefined,
  snapshot: snapshot ?? undefined,
  heatmap: includeHeatmap ? heatmap ?? undefined : undefined,
})

export const buildUsageLogsQuery = ({
  platform,
  model,
  startDate,
  endDate,
  page,
  pageSize,
  cursor,
  includeTotal,
}: {
  platform?: UsagePlatform
  model?: string
  startDate?: string
  endDate?: string
  page: number
  pageSize: number
  cursor: string | null
  includeTotal: boolean
}): UsageLogsQuery => ({
  platform,
  model,
  start_date: startDate,
  end_date: endDate,
  page,
  page_size: pageSize,
  cursor: cursor ?? undefined,
  include_total: includeTotal,
  mode: 'cursor',
})

export const normalizePaginatedLogs = (
  result: PaginatedLogs,
  page: number,
  pageSize: number,
  previousTotal: number | null,
): PaginatedLogs => ({
  ...result,
  total: result.total ?? previousTotal,
  page,
  page_size: result.page_size ?? pageSize,
  mode: 'cursor',
})

export const isCapabilityUnsupported = (capability: UsageFeatureCapability | null | undefined) => {
  return Boolean(capability && !capability.supported)
}
