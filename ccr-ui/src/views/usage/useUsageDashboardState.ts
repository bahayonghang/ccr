import { computed, onActivated, onDeactivated, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useUsageStore } from '@/stores/usage'
import { ensureLocaleLoaded } from '@/i18n'
import type { UsagePlatform } from '@/types/usage'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import { perfMark, perfMeasure } from '@/utils/perfTelemetry'
import {
  DEFAULT_USAGE_RANGE_PRESET,
  getLocalDateRangeWindow,
  getUsageRangePresetImportDays,
  type UsageRangePreset,
} from './dateWindow'
import { formatCost, formatTokens } from './usageSummaryCards'
import { shortenPath } from './usageOverviewInsights'
import type { UsageOpsAction } from './usageOpsCockpit'
import { useUsageFilters } from './state/useUsageFilters'
import { useUsageLogs } from './state/useUsageLogs'
import { useUsageCharts } from './state/useUsageCharts'
import { useUsageMeta } from './state/useUsageMeta'

const USAGE_CHART_DEFER_MS = 180

type IdleCapableWindow = Window & {
  requestIdleCallback?: (callback: () => void, options?: { timeout: number }) => number
  cancelIdleCallback?: (handle: number) => void
}

const hasTemplatePlaceholder = (value: string) => /\{[a-zA-Z_][a-zA-Z0-9_]*\}/.test(value)

const getTimeRange = getLocalDateRangeWindow

const scheduleAfterPaint = (callback: () => void): (() => void) => {
  if (
    typeof window === 'undefined' ||
    typeof window.requestAnimationFrame !== 'function' ||
    typeof window.cancelAnimationFrame !== 'function'
  ) {
    callback()
    return () => {}
  }

  let cancelled = false
  const frame = window.requestAnimationFrame(() => {
    window.requestAnimationFrame(() => {
      if (!cancelled) callback()
    })
  })

  return () => {
    cancelled = true
    window.cancelAnimationFrame(frame)
  }
}

const scheduleWhenIdle = (callback: () => void, timeout = 900): (() => void) => {
  if (typeof window === 'undefined') {
    callback()
    return () => {}
  }

  const win = window as IdleCapableWindow
  let cancelled = false
  const run = () => {
    if (!cancelled) callback()
  }

  if (typeof win.requestIdleCallback === 'function') {
    const handle = win.requestIdleCallback(run, { timeout })
    return () => {
      cancelled = true
      win.cancelIdleCallback?.(handle)
    }
  }

  const timer = window.setTimeout(run, USAGE_CHART_DEFER_MS)
  return () => {
    cancelled = true
    window.clearTimeout(timer)
  }
}

export const useUsageDashboardState = () => {
  const { t, locale } = useI18n()
  const store = useUsageStore()

  const activeTab = ref<string>('overview')
  const selectedPlatform = ref('')
  const selectedRange = ref<UsageRangePreset>(DEFAULT_USAGE_RANGE_PRESET)
  const logModelFilter = ref('')
  const localeReady = ref(false)
  const dashboardBootstrapped = ref(false)
  const translationRevision = ref(0)
  const runtimeUnavailable = computed(() => !isTauriRuntime())
  const trendChartReady = ref(false)
  const distributionChartReady = ref(false)
  const modelsChartVisited = ref(false)
  const cleanups: Array<() => void> = []
  let themeObserver: MutationObserver | null = null
  let firstContentMarked = false

  const dashboardReady = computed(() => localeReady.value && dashboardBootstrapped.value)
  // 图表水合门控只跟随各自的 *Ready 单调标志，不再耦合 activeTab：
  // tab 内容改由 KeepAlive 缓存后，切走的 tab 实例仍存活于缓存容器，若门控随 activeTab
  // 翻假会把已挂载的 ApexCharts 卸载，返回时又重建，反而抵消 KeepAlive 收益。
  // *Ready 只会 false→true 单调递增，首屏仍延迟到内容就绪后再挂图（契约不变）。
  const shouldRenderTrendChart = computed(() => trendChartReady.value)
  const shouldRenderDistributionChart = computed(() => distributionChartReady.value)

  const translateDashboardText = (
    key: string,
    values: Record<string, number | string> | undefined,
    fallback: string
  ) => {
    void translationRevision.value

    if (!localeReady.value) {
      return fallback
    }

    const resolved = values ? t(key, values) : t(key)
    if (resolved === key || hasTemplatePlaceholder(resolved)) {
      return fallback
    }

    return resolved
  }

  const hydrateUsageLocale = async () => {
    localeReady.value = false
    await ensureLocaleLoaded(locale.value)
    perfMark('usage_locale_ready')
    translationRevision.value += 1
    localeReady.value = true
  }

  const markFirstContentReadyAfterPaint = () => {
    if (firstContentMarked) return

    cleanups.push(
      scheduleAfterPaint(() => {
        if (firstContentMarked) return
        firstContentMarked = true
        perfMark('usage_first_content_ready')
      })
    )
  }

  const scheduleChartHydration = () => {
    if (!dashboardReady.value || store.dashboardUnsupported || store.hasNoUsageData) {
      return
    }

    if (trendChartReady.value && distributionChartReady.value) {
      return
    }

    cleanups.push(
      scheduleAfterPaint(() => {
        cleanups.push(
          scheduleWhenIdle(() => {
            trendChartReady.value = true
            perfMark('usage_trend_chart_gate_ready')
            cleanups.push(
              scheduleWhenIdle(() => {
                distributionChartReady.value = true
                perfMark('usage_distribution_chart_gate_ready')
              })
            )
          })
        )
      })
    )
  }

  const showInstallDialog = ref(false)

  const doImport = async () => {
    // Pre-check: is llmusage available?
    try {
      const { llmusageInstallDetect } = await import('@/api/domains/install')
      const detection = await llmusageInstallDetect()
      if (detection.status === 'absent') {
        // Show install dialog instead of starting import
        showInstallDialog.value = true
        return
      }
    } catch {
      // If the check itself fails (e.g. command not registered), proceed with import
      // and let the existing error handling surface the issue.
    }

    await store.startImportJob({
      platform: undefined,
      reason: 'manual',
      recentDays: getUsageRangePresetImportDays(selectedRange.value),
    })
  }

  const doImportAfterInstall = async () => {
    showInstallDialog.value = false
    await store.startImportJob({
      platform: undefined,
      reason: 'manual',
      recentDays: getUsageRangePresetImportDays(selectedRange.value),
    })
  }

  // 子 composable 按域拆分：logs/filters/charts/meta 通过显式参数共享同一批 ref 实例，
  // 保持响应式依赖图与拆分前一致；跨域编排（生命周期、watcher、图表水合门控）留在本组合根。
  const logs = useUsageLogs({
    store,
    t,
    locale,
    selectedPlatform,
    selectedRange,
    logModelFilter,
  })

  const filters = useUsageFilters({
    store,
    translateDashboardText,
    activeTab,
    selectedPlatform,
    selectedRange,
    loadLogs: logs.loadLogs,
  })

  const { syncChartTheme, ...chartsPublic } = useUsageCharts({
    store,
    t,
    locale,
    dashboardReady,
    selectedRange,
    selectedWindowLabel: filters.selectedWindowLabel,
    translateDashboardText,
  })

  const meta = useUsageMeta({
    store,
    t,
    locale,
    dashboardReady,
    selectedPlatform,
    selectedPlatformLabel: filters.selectedPlatformLabel,
    selectedWindowLabel: filters.selectedWindowLabel,
    translateDashboardText,
  })

  watch(activeTab, (tab) => {
    if (tab === 'logs') {
      void logs.loadLogs('reset')
    }

    if (tab === 'models') {
      const firstVisit = !modelsChartVisited.value
      modelsChartVisited.value = true
      if (!distributionChartReady.value && dashboardReady.value) {
        cleanups.push(
          scheduleWhenIdle(() => {
            distributionChartReady.value = true
            perfMark('usage_models_distribution_chart_gate_ready')
          })
        )
      } else if (firstVisit && dashboardReady.value) {
        perfMark('usage_models_distribution_chart_gate_ready')
      }
    }
  })

  watch(locale, () => {
    void hydrateUsageLocale()
  })

  watch(
    [dashboardReady, () => store.dashboardUnsupported, () => store.hasNoUsageData],
    ([ready]) => {
      if (ready) {
        perfMark('usage_dashboard_ready')
        markFirstContentReadyAfterPaint()
        scheduleChartHydration()
      }
    }
  )

  const handleOpsPrimaryAction = async (action: UsageOpsAction) => {
    if (action === 'import') {
      await doImport()
      return
    }

    if (action === 'diagnostics') {
      activeTab.value = 'logs'
      await logs.loadLogs('reset')
    }
  }

  let dashboardAutoRefreshActive = false

  const startDashboardAutoRefresh = (options?: { immediate?: boolean }) => {
    if (runtimeUnavailable.value) {
      stopDashboardAutoRefresh(true)
      return
    }

    if (dashboardAutoRefreshActive) {
      return
    }

    if (options) {
      store.startAutoRefresh(options)
    } else {
      store.startAutoRefresh()
    }
    dashboardAutoRefreshActive = true
  }

  const stopDashboardAutoRefresh = (force = false) => {
    if (!dashboardAutoRefreshActive && !force) {
      return
    }

    store.stopAutoRefresh()
    dashboardAutoRefreshActive = false
  }

  onMounted(async () => {
    perfMark('usage_route_mounted')
    syncChartTheme()
    if (typeof MutationObserver !== 'undefined') {
      themeObserver = new MutationObserver(syncChartTheme)
      themeObserver.observe(document.documentElement, {
        attributes: true,
        attributeFilter: ['class', 'data-theme'],
      })
    }

    await hydrateUsageLocale()

    if (runtimeUnavailable.value) {
      dashboardBootstrapped.value = true
      store.stopAutoRefresh()
      return
    }

    const { start, end } = getTimeRange(selectedRange.value)
    await store.initializeDashboard({
      platform: (selectedPlatform.value || undefined) as UsagePlatform | undefined,
      start,
      end,
    })
    perfMark('usage_dashboard_data_ready')
    perfMeasure('usage_dashboard_boot_ms', 'usage_route_mounted', 'usage_dashboard_data_ready')
    dashboardBootstrapped.value = true
    startDashboardAutoRefresh({ immediate: false })
  })

  onActivated(() => {
    if (dashboardBootstrapped.value) {
      startDashboardAutoRefresh()
      if (activeTab.value === 'logs') {
        void logs.loadLogs('same')
      }
    }
  })

  onDeactivated(() => {
    stopDashboardAutoRefresh()
  })

  onUnmounted(() => {
    stopDashboardAutoRefresh()
    cleanups.splice(0).forEach((cleanup) => cleanup())
    themeObserver?.disconnect()
  })

  return {
    dashboardReady,
    doImport,
    doImportAfterInstall,
    formatCost,
    formatTokens,
    localeReady,
    runtimeUnavailable,
    shortenPath,
    showInstallDialog,
    store,
    handleOpsPrimaryAction,
    shouldRenderTrendChart,
    shouldRenderDistributionChart,
    ...filters,
    ...logs,
    ...chartsPublic,
    ...meta,
  }
}
