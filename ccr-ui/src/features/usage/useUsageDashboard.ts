import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { hasTemplatePlaceholder } from '@/i18n/formatMessage'
import { llmusageInstallDetect } from '@/api/domains/install'
import type { UsagePlatform } from '@/types/usage'
import {
  DEFAULT_USAGE_RANGE_PRESET,
  getUsageRangePresetImportDays,
  getUsageRangePresetSpanDays,
  type UsageRangePreset,
} from '@/views/usage/dateWindow'
import {
  buildUsageDashboardPresentation,
  selectTrendGranularity,
  type UsageTrendSeriesItem,
} from '@/views/usage/usageDashboardPresentation'
import {
  buildChartTheme,
  buildDistributionPieOptions,
  buildTrendChartOptions,
  getTrendTickAmount,
} from '@/views/usage/usageChartOptions'
import { formatCost, formatTokens } from '@/views/usage/usageSummaryCards'
import { buildSelectedPlatformLabel, buildSelectedWindowLabel } from '@/views/usage/usageOverviewInsights'
import { buildUsageOpsCockpit } from '@/views/usage/usageOpsCockpit'
import { buildUsageDiagnosticsSummary } from '@/views/usage/usageDiagnostics'
import { buildDashboardMetaItems } from '@/views/usage/usageOverviewInsights'
import { hydrateUsageLocale, useUsageT } from './translate'
import { useUsageDashboardData } from './useUsageDashboardData'
import { useUsageViewStore } from './stores'

const trendSeriesKeyOf = (series: UsageTrendSeriesItem[]) =>
  series
    .map((item) => `${item.name}:${item.data.map((point) => `${point.x}=${point.y}`).join(',')}`)
    .join('\n')

const memoNumberSeries = (previous: number[] | undefined, next: number[]) =>
  previous && previous.join(',') === next.join(',') ? previous : next

export function useUsageDashboard() {
  const t = useUsageT()
  const locale = typeof navigator === 'undefined' ? 'zh-CN' : (document.documentElement.lang || 'zh-CN')
  const data = useUsageDashboardData()
  const platform = useUsageViewStore((state) => state.platform)
  const rangePreset = useUsageViewStore((state) => state.rangePreset)
  const setPlatform = useUsageViewStore((state) => state.setPlatform)
  const setRangePreset = useUsageViewStore((state) => state.setRangePreset)

  const [activeTab, setActiveTab] = useState('overview')
  const [logModelFilter, setLogModelFilter] = useState('')
  const [localeReady, setLocaleReady] = useState(false)
  const [showInstallDialog, setShowInstallDialog] = useState(false)
  const [chartTheme, setChartTheme] = useState(() => buildChartTheme())
  const [trendChartReady, setTrendChartReady] = useState(false)
  const [distributionChartReady, setDistributionChartReady] = useState(false)
  const previousTrendSeries = useRef<UsageTrendSeriesItem[] | undefined>(undefined)
  const previousPieSeries = useRef<number[] | undefined>(undefined)
  const previousTokenPieSeries = useRef<number[] | undefined>(undefined)

  const selectedPlatform = platform ?? ''
  const selectedRange = rangePreset ?? DEFAULT_USAGE_RANGE_PRESET
  const dashboardReady = localeReady && !data.loading

  const translateDashboardText = useCallback((
    key: string,
    values: Record<string, number | string> | undefined,
    fallback: string,
  ) => {
    if (!localeReady) return fallback
    const resolved = values ? t(key, values) : t(key)
    if (resolved === key || hasTemplatePlaceholder(resolved)) return fallback
    return resolved
  }, [localeReady, t])

  const selectedPlatformLabel = buildSelectedPlatformLabel(selectedPlatform, translateDashboardText)
  const selectedWindowLabel = buildSelectedWindowLabel(selectedRange, translateDashboardText)
  const trendGranularity = selectTrendGranularity(
    getUsageRangePresetSpanDays(selectedRange, data.trends.map((item) => item.date)),
  )
  const trendGranularityLabel = translateDashboardText(
    `usage.dashboard.chart.bucket.${trendGranularity}`,
    undefined,
    'Daily',
  )

  const presentation = useMemo(() => buildUsageDashboardPresentation({
    modelStats: dashboardReady ? data.modelStats : [],
    projectStats: dashboardReady ? data.projectStats : [],
    selectedWindowLabel,
    summary: dashboardReady ? data.summary : null,
    translate: translateDashboardText,
    trendGranularity,
    trendGranularityLabel,
    trends: dashboardReady ? data.trends : [],
  }), [dashboardReady, data.modelStats, data.projectStats, data.summary, data.trends, selectedWindowLabel, translateDashboardText, trendGranularity, trendGranularityLabel])

  const trendSeries = useMemo(() => {
    const next = presentation.trendSeries
    const previous = previousTrendSeries.current
    const stable = previous && trendSeriesKeyOf(previous) === trendSeriesKeyOf(next) ? previous : next
    previousTrendSeries.current = stable
    return stable
  }, [presentation.trendSeries])

  const pieSeries = useMemo(() => {
    const next = memoNumberSeries(previousPieSeries.current, presentation.pieSeries)
    previousPieSeries.current = next
    return next
  }, [presentation.pieSeries])

  const modelTokenPieSeries = useMemo(() => {
    const next = memoNumberSeries(previousTokenPieSeries.current, presentation.modelTokenPieSeries)
    previousTokenPieSeries.current = next
    return next
  }, [presentation.modelTokenPieSeries])

  const trendOptions = useMemo(() => buildTrendChartOptions({
    theme: chartTheme,
    locale,
    granularity: trendGranularity,
    tickAmount: getTrendTickAmount(presentation.trendBuckets.length),
    seriesNames: trendSeries.map((series) => series.name),
    costLabel: translateDashboardText('usage.dashboard.table.cost', undefined, 'Cost'),
    getBuckets: () => presentation.trendBuckets,
    formatTokens,
    formatCost,
  }), [chartTheme, locale, presentation.trendBuckets, trendGranularity, trendSeries, translateDashboardText])

  const pieColors = useMemo(() => {
    const palette = [
      chartTheme.primary,
      chartTheme.secondary,
      chartTheme.tertiary,
      chartTheme.quaternary,
      chartTheme.success,
      chartTheme.info,
      chartTheme.warning,
      chartTheme.muted,
    ]
    return palette.slice(0, Math.max(presentation.modelDistribution.length, presentation.modelTokenDistribution.length, 1))
  }, [chartTheme, presentation.modelDistribution.length, presentation.modelTokenDistribution.length])

  const pieOptions = useMemo(() => buildDistributionPieOptions({
    metric: 'cost',
    theme: chartTheme,
    colors: pieColors,
    labels: presentation.modelDistribution.map((slice) => slice.label),
    totalLabel: t('usage.dashboard.cards.totalCost'),
    formatTokens,
    formatCost,
  }), [chartTheme, pieColors, presentation.modelDistribution, t])

  const modelTokenPieOptions = useMemo(() => buildDistributionPieOptions({
    metric: 'tokens',
    theme: chartTheme,
    colors: pieColors,
    labels: presentation.modelTokenDistribution.map((slice) => slice.label),
    totalLabel: t('usage.dashboard.cards.totalTokens'),
    formatTokens,
    formatCost,
  }), [chartTheme, pieColors, presentation.modelTokenDistribution, t])

  const opsCockpit = useMemo(() => buildUsageOpsCockpit({
    archive: data.archive,
    importDetails: data.lastImportResults.filter((result) => result.error).map((result) => `${result.platform}: ${result.error}`),
    importing: data.importing,
    importJobBanner: null,
    importJobWarnings: data.currentImportJob?.warnings ?? [],
    lastUpdatedAt: data.lastUpdated?.toISOString() ?? null,
    loading: data.loading,
    locale,
    selectedPlatformLabel,
    selectedWindowLabel,
    snapshot: data.snapshot,
    translate: translateDashboardText,
    unsupportedSyncMessage: null,
    warningMessage: data.warning,
  }), [data.archive, data.currentImportJob, data.importing, data.lastImportResults, data.lastUpdated, data.loading, data.snapshot, data.warning, locale, selectedPlatformLabel, selectedWindowLabel, translateDashboardText])

  const logsRecords = useMemo(() => data.logs?.records ?? [], [data.logs])
  const diagnosticsSummary = useMemo(() => buildUsageDiagnosticsSummary({
    selectedPlatform: selectedPlatform as UsagePlatform | '',
    summary: data.summary,
    logsRecords,
    logsTotalCount: data.logs?.total ?? logsRecords.length,
    unknownModelStat: data.modelStats.find((item) => item.model === 'unknown') ?? null,
    archive: data.archive,
    locale,
    messages: {
      noRecentRecord: t('usage.dashboard.diagnostics.noRecentRecord'),
      rawLogsHint: t('usage.dashboard.diagnostics.rawLogsHint'),
      repairNeeded: t('usage.dashboard.diagnostics.repairNeeded'),
      codexRepairHint: (unknown) => t('usage.dashboard.diagnostics.codexRepairHint', { unknown }),
      healthy: t('usage.dashboard.diagnostics.healthy'),
    },
  }), [data.archive, data.logs, data.modelStats, data.summary, locale, logsRecords, selectedPlatform, t])

  const onFilterChange = useCallback(() => {
    if (activeTab === 'logs') void data.fetchLogs('reset')
  }, [activeTab, data])

  const updateSelectedPlatform = useCallback((value: string) => {
    setPlatform((value || undefined) as UsagePlatform | undefined)
    onFilterChange()
  }, [onFilterChange, setPlatform])

  const updateSelectedRange = useCallback((value: UsageRangePreset) => {
    setRangePreset(value)
    onFilterChange()
  }, [onFilterChange, setRangePreset])

  const doImport = useCallback(async () => {
    try {
      const detection = await llmusageInstallDetect()
      if (detection.status === 'absent') {
        setShowInstallDialog(true)
        return
      }
    } catch {
      // 检测失败时继续导入，由既有错误态承接。
    }
    await data.startImportJob({
      platform: undefined,
      reason: 'manual',
      recentDays: getUsageRangePresetImportDays(selectedRange),
    })
  }, [data, selectedRange])

  useEffect(() => {
    void hydrateUsageLocale().then(() => setLocaleReady(true))
    const observer = new MutationObserver(() => setChartTheme(buildChartTheme()))
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class', 'data-theme'] })
    const hydrateTimer = window.setTimeout(() => setTrendChartReady(true), 180)
    const distTimer = window.setTimeout(() => setDistributionChartReady(true), 360)
    return () => {
      observer.disconnect()
      window.clearTimeout(hydrateTimer)
      window.clearTimeout(distTimer)
    }
  }, [])

  const tabKeys = ['overview', 'tokens', 'cost', 'providers', 'models', 'projects', 'logs'] as const
  const summaryCards = presentation.summaryCards
  const costSummaryCard = summaryCards.find((card) => card.id === 'cost') ?? null
  const otherSummaryCards = summaryCards.filter((card) => card.id !== 'cost')
  const cacheCreationTokens = (dashboardReady ? data.trends : []).reduce(
    (sum, item) => sum + item.cache_creation_tokens,
    0,
  )

  return {
    ...data,
    t,
    locale,
    activeTab,
    setActiveTab,
    selectedPlatform,
    selectedRange,
    selectedPlatformLabel,
    selectedWindowLabel,
    logModelFilter,
    updateLogModelFilter: setLogModelFilter,
    localeReady,
    dashboardReady,
    showInstallDialog,
    setShowInstallDialog,
    tabKeys,
    summaryCards,
    costSummaryCard,
    otherSummaryCards,
    cacheCreationTokens,
    chartTheme,
    trendOptions,
    trendSeries,
    pieOptions,
    pieSeries,
    pieColors,
    modelTokenPieOptions,
    modelTokenPieSeries,
    modelDistribution: presentation.modelDistribution,
    modelTokenDistribution: presentation.modelTokenDistribution,
    overviewHighlights: presentation.overviewHighlights,
    topModelRankings: presentation.topModelRankings,
    topProjectRankings: presentation.topProjectRankings,
    trendSubtitle: dashboardReady
      ? translateDashboardText(
        'usage.dashboard.chart.trendSubtitle',
        { granularity: trendGranularityLabel, window: selectedWindowLabel, points: presentation.trendBuckets.length },
        `${selectedWindowLabel} · ${trendGranularityLabel}`,
      )
      : '',
    trendGranularityLabel,
    distributionSubtitle: presentation.modelDistribution.length
      ? translateDashboardText('usage.dashboard.chart.distributionAllVisible', { total: data.modelStats.length }, `${data.modelStats.length} models`)
      : '',
    hasRenderableTrendData: trendSeries.some((series) => series.data.length > 0),
    shouldRenderTrendChart: trendChartReady,
    shouldRenderDistributionChart: distributionChartReady,
    opsCockpit,
    diagnosticsSummary,
    dashboardMetaItems: dashboardReady
      ? buildDashboardMetaItems({
        archive: data.archive,
        locale,
        modelCount: data.modelStats.length,
        projectCount: data.projectStats.length,
        selectedPlatformLabel,
        selectedWindowLabel,
        translate: translateDashboardText,
      })
      : [],
    importButtonLabel: importLabelOf(data.isBootstrapping, data.importing, t),
    showEmptyState: data.hasNoUsageData,
    emptyStateTitle: t('usage.states.noData'),
    emptyStateDescription: t('usage.states.noDataHint', { platform: selectedPlatform || 'AI' }),
    unsupportedStateTitle: t('usage.unsupported.waiting_for_llmusage.title'),
    unsupportedStateDescription: t('usage.unsupported.waiting_for_llmusage.description'),
    logsRecords,
    formatCost,
    formatTokens,
    doImport,
    doImportAfterInstall: async () => {
      setShowInstallDialog(false)
      await data.startImportJob({
        platform: undefined,
        reason: 'manual',
        recentDays: getUsageRangePresetImportDays(selectedRange),
      })
    },
    updateSelectedPlatform,
    updateSelectedRange,
    onFilterChange,
    handleOpsPrimaryAction: async (action: 'import' | 'diagnostics' | 'none') => {
      if (action === 'none') return
      if (action === 'import') {
        await doImport()
        return
      }
      setActiveTab('logs')
      await data.fetchLogs('reset')
    },
    loadLogs: data.fetchLogs,
    repairCodexLogs: async () => {
      if (selectedPlatform !== 'codex' || data.importing) return
      await data.startImportJob({
        platform: 'codex',
        reason: 'manual',
        recentDays: getUsageRangePresetImportDays(selectedRange),
        resetSources: true,
      })
    },
    diagnosticsEmptyMessage: logModelFilter
      ? t('usage.dashboard.diagnostics.filteredNoResults')
      : t('usage.dashboard.logs.noLogs'),
    diagnosticsEmptyDetail: logModelFilter
      ? t('usage.dashboard.diagnostics.filteredNoResultsHint')
      : t('usage.dashboard.diagnostics.emptyHint'),
    repairCodexButtonLabel: data.importing
      ? t('usage.dashboard.diagnostics.repairingCodex')
      : t('usage.dashboard.diagnostics.repairCodex'),
  }
}

export type UsageDashboardController = ReturnType<typeof useUsageDashboard>

function importLabelOf(bootstrapping: boolean, importing: boolean, t: (key: string) => string) {
  if (bootstrapping) return t('usage.dashboard.bootstrapping')
  if (importing) return t('usage.dashboard.importing')
  return t('usage.dashboard.import')
}
