import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useUsageStore } from '@/stores/usage'
import { ensureLocaleLoaded } from '@/i18n'
import type { ModelStat, Platform, UsageFeatureCapability, UsageUnsupportedReason } from '@/types/usage'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import {
  aggregateDailyTrends,
  expandTrendBucketEnd,
  groupModelDistribution,
  selectTrendGranularity,
  type TrendGranularity,
} from './usageDashboardPresentation'

type UsageSummaryCardTone = 'rose' | 'violet' | 'sky' | 'amber'

/**
 * ApexCharts formatter callback 上下文。
 *
 * ApexCharts 自带 `ApexTooltip` / `ApexDataLabels` type 把第二个参数 `opts` 标成 `any`，
 * 等于让所有调用方手写 `any`。这里收口为最小可用形态——只列我们实际访问的字段，
 * 让 TypeScript 在 build 期发现错字 / 调错 API；对其余字段的访问保持 fail-fast。
 *
 * 字段都是 optional：donut 图没有 `dataPointIndex`，bar 图也不一定有 `globals.seriesTotals`，
 * 调用方按需 narrow。
 */
interface ApexFormatterContext {
  dataPointIndex?: number
  seriesIndex?: number
  globals?: {
    seriesTotals?: number[]
  }
}

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

const formatTokens = (value: number) =>
  value >= 1e6
    ? `${(value / 1e6).toFixed(1)}M`
    : value >= 1e3
      ? `${(value / 1e3).toFixed(1)}K`
      : value.toString()

const formatCost = (value: number) => (value >= 1 ? `$${value.toFixed(2)}` : `$${value.toFixed(4)}`)

const formatPercent = (value: number) => `${(value * 100).toFixed(1)}%`
const formatDateTime = (value: string, locale: string) => new Date(value).toLocaleString(locale)
const modelCost = (model: ModelStat) => model.cost_with_cache ?? 0
const outputSeriesName = 'Output'

const hasTemplatePlaceholder = (value: string) => /\{[a-zA-Z_][a-zA-Z0-9_]*\}/.test(value)

const shortenPath = (path: string) => {
  const parts = path.replace(/\\/g, '/').split('/')
  return parts.length > 2 ? `.../${parts.slice(-2).join('/')}` : path
}

const formatArchiveTimestamp = (value: string | null | undefined, locale: string) => {
  if (!value) return '—'
  return formatDateTime(value, locale)
}

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

export const formatLocalDate = (date: Date) => {
  const year = date.getFullYear()
  const month = `${date.getMonth() + 1}`.padStart(2, '0')
  const day = `${date.getDate()}`.padStart(2, '0')
  return `${year}-${month}-${day}`
}

export const getLocalDateWindow = (days: number, endDate = new Date()) => {
  const normalizedDays = Math.max(1, Math.floor(days))
  const end = new Date(endDate.getFullYear(), endDate.getMonth(), endDate.getDate())
  const start = new Date(end)
  start.setDate(end.getDate() - (normalizedDays - 1))
  return { start: formatLocalDate(start), end: formatLocalDate(end) }
}

const getTimeRange = getLocalDateWindow

const parseUtcDate = (value: string) => {
  const [year, month, day] = value.split('-').map(Number)
  return new Date(Date.UTC(year, (month || 1) - 1, day || 1))
}

const buildDateFormatters = (locale: string) => ({
  day: new Intl.DateTimeFormat(locale, { month: 'short', day: 'numeric', timeZone: 'UTC' }),
  month: new Intl.DateTimeFormat(locale, { month: 'short', year: 'numeric', timeZone: 'UTC' }),
})

const formatTrendAxisLabel = (value: number, granularity: TrendGranularity, locale: string) => {
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
  locale: string
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
  primary: readCssVar('--color-accent-primary', '#0071E3'),
  secondary: readCssVar('--color-accent-secondary', '#2997FF'),
  tertiary: readCssVar('--color-info', '#5AC8FA'),
  quaternary: readCssVar('--color-warning', '#FF9F0A'),
  textPrimary: readCssVar('--color-text-primary', '#1D1D1F'),
  textSecondary: readCssVar('--color-text-secondary', '#424245'),
  textMuted: readCssVar('--color-text-muted', '#6E6E73'),
  grid: readCssVar('--color-border-subtle', 'rgb(29 29 31 / 8%)'),
  border: readCssVar('--color-border-default', 'rgb(29 29 31 / 12%)'),
})

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

  const selectedPlatformLabel = computed(() => {
    if (!selectedPlatform.value) {
      return translateDashboardText('usage.dashboard.allPlatforms', undefined, 'All Platforms')
    }

    const fallbackLabels: Record<string, string> = {
      claude: 'Claude',
      codex: 'Codex',
      gemini: 'Gemini',
      opencode: 'OpenCode',
    }

    return translateDashboardText(
      `usage.platforms.${selectedPlatform.value}`,
      undefined,
      fallbackLabels[selectedPlatform.value] ?? selectedPlatform.value
    )
  })

  const summaryCards = computed<UsageSummaryCard[]>(() => {
    if (!dashboardReady.value) return []

    const summary = store.summary
    if (!summary) return []

    const averageCostPerRequest =
      summary.total_requests > 0
        ? formatCost(summary.total_cost_usd / summary.total_requests)
        : formatCost(0)

    return [
      {
        id: 'requests',
        label: translateDashboardText(
          'usage.dashboard.cards.totalRequests',
          undefined,
          'Total Requests'
        ),
        value: summary.total_requests.toLocaleString(),
        detail: translateDashboardText(
          'usage.dashboard.cards.requestsDetail',
          {
            models: store.modelStats.length,
            projects: store.projectStats.length,
          },
          `${store.modelStats.length} models · ${store.projectStats.length} projects`
        ),
        icon: 'Activity',
        tone: 'rose',
      },
      {
        id: 'tokens',
        label: translateDashboardText(
          'usage.dashboard.cards.totalTokens',
          undefined,
          'Total Tokens'
        ),
        value: formatTokens(summary.total_tokens),
        detail: translateDashboardText(
          'usage.dashboard.cards.tokensDetail',
          {
            input: formatTokens(summary.total_input_tokens),
            output: formatTokens(summary.total_output_tokens),
            cache: formatTokens(summary.total_cache_read_tokens),
          },
          `${formatTokens(summary.total_input_tokens)} in · ${formatTokens(summary.total_output_tokens)} out · ${formatTokens(summary.total_cache_read_tokens)} cache read`
        ),
        icon: 'Layers',
        tone: 'violet',
      },
      {
        id: 'cost',
        label: translateDashboardText('usage.dashboard.cards.totalCost', undefined, 'Total Cost'),
        value: formatCost(summary.total_cost_usd),
        detail: translateDashboardText(
          'usage.dashboard.cards.costDetail',
          {
            average: averageCostPerRequest,
          },
          `${averageCostPerRequest} per request`
        ),
        icon: 'Wallet',
        tone: 'sky',
      },
      {
        id: 'cache',
        label: translateDashboardText(
          'usage.dashboard.cards.cacheEfficiency',
          undefined,
          'Cache Reuse Rate'
        ),
        value: formatPercent(summary.cache_efficiency),
        detail: translateDashboardText(
          'usage.dashboard.cards.cacheDetail',
          {
            tokens: formatTokens(summary.total_cache_read_tokens),
          },
          `cache read / (input + cache read) · ${formatTokens(summary.total_cache_read_tokens)} cache read`
        ),
        icon: 'Cpu',
        tone: 'amber',
      },
    ]
  })

  const selectedWindowLabel = computed(() => {
    const labels: Record<number, { key: string; fallback: string }> = {
      7: { key: 'usage.dashboard.days7', fallback: '7 Days' },
      30: { key: 'usage.dashboard.days30', fallback: '30 Days' },
      90: { key: 'usage.dashboard.days90', fallback: '90 Days' },
      365: { key: 'usage.dashboard.days365', fallback: '365 Days' },
    }

    const selected = labels[selectedDays.value]
    if (!selected) return `${selectedDays.value}d`

    return translateDashboardText(selected.key, undefined, selected.fallback)
  })

  const dashboardMetaItems = computed<DashboardMetaItem[]>(() => {
    if (!dashboardReady.value) return []

    const archive = store.archive

    return [
      {
        id: 'scope',
        label: translateDashboardText('usage.dashboard.meta.scope', undefined, 'Scope'),
        value: selectedPlatformLabel.value,
      },
      {
        id: 'window',
        label: translateDashboardText('usage.dashboard.meta.window', undefined, 'Window'),
        value: selectedWindowLabel.value,
      },
      {
        id: 'models',
        label: translateDashboardText('usage.dashboard.meta.models', undefined, 'Models'),
        value: store.modelStats.length.toLocaleString(),
      },
      {
        id: 'projects',
        label: translateDashboardText('usage.dashboard.meta.projects', undefined, 'Projects'),
        value: store.projectStats.length.toLocaleString(),
      },
      {
        id: 'archive',
        label: 'Archive',
        value: archive
          ? `L ${archive.live_sources} · M ${archive.missing_sources} · D ${archive.deleted_sources}`
          : '—',
      },
      {
        id: 'archive-root',
        label: 'Archive Root',
        value: archive ? shortenPath(archive.archive_root) : '—',
      },
      {
        id: 'archive-time',
        label: 'Last Sync',
        value: archive
          ? formatArchiveTimestamp(
              archive.history_completed_at ?? archive.recent_completed_at,
              locale.value
            )
          : '—',
      },
    ]
  })

  const trendGranularity = computed(() => selectTrendGranularity(selectedDays.value))

  const trendBuckets = computed(() =>
    aggregateDailyTrends(store.trends, trendGranularity.value).map((bucket) => ({
      ...bucket,
      displayEndDate: expandTrendBucketEnd(bucket, trendGranularity.value),
    }))
  )

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

  const pieOptions = computed(() => ({
    chart: { background: 'transparent', fontFamily: 'inherit' },
    theme: { mode: chartTheme.value.mode },
    colors: pieColors.value,
    labels: modelDistribution.value.map((item) => item.label),
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
              formatter: (value: string) => formatCost(Number(value)),
            },
            total: {
              show: true,
              label: t('usage.dashboard.cards.totalCost'),
              fontSize: '10px',
              color: chartTheme.value.textMuted,
              formatter: (context: ApexFormatterContext) =>
                formatCost(
                  (context.globals?.seriesTotals ?? []).reduce(
                    (sum: number, item: number) => sum + item,
                    0,
                  ),
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
      y: { formatter: (value: number) => formatCost(value) },
    },
  }))

  const overviewHighlights = computed(() => {
    if (!dashboardReady.value) return []

    const topModel = store.modelStats[0]
    const topProject = store.projectStats[0]

    return [
      {
        id: 'density',
        label: translateDashboardText(
          'usage.dashboard.highlights.density',
          undefined,
          'Trend Density'
        ),
        value: trendGranularityLabel.value,
        detail: translateDashboardText(
          'usage.dashboard.highlights.densityDetail',
          {
            points: trendBuckets.value.length,
            window: selectedWindowLabel.value,
          },
          `${trendBuckets.value.length} points across ${selectedWindowLabel.value}`
        ),
      },
      {
        id: 'top-model',
        label: translateDashboardText(
          'usage.dashboard.highlights.topModel',
          undefined,
          'Top Model'
        ),
        value:
          topModel?.model ??
          translateDashboardText('usage.dashboard.table.noData', undefined, 'No data'),
        detail: topModel
          ? `${formatCost(modelCost(topModel))} · ${formatTokens(topModel.total_tokens)}`
          : translateDashboardText('usage.dashboard.table.noData', undefined, 'No data'),
      },
      {
        id: 'top-project',
        label: translateDashboardText(
          'usage.dashboard.highlights.topProject',
          undefined,
          'Top Project'
        ),
        value: topProject
          ? shortenPath(topProject.project_path)
          : translateDashboardText('usage.dashboard.table.noData', undefined, 'No data'),
        detail: topProject
          ? `${formatCost(topProject.total_cost)} · ${topProject.request_count.toLocaleString()} ${translateDashboardText('usage.dashboard.table.requests', undefined, 'requests')}`
          : translateDashboardText('usage.dashboard.table.noData', undefined, 'No data'),
      },
      {
        id: 'cache',
        label: translateDashboardText(
          'usage.dashboard.highlights.cacheRead',
          undefined,
          'Cache Read'
        ),
        value: store.summary
          ? formatTokens(store.summary.total_cache_read_tokens)
          : translateDashboardText('usage.dashboard.table.noData', undefined, 'No data'),
        detail: store.summary
          ? translateDashboardText(
              'usage.dashboard.highlights.cacheReadDetail',
              {
                percent: formatPercent(store.summary.cache_efficiency),
              },
              `Cache reuse ${formatPercent(store.summary.cache_efficiency)}`
            )
          : translateDashboardText('usage.dashboard.table.noData', undefined, 'No data'),
      },
    ]
  })

  const topModelRankings = computed<OverviewRankItem[]>(() => {
    if (!dashboardReady.value) return []

    const totalCost = store.modelStats.reduce((sum, item) => sum + modelCost(item), 0)

    return [...store.modelStats]
      .sort(
        (left, right) =>
          modelCost(right) - modelCost(left) ||
          right.total_tokens - left.total_tokens ||
          right.request_count - left.request_count
      )
      .slice(0, 5)
      .map((item) => ({
        id: item.model,
        label: item.model,
        title: item.model,
        detail: `${item.request_count.toLocaleString()} ${translateDashboardText('usage.dashboard.table.requests', undefined, 'requests')} · ${formatTokens(item.total_tokens)}`,
        value: formatCost(modelCost(item)),
        share: totalCost > 0 ? modelCost(item) / totalCost : 0,
      }))
  })

  const topProjectRankings = computed<OverviewRankItem[]>(() => {
    if (!dashboardReady.value) return []

    const totalCost = store.projectStats.reduce((sum, item) => sum + item.total_cost, 0)

    return [...store.projectStats]
      .sort(
        (left, right) =>
          right.total_cost - left.total_cost ||
          right.total_tokens - left.total_tokens ||
          right.request_count - left.request_count
      )
      .slice(0, 5)
      .map((item) => ({
        id: item.project_path,
        label: shortenPath(item.project_path),
        title: item.project_path,
        detail: `${formatTokens(item.total_tokens)} · ${item.request_count.toLocaleString()} ${translateDashboardText('usage.dashboard.table.requests', undefined, 'requests')}`,
        value: formatCost(item.total_cost),
        share: totalCost > 0 ? item.total_cost / totalCost : 0,
      }))
  })

  const logsRecords = computed(() => store.logs?.records ?? [])
  const unknownModelStat = computed(
    () => store.modelStats.find((item) => item.model === 'unknown') ?? null
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
    const archive = store.archive
    const archiveDetail = archive
      ? `Archive ${archive.archived_sessions} · L ${archive.live_sources} / M ${archive.missing_sources} / D ${archive.deleted_sources}`
      : t('usage.dashboard.diagnostics.rawLogsHint')

    if (codexRepairRecommended.value) {
      return {
        totalRecords: logsTotalCount.value.toLocaleString(),
        latestRecordAt,
        healthLabel: t('usage.dashboard.diagnostics.repairNeeded'),
        healthDetail: `${t('usage.dashboard.diagnostics.codexRepairHint', {
          unknown: (unknownModelStat.value?.request_count ?? 0).toLocaleString(),
        })} · ${archiveDetail}`,
        repairRecommended: true,
        canRepairCodex: true,
      }
    }

    return {
        totalRecords: logsTotalCount.value.toLocaleString(),
        latestRecordAt,
        healthLabel: t('usage.dashboard.diagnostics.healthy'),
        healthDetail: archiveDetail,
        repairRecommended: false,
        canRepairCodex: selectedPlatform.value === 'codex',
      }
  })
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
    store.startAutoRefresh()
  })

  onUnmounted(() => {
    store.stopAutoRefresh()
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
    updateLogModelFilter,
    warningMessage,
  }
}
