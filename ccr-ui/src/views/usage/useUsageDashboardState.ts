import { computed, onActivated, onDeactivated, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useUsageStore } from '@/stores/usage'
import { ensureLocaleLoaded } from '@/i18n'
import type { Platform, UsageFeatureCapability, UsageUnsupportedReason } from '@/types/usage'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import { perfMark, perfMeasure } from '@/utils/perfTelemetry'
import {
  buildUsageDashboardPresentation,
  selectTrendGranularity,
  type ModelDistributionSlice,
  type TrendGranularity,
} from './usageDashboardPresentation'
import { getLocalDateWindow } from './dateWindow'
import {
  buildChartTheme,
  buildTrendTooltipHtml,
  formatTrendAxisLabel,
  formatTrendTooltipLabel,
  getTrendTickAmount,
  type ApexCustomTooltipContext,
  type ApexFormatterContext,
  type ChartThemeState,
  type ModelDistributionMetric,
} from './usageChartOptions'
import {
  buildUsageDiagnosticsSummary,
  type UsageDiagnosticsSummary,
} from './usageDiagnostics'
import {
  formatCost,
  formatTokens,
} from './usageSummaryCards'
import {
  buildDashboardMetaItems,
  buildSelectedPlatformLabel,
  buildSelectedWindowLabel,
  shortenPath,
  type DashboardMetaItem,
  type OverviewRankItem,
} from './usageOverviewInsights'

const USAGE_CHART_DEFER_MS = 180

type IdleCapableWindow = Window & {
  requestIdleCallback?: (callback: () => void, options?: { timeout: number }) => number
  cancelIdleCallback?: (handle: number) => void
}

const hasTemplatePlaceholder = (value: string) => /\{[a-zA-Z_][a-zA-Z0-9_]*\}/.test(value)

const normalizeUnsupportedReason = (reason: UsageUnsupportedReason | null | undefined) => {
  if (
    reason === 'cli_missing' ||
    reason === 'db_missing' ||
    reason === 'db_unreadable' ||
    reason === 'schema_unsupported' ||
    reason === 'missing_table' ||
    reason === 'missing_column' ||
    reason === 'waiting_for_llmusage'
  ) {
    return reason
  }

  return 'waiting_for_llmusage'
}

const getTimeRange = getLocalDateWindow

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

  const tabKeys = ['overview', 'models', 'projects', 'logs'] as const
  const activeTab = ref<string>('overview')
  const selectedPlatform = ref('')
  const selectedDays = ref(30)
  const logModelFilter = ref('')
  const localeReady = ref(false)
  const dashboardBootstrapped = ref(false)
  const translationRevision = ref(0)
  const runtimeUnavailable = computed(() => !isTauriRuntime())
  const chartTheme = ref<ChartThemeState>(buildChartTheme())
  const trendChartReady = ref(false)
  const distributionChartReady = ref(false)
  const modelsChartVisited = ref(false)
  const cleanups: Array<() => void> = []
  let themeObserver: MutationObserver | null = null
  let firstContentMarked = false

  const dashboardReady = computed(() => localeReady.value && dashboardBootstrapped.value)
  const shouldRenderTrendChart = computed(
    () => activeTab.value === 'overview' && trendChartReady.value
  )
  const shouldRenderDistributionChart = computed(
    () =>
      (activeTab.value === 'overview' && distributionChartReady.value) ||
      (activeTab.value === 'models' && modelsChartVisited.value && distributionChartReady.value)
  )

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

    cleanups.push(scheduleAfterPaint(() => {
      if (firstContentMarked) return
      firstContentMarked = true
      perfMark('usage_first_content_ready')
    }))
  }

  const scheduleChartHydration = () => {
    if (!dashboardReady.value || store.dashboardUnsupported || store.hasNoUsageData) {
      return
    }

    if (trendChartReady.value && distributionChartReady.value) {
      return
    }

    cleanups.push(scheduleAfterPaint(() => {
      cleanups.push(scheduleWhenIdle(() => {
        trendChartReady.value = true
        perfMark('usage_trend_chart_gate_ready')
        cleanups.push(scheduleWhenIdle(() => {
          distributionChartReady.value = true
          perfMark('usage_distribution_chart_gate_ready')
        }))
      }))
    }))
  }

  const onFilterChange = () => {
    const { start, end } = getTimeRange(selectedDays.value)
    store.setFilters({
      platform: (selectedPlatform.value || undefined) as Platform | undefined,
      start,
      end,
    })

    if (activeTab.value === 'logs') {
      void loadLogs('reset')
    }
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
      recentDays: selectedDays.value,
    })
  }

  const doImportAfterInstall = async () => {
    showInstallDialog.value = false
    await store.startImportJob({
      platform: undefined,
      reason: 'manual',
      recentDays: selectedDays.value,
    })
  }

  const loadLogs = async (direction: 'reset' | 'next' | 'prev' | 'same' = 'reset') => {
    store.setLogsModelFilter(logModelFilter.value || undefined)
    await store.fetchLogs(direction)
  }

  const updateLogModelFilter = (value: string) => {
    logModelFilter.value = value
  }

  watch(activeTab, (tab) => {
    if (tab === 'logs') {
      void loadLogs('reset')
    }

    if (tab === 'models') {
      const firstVisit = !modelsChartVisited.value
      modelsChartVisited.value = true
      if (!distributionChartReady.value && dashboardReady.value) {
        cleanups.push(scheduleWhenIdle(() => {
          distributionChartReady.value = true
          perfMark('usage_models_distribution_chart_gate_ready')
        }))
      } else if (firstVisit && dashboardReady.value) {
        perfMark('usage_models_distribution_chart_gate_ready')
      }
    }
  })

  watch(locale, () => {
    void hydrateUsageLocale()
  })

  watch([dashboardReady, () => store.dashboardUnsupported, () => store.hasNoUsageData], ([ready]) => {
    if (ready) {
      perfMark('usage_dashboard_ready')
      markFirstContentReadyAfterPaint()
      scheduleChartHydration()
    }
  })

  const selectedPlatformLabel = computed(() =>
    buildSelectedPlatformLabel(selectedPlatform.value, translateDashboardText)
  )

  const selectedWindowLabel = computed(() =>
    buildSelectedWindowLabel(selectedDays.value, translateDashboardText)
  )

  const dashboardMetaItems = computed<DashboardMetaItem[]>(() => {
    if (!dashboardReady.value) return []

    return buildDashboardMetaItems({
      archive: store.archive,
      locale: locale.value,
      modelCount: store.modelStats.length,
      projectCount: store.projectStats.length,
      selectedPlatformLabel: selectedPlatformLabel.value,
      selectedWindowLabel: selectedWindowLabel.value,
      translate: translateDashboardText,
    })
  })

  const trendGranularity = computed(() => selectTrendGranularity(selectedDays.value))

  const trendGranularityLabel = computed(() => {
    const fallbacks: Record<TrendGranularity, string> = {
      day: 'Daily',
      week: 'Weekly',
      month: 'Monthly',
    }

    return translateDashboardText(
      `usage.dashboard.chart.bucket.${trendGranularity.value}`,
      undefined,
      fallbacks[trendGranularity.value]
    )
  })

  const dashboardPresentation = computed(() =>
    buildUsageDashboardPresentation({
      modelStats: dashboardReady.value ? store.modelStats : [],
      projectStats: dashboardReady.value ? store.projectStats : [],
      selectedWindowLabel: selectedWindowLabel.value,
      summary: dashboardReady.value ? store.summary : null,
      translate: translateDashboardText,
      trendGranularity: trendGranularity.value,
      trendGranularityLabel: trendGranularityLabel.value,
      trends: dashboardReady.value ? store.trends : [],
    })
  )

  const trendBuckets = computed(() => dashboardPresentation.value.trendBuckets)
  const summaryCards = computed(() => dashboardPresentation.value.summaryCards)
  const trendSeries = computed(() => dashboardPresentation.value.trendSeries)

  const hasRenderableTrendData = computed(
    () => dashboardReady.value && trendSeries.value.some((series) => series.data.length > 0)
  )

  const trendSubtitle = computed(() => {
    if (!dashboardReady.value) return ''

    return translateDashboardText(
      'usage.dashboard.chart.trendSubtitle',
      {
        granularity: trendGranularityLabel.value,
        window: selectedWindowLabel.value,
        points: trendBuckets.value.length,
      },
      `${selectedWindowLabel.value} · ${trendGranularityLabel.value} · ${trendBuckets.value.length} points`
    )
  })

  const chartBaseOptions = () => ({
    background: 'transparent',
    fontFamily: 'inherit',
    parentHeightOffset: 0,
    redrawOnParentResize: false,
    redrawOnWindowResize: false,
    animations: { enabled: false },
  })

  const trendOptions = computed(() => ({
    chart: {
      ...chartBaseOptions(),
      toolbar: { show: false },
    },
    theme: { mode: chartTheme.value.mode },
    colors: [chartTheme.value.primary, chartTheme.value.secondary, chartTheme.value.tertiary],
    xaxis: {
      type: 'datetime' as const,
      tickAmount: getTrendTickAmount(trendBuckets.value.length),
      labels: {
        style: { colors: chartTheme.value.textMuted, fontSize: '11px' },
        datetimeUTC: false,
        formatter: (_value: string, timestamp?: number) => {
          if (timestamp == null) return ''
          return formatTrendAxisLabel(timestamp, trendGranularity.value, locale.value)
        },
      },
      axisBorder: { show: false },
      axisTicks: { color: chartTheme.value.grid },
    },
    yaxis: [
      {
        seriesName: trendSeries.value[0]?.name,
        labels: {
          style: { colors: chartTheme.value.textMuted },
          formatter: (value: number) => formatTokens(value),
        },
      },
      {
        seriesName: trendSeries.value[1]?.name,
        opposite: true,
        showAlways: true,
        labels: {
          style: { colors: chartTheme.value.textMuted },
          formatter: (value: number) => formatTokens(value),
        },
      },
      {
        seriesName: trendSeries.value[2]?.name,
        show: false,
        labels: {
          style: { colors: chartTheme.value.textMuted },
          formatter: (value: number) => formatTokens(value),
        },
      },
    ],
    markers: {
      size: 0,
      hover: {
        size: 0,
        sizeOffset: 0,
      },
    },
    stroke: { curve: 'smooth' as const, width: 2.2 },
    fill: { type: 'gradient', gradient: { opacityFrom: 0.32, opacityTo: 0.04 } },
    dataLabels: { enabled: false },
    tooltip: {
      theme: chartTheme.value.mode,
      shared: true,
      intersect: false,
      custom: (context: ApexCustomTooltipContext) => {
        return buildTrendTooltipHtml({
          context,
          buckets: trendBuckets.value,
          granularity: trendGranularity.value,
          locale: locale.value,
          seriesNames: trendSeries.value.map((series) => series.name),
          fallbackColor: chartTheme.value.primary,
          costLabel: translateDashboardText('usage.dashboard.table.cost', undefined, 'Cost'),
          formatTokens,
          formatCost,
        })
      },
      x: {
        // ApexCharts 把 formatter 的 context 类型标成 any，这里收口到本地 ApexFormatterContext。
        formatter: (_value: string, context: ApexFormatterContext) => {
          const bucket = trendBuckets.value[context?.dataPointIndex ?? -1]
          if (!bucket) return _value
          return formatTrendTooltipLabel(
            bucket.startDate,
            bucket.displayEndDate,
            trendGranularity.value,
            locale.value
          )
        },
      },
    },
    grid: {
      borderColor: chartTheme.value.grid,
      strokeDashArray: 4,
      padding: { left: 4, right: 6, bottom: 2, top: 6 },
    },
    legend: {
      show: true,
      showForSingleSeries: true,
      position: 'top' as const,
      horizontalAlign: 'right' as const,
      labels: { colors: chartTheme.value.textSecondary },
      markers: { strokeWidth: 0 },
    },
  }))

  const modelDistribution = computed(() => dashboardPresentation.value.modelDistribution)
  const modelTokenDistribution = computed(() => dashboardPresentation.value.modelTokenDistribution)
  const pieSeries = computed(() => dashboardPresentation.value.pieSeries)
  const modelTokenPieSeries = computed(() => dashboardPresentation.value.modelTokenPieSeries)

  const pieColors = computed(() => {
    const palette = [
      chartTheme.value.primary,
      chartTheme.value.secondary,
      chartTheme.value.tertiary,
      chartTheme.value.quaternary,
      '#D8B4FE',
      '#FDA4AF',
      '#2DD4BF',
      '#93C5FD',
    ]

    return palette.slice(
      0,
      Math.max(modelDistribution.value.length, modelTokenDistribution.value.length, 1)
    )
  })

  const distributionSubtitle = computed(() => {
    if (!dashboardReady.value) return ''

    if (store.modelStats.length <= 6) {
      return translateDashboardText(
        'usage.dashboard.chart.distributionAllVisible',
        {
          total: store.modelStats.length,
        },
        `${store.modelStats.length} models visible in this window`
      )
    }

    return translateDashboardText(
      'usage.dashboard.chart.distributionSubtitle',
      {
        visible: 6,
        total: store.modelStats.length - 6,
      },
      `Showing the top 6 models; ${store.modelStats.length - 6} grouped into Others`
    )
  })

  const buildDistributionPieOptions = (
    metric: ModelDistributionMetric,
    distribution: ModelDistributionSlice[],
  ) => ({
    chart: chartBaseOptions(),
    theme: { mode: chartTheme.value.mode },
    colors: pieColors.value,
    labels: distribution.map((item) => item.label),
    legend: { show: false },
    plotOptions: {
      pie: {
        donut: {
          size: '72%',
          labels: {
            show: true,
            name: {
              show: true,
              fontSize: '11px',
              color: chartTheme.value.textMuted,
              offsetY: -2,
            },
            value: {
              show: true,
              fontSize: '15px',
              fontWeight: 600,
              color: chartTheme.value.textPrimary,
              formatter: (value: string) =>
                metric === 'tokens' ? formatTokens(Number(value)) : formatCost(Number(value)),
            },
            total: {
              show: true,
              label: t(
                metric === 'tokens'
                  ? 'usage.dashboard.cards.totalTokens'
                  : 'usage.dashboard.cards.totalCost'
              ),
              fontSize: '10px',
              color: chartTheme.value.textMuted,
              formatter: (context: ApexFormatterContext) =>
                metric === 'tokens'
                  ? formatTokens(
                      (context.globals?.seriesTotals ?? []).reduce(
                        (sum: number, item: number) => sum + item,
                        0,
                      )
                    )
                  : formatCost(
                      (context.globals?.seriesTotals ?? []).reduce(
                        (sum: number, item: number) => sum + item,
                        0,
                      )
                    ),
            },
          },
        },
      },
    },
    dataLabels: {
      enabled: true,
      formatter: (
        _: number,
        options: { seriesIndex: number; w: { globals: { series: number[] } } }
      ) => {
        const total = options.w.globals.series.reduce((sum: number, item: number) => sum + item, 0)
        if (total <= 0) return '0%'

        const percent = (options.w.globals.series[options.seriesIndex] / total) * 100
        return percent >= 7 ? `${percent.toFixed(0)}%` : ''
      },
      style: { fontSize: '11px', fontWeight: 600 },
      dropShadow: { enabled: false },
    },
    tooltip: {
      theme: chartTheme.value.mode,
      y: {
        formatter: (value: number) =>
          metric === 'tokens' ? formatTokens(value) : formatCost(value),
      },
    },
  })

  const pieOptions = computed(() => buildDistributionPieOptions('cost', modelDistribution.value))
  const modelTokenPieOptions = computed(() =>
    buildDistributionPieOptions('tokens', modelTokenDistribution.value)
  )

  const overviewHighlights = computed(() => dashboardPresentation.value.overviewHighlights)

  const topModelRankings = computed<OverviewRankItem[]>(() =>
    dashboardPresentation.value.topModelRankings
  )

  const topProjectRankings = computed<OverviewRankItem[]>(() =>
    dashboardPresentation.value.topProjectRankings
  )
  const logsRecords = computed(() => store.logs?.records ?? [])
  const unknownModelStat = computed(
    () => store.modelStats.find((item) => item.model === 'unknown') ?? null
  )
  const logsTotalCount = computed(() => store.logs?.total ?? logsRecords.value.length)
  const diagnosticsSummary = computed<UsageDiagnosticsSummary>(() =>
    buildUsageDiagnosticsSummary({
      selectedPlatform: selectedPlatform.value as Platform | '',
      summary: store.summary,
      logsRecords: logsRecords.value,
      logsTotalCount: logsTotalCount.value,
      unknownModelStat: unknownModelStat.value,
      archive: store.archive,
      locale: locale.value,
      messages: {
        noRecentRecord: t('usage.dashboard.diagnostics.noRecentRecord'),
        rawLogsHint: t('usage.dashboard.diagnostics.rawLogsHint'),
        repairNeeded: t('usage.dashboard.diagnostics.repairNeeded'),
        codexRepairHint: (unknown) => t('usage.dashboard.diagnostics.codexRepairHint', { unknown }),
        healthy: t('usage.dashboard.diagnostics.healthy'),
      },
    })
  )
  const diagnosticsEmptyMessage = computed(() =>
    logModelFilter.value
      ? t('usage.dashboard.diagnostics.filteredNoResults')
      : t('usage.dashboard.logs.noLogs')
  )
  const diagnosticsEmptyDetail = computed(() =>
    logModelFilter.value
      ? t('usage.dashboard.diagnostics.filteredNoResultsHint')
      : t('usage.dashboard.diagnostics.emptyHint')
  )
  const repairCodexButtonLabel = computed(() =>
    store.importing
      ? t('usage.dashboard.diagnostics.repairingCodex')
      : t('usage.dashboard.diagnostics.repairCodex')
  )
  const repairCodexLogs = async () => {
    if (selectedPlatform.value !== 'codex' || store.importing) return

    await store.startImportJob({
      platform: 'codex',
      reason: 'manual',
      recentDays: selectedDays.value,
      resetSources: true,
    })
  }

  const importButtonLabel = computed(() => {
    if (store.isBootstrapping) return t('usage.dashboard.bootstrapping')
    if (store.importing) return t('usage.dashboard.importing')
    return t('usage.dashboard.import')
  })

  const dashboardUnsupportedCapability = computed<UsageFeatureCapability | null>(() => {
    const capability = store.dashboardCapability
    if (!capability || capability.supported) return null
    return capability
  })

  const syncUnsupportedCapability = computed<UsageFeatureCapability | null>(() => {
    const capability = store.syncCapability
    if (!capability || capability.supported) return null
    return capability
  })

  const unsupportedStateTitle = computed(() => {
    const reason = normalizeUnsupportedReason(dashboardUnsupportedCapability.value?.reason)
    return translateDashboardText(
      `usage.unsupported.${reason}.title`,
      undefined,
      translateDashboardText(
        'usage.unsupported.waiting_for_llmusage.title',
        undefined,
        'Waiting for llmusage support'
      )
    )
  })

  const unsupportedStateDescription = computed(() => {
    const capability = dashboardUnsupportedCapability.value
    const reason = normalizeUnsupportedReason(capability?.reason)
    const translated = translateDashboardText(
      `usage.unsupported.${reason}.description`,
      undefined,
      translateDashboardText(
        'usage.unsupported.waiting_for_llmusage.description',
        undefined,
        'The installed llmusage runtime does not expose this usage view yet.'
      )
    )

    return capability?.detail ? `${translated} ${capability.detail}` : translated
  })

  const unsupportedSyncMessage = computed(() => {
    const capability = syncUnsupportedCapability.value
    if (!capability) return null

    const reason = normalizeUnsupportedReason(capability.reason)
    const translated = translateDashboardText(
      `usage.unsupported.${reason}.description`,
      undefined,
      translateDashboardText(
        'usage.unsupported.cli_missing.description',
        undefined,
        'Install llmusage and run a sync before usage data can be imported.'
      )
    )

    return capability.detail ? `${translated} ${capability.detail}` : translated
  })

  const importDetails = computed(() =>
    store.lastImportResults
      .filter((result) => result.error)
      .map((result) => `${result.platform}: ${result.error}`)
  )

  const warningMessage = computed(() => store.warning || null)
  const importJobBanner = computed(() => {
    const job = store.currentImportJob
    if (!job || !store.importing) return null

    const totalFiles = Math.max(job.files_total, job.files_scanned)
    const params = {
      scanned: job.files_scanned,
      total: totalFiles,
      records: job.records_imported.toLocaleString(),
    }

    return job.status === 'recent_ready'
      ? t('usage.dashboard.importJobBanner.recent', params)
      : t('usage.dashboard.importJobBanner.running', params)
  })
  const importJobWarnings = computed(() => store.currentImportJob?.warnings ?? [])
  const showEmptyState = computed(() => store.hasNoUsageData)

  const emptyStateTitle = computed(() => {
    if (
      store.lastImportSummary &&
      store.lastImportSummary.processed_files === 0 &&
      store.lastImportSummary.imported_records === 0
    ) {
      return t('usage.dashboard.status.noLogsTitle')
    }
    return t('usage.states.noData')
  })

  const emptyStateDescription = computed(() => {
    if (
      store.lastImportSummary &&
      store.lastImportSummary.processed_files === 0 &&
      store.lastImportSummary.imported_records === 0
    ) {
      return t('usage.dashboard.status.noLogs')
    }
    return t('usage.states.noDataHint', { platform: selectedPlatform.value || 'AI' })
  })

  const syncChartTheme = () => {
    chartTheme.value = buildChartTheme()
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

    const { start, end } = getTimeRange(selectedDays.value)
    await store.initializeDashboard({
      platform: (selectedPlatform.value || undefined) as Platform | undefined,
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
        void loadLogs('same')
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
    activeTab,
    dashboardReady,
    dashboardMetaItems,
    doImport,
    doImportAfterInstall,
    emptyStateDescription,
    emptyStateTitle,
    formatCost,
    formatTokens,
    importButtonLabel,
    importDetails,
    importJobBanner,
    importJobWarnings,
    unsupportedStateDescription,
    unsupportedStateTitle,
    unsupportedSyncMessage,
    loadLogs,
    diagnosticsEmptyDetail,
    diagnosticsEmptyMessage,
    diagnosticsSummary,
    logsRecords,
    logModelFilter,
    onFilterChange,
    overviewHighlights,
    pieColors,
    pieOptions,
    pieSeries,
    hasRenderableTrendData,
    localeReady,
    runtimeUnavailable,
    selectedDays,
    selectedPlatformLabel,
    selectedPlatform,
    selectedWindowLabel,
    repairCodexButtonLabel,
    repairCodexLogs,
    shortenPath,
    shouldRenderDistributionChart,
    shouldRenderTrendChart,
    showEmptyState,
    showInstallDialog,
    store,
    trendSubtitle,
    summaryCards,
    tabKeys,
    trendGranularityLabel,
    trendOptions,
    trendSeries,
    topModelRankings,
    topProjectRankings,
    distributionSubtitle,
    modelDistribution,
    modelTokenDistribution,
    modelTokenPieOptions,
    modelTokenPieSeries,
    updateLogModelFilter,
    warningMessage,
  }
}
