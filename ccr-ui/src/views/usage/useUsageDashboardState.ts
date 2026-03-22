import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import type { ComponentPublicInstance } from 'vue'
import { useI18n } from 'vue-i18n'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { useUsageStore } from '@/stores/usage'
import type { Platform } from '@/types/usage'

const CHART_COLORS = ['#F9A8D4', '#C4B5FD', '#6EE7B7', '#FCD34D', '#D8B4FE', '#FDA4AF', '#22D3EE', '#2DD4BF']

const formatTokens = (value: number) => (
  value >= 1e6 ? `${(value / 1e6).toFixed(1)}M`
    : value >= 1e3 ? `${(value / 1e3).toFixed(1)}K`
      : value.toString()
)

const formatCost = (value: number) => (
  value >= 1 ? `$${value.toFixed(2)}` : `$${value.toFixed(4)}`
)

const formatPercent = (value: number) => `${(value * 100).toFixed(1)}%`

const shortenPath = (path: string) => {
  const parts = path.replace(/\\/g, '/').split('/')
  return parts.length > 2 ? `.../${parts.slice(-2).join('/')}` : path
}

const getTimeRange = (days: number) => {
  const end = new Date()
  const start = new Date(end.getTime() - days * 86400000)
  return { start: start.toISOString().slice(0, 10), end: end.toISOString().slice(0, 10) }
}

export const useUsageDashboardState = () => {
  const { t } = useI18n()
  const store = useUsageStore()

  const tabKeys = ['overview', 'models', 'projects', 'logs'] as const
  const activeTab = ref<string>('overview')
  const selectedPlatform = ref('')
  const selectedDays = ref(30)
  const logModelFilter = ref('')
  const logsScrollRef = ref<HTMLElement | null>(null)

  const shouldLoadCharts = computed(() => activeTab.value === 'overview' || activeTab.value === 'models')

  const onFilterChange = () => {
    const { start, end } = getTimeRange(selectedDays.value)
    store.setFilters({
      platform: (selectedPlatform.value || undefined) as Platform | undefined,
      start,
      end,
    })
  }

  const doImport = async () => {
    await store.triggerImport(undefined, 'manual')
  }

  const loadLogs = (direction: 'reset' | 'next' | 'prev' | 'same' = 'reset') => {
    store.logsModelFilter = logModelFilter.value || undefined
    store.fetchLogs(direction)
  }

  const updateLogModelFilter = (value: string) => {
    logModelFilter.value = value
  }

  watch(activeTab, (tab) => {
    if (tab === 'logs' && !store.logs) {
      loadLogs('reset')
    }
  })

  const onVisibilityChange = () => {
    if (document.hidden) {
      store.stopAutoRefresh()
      return
    }

    store.startAutoRefresh()
    if (activeTab.value === 'logs') {
      loadLogs('same')
    }
  }

  const summaryCards = computed(() => {
    const summary = store.summary
    if (!summary) return []

    return [
      { label: t('usage.dashboard.cards.totalRequests'), value: summary.total_requests.toLocaleString() },
      { label: t('usage.dashboard.cards.totalTokens'), value: formatTokens(summary.total_input_tokens + summary.total_output_tokens) },
      { label: t('usage.dashboard.cards.totalCost'), value: formatCost(summary.total_cost_usd) },
      { label: t('usage.dashboard.cards.cacheEfficiency'), value: formatPercent(summary.cache_efficiency) },
    ]
  })

  const trendSeries = computed(() => [
    { name: t('usage.dashboard.chart.input'), data: store.trends.map((item) => item.input_tokens) },
    { name: t('usage.dashboard.chart.output'), data: store.trends.map((item) => item.output_tokens) },
    { name: t('usage.dashboard.chart.cache'), data: store.trends.map((item) => item.cache_read_tokens) },
  ])

  const trendOptions = computed(() => ({
    chart: { background: 'transparent', toolbar: { show: false }, fontFamily: 'inherit' },
    theme: { mode: 'dark' as const },
    colors: CHART_COLORS.slice(0, 3),
    xaxis: {
      categories: store.trends.map((item) => item.date),
      labels: {
        style: { colors: '#94a3b8', fontSize: '11px' },
        rotate: store.trends.length > 14 ? -45 : 0,
        rotateAlways: false,
        maxHeight: 60,
        formatter: (value: string) => value?.slice(5) ?? value,
      },
      axisBorder: { show: false },
      axisTicks: { color: 'rgba(249,168,212,0.1)' },
    },
    yaxis: {
      labels: {
        style: { colors: '#94a3b8' },
        formatter: (value: number) => formatTokens(value),
      },
    },
    stroke: { curve: 'smooth' as const, width: 2 },
    fill: { type: 'gradient', gradient: { opacityFrom: 0.35, opacityTo: 0.05 } },
    dataLabels: { enabled: false },
    tooltip: { theme: 'dark' },
    grid: { borderColor: 'rgba(249,168,212,0.08)', strokeDashArray: 4, padding: { bottom: 4 } },
    legend: { labels: { colors: '#94a3b8' }, markers: { size: 4 } },
  }))

  const pieSeries = computed(() => store.modelStats.map((item) => item.total_cost))

  const pieOptions = computed(() => ({
    chart: { background: 'transparent', fontFamily: 'inherit' },
    theme: { mode: 'dark' as const },
    colors: CHART_COLORS,
    labels: store.modelStats.map((item) => item.model),
    legend: {
      position: 'bottom' as const,
      labels: { colors: '#94a3b8' },
      fontSize: '11px',
      markers: { size: 5, offsetX: -2 },
      formatter: (name: string) => (name.length > 22 ? `${name.slice(0, 22)}…` : name),
      itemMargin: { horizontal: 6, vertical: 2 },
    },
    plotOptions: {
      pie: {
        donut: {
          size: '58%',
          labels: {
            show: true,
            name: { show: true, fontSize: '12px', color: '#94a3b8', offsetY: -4 },
            value: {
              show: true,
              fontSize: '16px',
              fontWeight: 500,
              color: '#FDF2F8',
              formatter: (value: string) => formatCost(Number(value)),
            },
            total: {
              show: true,
              label: 'Total',
              fontSize: '11px',
              color: '#94a3b8',
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
        return total > 0 ? `${((options.w.globals.series[options.seriesIndex] / total) * 100).toFixed(0)}%` : '0%'
      },
      style: { fontSize: '11px', fontWeight: 500 },
      dropShadow: { enabled: false },
    },
    tooltip: { theme: 'dark', y: { formatter: (value: number) => formatCost(value) } },
  }))

  const logsRecords = computed(() => store.logs?.records ?? [])
  const logsVirtualizer = useVirtualizer(computed(() => ({
    count: logsRecords.value.length,
    getScrollElement: () => logsScrollRef.value,
    estimateSize: () => 44,
    overscan: 10,
  })))

  const setLogsScrollRef = (element: Element | ComponentPublicInstance | null) => {
    logsScrollRef.value = element instanceof HTMLElement ? element : null
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

  onMounted(async () => {
    const { start, end } = getTimeRange(selectedDays.value)
    await store.initializeDashboard({
      platform: (selectedPlatform.value || undefined) as Platform | undefined,
      start,
      end,
    })
    store.startAutoRefresh()
    document.addEventListener('visibilitychange', onVisibilityChange)
  })

  onUnmounted(() => {
    store.stopAutoRefresh()
    document.removeEventListener('visibilitychange', onVisibilityChange)
  })

  return {
    activeTab,
    doImport,
    emptyStateDescription,
    emptyStateTitle,
    formatCost,
    formatTokens,
    importButtonLabel,
    importDetails,
    loadLogs,
    logsRecords,
    logsScrollRef,
    setLogsScrollRef,
    logsVirtualizer,
    logModelFilter,
    onFilterChange,
    pieOptions,
    pieSeries,
    selectedDays,
    selectedPlatform,
    shortenPath,
    shouldLoadCharts,
    showEmptyState,
    store,
    summaryCards,
    tabKeys,
    trendOptions,
    trendSeries,
    updateLogModelFilter,
    warningMessage,
  }
}
