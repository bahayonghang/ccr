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
  ImportResult,
  ModelStat,
  PaginatedLogs,
  Platform,
  ProjectStat,
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
} from '@/api/modules/usageV2'

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
    console.debug('[usage-perf]', detail)
  }
}

interface FetchOptions {
  includeHeatmap?: boolean
  reason?: string
}

export const useUsageStore = defineStore('usage', () => {
  // ═══ State ═══
  const summary = ref<UsageSummary | null>(null)
  const trends = ref<DailyTrend[]>([])
  const modelStats = ref<ModelStat[]>([])
  const projectStats = ref<ProjectStat[]>([])
  const heatmap = ref<HeatmapResponse | null>(null)
  const logs = ref<PaginatedLogs | null>(null)

  const loading = ref(false)
  const logsLoading = ref(false)
  const error = ref<string | null>(null)
  const lastUpdated = ref<Date | null>(null)

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

  const buildFetchKey = (includeHeatmap: boolean) =>
    [
      platform.value ?? 'all',
      timeRange.value.start ?? '',
      timeRange.value.end ?? '',
      includeHeatmap ? 'heatmap' : 'core',
    ].join('|')

  const applyDashboardPayload = (
    data: {
      summary: UsageSummary
      trends: DailyTrend[]
      model_stats: ModelStat[]
      project_stats: ProjectStat[]
      heatmap: HeatmapResponse
    },
    includeHeatmap: boolean,
  ) => {
    summary.value = data.summary
    trends.value = data.trends
    modelStats.value = data.model_stats
    projectStats.value = data.project_stats
    if (includeHeatmap) {
      heatmap.value = data.heatmap
    }
  }

  // ═══ Actions ═══
  async function fetchHeatmap(reason: string = 'manual') {
    const startedAt = nowMs()
    heatmap.value = await getUsageHeatmapV2(platform.value, HEATMAP_DAYS)
    recordPerfMetric('usage_heatmap_load_ms', nowMs() - startedAt, { reason })
  }

  /** 拉取汇总、趋势、模型、项目和热力图 */
  async function fetchAll(options: FetchOptions = {}) {
    const includeHeatmap = options.includeHeatmap ?? true
    const reason = options.reason ?? 'manual'
    const startedAt = nowMs()
    const key = buildFetchKey(includeHeatmap)

    if (inFlightPromise && inFlightKey === key) {
      return inFlightPromise
    }

    const requestId = ++requestSerial
    loading.value = true
    error.value = null

    const promise = (async () => {
      try {
        let requestCount = 0

        if (USE_DASHBOARD_API) {
          requestCount = 1
          const data = await getUsageDashboardV2(
            platform.value,
            timeRange.value.start,
            timeRange.value.end,
            HEATMAP_DAYS,
            includeHeatmap,
          )
          if (requestId !== requestSerial) return
          applyDashboardPayload(data, includeHeatmap)
        } else {
          requestCount = includeHeatmap ? 5 : 4
          const [summaryData, trendsData, modelData, projectData] = await Promise.all([
            getUsageSummaryV2(platform.value, timeRange.value.start, timeRange.value.end),
            getUsageTrendsV2(platform.value, timeRange.value.start, timeRange.value.end),
            getUsageByModelV2(platform.value, timeRange.value.start, timeRange.value.end),
            getUsageByProjectV2(platform.value, timeRange.value.start, timeRange.value.end),
          ])
          if (requestId !== requestSerial) return
          summary.value = summaryData
          trends.value = trendsData
          modelStats.value = modelData
          projectStats.value = projectData
          if (includeHeatmap) {
            await fetchHeatmap(reason)
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
      if (USE_CURSOR_LOGS) {
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

        const result = await getUsageLogsV2(
          platform.value,
          logsPage.value,
          logsPageSize.value,
          logsModelFilter.value,
          logsCursor.value,
          false,
        )
        logs.value = { ...result, page: logsPage.value, page_size: logsPageSize.value }
      } else {
        if (direction === 'next') logsPage.value += 1
        if (direction === 'prev') logsPage.value = Math.max(1, logsPage.value - 1)
        if (direction === 'reset') logsPage.value = 1
        logs.value = await getUsageLogsV2(
          platform.value,
          logsPage.value,
          logsPageSize.value,
          logsModelFilter.value,
          undefined,
          true,
        )
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

  /** 触发数据导入 */
  async function triggerImport(p?: string): Promise<ImportResult[]> {
    if (p) {
      const r = await importUsageV2(p)
      return [r]
    }
    return importAllUsageV2()
  }

  /** 设置筛选条件并刷新（300ms 防抖） */
  function setFilters(opts: { platform?: Platform; start?: string; end?: string }) {
    platform.value = opts.platform
    timeRange.value = { start: opts.start, end: opts.end }

    if (filterDebounceTimer) {
      clearTimeout(filterDebounceTimer)
      filterDebounceTimer = null
    }

    const startedAt = nowMs()
    filterDebounceTimer = setTimeout(async () => {
      await fetchAll({ includeHeatmap: true, reason: 'filter' })
      recordPerfMetric('usage_filter_apply_ms', nowMs() - startedAt)
      filterDebounceTimer = null
    }, FILTER_DEBOUNCE_MS)
  }

  /** 启动自动刷新 */
  function startAutoRefresh() {
    stopAutoRefresh()
    coreRefreshTimer = setInterval(() => {
      fetchAll({ includeHeatmap: false, reason: 'auto-refresh-core' })
    }, REFRESH_INTERVAL)
    heatmapRefreshTimer = setInterval(() => {
      fetchHeatmap('auto-refresh-heatmap')
    }, HEATMAP_REFRESH_INTERVAL)
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
    lastUpdated,
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
    // flags
    useCursorLogs: USE_CURSOR_LOGS,
    useDashboardApi: USE_DASHBOARD_API,
    // actions
    fetchAll,
    fetchHeatmap,
    fetchLogs,
    nextLogsPage,
    prevLogsPage,
    triggerImport,
    setFilters,
    startAutoRefresh,
    stopAutoRefresh,
  }
})
