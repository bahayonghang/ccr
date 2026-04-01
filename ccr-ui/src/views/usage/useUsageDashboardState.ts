import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useUsageStore } from '@/stores/usage'
import type { Platform } from '@/types/usage'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import {
  aggregateDailyTrends,
  expandTrendBucketEnd,
  groupModelDistribution,
  selectTrendGranularity,
  type TrendGranularity,
} from './usageDashboardPresentation'

type UsageSummaryCardTone = 'rose' | 'violet' | 'sky' | 'amber'

type UsageSummaryCard = {
  id: string
  label: string
  value: string
  detail: string
  icon: string
  tone: UsageSummaryCardTone
}

type DashboardMetaItem = {
  id: string
  label: string
  value: string
}

type OverviewRankItem = {
  id: string
  label: string
  title: string
  detail: string
  value: string
  share: number
}

type ChartThemeState = {
  mode: 'light' | 'dark'
  primary: string
  secondary: string
  tertiary: string
  quaternary: string
  textPrimary: string
  textSecondary: string
  textMuted: string
  grid: string
  border: string
}

type UsageDiagnosticsSummary = {
  totalRecords: string
  latestRecordAt: string
  healthLabel: string
  healthDetail: string
  repairRecommended: boolean
  canRepairCodex: boolean
}

const formatTokens = (value: number) => (
  value >= 1e6 ? `${(value / 1e6).toFixed(1)}M`
    : value >= 1e3 ? `${(value / 1e3).toFixed(1)}K`
      : value.toString()
)

const formatCost = (value: number) => (
  value >= 1 ? `$${value.toFixed(2)}` : `$${value.toFixed(4)}`
)

const formatPercent = (value: number) => `${(value * 100).toFixed(1)}%`
const formatDateTime = (value: string, locale: string) =>
  new Date(value).toLocaleString(locale)

const shortenPath = (path: string) => {
  const parts = path.replace(/\\/g, '/').split('/')
  return parts.length > 2 ? `.../${parts.slice(-2).join('/')}` : path
}

const getTimeRange = (days: number) => {
  const end = new Date()
  const start = new Date(end.getTime() - days * 86400000)
  return { start: start.toISOString().slice(0, 10), end: end.toISOString().slice(0, 10) }
}

const parseUtcDate = (value: string) => {
  const [year, month, day] = value.split('-').map(Number)
  return new Date(Date.UTC(year, (month || 1) - 1, day || 1))
}

const buildDateFormatters = (locale: string) => ({
  day: new Intl.DateTimeFormat(locale, { month: 'short', day: 'numeric', timeZone: 'UTC' }),
  month: new Intl.DateTimeFormat(locale, { month: 'short', year: 'numeric', timeZone: 'UTC' }),
})

const formatTrendAxisLabel = (
  value: number,
  granularity: TrendGranularity,
  locale: string,
) => {
  const formatters = buildDateFormatters(locale)
  const date = new Date(value)

  if (Number.isNaN(date.getTime())) return ''
  if (granularity === 'month') return formatters.month.format(date)
  return formatters.day.format(date)
}

const formatTrendTooltipLabel = (
  startDate: string,
  endDate: string,
  granularity: TrendGranularity,
  locale: string,
) => {
  const formatters = buildDateFormatters(locale)
  const start = parseUtcDate(startDate)
  const end = parseUtcDate(endDate)

  if (granularity === 'month') {
    return formatters.month.format(start)
  }

  const startLabel = formatters.day.format(start)
  const endLabel = formatters.day.format(end)
  return startLabel === endLabel ? startLabel : `${startLabel} - ${endLabel}`
}

const getTrendTickAmount = (pointCount: number) => {
  if (pointCount <= 0) return undefined
  if (pointCount <= 8) return pointCount
  if (pointCount <= 16) return 8
  return 6
}

const readCssVar = (name: string, fallback: string) => {
  if (typeof window === 'undefined' || typeof document === 'undefined') {
    return fallback
  }

  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
  return value || fallback
}

const detectThemeMode = (): 'light' | 'dark' => {
  if (typeof document === 'undefined') {
    return 'light'
  }

  return document.documentElement.getAttribute('data-theme') === 'dark' ||
    document.documentElement.classList.contains('dark')
    ? 'dark'
    : 'light'
}

const buildChartTheme = (): ChartThemeState => ({
  mode: detectThemeMode(),
  primary: readCssVar('--color-accent-primary', '#CA8FD1'),
  secondary: readCssVar('--color-accent-secondary', '#7F78D8'),
  tertiary: readCssVar('--color-info', '#69A8D8'),
  quaternary: readCssVar('--color-warning', '#D6A143'),
  textPrimary: readCssVar('--color-text-primary', '#1D1A27'),
  textSecondary: readCssVar('--color-text-secondary', '#5C5470'),
  textMuted: readCssVar('--color-text-muted', '#7F7694'),
  grid: readCssVar('--color-border-subtle', 'rgb(210 202 218 / 28%)'),
  border: readCssVar('--color-border-default', 'rgb(179 166 194 / 44%)'),
})

export const useUsageDashboardState = () => {
  const { t, locale } = useI18n()
  const store = useUsageStore()

  const tabKeys = ['overview', 'models', 'projects', 'logs'] as const
  const activeTab = ref<string>('overview')
  const selectedPlatform = ref('')
  const selectedDays = ref(30)
  const logModelFilter = ref('')
  const runtimeUnavailable = computed(() => !isTauriRuntime())
  const chartTheme = ref<ChartThemeState>(buildChartTheme())
  let themeObserver: MutationObserver | null = null

  const shouldLoadCharts = computed(() => activeTab.value === 'overview' || activeTab.value === 'models')

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

  const doImport = async () => {
    await store.startImportJob({
      platform: undefined,
      reason: 'manual',
      recentDays: selectedDays.value,
    })
  }

  const loadLogs = async (direction: 'reset' | 'next' | 'prev' | 'same' = 'reset') => {
    store.logsModelFilter = logModelFilter.value || undefined
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

  const selectedPlatformLabel = computed(() => {
    if (!selectedPlatform.value) {
      return t('usage.dashboard.allPlatforms')
    }

    return t(`usage.platforms.${selectedPlatform.value}`)
  })

  const summaryCards = computed<UsageSummaryCard[]>(() => {
    const summary = store.summary
    if (!summary) return []

    const totalTokens = summary.total_input_tokens + summary.total_output_tokens
    const averageCostPerRequest = summary.total_requests > 0
      ? formatCost(summary.total_cost_usd / summary.total_requests)
      : formatCost(0)

    return [
      {
        id: 'requests',
        label: t('usage.dashboard.cards.totalRequests'),
        value: summary.total_requests.toLocaleString(),
        detail: t('usage.dashboard.cards.requestsDetail', {
          models: store.modelStats.length,
          projects: store.projectStats.length,
        }),
        icon: 'Activity',
        tone: 'rose',
      },
      {
        id: 'tokens',
        label: t('usage.dashboard.cards.totalTokens'),
        value: formatTokens(totalTokens),
        detail: t('usage.dashboard.cards.tokensDetail', {
          input: formatTokens(summary.total_input_tokens),
          output: formatTokens(summary.total_output_tokens),
        }),
        icon: 'Layers',
        tone: 'violet',
      },
      {
        id: 'cost',
        label: t('usage.dashboard.cards.totalCost'),
        value: formatCost(summary.total_cost_usd),
        detail: t('usage.dashboard.cards.costDetail', {
          average: averageCostPerRequest,
        }),
        icon: 'Wallet',
        tone: 'sky',
      },
      {
        id: 'cache',
        label: t('usage.dashboard.cards.cacheEfficiency'),
        value: formatPercent(summary.cache_efficiency),
        detail: t('usage.dashboard.cards.cacheDetail', {
          tokens: formatTokens(summary.total_cache_read_tokens),
        }),
        icon: 'Cpu',
        tone: 'amber',
      },
    ]
  })

  const selectedWindowLabel = computed(() => {
    const labels: Record<number, string> = {
      7: t('usage.dashboard.days7'),
      30: t('usage.dashboard.days30'),
      90: t('usage.dashboard.days90'),
      365: t('usage.dashboard.days365'),
    }

    return labels[selectedDays.value] ?? `${selectedDays.value}d`
  })

  const dashboardMetaItems = computed<DashboardMetaItem[]>(() => [
    {
      id: 'scope',
      label: t('usage.dashboard.meta.scope'),
      value: selectedPlatformLabel.value,
    },
    {
      id: 'window',
      label: t('usage.dashboard.meta.window'),
      value: selectedWindowLabel.value,
    },
    {
      id: 'models',
      label: t('usage.dashboard.meta.models'),
      value: store.modelStats.length.toLocaleString(),
    },
    {
      id: 'projects',
      label: t('usage.dashboard.meta.projects'),
      value: store.projectStats.length.toLocaleString(),
    },
  ])

  const trendGranularity = computed(() => selectTrendGranularity(selectedDays.value))

  const trendBuckets = computed(() =>
    aggregateDailyTrends(store.trends, trendGranularity.value).map((bucket) => ({
      ...bucket,
      displayEndDate: expandTrendBucketEnd(bucket, trendGranularity.value),
    })),
  )

  const trendSeries = computed(() => [
    {
      name: t('usage.dashboard.chart.input'),
      data: trendBuckets.value.map((item) => ({ x: `${item.startDate}T00:00:00Z`, y: item.inputTokens })),
    },
    {
      name: t('usage.dashboard.chart.output'),
      data: trendBuckets.value.map((item) => ({ x: `${item.startDate}T00:00:00Z`, y: item.outputTokens })),
    },
    {
      name: t('usage.dashboard.chart.cache'),
      data: trendBuckets.value.map((item) => ({ x: `${item.startDate}T00:00:00Z`, y: item.cacheReadTokens })),
    },
  ])

  const trendGranularityLabel = computed(() => t(`usage.dashboard.chart.bucket.${trendGranularity.value}`))

  const trendSubtitle = computed(() =>
    t('usage.dashboard.chart.trendSubtitle', {
      granularity: trendGranularityLabel.value,
      window: selectedWindowLabel.value,
      points: trendBuckets.value.length,
    }),
  )

  const trendOptions = computed(() => ({
    chart: {
      background: 'transparent',
      toolbar: { show: false },
      fontFamily: 'inherit',
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
    yaxis: {
      labels: {
        style: { colors: chartTheme.value.textMuted },
        formatter: (value: number) => formatTokens(value),
      },
    },
    stroke: { curve: 'smooth' as const, width: 2.4 },
    fill: { type: 'gradient', gradient: { opacityFrom: 0.32, opacityTo: 0.04 } },
    dataLabels: { enabled: false },
    tooltip: {
      theme: chartTheme.value.mode,
      x: {
        // ApexCharts 会把第二个参数作为上下文对象传入，这里只取数据点索引。
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        formatter: (_value: string, context: any) => {
          const bucket = trendBuckets.value[context?.dataPointIndex ?? -1]
          if (!bucket) return _value
          return formatTrendTooltipLabel(
            bucket.startDate,
            bucket.displayEndDate,
            trendGranularity.value,
            locale.value,
          )
        },
      },
    },
    grid: {
      borderColor: chartTheme.value.grid,
      strokeDashArray: 4,
      padding: { bottom: 4 },
    },
    legend: {
      labels: { colors: chartTheme.value.textSecondary },
      markers: { size: 4 },
    },
  }))

  const modelDistribution = computed(() =>
    groupModelDistribution(store.modelStats, 6).map((item) => ({
      ...item,
      label: item.isOther ? t('usage.dashboard.chart.others') : item.label,
    })),
  )

  const pieSeries = computed(() => modelDistribution.value.map((item) => item.totalCost))

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

    return palette.slice(0, Math.max(modelDistribution.value.length, 1))
  })

  const distributionSubtitle = computed(() => {
    if (store.modelStats.length <= 6) {
      return t('usage.dashboard.chart.distributionAllVisible', {
        total: store.modelStats.length,
      })
    }

    return t('usage.dashboard.chart.distributionSubtitle', {
      visible: 6,
      total: store.modelStats.length - 6,
    })
  })

  const pieOptions = computed(() => ({
    chart: { background: 'transparent', fontFamily: 'inherit' },
    theme: { mode: chartTheme.value.mode },
    colors: pieColors.value,
    labels: modelDistribution.value.map((item) => item.label),
    legend: { show: false },
    plotOptions: {
      pie: {
        donut: {
          size: '64%',
          labels: {
            show: true,
            name: {
              show: true,
              fontSize: '12px',
              color: chartTheme.value.textMuted,
              offsetY: -4,
            },
            value: {
              show: true,
              fontSize: '16px',
              fontWeight: 600,
              color: chartTheme.value.textPrimary,
              formatter: (value: string) => formatCost(Number(value)),
            },
            total: {
              show: true,
              label: t('usage.dashboard.cards.totalCost'),
              fontSize: '11px',
              color: chartTheme.value.textMuted,
              // eslint-disable-next-line @typescript-eslint/no-explicit-any
              formatter: (context: any) => formatCost(context.globals.seriesTotals.reduce((sum: number, item: number) => sum + item, 0)),
            },
          },
        },
      },
    },
    dataLabels: {
      enabled: true,
      formatter: (_: number, options: { seriesIndex: number; w: { globals: { series: number[] } } }) => {
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
      y: { formatter: (value: number) => formatCost(value) },
    },
  }))

  const overviewHighlights = computed(() => {
    const topModel = store.modelStats[0]
    const topProject = store.projectStats[0]

    return [
      {
        id: 'density',
        label: t('usage.dashboard.highlights.density'),
        value: trendGranularityLabel.value,
        detail: t('usage.dashboard.highlights.densityDetail', {
          points: trendBuckets.value.length,
          window: selectedWindowLabel.value,
        }),
      },
      {
        id: 'top-model',
        label: t('usage.dashboard.highlights.topModel'),
        value: topModel?.model ?? t('usage.dashboard.table.noData'),
        detail: topModel
          ? `${formatCost(topModel.total_cost)} · ${formatTokens(topModel.total_tokens)}`
          : t('usage.dashboard.table.noData'),
      },
      {
        id: 'top-project',
        label: t('usage.dashboard.highlights.topProject'),
        value: topProject ? shortenPath(topProject.project_path) : t('usage.dashboard.table.noData'),
        detail: topProject
          ? `${formatCost(topProject.total_cost)} · ${topProject.request_count.toLocaleString()} ${t('usage.dashboard.table.requests')}`
          : t('usage.dashboard.table.noData'),
      },
      {
        id: 'cache',
        label: t('usage.dashboard.highlights.cacheRead'),
        value: store.summary ? formatTokens(store.summary.total_cache_read_tokens) : t('usage.dashboard.table.noData'),
        detail: store.summary
          ? t('usage.dashboard.highlights.cacheReadDetail', {
            percent: formatPercent(store.summary.cache_efficiency),
          })
          : t('usage.dashboard.table.noData'),
      },
    ]
  })

  const topModelRankings = computed<OverviewRankItem[]>(() => {
    const totalCost = store.modelStats.reduce((sum, item) => sum + item.total_cost, 0)

    return [...store.modelStats]
      .sort((left, right) =>
        right.total_cost - left.total_cost ||
        right.total_tokens - left.total_tokens ||
        right.request_count - left.request_count,
      )
      .slice(0, 5)
      .map((item) => ({
        id: item.model,
        label: item.model,
        title: item.model,
        detail: `${item.request_count.toLocaleString()} ${t('usage.dashboard.table.requests')} · ${formatTokens(item.total_tokens)}`,
        value: formatCost(item.total_cost),
        share: totalCost > 0 ? item.total_cost / totalCost : 0,
      }))
  })

  const topProjectRankings = computed<OverviewRankItem[]>(() => {
    const totalCost = store.projectStats.reduce((sum, item) => sum + item.total_cost, 0)

    return [...store.projectStats]
      .sort((left, right) =>
        right.total_cost - left.total_cost ||
        right.total_tokens - left.total_tokens ||
        right.request_count - left.request_count,
      )
      .slice(0, 5)
      .map((item) => ({
        id: item.project_path,
        label: shortenPath(item.project_path),
        title: item.project_path,
        detail: `${formatTokens(item.total_tokens)} · ${item.request_count.toLocaleString()} ${t('usage.dashboard.table.requests')}`,
        value: formatCost(item.total_cost),
        share: totalCost > 0 ? item.total_cost / totalCost : 0,
      }))
  })

  const logsRecords = computed(() => store.logs?.records ?? [])
  const unknownModelStat = computed(() =>
    store.modelStats.find((item) => item.model === 'unknown') ?? null,
  )
  const logsTotalCount = computed(() => store.logs?.total ?? logsRecords.value.length)
  const latestLogTimestamp = computed(() => logsRecords.value[0]?.recorded_at ?? null)
  const codexRepairRecommended = computed(() => {
    if (selectedPlatform.value !== 'codex') return false
    const summary = store.summary
    if (!summary || summary.total_requests <= 0) return false

    return (unknownModelStat.value?.request_count ?? 0) > 0 || summary.total_cost_usd === 0
  })
  const diagnosticsSummary = computed<UsageDiagnosticsSummary>(() => {
    const latestRecordAt = latestLogTimestamp.value
      ? formatDateTime(latestLogTimestamp.value, locale.value)
      : t('usage.dashboard.diagnostics.noRecentRecord')

    if (codexRepairRecommended.value) {
      return {
        totalRecords: logsTotalCount.value.toLocaleString(),
        latestRecordAt,
        healthLabel: t('usage.dashboard.diagnostics.repairNeeded'),
        healthDetail: t('usage.dashboard.diagnostics.codexRepairHint', {
          unknown: (unknownModelStat.value?.request_count ?? 0).toLocaleString(),
        }),
        repairRecommended: true,
        canRepairCodex: true,
      }
    }

    return {
      totalRecords: logsTotalCount.value.toLocaleString(),
      latestRecordAt,
      healthLabel: t('usage.dashboard.diagnostics.healthy'),
      healthDetail: t('usage.dashboard.diagnostics.rawLogsHint'),
      repairRecommended: false,
      canRepairCodex: selectedPlatform.value === 'codex',
    }
  })
  const diagnosticsEmptyMessage = computed(() =>
    logModelFilter.value
      ? t('usage.dashboard.diagnostics.filteredNoResults')
      : t('usage.dashboard.logs.noLogs'),
  )
  const diagnosticsEmptyDetail = computed(() =>
    logModelFilter.value
      ? t('usage.dashboard.diagnostics.filteredNoResultsHint')
      : t('usage.dashboard.diagnostics.emptyHint'),
  )
  const repairCodexButtonLabel = computed(() =>
    store.importing
      ? t('usage.dashboard.diagnostics.repairingCodex')
      : t('usage.dashboard.diagnostics.repairCodex'),
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

  const importDetails = computed(() =>
    store.lastImportResults
      .filter((result) => result.error)
      .map((result) => `${result.platform}: ${result.error}`),
  )

  const warningMessage = computed(() => store.warning || null)
  const importJobBanner = computed(() => {
    const job = store.currentImportJob
    if (!job || !store.importing) return null

    const totalFiles = Math.max(job.files_total, job.files_scanned)
    const isZh = locale.value.startsWith('zh')

    if (job.status === 'recent_ready') {
      return isZh
        ? `最近数据已就绪，历史数据仍在后台补齐中。已扫描 ${job.files_scanned}/${totalFiles} 个文件，累计导入 ${job.records_imported.toLocaleString()} 条记录。`
        : `Recent data is ready. Historical data is still backfilling in the background. Scanned ${job.files_scanned}/${totalFiles} files and imported ${job.records_imported.toLocaleString()} records so far.`
    }

    return isZh
      ? `正在后台导入 usage 数据。已扫描 ${job.files_scanned}/${totalFiles} 个文件，累计导入 ${job.records_imported.toLocaleString()} 条记录；你可以继续切换页面。`
      : `Usage import is running in the background. Scanned ${job.files_scanned}/${totalFiles} files and imported ${job.records_imported.toLocaleString()} records so far; you can keep navigating.`
  })
  const importJobWarnings = computed(() => store.currentImportJob?.warnings ?? [])
  const showEmptyState = computed(() => store.hasNoUsageData)

  const emptyStateTitle = computed(() => {
    if (store.lastImportSummary && store.lastImportSummary.processed_files === 0 && store.lastImportSummary.imported_records === 0) {
      return t('usage.dashboard.status.noLogsTitle')
    }
    return t('usage.states.noData')
  })

  const emptyStateDescription = computed(() => {
    if (store.lastImportSummary && store.lastImportSummary.processed_files === 0 && store.lastImportSummary.imported_records === 0) {
      return t('usage.dashboard.status.noLogs')
    }
    return t('usage.states.noDataHint', { platform: selectedPlatform.value || 'AI' })
  })

  const syncChartTheme = () => {
    chartTheme.value = buildChartTheme()
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

    if (runtimeUnavailable.value) {
      store.stopAutoRefresh()
      return
    }

    const { start, end } = getTimeRange(selectedDays.value)
    await store.initializeDashboard({
      platform: (selectedPlatform.value || undefined) as Platform | undefined,
      start,
      end,
    })
    store.startAutoRefresh()
  })

  onUnmounted(() => {
    store.stopAutoRefresh()
    themeObserver?.disconnect()
  })

  return {
    activeTab,
    dashboardMetaItems,
    doImport,
    emptyStateDescription,
    emptyStateTitle,
    formatCost,
    formatTokens,
    importButtonLabel,
    importDetails,
    importJobBanner,
    importJobWarnings,
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
    updateLogModelFilter,
    warningMessage,
  }
}
