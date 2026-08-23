/**
 * Usage Analytics Store (V2)
 *
 * 统一管理使用统计数据，对接后端 v2 聚合 API
 * 支持请求去重、筛选防抖、分频自动刷新和日志分页诊断
 */

import { defineStore } from 'pinia'
import { getErrorMessage } from '@/utils/errorHandler'
import { computed, ref, shallowRef } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type {
  UsageArchiveDiagnostics,
  DailyTrend,
  HeatmapResponse,
  ImportAllUsageResponse,
  UsageImportResult,
  ModelStat,
  PaginatedLogs,
  UsagePlatform,
  ProjectStat,
  ProviderBreakdown,
  SourceBreakdown,
  UsageCapabilityReport,
  UsageImportJobSnapshot,
  UsageImportSummary,
  UsageSnapshotProjection,
  UsageSnapshotUpdatedPayload,
  UsageSummary,
} from '@/types/usage'
import {
  getUsageImportJobStatusV2,
  getUsageByModelV2,
  getUsageByProviderV2,
  getUsageByProjectV2,
  getUsageCapabilitiesV2,
  getUsageDashboardV2,
  getUsageHeatmapV2,
  getUsageLogsV2,
  getUsageSummaryV2,
  getUsageTrendsV2,
  importAllUsageV2,
  importUsageV2,
  startUsageImportJobV2,
} from '@/api'
import { usePolledData } from '@/composables/usePolledData'
import {
  buildDashboardCachePayload,
  buildDashboardFetchKey,
  buildUsageLogsQuery,
  isCapabilityUnsupported,
  normalizeDashboardPayload,
  normalizePaginatedLogs,
  parseEnvFlag,
  type UsageDashboardCacheEntry,
  type UsageDashboardPayload,
} from '@/utils/usageDashboardPayload'
import {
  buildImportSummary,
  normalizeImportResponse,
  normalizeUserVisibleImportJob,
} from '@/utils/usageImportNormalization'
import { logger } from '@/utils/logger'
import { isTauriRuntime } from '@/utils/tauriRuntime'

const REFRESH_INTERVAL = 30_000 // 30s
const HEATMAP_REFRESH_INTERVAL = 10 * 60_000 // 10min
const FILTER_DEBOUNCE_MS = 300
const HEATMAP_DAYS = 365
const IMPORT_PROGRESS_REFRESH_INTERVAL_MS = 2_000
const DASHBOARD_CACHE_TTL_MS = 30_000
const AUTO_REFRESH_FRESHNESS_MS = DASHBOARD_CACHE_TTL_MS

const USE_DASHBOARD_API = parseEnvFlag(import.meta.env.VITE_USAGE_DASHBOARD_AGGREGATED_API, true)
const LAZY_HEATMAP_LOAD = parseEnvFlag(import.meta.env.VITE_PERF_HEATMAP_LAZY_LOAD, true)

const nowMs = () => (typeof performance !== 'undefined' ? performance.now() : Date.now())

const recordPerfMetric = (
  name: string,
  value: number,
  extra: Record<string, string | number | boolean> = {}
) => {
  const detail = { name, value, ts: Date.now(), ...extra }
  if (typeof window !== 'undefined') {
    window.dispatchEvent(new CustomEvent('ccr:perf', { detail }))
  }
  if (import.meta.env.DEV) {
    logger.debug('[usage-perf]', detail)
  }
}

const recordPerfMark = (name: string, extra: Record<string, string | number | boolean> = {}) => {
  recordPerfMetric(name, 0, extra)
}

interface FetchOptions {
  includeHeatmap?: boolean
  reason?: string
  preserveError?: boolean
  background?: boolean
  force?: boolean
}

type IdleCapableWindow = Window & {
  requestIdleCallback?: (callback: () => void, options?: { timeout: number }) => number
}

type ProviderStatsCacheEntry = {
  payload: ProviderBreakdown[]
  ts: number
}

export const useUsageStore = defineStore('usage', () => {
  // ═══ State ═══
  const summary = ref<UsageSummary | null>(null)
  const trends = shallowRef<DailyTrend[]>([])
  const modelStats = shallowRef<ModelStat[]>([])
  const projectStats = shallowRef<ProjectStat[]>([])
  const providerStats = shallowRef<ProviderBreakdown[]>([])
  const sourceStats = shallowRef<SourceBreakdown[]>([])
  const heatmap = shallowRef<HeatmapResponse | null>(null)
  const logs = shallowRef<PaginatedLogs | null>(null)
  const archive = ref<UsageArchiveDiagnostics | null>(null)
  const snapshot = shallowRef<UsageSnapshotProjection | null>(null)
  const usageCapabilities = ref<UsageCapabilityReport | null>(null)

  const loading = ref(true)
  const logsLoading = ref(false)
  const error = ref<string | null>(null)
  const warning = ref<string | null>(null)
  const lastUpdated = ref<Date | null>(null)
  const importing = ref(false)
  const isBootstrapping = ref(false)
  const bootstrapAttempted = ref(false)
  const lastImportSummary = ref<UsageImportSummary | null>(null)
  const lastImportResults = ref<UsageImportResult[]>([])
  const currentImportJob = ref<UsageImportJobSnapshot | null>(null)

  // 筛选条件
  const platform = ref<UsagePlatform | undefined>(undefined)
  const timeRange = ref<{ start?: string; end?: string }>({})
  const logsPage = ref(1)
  const logsPageSize = ref(50)
  const logsModelFilter = ref<string | undefined>(undefined)

  let filterDebounceTimer: ReturnType<typeof setTimeout> | null = null
  let importJobUnlisteners: UnlistenFn[] = []
  let usageSnapshotUnlistener: UnlistenFn | null = null
  let activeImportReason: 'manual' | 'bootstrap' | null = null
  let lastProgressRefreshAt = 0
  let lastProgressRefreshRecords = 0
  let snapshotRefreshPromise: Promise<void> | null = null

  let requestSerial = 0
  let inFlightKey: string | null = null
  let inFlightPromise: Promise<void> | null = null
  // 使用 ref 而不是 setup-store 闭包 let，HMR 重载后游标栈状态可预测；
  // 此 cursor 仅在 store 内部使用，不进 return（消费者不应直接读写）。
  const logsCursorStack = ref<Array<string | null>>([null])
  const dashboardCache = new Map<string, UsageDashboardCacheEntry>()
  const providerStatsCache = new Map<string, ProviderStatsCacheEntry>()

  // ═══ Computed ═══
  const totalTokens = computed(() => summary.value?.total_tokens ?? 0)

  const logsTotalPages = computed(() => {
    const total = logs.value?.total
    if (!total || total <= 0) return 1
    return Math.max(1, Math.ceil(total / logsPageSize.value))
  })

  const hasLogsTotal = computed(() => logs.value?.total != null)

  const canPrevLogs = computed(() => logsPage.value > 1)

  const canNextLogs = computed(() => {
    if (!logs.value) return false
    if (logs.value.mode === 'cursor') {
      return Boolean(logs.value.next_cursor)
    }
    const total = logs.value.total
    if (!total || total <= 0) return logs.value.records.length >= logsPageSize.value
    return logsPage.value < logsTotalPages.value
  })

  const showLogsPager = computed(() => {
    if (!logs.value) return false
    return (
      canPrevLogs.value || canNextLogs.value || (hasLogsTotal.value && logsTotalPages.value > 1)
    )
  })

  const hasUsageData = computed(() => (summary.value?.total_requests ?? 0) > 0)
  const hasNoUsageData = computed(() => !loading.value && !error.value && !hasUsageData.value)
  const dashboardCapability = computed(() => usageCapabilities.value?.features.overview ?? null)
  const syncCapability = computed(() => usageCapabilities.value?.features.sync_json_events ?? null)
  const dashboardUnsupported = computed(() => isCapabilityUnsupported(dashboardCapability.value))

  const applyDashboardPayload = (data: UsageDashboardPayload, includeHeatmap: boolean) => {
    const normalized = normalizeDashboardPayload(data, includeHeatmap)
    summary.value = normalized.summary
    trends.value = normalized.trends
    modelStats.value = normalized.modelStats
    projectStats.value = normalized.projectStats
    providerStats.value = normalized.providerStats
    sourceStats.value = normalized.sourceStats
    archive.value = normalized.archive
    snapshot.value = normalized.snapshot
    if (normalized.heatmap !== undefined) {
      heatmap.value = normalized.heatmap
    }
  }

  const clearDashboardPayload = () => {
    summary.value = null
    trends.value = []
    modelStats.value = []
    projectStats.value = []
    providerStats.value = []
    providerStatsCache.clear()
    sourceStats.value = []
    heatmap.value = null
    logs.value = null
    archive.value = null
    snapshot.value = null
  }

  const refreshUsageCapabilities = async (force = false) => {
    if (!isTauriRuntime()) return usageCapabilities.value
    if (!force && usageCapabilities.value) return usageCapabilities.value

    try {
      usageCapabilities.value = await getUsageCapabilitiesV2()
    } catch (capabilityError) {
      logger.error('[usage] failed to load llmusage capabilities', capabilityError)
    }

    return usageCapabilities.value
  }

  const applyFilters = (opts: { platform?: UsagePlatform; start?: string; end?: string }) => {
    platform.value = opts.platform
    timeRange.value = { start: opts.start, end: opts.end }
  }

  const clearImportFeedback = () => {
    warning.value = null
    lastImportSummary.value = null
    lastImportResults.value = []
  }

  const resetProgressRefreshState = () => {
    lastProgressRefreshAt = 0
    lastProgressRefreshRecords = 0
  }

  const isTerminalImportJob = (job: UsageImportJobSnapshot | null | undefined) =>
    job?.status === 'finished' || job?.status === 'failed' || job?.status === 'cancelled'

  const syncImportFeedbackFromJob = (job: UsageImportJobSnapshot | null) => {
    if (!job) {
      currentImportJob.value = null
      importing.value = false
      isBootstrapping.value = false
      return
    }

    importing.value = !isTerminalImportJob(job)
    isBootstrapping.value = activeImportReason === 'bootstrap' && importing.value

    const visibleJob = normalizeUserVisibleImportJob(job)
    const visibleResults = visibleJob.results
    const visibleSummary = visibleJob.summary ? buildImportSummary(visibleResults) : null
    currentImportJob.value = {
      ...visibleJob,
      summary: visibleSummary ?? visibleJob.summary,
    }

    if (visibleResults.length > 0) {
      lastImportResults.value = visibleResults
    }
    if (visibleSummary) {
      lastImportSummary.value = visibleSummary
      const failedDetails = visibleResults
        .filter((result) => result.error)
        .map((result) => `${result.platform}: ${result.error}`)
        .join('\n')

      if (visibleSummary.failure_count === visibleResults.length && visibleResults.length > 0) {
        error.value =
          failedDetails || job.error || '未能导入本地 usage 日志，请检查日志目录或导入错误'
        warning.value = null
      } else if (visibleSummary.has_partial) {
        warning.value =
          failedDetails || visibleJob.warnings[0] || '仅导入部分 usage 数据，可重试继续导入'
      } else {
        warning.value = null
      }
    }

    if (job.status === 'failed' && (!visibleSummary || visibleSummary.failure_count > 0)) {
      error.value = visibleJob.error || '后台导入任务失败'
    }
  }

  const clearImportJobListeners = async () => {
    const unlisteners = importJobUnlisteners
    importJobUnlisteners = []
    await Promise.all(unlisteners.map((unlisten) => unlisten()))
  }

  const shouldRefreshOnImportProgress = (job: UsageImportJobSnapshot) => {
    if (job.records_imported <= 0) return false

    const now = Date.now()
    const hasNewRecords = job.records_imported > lastProgressRefreshRecords
    const isFirstPositiveRefresh = lastProgressRefreshRecords === 0
    const throttleExpired = now - lastProgressRefreshAt >= IMPORT_PROGRESS_REFRESH_INTERVAL_MS

    if (!hasNewRecords || (!isFirstPositiveRefresh && !throttleExpired)) {
      return false
    }

    lastProgressRefreshRecords = job.records_imported
    lastProgressRefreshAt = now
    return true
  }

  const refreshAfterImportSignal = async (
    reason: string,
    includeHeatmap = false,
    force = false
  ) => {
    dashboardCache.clear()
    providerStatsCache.clear()
    await fetchAll({
      background: true,
      includeHeatmap,
      force,
      reason,
      preserveError: true,
    })
  }

  const ensureUsageSnapshotListener = async () => {
    if (!isTauriRuntime() || usageSnapshotUnlistener) return

    usageSnapshotUnlistener = await listen<UsageSnapshotUpdatedPayload>(
      'usage:snapshot-updated',
      (event) => {
        dashboardCache.clear()
        providerStatsCache.clear()
        if (!lastUpdated.value && !summary.value) return
        if (snapshotRefreshPromise) return

        snapshotRefreshPromise = fetchAll({
          background: true,
          includeHeatmap: false,
          force: true,
          reason: `snapshot-updated:${event.payload.reason}`,
          preserveError: true,
        }).finally(() => {
          snapshotRefreshPromise = null
        })
      }
    )
  }

  const handleImportJobSnapshot = async (
    job: UsageImportJobSnapshot,
    trigger: 'progress' | 'recent-ready' | 'finished' | 'failed'
  ) => {
    syncImportFeedbackFromJob(job)

    if (trigger === 'progress' && shouldRefreshOnImportProgress(job)) {
      await refreshAfterImportSignal('job-progress')
    }

    if (trigger === 'recent-ready') {
      await refreshAfterImportSignal('job-recent-ready', false, true)
      if (logs.value) {
        await fetchLogs('reset')
      }
    }

    if (trigger === 'finished') {
      await refreshAfterImportSignal('job-finished', !LAZY_HEATMAP_LOAD, true)
      if (logs.value) {
        await fetchLogs('reset')
      }
      activeImportReason = null
      resetProgressRefreshState()
      await clearImportJobListeners()
    }

    if (trigger === 'failed') {
      activeImportReason = null
      resetProgressRefreshState()
      await clearImportJobListeners()
    }
  }

  const ensureImportJobListeners = async (jobId: string) => {
    if (!isTauriRuntime()) return

    await clearImportJobListeners()

    importJobUnlisteners = await Promise.all([
      listen<UsageImportJobSnapshot>('usage:job-progress', (event) => {
        if (event.payload.job_id === jobId) {
          void handleImportJobSnapshot(event.payload, 'progress')
        }
      }),
      listen<UsageImportJobSnapshot>('usage:job-recent-ready', (event) => {
        if (event.payload.job_id === jobId) {
          void handleImportJobSnapshot(event.payload, 'recent-ready')
        }
      }),
      listen<UsageImportJobSnapshot>('usage:job-finished', (event) => {
        if (event.payload.job_id === jobId) {
          void handleImportJobSnapshot(event.payload, 'finished')
        }
      }),
      listen<UsageImportJobSnapshot>('usage:job-failed', (event) => {
        if (event.payload.job_id === jobId) {
          void handleImportJobSnapshot(event.payload, 'failed')
        }
      }),
    ])

    const latest = await getUsageImportJobStatusV2(jobId)
    const trigger =
      latest.status === 'failed' || latest.status === 'cancelled'
        ? 'failed'
        : latest.status === 'finished'
          ? 'finished'
          : latest.status === 'recent_ready'
            ? 'recent-ready'
            : 'progress'
    await handleImportJobSnapshot(latest, trigger)
  }

  // ═══ Actions ═══
  async function fetchHeatmap(reason: string = 'manual') {
    const startedAt = nowMs()
    heatmap.value = await getUsageHeatmapV2(platform.value, HEATMAP_DAYS)
    recordPerfMetric('usage_heatmap_load_ms', nowMs() - startedAt, { reason })
  }

  function scheduleHeatmapLoad(reason: string) {
    if (!LAZY_HEATMAP_LOAD) return
    const win = typeof window !== 'undefined' ? (window as IdleCapableWindow) : null
    if (typeof win?.requestIdleCallback === 'function') {
      win.requestIdleCallback(
        () => {
          void fetchHeatmap(reason)
        },
        { timeout: 1000 }
      )
    } else {
      setTimeout(() => {
        void fetchHeatmap(reason)
      }, 200)
    }
  }

  /** 拉取汇总、趋势、模型、项目和热力图 */
  async function fetchAll(options: FetchOptions = {}) {
    await ensureUsageSnapshotListener()
    const includeHeatmap = options.includeHeatmap ?? !LAZY_HEATMAP_LOAD
    const reason = options.reason ?? 'manual'
    const preserveError = options.preserveError ?? false
    const background = options.background ?? false
    const force = options.force ?? false
    const startedAt = nowMs()
    const key = buildDashboardFetchKey({
      platform: platform.value,
      start: timeRange.value.start,
      end: timeRange.value.end,
      includeHeatmap,
    })
    const cached = dashboardCache.get(key)
    const hasFreshCache = cached && Date.now() - cached.ts < DASHBOARD_CACHE_TTL_MS

    if (!force && inFlightPromise && inFlightKey === key) {
      return inFlightPromise
    }

    if (!force && hasFreshCache) {
      applyDashboardPayload(cached.payload, includeHeatmap)
      if (!preserveError) {
        error.value = null
      }
      loading.value = false
      lastUpdated.value = new Date(cached.ts)
      return
    }

    const requestId = ++requestSerial
    if (!background) {
      loading.value = true
    }
    if (!preserveError) {
      error.value = null
    }

    const promise = (async () => {
      try {
        let requestCount = 0
        const capabilities = await refreshUsageCapabilities(force)
        const overviewCapability = capabilities?.features.overview

        if (overviewCapability && !overviewCapability.supported) {
          clearDashboardPayload()
          dashboardCache.delete(key)
          if (!preserveError) {
            error.value = null
          }
          lastUpdated.value = new Date()
          recordPerfMetric('usage_dashboard_unsupported', 1, {
            reason: overviewCapability.reason ?? 'unknown',
          })
          return
        }

        if (USE_DASHBOARD_API) {
          requestCount = 1
          const data = await getUsageDashboardV2(
            platform.value,
            timeRange.value.start,
            timeRange.value.end,
            HEATMAP_DAYS,
            includeHeatmap
          )
          if (requestId !== requestSerial) return
          applyDashboardPayload(data, includeHeatmap)
          dashboardCache.set(key, { payload: data, ts: Date.now() })
          if (LAZY_HEATMAP_LOAD && !includeHeatmap) {
            scheduleHeatmapLoad(`${reason}-lazy-heatmap`)
          }
        } else {
          requestCount = includeHeatmap ? 5 : 4
          const [summaryData, trendsData, modelData, projectData] = await Promise.all([
            getUsageSummaryV2(platform.value, timeRange.value.start, timeRange.value.end),
            getUsageTrendsV2(platform.value, timeRange.value.start, timeRange.value.end),
            getUsageByModelV2(platform.value, timeRange.value.start, timeRange.value.end),
            getUsageByProjectV2(platform.value, timeRange.value.start, timeRange.value.end),
          ])
          if (requestId !== requestSerial) return
          summary.value = summaryData ?? null
          trends.value = trendsData ?? []
          modelStats.value = modelData ?? []
          projectStats.value = projectData ?? []
          providerStats.value = []
          sourceStats.value = []
          if (includeHeatmap) {
            await fetchHeatmap(reason)
          } else if (LAZY_HEATMAP_LOAD) {
            scheduleHeatmapLoad(`${reason}-lazy-heatmap`)
          }
          dashboardCache.set(key, {
            payload: buildDashboardCachePayload({
              summary: summaryData ?? null,
              trends: trendsData ?? [],
              modelStats: modelData ?? [],
              projectStats: projectData ?? [],
              providerStats: [],
              sourceStats: [],
              archive: archive.value,
              snapshot: snapshot.value,
              heatmap: heatmap.value,
              includeHeatmap,
            }),
            ts: Date.now(),
          })
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
        error.value = getErrorMessage(e)
      } finally {
        if (requestId === requestSerial && !background) {
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
  async function fetchLogs(direction: 'reset' | 'next' | 'prev' | 'same' = 'same') {
    logsLoading.value = true
    error.value = null
    try {
      let targetPage = logsPage.value

      if (direction === 'reset') {
        targetPage = 1
        logsCursorStack.value = [null]
      } else if (direction === 'next') {
        if (!logs.value?.next_cursor) return
        // next_cursor 类型为 string | null | undefined；写 undefined 进 stack 后
        // prev 翻页取出会变成 undefined，进 getUsageLogsV2 时与 null 行为不一致，
        // 导致分页错位。这里强制规范化为 null，与 stack 元素类型对齐。
        logsCursorStack.value[targetPage] = logs.value.next_cursor ?? null
        targetPage += 1
      } else if (direction === 'prev') {
        targetPage = Math.max(1, targetPage - 1)
      }

      const currentCursor = logsCursorStack.value[targetPage - 1] ?? null
      const previousTotal = logs.value?.total ?? null
      logsPage.value = targetPage

      const result = await getUsageLogsV2(
        buildUsageLogsQuery({
          platform: platform.value,
          model: logsModelFilter.value,
          startDate: timeRange.value.start,
          endDate: timeRange.value.end,
          page: targetPage,
          pageSize: logsPageSize.value,
          cursor: currentCursor,
          includeTotal: targetPage === 1 && previousTotal == null,
        })
      )
      logs.value = normalizePaginatedLogs(result, targetPage, logsPageSize.value, previousTotal)
    } catch (e) {
      error.value = getErrorMessage(e)
    } finally {
      logsLoading.value = false
    }
  }

  async function fetchProviderStats() {
    const key = buildDashboardFetchKey({
      platform: platform.value,
      start: timeRange.value.start,
      end: timeRange.value.end,
      includeHeatmap: false,
    })
    const cached = providerStatsCache.get(key)
    if (cached && Date.now() - cached.ts < DASHBOARD_CACHE_TTL_MS) {
      providerStats.value = cached.payload
      return providerStats.value
    }

    const capabilities = await refreshUsageCapabilities()
    const providerCapability = capabilities?.features.provider_breakdown
    if (providerCapability && !providerCapability.supported) {
      providerStats.value = []
      providerStatsCache.set(key, { payload: providerStats.value, ts: Date.now() })
      return providerStats.value
    }

    providerStats.value = await getUsageByProviderV2(
      platform.value,
      timeRange.value.start,
      timeRange.value.end
    )
    providerStatsCache.set(key, { payload: providerStats.value, ts: Date.now() })
    return providerStats.value
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
    requestedPlatform?: UsagePlatform,
    reason: 'manual' | 'bootstrap' = 'manual'
  ): Promise<ImportAllUsageResponse> {
    importing.value = true
    isBootstrapping.value = reason === 'bootstrap'
    error.value = null

    try {
      const response = requestedPlatform
        ? normalizeImportResponse(await importUsageV2(requestedPlatform), requestedPlatform)
        : normalizeImportResponse(await importAllUsageV2())

      lastImportSummary.value = response.summary
      lastImportResults.value = response.results

      const failedResults = response.results.filter((result) => result.error)
      const failedDetails = failedResults
        .map((result) => `${result.platform}: ${result.error}`)
        .join('\n')

      if (
        response.summary.failure_count === response.results.length &&
        response.results.length > 0
      ) {
        error.value = failedDetails || '未能导入本地 usage 日志，请检查日志目录或导入错误'
        warning.value = null
        return response
      }

      if (response.summary.has_partial) {
        warning.value =
          failedDetails || '仅导入部分 usage 数据，达到时间预算或部分平台失败，可重试继续导入'
      } else if (
        response.summary.processed_files === 0 &&
        response.summary.imported_records === 0
      ) {
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
      const message = getErrorMessage(e)
      const fallback = normalizeImportResponse({
        results: [
          {
            platform: requestedPlatform ?? 'all',
            files_processed: 0,
            records_imported: 0,
            records_skipped: 0,
            duration_ms: 0,
            completed: false,
            error: message,
            is_optional_absent: false,
          },
        ],
        summary: {
          success_count: 0,
          failure_count: 1,
          imported_records: 0,
          processed_files: 0,
          // 全失败时只有一条失败结果，没有"部分成功"的语义；改成 false 与 build_import_summary 对齐。
          has_partial: false,
        },
      })

      lastImportSummary.value = fallback.summary
      lastImportResults.value = fallback.results
      warning.value = null
      error.value = fallback.summary.failure_count > 0 ? message : null
      return fallback
    } finally {
      importing.value = false
      isBootstrapping.value = false
    }
  }

  async function startImportJob(opts: {
    platform?: UsagePlatform
    recentDays?: number
    reason?: 'manual' | 'bootstrap'
    resetSources?: boolean
  }) {
    const reason = opts.reason ?? 'manual'
    activeImportReason = reason
    importing.value = true
    isBootstrapping.value = reason === 'bootstrap'
    error.value = null
    clearImportFeedback()
    resetProgressRefreshState()

    try {
      const response = await startUsageImportJobV2(
        opts.platform,
        opts.recentDays,
        opts.resetSources
      )

      syncImportFeedbackFromJob(response.snapshot)
      await ensureImportJobListeners(response.job_id)
      return response.snapshot
    } catch (e) {
      const message = getErrorMessage(e)
      error.value = message
      importing.value = false
      isBootstrapping.value = false
      activeImportReason = null
      throw e
    }
  }

  /** 初始化仪表盘并在空库时做一次自举导入 */
  async function initializeDashboard(opts: {
    platform?: UsagePlatform
    start?: string
    end?: string
  }) {
    applyFilters(opts)
    await fetchAll({ includeHeatmap: !LAZY_HEATMAP_LOAD, reason: 'initialize' })

    if (dashboardUnsupported.value) {
      return
    }

    if ((summary.value?.total_requests ?? 0) > 0) {
      return
    }

    if (bootstrapAttempted.value) {
      return
    }

    bootstrapAttempted.value = true

    const sync = syncCapability.value
    if (sync && !sync.supported) {
      return
    }

    await startImportJob({
      platform: undefined,
      reason: 'bootstrap',
      recentDays: 30,
    })
  }

  /** 设置筛选条件并刷新（300ms 防抖） */
  function setFilters(opts: { platform?: UsagePlatform; start?: string; end?: string }) {
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

  /**
   * 设置日志模型过滤器。统一通过 store action 写入，禁止外部 composable 直接 mutate
   * `store.logsModelFilter`，保持 store state 单向流动。
   */
  function setLogsModelFilter(value: string | undefined) {
    logsModelFilter.value = value
  }

  /** 启动自动刷新 */
  const coreAutoRefresh = usePolledData(
    async () => {
      recordPerfMark('usage_auto_refresh_start')
      await fetchAll({
        includeHeatmap: false,
        reason: 'auto-refresh-core',
        preserveError: Boolean(error.value && !hasUsageData.value),
      })
      recordPerfMark('usage_auto_refresh_end')
      return lastUpdated.value?.toISOString() ?? null
    },
    {
      key: 'usage:auto-refresh:core',
      intervalMs: REFRESH_INTERVAL,
      pauseWhenHidden: true,
      immediate: false,
    }
  )

  const heatmapAutoRefresh = LAZY_HEATMAP_LOAD
    ? null
    : usePolledData(
        async () => {
          await fetchHeatmap('auto-refresh-heatmap')
          return heatmap.value?.data ?? null
        },
        {
          key: 'usage:auto-refresh:heatmap',
          intervalMs: HEATMAP_REFRESH_INTERVAL,
          pauseWhenHidden: true,
          immediate: false,
        }
      )

  const shouldAutoRefreshImmediately = () => {
    if (!lastUpdated.value) return true
    return Date.now() - lastUpdated.value.getTime() >= AUTO_REFRESH_FRESHNESS_MS
  }

  function startAutoRefresh(options: { immediate?: boolean } = {}) {
    stopAutoRefresh()
    const immediate = options.immediate ?? shouldAutoRefreshImmediately()
    recordPerfMark('usage_auto_refresh_resume', { immediate })
    coreAutoRefresh.resume({ immediate })
    heatmapAutoRefresh?.resume({ immediate })
  }

  /** 停止自动刷新 */
  function stopAutoRefresh() {
    coreAutoRefresh.pause()
    heatmapAutoRefresh?.pause()
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
    providerStats,
    sourceStats,
    heatmap,
    logs,
    archive,
    snapshot,
    usageCapabilities,
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
    currentImportJob,
    platform,
    timeRange,
    logsPage,
    logsPageSize,
    logsModelFilter,
    // computed
    totalTokens,
    logsTotalPages,
    hasLogsTotal,
    canPrevLogs,
    canNextLogs,
    showLogsPager,
    hasUsageData,
    hasNoUsageData,
    dashboardCapability,
    syncCapability,
    dashboardUnsupported,
    // flags
    useDashboardApi: USE_DASHBOARD_API,
    // actions
    initializeDashboard,
    fetchAll,
    fetchProviderStats,
    refreshUsageCapabilities,
    fetchHeatmap,
    fetchLogs,
    nextLogsPage,
    prevLogsPage,
    startImportJob,
    triggerImport,
    clearImportFeedback,
    setFilters,
    setLogsModelFilter,
    startAutoRefresh,
    stopAutoRefresh,
  }
})
