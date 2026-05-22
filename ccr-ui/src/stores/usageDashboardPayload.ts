import type {
  DailyTrend,
  HeatmapResponse,
  ModelStat,
  PaginatedLogs,
  Platform,
  ProjectStat,
  SourceBreakdown,
  UsageArchiveDiagnostics,
  UsageDashboardResponse,
  UsageFeatureCapability,
  UsageLogsQuery,
  UsageSummary,
} from '@/types/usage'

export type UsageDashboardPayload = Omit<
  UsageDashboardResponse,
  'summary' | 'heatmap' | 'generated_at' | 'archive' | 'source_stats'
> & {
  summary?: UsageSummary | null
  archive?: UsageArchiveDiagnostics | null
  heatmap?: HeatmapResponse
  by_model?: ModelStat[]
  by_project?: ProjectStat[]
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
  sourceStats: SourceBreakdown[]
  archive: UsageArchiveDiagnostics | null
  heatmap: HeatmapResponse | null | undefined
}

export const parseEnvFlag = (value: string | undefined, defaultValue: boolean): boolean => {
  if (value == null || value === '') return defaultValue
  return ['1', 'true', 'yes', 'on'].includes(value.toLowerCase())
}

export const buildDashboardFetchKey = ({
  platform,
  start,
  end,
  includeHeatmap,
}: {
  platform?: Platform
  start?: string
  end?: string
  includeHeatmap: boolean
}) => [platform ?? 'all', start ?? '', end ?? '', includeHeatmap ? 'heatmap' : 'core'].join('|')

export const normalizeDashboardPayload = (
  data: UsageDashboardPayload,
  includeHeatmap: boolean,
): NormalizedUsageDashboardPayload => ({
  summary: data.summary ?? null,
  trends: data.trends ?? [],
  // 兼容后端 "by_model" / "model_stats" 两种字段名。
  modelStats: data.model_stats ?? data.by_model ?? [],
  projectStats: data.project_stats ?? data.by_project ?? [],
  sourceStats: data.source_stats ?? [],
  archive: data.archive ?? null,
  heatmap: includeHeatmap ? data.heatmap ?? null : undefined,
})

export const buildDashboardCachePayload = ({
  summary,
  trends,
  modelStats,
  projectStats,
  sourceStats = [],
  archive,
  heatmap,
  includeHeatmap,
}: {
  summary: UsageSummary | null
  trends: DailyTrend[]
  modelStats: ModelStat[]
  projectStats: ProjectStat[]
  sourceStats?: SourceBreakdown[]
  archive: UsageArchiveDiagnostics | null
  heatmap: HeatmapResponse | null
  includeHeatmap: boolean
}): UsageDashboardPayload => ({
  summary,
  trends,
  model_stats: modelStats,
  project_stats: projectStats,
  source_stats: sourceStats,
  archive: archive ?? undefined,
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
  platform?: Platform
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
