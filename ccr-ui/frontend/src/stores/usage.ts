/**
 * Usage Analytics Store (V2)
 *
 * 统一管理使用统计数据，对接后端 v2 聚合 API
 * 支持 30s 自动刷新
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type {
  UsageSummary,
  DailyTrend,
  ModelStat,
  ProjectStat,
  PaginatedLogs,
  HeatmapResponse,
  ImportResult,
  Platform,
} from '@/types/usage'
import {
  getUsageSummaryV2,
  getUsageTrendsV2,
  getUsageByModelV2,
  getUsageByProjectV2,
  getUsageHeatmapV2,
  getUsageLogsV2,
  importUsageV2,
  importAllUsageV2,
} from '@/api/modules/usageV2'

const REFRESH_INTERVAL = 30_000 // 30s

export const useUsageStore = defineStore('usage', () => {
  // ═══ State ═══
  const summary = ref<UsageSummary | null>(null)
  const trends = ref<DailyTrend[]>([])
  const modelStats = ref<ModelStat[]>([])
  const projectStats = ref<ProjectStat[]>([])
  const heatmap = ref<HeatmapResponse | null>(null)
  const logs = ref<PaginatedLogs | null>(null)

  const loading = ref(false)
  const error = ref<string | null>(null)
  const lastUpdated = ref<Date | null>(null)

  // 筛选条件
  const platform = ref<Platform | undefined>(undefined)
  const timeRange = ref<{ start?: string; end?: string }>({})
  const logsPage = ref(1)
  const logsPageSize = ref(50)
  const logsModelFilter = ref<string | undefined>(undefined)

  let refreshTimer: ReturnType<typeof setInterval> | null = null

  // ═══ Computed ═══
  const totalTokens = computed(() => {
    if (!summary.value) return 0
    return summary.value.total_input_tokens + summary.value.total_output_tokens
  })

  // ═══ Actions ═══
  async function fetchSummary() {
    summary.value = await getUsageSummaryV2(platform.value, timeRange.value.start, timeRange.value.end)
  }

  async function fetchTrends() {
    trends.value = await getUsageTrendsV2(platform.value, timeRange.value.start, timeRange.value.end)
  }

  async function fetchModelStats() {
    modelStats.value = await getUsageByModelV2(platform.value, timeRange.value.start, timeRange.value.end)
  }

  async function fetchProjectStats() {
    projectStats.value = await getUsageByProjectV2(platform.value, timeRange.value.start, timeRange.value.end)
  }

  async function fetchHeatmap() {
    heatmap.value = await getUsageHeatmapV2(platform.value)
  }

  async function fetchLogs() {
    logs.value = await getUsageLogsV2(platform.value, logsPage.value, logsPageSize.value, logsModelFilter.value)
  }

  /** 并行获取所有数据 */
  async function fetchAll() {
    loading.value = true
    error.value = null
    try {
      await Promise.all([fetchSummary(), fetchTrends(), fetchModelStats(), fetchProjectStats(), fetchHeatmap()])
      lastUpdated.value = new Date()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  /** 触发数据导入 */
  async function triggerImport(p?: string): Promise<ImportResult[]> {
    if (p) {
      const r = await importUsageV2(p)
      return [r]
    }
    return importAllUsageV2()
  }

  /** 设置筛选条件并刷新 */
  function setFilters(opts: { platform?: Platform; start?: string; end?: string }) {
    platform.value = opts.platform
    timeRange.value = { start: opts.start, end: opts.end }
    fetchAll()
  }

  /** 启动自动刷新 */
  function startAutoRefresh() {
    stopAutoRefresh()
    refreshTimer = setInterval(fetchAll, REFRESH_INTERVAL)
  }

  /** 停止自动刷新 */
  function stopAutoRefresh() {
    if (refreshTimer) {
      clearInterval(refreshTimer)
      refreshTimer = null
    }
  }

  return {
    // state
    summary, trends, modelStats, projectStats, heatmap, logs,
    loading, error, lastUpdated,
    platform, timeRange, logsPage, logsPageSize, logsModelFilter,
    // computed
    totalTokens,
    // actions
    fetchAll, fetchSummary, fetchTrends, fetchModelStats, fetchProjectStats,
    fetchHeatmap, fetchLogs, triggerImport, setFilters,
    startAutoRefresh, stopAutoRefresh,
  }
})
