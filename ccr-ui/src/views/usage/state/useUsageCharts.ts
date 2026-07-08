import { computed, ref, type ComputedRef, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useUsageStore } from '@/stores/usage'
import {
  buildUsageDashboardPresentation,
  selectTrendGranularity,
  type TrendGranularity,
} from '../usageDashboardPresentation'
import { getUsageRangePresetSpanDays, type UsageRangePreset } from '../dateWindow'
import {
  buildChartTheme,
  buildDistributionPieOptions,
  buildTrendChartOptions,
  getTrendTickAmount,
  type ChartThemeState,
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

  // tickAmount 只随分桶数量变化（坐标轴相关）：Vue 3.5 computed 在返回值相等时不向下游传播，
  // 因此“同窗口数据刷新”（点数不变）不会把 trendOptions 拖脏。
  const trendTickAmount = computed(() => getTrendTickAmount(trendBuckets.value.length))

  // 系列名是 presentation 的翻译标签（随 locale 变化，与数据值无关）。
  // 用 join key 记忆化：数据刷新产生的新数组引用不会传导到 options（名称串未变即不重建）。
  const trendSeriesNameKey = computed(() =>
    trendSeries.value.map((series) => series.name).join('\n')
  )
  const trendSeriesNames = computed(() =>
    trendSeriesNameKey.value ? trendSeriesNameKey.value.split('\n') : []
  )

  const trendOptions = computed(() =>
    buildTrendChartOptions({
      theme: chartTheme.value,
      locale: locale.value,
      granularity: trendGranularity.value,
      tickAmount: trendTickAmount.value,
      seriesNames: trendSeriesNames.value,
      costLabel: translateDashboardText('usage.dashboard.table.cost', undefined, 'Cost'),
      // 取值器：tooltip 闭包在渲染时读当前 buckets，数据因此不进入 options 的构建期依赖。
      getBuckets: () => trendBuckets.value,
      formatTokens,
      formatCost,
    })
  )

  const modelDistribution = computed(() => dashboardPresentation.value.modelDistribution)
  const modelTokenDistribution = computed(() => dashboardPresentation.value.modelTokenDistribution)
  const pieSeries = computed(() => dashboardPresentation.value.pieSeries)
  const modelTokenPieSeries = computed(() => dashboardPresentation.value.modelTokenPieSeries)

  // 取色个数只取决于切片数量（与数值无关）：先用数量 computed 记忆化，
  // 数据刷新但切片数不变时 pieColors 不重算，引用稳定，不会拖脏 options。
  const pieColorCount = computed(() =>
    Math.max(modelDistribution.value.length, modelTokenDistribution.value.length, 1)
  )
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

    return palette.slice(0, pieColorCount.value)
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

  // labels 是模型名（数据派生）：join key 记忆化，模型集合/排序不变则 labels 引用稳定，
  // 数据刷新只走 updateSeries，不重建 pie options。
  const modelDistributionLabelKey = computed(() =>
    modelDistribution.value.map((slice) => slice.label).join('\n')
  )
  const modelDistributionLabels = computed(() =>
    modelDistributionLabelKey.value ? modelDistributionLabelKey.value.split('\n') : []
  )
  const modelTokenDistributionLabelKey = computed(() =>
    modelTokenDistribution.value.map((slice) => slice.label).join('\n')
  )
  const modelTokenDistributionLabels = computed(() =>
    modelTokenDistributionLabelKey.value ? modelTokenDistributionLabelKey.value.split('\n') : []
  )

  // 中心总计文案随 locale 变化（不含数据），单独 computed 让 options 仅在 locale 变化时重建。
  const pieTotalCostLabel = computed(() => t('usage.dashboard.cards.totalCost'))
  const pieTotalTokensLabel = computed(() => t('usage.dashboard.cards.totalTokens'))

  const pieOptions = computed(() =>
    buildDistributionPieOptions({
      metric: 'cost',
      theme: chartTheme.value,
      colors: pieColors.value,
      labels: modelDistributionLabels.value,
      totalLabel: pieTotalCostLabel.value,
      formatTokens,
      formatCost,
    })
  )
  const modelTokenPieOptions = computed(() =>
    buildDistributionPieOptions({
      metric: 'tokens',
      theme: chartTheme.value,
      colors: pieColors.value,
      labels: modelTokenDistributionLabels.value,
      totalLabel: pieTotalTokensLabel.value,
      formatTokens,
      formatCost,
    })
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
