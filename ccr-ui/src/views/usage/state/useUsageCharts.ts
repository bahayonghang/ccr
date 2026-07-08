import { computed, ref, type ComputedRef, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useUsageStore } from '@/stores/usage'
import {
  buildUsageDashboardPresentation,
  selectTrendGranularity,
  type ModelDistributionSlice,
  type TrendGranularity,
} from '../usageDashboardPresentation'
import { getUsageRangePresetSpanDays, type UsageRangePreset } from '../dateWindow'
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
} from '../usageChartOptions'
import { formatCost, formatTokens } from '../usageSummaryCards'
import type { OverviewRankItem } from '../usageOverviewInsights'

type UsageStore = ReturnType<typeof useUsageStore>
type I18nComposer = ReturnType<typeof useI18n>
type TranslateDashboardText = (
  key: string,
  values: Record<string, number | string> | undefined,
  fallback: string
) => string

export const useUsageCharts = (params: {
  store: UsageStore
  t: I18nComposer['t']
  locale: I18nComposer['locale']
  dashboardReady: ComputedRef<boolean>
  selectedRange: Ref<UsageRangePreset>
  selectedWindowLabel: ComputedRef<string>
  translateDashboardText: TranslateDashboardText
}) => {
  const {
    store,
    t,
    locale,
    dashboardReady,
    selectedRange,
    selectedWindowLabel,
    translateDashboardText,
  } = params

  const chartTheme = ref<ChartThemeState>(buildChartTheme())

  const trendRangeSpanDays = computed(() =>
    getUsageRangePresetSpanDays(
      selectedRange.value,
      store.trends.map((item) => item.date)
    )
  )

  const trendGranularity = computed(() => selectTrendGranularity(trendRangeSpanDays.value))

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
  const cacheCreationTokens = computed(() =>
    (dashboardReady.value ? store.trends : []).reduce(
      (sum, item) => sum + item.cache_creation_tokens,
      0
    )
  )

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
    colors: [
      chartTheme.value.inputToken,
      chartTheme.value.outputToken,
      chartTheme.value.cacheReadToken,
    ],
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
      chartTheme.value.success,
      chartTheme.value.info,
      chartTheme.value.warning,
      chartTheme.value.muted,
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
    distribution: ModelDistributionSlice[]
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
                        0
                      )
                    )
                  : formatCost(
                      (context.globals?.seriesTotals ?? []).reduce(
                        (sum: number, item: number) => sum + item,
                        0
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

  const topModelRankings = computed<OverviewRankItem[]>(
    () => dashboardPresentation.value.topModelRankings
  )

  const topProjectRankings = computed<OverviewRankItem[]>(
    () => dashboardPresentation.value.topProjectRankings
  )
  const sourceStats = computed(() => (dashboardReady.value ? store.sourceStats : []))

  const syncChartTheme = () => {
    chartTheme.value = buildChartTheme()
  }

  return {
    syncChartTheme,
    cacheCreationTokens,
    hasRenderableTrendData,
    trendSubtitle,
    summaryCards,
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
    pieColors,
    pieOptions,
    pieSeries,
    overviewHighlights,
    sourceStats,
  }
}
