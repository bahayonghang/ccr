/**
 * Usage Analytics Store (V2)
 *
 * 统一管理使用统计数据，对接后端 v2 聚合 API
 * 支持请求去重、筛选防抖、分频自动刷新和日志游标分页
 */

import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import type {
  DailyTrend,
  HeatmapResponse,
  ImportAllUsageResponse,
  ImportResult,
  ModelStat,
  PaginatedLogs,
  Platform,
  ProjectStat,
  UsageDashboardResponse,
  UsageImportSummary,
  UsageLogsQuery,
  UsageSummary,
} from '@/types/usage'
import {
  getUsageByModelV2,
  getUsageByProjectV2,
  getUsageDashboardV2,
  getUsageHeatmapV2,
  getUsageLogsV2,
  getUsageSummaryV2,
  getUsageTrendsV2,
  importAllUsageV2,
  importUsageV2,
} from '@/api'
import { logger } from '@/utils/logger'

const REFRESH_INTERVAL = 30_000 // 30s
const HEATMAP_REFRESH_INTERVAL = 10 * 60_000 // 10min
const FILTER_DEBOUNCE_MS = 300
const HEATMAP_DAYS = 365

const parseEnvFlag = (value: string | undefined, defaultValue: boolean): boolean => {
  if (value == null || value === '') return defaultValue
  return ['1', 'true', 'yes', 'on'].includes(value.toLowerCase())
}

const USE_DASHBOARD_API = parseEnvFlag(
  import.meta.env.VITE_USAGE_DASHBOARD_AGGREGATED_API,
  true,
)
const USE_CURSOR_LOGS = parseEnvFlag(import.meta.env.VITE_USAGE_LOGS_CURSOR_PAGING, true)
const LAZY_HEATMAP_LOAD = parseEnvFlag(import.meta.env.VITE_PERF_HEATMAP_LAZY_LOAD, true)

const nowMs = () => (typeof performance !== 'undefined' ? performance.now() : Date.now())

const recordPerfMetric = (
  name: string,
  value: number,
  extra: Record<string, string | number | boolean> = {},
) => {
  const detail = { name, value, ts: Date.now(), ...extra }
  if (typeof window !== 'undefined') {
    window.dispatchEvent(new CustomEvent('ccr:perf', { detail }))
  }
  if (import.meta.env.DEV) {
    logger.debug('[usage-perf]', detail)
  }
}

interface FetchOptions {
  includeHeatmap?: boolean
  reason?: string
  preserveError?: boolean
}

type UsageDashboardPayload = Omit<UsageDashboardResponse, 'heatmap' | 'generated_at'> & {
  heatmap?: HeatmapResponse
  by_model?: ModelStat[]
  by_project?: ProjectStat[]
}

const isImportAllUsageResponse = (
  payload: ImportResult | ImportAllUsageResponse,
): payload is ImportAllUsageResponse => {
  return 'results' in payload && Array.isArray(payload.results)
}

type IdleCapableWindow = Window & {
  requestIdleCallback?: (callback: () => void, options?: { timeout: number }) => number
}

export const useUsageStore = defineStore('usage', () => {
  // ═══ State ═══
  const summary = ref<UsageSummary | null>(null)
  const trends = ref<DailyTrend[]>([])
  const modelStats = ref<ModelStat[]>([])
  const projectStats = ref<ProjectStat[]>([])
  const heatmap = ref<HeatmapResponse | null>(null)
  const logs = ref<PaginatedLogs | null>(null)

  const loading = ref(true)
  const logsLoading = ref(false)
  const error = ref<string | null>(null)
  const warning = ref<string | null>(null)
  const lastUpdated = ref<Date | null>(null)
  const importing = ref(false)
  const isBootstrapping = ref(false)
  const bootstrapAttempted = ref(false)
  const lastImportSummary = ref<UsageImportSummary | null>(null)
  const lastImportResults = ref<ImportResult[]>([])

  // 筛选条件
  const platform = ref<Platform | undefined>(undefined)
  const timeRange = ref<{ start?: string; end?: string }>({})
  const logsPage = ref(1)
  const logsPageSize = ref(50)
  const logsModelFilter = ref<string | undefined>(undefined)
  const logsCursor = ref<string | undefined>(undefined)
  const logsCursorHistory = ref<string[]>([])

  let coreRefreshTimer: ReturnType<typeof setInterval> | null = null
  let heatmapRefreshTimer: ReturnType<typeof setInterval> | null = null
  let filterDebounceTimer: ReturnType<typeof setTimeout> | null = null

  let requestSerial = 0
  let inFlightKey: string | null = null
  let inFlightPromise: Promise<void> | null = null

  // ═══ Computed ═══
  const totalTokens = computed(() => {
    if (!summary.value) return 0
    return summary.value.total_input_tokens + summary.value.total_output_tokens
  })

  const logsTotalPages = computed(() => {
    const total = logs.value?.total
    if (!total || total <= 0) return 1
    return Math.max(1, Math.ceil(total / logsPageSize.value))
  })

  const canPrevLogs = computed(() => {
    if (USE_CURSOR_LOGS) return logsCursorHistory.value.length > 0
    return logsPage.value > 1
  })

  const canNextLogs = computed(() => {
    if (!logs.value) return false
    if (USE_CURSOR_LOGS) return Boolean(logs.value.next_cursor)
    const total = logs.value.total
    if (!total || total <= 0) return logs.value.records.length >= logsPageSize.value
    return logsPage.value < logsTotalPages.value
  })

  const hasUsageData = computed(() => (summary.value?.total_requests ?? 0) > 0)
  const hasNoUsageData = computed(() => !loading.value && !error.value && !hasUsageData.value)

  const buildFetchKey = (includeHeatmap: boolean) =>
    [
      platform.value ?? 'all',
      timeRange.value.start ?? '',
      timeRange.value.end ?? '',
      includeHeatmap ? 'heatmap' : 'core',
    ].join('|')

   
  const applyDashboardPayload = (data: UsageDashboardPayload, includeHeatmap: boolean) => {
    summary.value = data.summary ?? null
    trends.value = data.trends ?? []
    // 兼容后端 "by_model" / "model_stats" 两种字段名
    modelStats.value = data.model_stats ?? data.by_model ?? []
    projectStats.value = data.project_stats ?? data.by_project ?? []
    if (includeHeatmap && data.heatmap) {
      heatmap.value = data.heatmap
    }
  }

  const applyFilters = (opts: { platform?: Platform; start?: string; end?: string }) => {
    platform.value = opts.platform
    timeRange.value = { start: opts.start, end: opts.end }
  }

  const clearImportFeedback = () => {
    warning.value = null
    lastImportSummary.value = null
    lastImportResults.value = []
  }

  const buildImportSummary = (results: ImportResult[]): UsageImportSummary => {
    const successCount = results.filter(result => !result.error).length
    const failureCount = results.length - successCount
    const importedRecords = results.reduce((sum, result) => sum + result.records_imported, 0)
    const processedFiles = results.reduce((sum, result) => sum + result.files_processed, 0)
    const hasPartial = results.some(result =>
      Boolean(result.error)
      || !result.completed
      || (result.files_processed > 0 && result.records_imported === 0 && result.records_skipped > 0),
    )

    return {
      success_count: successCount,
      failure_count: failureCount,
      imported_records: importedRecords,
      processed_files: processedFiles,
      has_partial: hasPartial,
    }
  }

  const normalizeImportResponse = (
    payload: ImportResult | ImportAllUsageResponse,
    platformOverride?: Platform,
  ): ImportAllUsageResponse => {
    if (isImportAllUsageResponse(payload)) {
      return payload
    }

    const result: ImportResult = platformOverride
      ? { ...payload, platform: platformOverride }
      : payload
    return {
      results: [result],
      summary: buildImportSummary([result]),
    }
  }

  // ═══ Actions ═══
  async function fetchHeatmap(reason: string = 'manual') {
    const startedAt = nowMs()
    heatmap.value = await getUsageHeatmapV2<HeatmapResponse>(platform.value, HEATMAP_DAYS)
    recordPerfMetric('usage_heatmap_load_ms', nowMs() - startedAt, { reason })
  }

  function scheduleHeatmapLoad(reason: string) {
    if (!LAZY_HEATMAP_LOAD) return
    const win = typeof window !== 'undefined' ? (window as IdleCapableWindow) : null
    if (typeof win?.requestIdleCallback === 'function') {
      win.requestIdleCallback(() => { void fetchHeatmap(reason) }, { timeout: 1000 })
    } else {
      setTimeout(() => {
        void fetchHeatmap(reason)
      }, 200)
    }
  }

  /** 拉取汇总、趋势、模型、项目和热力图 */
  async function fetchAll(options: FetchOptions = {}) {
    const includeHeatmap = options.includeHeatmap ?? !LAZY_HEATMAP_LOAD
    const reason = options.reason ?? 'manual'
    const preserveError = options.preserveError ?? false
    const startedAt = nowMs()
    const key = buildFetchKey(includeHeatmap)

    if (inFlightPromise && inFlightKey === key) {
      return inFlightPromise
    }

    const requestId = ++requestSerial
    loading.value = true
    if (!preserveError) {
      error.value = null
    }

    const promise = (async () => {
      try {
        let requestCount = 0

        if (USE_DASHBOARD_API) {
          requestCount = 1
          const data = await getUsageDashboardV2<UsageDashboardPayload>(
            platform.value,
            timeRange.value.start,
            timeRange.value.end,
            HEATMAP_DAYS,
            includeHeatmap,
          )
          if (requestId !== requestSerial) return
          applyDashboardPayload(data, includeHeatmap)
          if (LAZY_HEATMAP_LOAD && !includeHeatmap) {
            scheduleHeatmapLoad(`${reason}-lazy-heatmap`)
          }
        } else {
          requestCount = includeHeatmap ? 5 : 4
          const [summaryData, trendsData, modelData, projectData] = await Promise.all([
            getUsageSummaryV2<UsageSummary>(platform.value, timeRange.value.start, timeRange.value.end),
            getUsageTrendsV2<DailyTrend[]>(platform.value, timeRange.value.start, timeRange.value.end),
            getUsageByModelV2<ModelStat[]>(platform.value, timeRange.value.start, timeRange.value.end),
            getUsageByProjectV2<ProjectStat[]>(platform.value, timeRange.value.start, timeRange.value.end),
          ])
          if (requestId !== requestSerial) return
          summary.value = summaryData ?? null
          trends.value = trendsData ?? []
          modelStats.value = modelData ?? []
          projectStats.value = projectData ?? []
          if (includeHeatmap) {
            await fetchHeatmap(reason)
          } else if (LAZY_HEATMAP_LOAD) {
            scheduleHeatmapLoad(`${reason}-lazy-heatmap`)
          }
        }

        if (requestId !== requestSerial) return
        lastUpdated.value = new Date()
        recordPerfMetric('usage_dashboard_load_ms', nowMs() - startedAt, {
          reason,
          include_heatmap: includeHeatmap,
          requests: requestCount,
        })
        recordPerfMetric('usage_requests_per_refresh', requestCount, { reason })
      } catch (e) {
        if (requestId !== requestSerial) return
        error.value = e instanceof Error ? e.message : String(e)
      } finally {
        if (requestId === requestSerial) {
          loading.value = false
        }
      }
    })()

    inFlightKey = key
    inFlightPromise = promise.finally(() => {
      if (inFlightPromise === promise) {
        inFlightPromise = null
        inFlightKey = null
      }
    })
    return inFlightPromise
  }

  /** 拉取日志 */
  async function fetchLogs(
    direction: 'reset' | 'next' | 'prev' | 'same' = 'same',
  ) {
    logsLoading.value = true
    error.value = null
    try {
      const preferredMode: 'cursor' | 'offset' = USE_CURSOR_LOGS ? 'cursor' : 'offset'
      const prepareQuery = (mode: 'cursor' | 'offset', includeTotal: boolean): UsageLogsQuery => ({
        platform: platform.value,
        model: logsModelFilter.value,
        start_date: timeRange.value.start,
        end_date: timeRange.value.end,
        page: logsPage.value,
        page_size: logsPageSize.value,
        cursor: mode === 'cursor' ? logsCursor.value : undefined,
        include_total: includeTotal,
        mode,
      })

      if (preferredMode === 'cursor') {
        if (direction === 'reset') {
          logsPage.value = 1
          logsCursor.value = undefined
          logsCursorHistory.value = []
        } else if (direction === 'next') {
          const nextCursor = logs.value?.next_cursor || undefined
          if (!nextCursor) return
          logsCursorHistory.value.push(logsCursor.value ?? '')
          logsCursor.value = nextCursor
          logsPage.value += 1
        } else if (direction === 'prev') {
          const previous = logsCursorHistory.value.pop()
          if (previous == null) return
          logsCursor.value = previous || undefined
          logsPage.value = Math.max(1, logsPage.value - 1)
        }

        try {
          const result = await getUsageLogsV2<PaginatedLogs>(prepareQuery('cursor', false))
          logs.value = { ...result, page: logsPage.value, page_size: logsPageSize.value, mode: 'cursor' }
        } catch (cursorError) {
          // 兼容兜底：cursor 失败时退回 offset 模式
          const offsetResult = await getUsageLogsV2<PaginatedLogs>({
            ...prepareQuery('offset', true),
            cursor: undefined,
          })
          logs.value = { ...offsetResult, page: logsPage.value, page_size: logsPageSize.value, mode: 'offset' }
          if (import.meta.env.DEV) {
            logger.warn('[usage] cursor logs failed, fallback to offset', cursorError)
          }
        }
      } else {
        if (direction === 'next') logsPage.value += 1
        if (direction === 'prev') logsPage.value = Math.max(1, logsPage.value - 1)
        if (direction === 'reset') logsPage.value = 1
        const result = await getUsageLogsV2<PaginatedLogs>(prepareQuery('offset', true))
        logs.value = { ...result, mode: 'offset' }
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      logsLoading.value = false
    }
  }

  /** 下一页日志 */
  async function nextLogsPage() {
    await fetchLogs('next')
  }

  /** 上一页日志 */
  async function prevLogsPage() {
    await fetchLogs('prev')
  }

  async function triggerImport(
    requestedPlatform?: Platform,
    reason: 'manual' | 'bootstrap' = 'manual',
  ): Promise<ImportAllUsageResponse> {
    importing.value = true
    isBootstrapping.value = reason === 'bootstrap'
    error.value = null

    try {
      const response = requestedPlatform
        ? normalizeImportResponse(
          await importUsageV2<ImportResult>(requestedPlatform),
          requestedPlatform,
        )
        : normalizeImportResponse(await importAllUsageV2<ImportAllUsageResponse>())

      lastImportSummary.value = response.summary
      lastImportResults.value = response.results

      const failedResults = response.results.filter(result => result.error)
      const failedDetails = failedResults
        .map(result => `${result.platform}: ${result.error}`)
        .join('\n')

      if (response.summary.failure_count === response.results.length && response.results.length > 0) {
        error.value = failedDetails || '未能导入本地 usage 日志，请检查日志目录或导入错误'
        warning.value = null
        return response
      }

      if (response.summary.has_partial) {
        warning.value = failedDetails || '仅导入部分 usage 数据，达到时间预算或部分平台失败，可重试继续导入'
      } else if (response.summary.processed_files === 0 && response.summary.imported_records === 0) {
        warning.value = null
      } else {
        warning.value = null
      }

      if (response.summary.success_count > 0 || response.summary.has_partial) {
        await fetchAll({
          includeHeatmap: !LAZY_HEATMAP_LOAD,
          reason: `${reason}-import-refresh`,
          preserveError: response.summary.failure_count === response.results.length,
        })
      }

      return response
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e)
      const fallback: ImportAllUsageResponse = {
        results: [{
          platform: requestedPlatform ?? 'all',
          files_processed: 0,
          records_imported: 0,
          records_skipped: 0,
          duration_ms: 0,
          completed: false,
          error: message,
        }],
        summary: {
          success_count: 0,
          failure_count: 1,
          imported_records: 0,
          processed_files: 0,
          has_partial: true,
        },
      }

      lastImportSummary.value = fallback.summary
      lastImportResults.value = fallback.results
      warning.value = null
      error.value = message
      return fallback
    } finally {
      importing.value = false
      isBootstrapping.value = false
    }
  }

  /** 初始化仪表盘并在空库时做一次自举导入 */
  async function initializeDashboard(opts: { platform?: Platform; start?: string; end?: string }) {
    applyFilters(opts)
    await fetchAll({ includeHeatmap: !LAZY_HEATMAP_LOAD, reason: 'initialize' })

    if ((summary.value?.total_requests ?? 0) > 0) {
      return
    }

    if (bootstrapAttempted.value) {
      return
    }

    bootstrapAttempted.value = true
    await triggerImport(undefined, 'bootstrap')
  }

  /** 设置筛选条件并刷新（300ms 防抖） */
  function setFilters(opts: { platform?: Platform; start?: string; end?: string }) {
    applyFilters(opts)

    if (filterDebounceTimer) {
      clearTimeout(filterDebounceTimer)
      filterDebounceTimer = null
    }

    const startedAt = nowMs()
    filterDebounceTimer = setTimeout(async () => {
      await fetchAll({ includeHeatmap: !LAZY_HEATMAP_LOAD, reason: 'filter' })
      recordPerfMetric('usage_filter_apply_ms', nowMs() - startedAt)
      filterDebounceTimer = null
    }, FILTER_DEBOUNCE_MS)
  }

  /** 启动自动刷新 */
  function startAutoRefresh() {
    stopAutoRefresh()
    coreRefreshTimer = setInterval(() => {
      fetchAll({
        includeHeatmap: false,
        reason: 'auto-refresh-core',
        preserveError: Boolean(error.value && !hasUsageData.value),
      })
    }, REFRESH_INTERVAL)
    if (!LAZY_HEATMAP_LOAD) {
      heatmapRefreshTimer = setInterval(() => {
        fetchHeatmap('auto-refresh-heatmap')
      }, HEATMAP_REFRESH_INTERVAL)
    }
  }

  /** 停止自动刷新 */
  function stopAutoRefresh() {
    if (coreRefreshTimer) {
      clearInterval(coreRefreshTimer)
      coreRefreshTimer = null
    }
    if (heatmapRefreshTimer) {
      clearInterval(heatmapRefreshTimer)
      heatmapRefreshTimer = null
    }
    if (filterDebounceTimer) {
      clearTimeout(filterDebounceTimer)
      filterDebounceTimer = null
    }
  }

  return {
    // state
    summary,
    trends,
    modelStats,
    projectStats,
    heatmap,
    logs,
    loading,
    logsLoading,
    error,
    warning,
    lastUpdated,
    importing,
    isBootstrapping,
    bootstrapAttempted,
    lastImportSummary,
    lastImportResults,
    platform,
    timeRange,
    logsPage,
    logsPageSize,
    logsModelFilter,
    // computed
    totalTokens,
    logsTotalPages,
    canPrevLogs,
    canNextLogs,
    hasUsageData,
    hasNoUsageData,
    // flags
    useCursorLogs: USE_CURSOR_LOGS,
    useDashboardApi: USE_DASHBOARD_API,
    // actions
    initializeDashboard,
    fetchAll,
    fetchHeatmap,
    fetchLogs,
    nextLogsPage,
    prevLogsPage,
    triggerImport,
    clearImportFeedback,
    setFilters,
    startAutoRefresh,
    stopAutoRefresh,
  }
})
