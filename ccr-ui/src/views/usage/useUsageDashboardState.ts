import { computed, onActivated, onDeactivated, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useUsageStore } from '@/stores/usage'
import { ensureLocaleLoaded } from '@/i18n'
import type { Platform, UsageFeatureCapability, UsageUnsupportedReason } from '@/types/usage'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import {
  aggregateDailyTrends,
  expandTrendBucketEnd,
  groupModelDistribution,
  groupModelTokenDistribution,
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
  buildSummarySparklinePoints,
  buildUsageSummaryCards,
  formatCost,
  formatTokens,
  type UsageSummaryCard,
} from './usageSummaryCards'
import {
  buildDashboardMetaItems,
  buildOverviewHighlights,
  buildSelectedPlatformLabel,
  buildSelectedWindowLabel,
  buildTopModelRankings,
  buildTopProjectRankings,
  shortenPath,
  type DashboardMetaItem,
  type OverviewRankItem,
} from './usageOverviewInsights'

const outputSeriesName = 'Output'

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
  let themeObserver: MutationObserver | null = null

  const dashboardReady = computed(() => localeReady.value && dashboardBootstrapped.value)
  const shouldLoadCharts = computed(
    () => activeTab.value === 'overview' || activeTab.value === 'models'
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
    translationRevision.value += 1
    localeReady.value = true
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
  })

  watch(locale, () => {
    void hydrateUsageLocale()
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

  const trendBuckets = computed(() =>
    aggregateDailyTrends(store.trends, trendGranularity.value).map((bucket) => ({
      ...bucket,
      displayEndDate: expandTrendBucketEnd(bucket, trendGranularity.value),
    }))
  )

  const summarySparklinePoints = computed(() => buildSummarySparklinePoints(trendBuckets.value))

  const summaryCards = computed<UsageSummaryCard[]>(() => {
    if (!dashboardReady.value) return []

    const summary = store.summary
    if (!summary) return []

    return buildUsageSummaryCards({
      summary,
      modelCount: store.modelStats.length,
      projectCount: store.projectStats.length,
      sparklinePoints: summarySparklinePoints.value,
      selectedWindowLabel: selectedWindowLabel.value,
      translate: translateDashboardText,
    })
  })

  const trendSeries = computed(() => {
    const inputName = translateDashboardText('usage.dashboard.chart.input', undefined, 'Input')
    const outputName = translateDashboardText('usage.dashboard.chart.output', undefined, outputSeriesName)
    const cacheName = translateDashboardText('usage.dashboard.chart.cache', undefined, 'Cache Read')

    return [
      {
        name: inputName,
        data: trendBuckets.value.map((item) => ({
          x: `${item.startDate}T00:00:00Z`,
          y: item.inputTokens,
        })),
      },
      {
        name: outputName,
        data: trendBuckets.value.map((item) => ({
          x: `${item.startDate}T00:00:00Z`,
          y: item.outputTokens,
        })),
      },
      {
        name: cacheName,
        data: trendBuckets.value.map((item) => ({
          x: `${item.startDate}T00:00:00Z`,
          y: item.cacheReadTokens,
        })),
      },
    ]
  })

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

  const trendOptions = computed(() => ({
    chart: {
      background: 'transparent',
      toolbar: { show: false },
      fontFamily: 'inherit',
      parentHeightOffset: 0,
      redrawOnParentResize: true,
      redrawOnWindowResize: true,
      animations: {
        enabled: true,
        speed: 220,
        easing: 'easeinout',
      },
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

  const modelDistribution = computed(() =>
    groupModelDistribution(store.modelStats, 6).map((item) => ({
      ...item,
      label: item.isOther
        ? translateDashboardText('usage.dashboard.chart.others', undefined, 'Others')
        : item.label,
    }))
  )

  const modelTokenDistribution = computed(() =>
    groupModelTokenDistribution(store.modelStats, 6).map((item) => ({
      ...item,
      label: item.isOther
        ? translateDashboardText('usage.dashboard.chart.others', undefined, 'Others')
        : item.label,
    }))
  )

  const pieSeries = computed(() => modelDistribution.value.map((item) => item.totalCost))
  const modelTokenPieSeries = computed(() =>
    modelTokenDistribution.value.map((item) => item.totalTokens)
  )

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
    chart: { background: 'transparent', fontFamily: 'inherit' },
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

  const overviewHighlights = computed(() => {
    if (!dashboardReady.value) return []

    return buildOverviewHighlights({
      modelStats: store.modelStats,
      projectStats: store.projectStats,
      summary: store.summary,
      trendBuckets: trendBuckets.value,
      trendGranularityLabel: trendGranularityLabel.value,
      selectedWindowLabel: selectedWindowLabel.value,
      translate: translateDashboardText,
    })
  })

  const topModelRankings = computed<OverviewRankItem[]>(() => {
    if (!dashboardReady.value) return []
    return buildTopModelRankings(store.modelStats, translateDashboardText)
  })

  const topProjectRankings = computed<OverviewRankItem[]>(() => {
    if (!dashboardReady.value) return []
    return buildTopProjectRankings(store.projectStats, translateDashboardText)
  })

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

  const startDashboardAutoRefresh = () => {
    if (runtimeUnavailable.value) {
      stopDashboardAutoRefresh(true)
      return
    }

    if (dashboardAutoRefreshActive) {
      return
    }

    store.startAutoRefresh()
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
    dashboardBootstrapped.value = true
    startDashboardAutoRefresh()
  })

  onActivated(() => {
    if (dashboardBootstrapped.value) {
      startDashboardAutoRefresh()
    }
  })

  onDeactivated(() => {
    stopDashboardAutoRefresh()
  })

  onUnmounted(() => {
    stopDashboardAutoRefresh()
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
    shouldLoadCharts,
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
