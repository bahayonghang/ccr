import { useCallback, useMemo } from 'react'
import { getErrorMessage } from '@/utils/errorHandler'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import { isCapabilityUnsupported, normalizeDashboardPayload } from '@/utils/usageDashboardPayload'
import {
  useInvalidateUsage,
  useUsageCapabilities,
  useUsageDashboard,
  useUsageHeatmap,
} from './queries'
import { useUsageImport } from './useUsageImport'
import { useUsageLogsPager } from './useUsageLogsPager'
import { useUsageViewStore } from './stores'
import {
  useUsageAutoRefresh,
  useUsageBootstrapImport,
  useUsageSnapshotRefresh,
} from './useUsageDashboardEffects'

export function useUsageDashboardData() {
  const platform = useUsageViewStore((state) => state.platform)
  const timeRange = useUsageViewStore((state) => state.timeRange)
  const dashboardQuery = useUsageDashboard(platform, timeRange.start, timeRange.end)
  const capabilitiesQuery = useUsageCapabilities()
  const heatmapQuery = useUsageHeatmap(platform, 365)
  const invalidate = useInvalidateUsage()

  const refresh = useCallback(async () => {
    await invalidate()
  }, [invalidate])

  const importState = useUsageImport(refresh)

  const payload = useMemo(
    () => (dashboardQuery.data ? normalizeDashboardPayload(dashboardQuery.data, true) : null),
    [dashboardQuery.data],
  )

  const capabilities = capabilitiesQuery.data ?? null
  const flags = dashboardFlags({
    capabilities,
    queryError: dashboardQuery.error,
    importError: importState.error,
    loading: dashboardQuery.isLoading,
    payload,
  })

  const pager = useUsageLogsPager({
    platform,
    start: timeRange.start,
    end: timeRange.end,
    onError: importState.setError,
  })

  useUsageSnapshotRefresh(refresh)
  useUsageBootstrapImport({
    unsupported: flags.dashboardUnsupported,
    hasUsageData: flags.hasUsageData,
    isLoading: dashboardQuery.isLoading,
    isFetched: dashboardQuery.isFetched,
    syncCapability: flags.syncCapability,
    startImportJob: importState.startImportJob,
  })
  useUsageAutoRefresh(dashboardQuery.refetch)

  return {
    summary: flags.summary,
    ...fieldsFromPayload(payload, heatmapQuery.data),
    usageCapabilities: capabilities,
    loading: flags.loading,
    error: flags.error,
    warning: importState.warning,
    lastUpdated: dashboardQuery.dataUpdatedAt ? new Date(dashboardQuery.dataUpdatedAt) : null,
    importing: importState.importing,
    isBootstrapping: importState.isBootstrapping,
    lastImportSummary: importState.lastImportSummary,
    lastImportResults: importState.lastImportResults,
    currentImportJob: importState.currentImportJob,
    platform,
    timeRange,
    ...pager,
    hasUsageData: flags.hasUsageData,
    hasNoUsageData: flags.hasNoUsageData,
    dashboardCapability: flags.dashboardCapability,
    syncCapability: flags.syncCapability,
    dashboardUnsupported: flags.dashboardUnsupported,
    startImportJob: importState.startImportJob,
    refresh,
    runtimeUnavailable: !isTauriRuntime(),
  }
}

export type UsageDashboardData = ReturnType<typeof useUsageDashboardData>

function emptyList<T>(value: T[] | undefined): T[] {
  return value ?? []
}

function fieldsFromPayload(
  payload: ReturnType<typeof normalizeDashboardPayload> | null,
  heatmap: ReturnType<typeof useUsageHeatmap>['data'],
) {
  if (!payload) {
    return {
      trends: emptyList([]),
      modelStats: emptyList([]),
      projectStats: emptyList([]),
      providerStats: emptyList([]),
      sourceStats: emptyList([]),
      heatmap: heatmap ?? null,
      archive: null,
      snapshot: null,
    }
  }
  return {
    trends: payload.trends,
    modelStats: payload.modelStats,
    projectStats: payload.projectStats,
    providerStats: payload.providerStats,
    sourceStats: payload.sourceStats,
    heatmap: payload.heatmap ?? heatmap ?? null,
    archive: payload.archive,
    snapshot: payload.snapshot,
  }
}

function dashboardFlags(input: {
  capabilities: ReturnType<typeof useUsageCapabilities>['data'] | null
  queryError: unknown
  importError: string | null
  loading: boolean
  payload: ReturnType<typeof normalizeDashboardPayload> | null
}) {
  const features = input.capabilities?.features
  const dashboardCapability = features?.overview ?? null
  const syncCapability = features?.sync_json_events ?? null
  const dashboardUnsupported = isCapabilityUnsupported(dashboardCapability)
  const queryError = input.queryError ? getErrorMessage(input.queryError) : null
  const error = input.importError ?? queryError
  const loading = input.loading && !input.payload
  const summary = input.payload?.summary ?? null
  const hasUsageData = (summary?.total_requests ?? 0) > 0
  const hasNoUsageData = !loading && !error && !hasUsageData && !dashboardUnsupported
  return {
    dashboardCapability,
    syncCapability,
    dashboardUnsupported,
    error,
    loading,
    summary,
    hasUsageData,
    hasNoUsageData,
  }
}
