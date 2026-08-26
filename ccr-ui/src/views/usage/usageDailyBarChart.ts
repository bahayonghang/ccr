import {
  buildChartAnimations,
  formatTrendAxisLabel,
  parseUtcDate,
  type ChartThemeState,
} from './usageChartOptions'
import type { TrendGranularity } from './usageDashboardPresentation'

export type DailyBarPoint = {
  x: number
  y: number
}

export type DailyBarSeries = {
  name: string
  data: DailyBarPoint[]
}

export type DailyBarPalette = 'tokens' | 'cost'

const DAILY_BAR_CHART_BASE = Object.freeze({
  background: 'transparent',
  fontFamily: 'inherit',
  parentHeightOffset: 0,
  redrawOnParentResize: false,
  redrawOnWindowResize: false,
  toolbar: { show: false },
})
const DAILY_BAR_DATA_LABELS = Object.freeze({ enabled: false })
const DAILY_BAR_AXIS_BORDER = Object.freeze({ show: false })
const DAILY_BAR_PLOT = Object.freeze({
  bar: { columnWidth: '58%', borderRadius: 1 },
})

export const toDailyBarPoints = <T extends { date: string }>(
  rows: readonly T[],
  pick: (row: T) => number,
): DailyBarPoint[] =>
  rows.map((row) => ({
    x: parseUtcDate(row.date).getTime(),
    y: pick(row),
  }))

export const dailyBarSeriesKey = (series: DailyBarSeries[]) =>
  series
    .map((item) => `${item.name}:${item.data.map((point) => `${point.x}=${point.y}`).join(',')}`)
    .join('\n')

export const stabilizeDailyBarSeries = (
  previous: DailyBarSeries[] | undefined,
  next: DailyBarSeries[],
): DailyBarSeries[] =>
  previous && dailyBarSeriesKey(previous) === dailyBarSeriesKey(next) ? previous : next

const colorsForPalette = (theme: ChartThemeState, palette: DailyBarPalette) => {
  if (palette === 'cost') return [theme.primary]
  return [theme.inputToken, theme.outputToken, theme.cacheReadToken]
}

export interface DailyBarChartOptionsInput {
  theme: ChartThemeState
  locale: string
  granularity: TrendGranularity
  tickAmount: number | undefined
  stacked: boolean
  palette: DailyBarPalette
  formatY: (value: number) => string
}

export const buildDailyBarChartOptions = ({
  theme,
  locale,
  granularity,
  tickAmount,
  stacked,
  palette,
  formatY,
}: DailyBarChartOptionsInput) => ({
  chart: {
    ...DAILY_BAR_CHART_BASE,
    stacked,
    animations: buildChartAnimations(),
  },
  theme: { mode: theme.mode },
  colors: colorsForPalette(theme, palette),
  dataLabels: DAILY_BAR_DATA_LABELS,
  plotOptions: DAILY_BAR_PLOT,
  xaxis: {
    type: 'datetime' as const,
    tickAmount,
    labels: {
      trim: false,
      datetimeUTC: true,
      rotate: 0,
      hideOverlappingLabels: true,
      style: { colors: theme.textMuted, fontSize: '11px' },
      formatter: (value: string, timestamp?: number) =>
        formatTrendAxisLabel(timestamp ?? Number(value), granularity, locale),
    },
    axisBorder: DAILY_BAR_AXIS_BORDER,
    axisTicks: { color: theme.grid },
  },
  yaxis: {
    min: 0,
    labels: {
      style: { colors: theme.textMuted },
      formatter: formatY,
    },
  },
  grid: {
    borderColor: theme.grid,
    strokeDashArray: 4,
    padding: { left: 4, right: 6, bottom: 2, top: 6 },
  },
  legend: {
    show: true,
    showForSingleSeries: true,
    position: 'top' as const,
    horizontalAlign: 'right' as const,
    labels: { colors: theme.textSecondary },
    markers: { strokeWidth: 0 },
  },
  tooltip: {
    theme: theme.mode,
    shared: stacked,
    intersect: false,
    x: {
      formatter: (value: number) => formatTrendAxisLabel(value, granularity, locale),
    },
  },
})
