import type { UsageTrendBucket, TrendGranularity } from './usageDashboardPresentation'

type DisplayUsageTrendBucket = UsageTrendBucket & { displayEndDate: string }

export interface ApexFormatterContext {
  dataPointIndex?: number
  seriesIndex?: number
  globals?: {
    seriesTotals?: number[]
  }
}

export interface ApexCustomTooltipContext {
  dataPointIndex?: number
  series?: number[][]
  w?: {
    globals?: {
      colors?: string[]
      seriesNames?: string[]
    }
  }
}

export type ChartThemeState = {
  mode: 'light' | 'dark'
  primary: string
  secondary: string
  tertiary: string
  quaternary: string
  success: string
  info: string
  warning: string
  reasoningToken: string
  muted: string
  inputToken: string
  outputToken: string
  cacheReadToken: string
  cacheCreationToken: string
  textPrimary: string
  textSecondary: string
  textMuted: string
  grid: string
  border: string
}

export type ModelDistributionMetric = 'cost' | 'tokens'
export type UsageTokenCategory = 'input' | 'output' | 'cacheRead' | 'cacheCreation' | 'reasoning'

export type UsageTokenCategoryColor = {
  cssVar: string
  rgbVar: string
  fallback: string
}

export const usageTokenCategoryColors: Record<UsageTokenCategory, UsageTokenCategoryColor> = {
  input: {
    cssVar: '--color-success',
    rgbVar: '--color-success-rgb',
    fallback: '#5B8A62',
  },
  output: {
    cssVar: '--color-info',
    rgbVar: '--color-info-rgb',
    fallback: '#7D97B6',
  },
  cacheRead: {
    cssVar: '--color-accent-secondary',
    rgbVar: '--color-accent-secondary-rgb',
    fallback: '#B99666',
  },
  cacheCreation: {
    cssVar: '--color-warning',
    rgbVar: '--color-warning-rgb',
    fallback: '#BC8540',
  },
  reasoning: {
    cssVar: '--color-accent-primary',
    rgbVar: '--color-accent-primary-rgb',
    fallback: '#8A7D66',
  },
}

const parseUtcDate = (value: string) => {
  const [year, month, day] = value.split('-').map(Number)
  return new Date(Date.UTC(year, (month || 1) - 1, day || 1))
}

const buildDateFormatters = (locale: string) => ({
  day: new Intl.DateTimeFormat(locale, { month: 'short', day: 'numeric', timeZone: 'UTC' }),
  month: new Intl.DateTimeFormat(locale, { month: 'short', year: 'numeric', timeZone: 'UTC' }),
})

export const formatTrendAxisLabel = (
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

export const formatTrendTooltipLabel = (
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

export const getTrendTickAmount = (pointCount: number) => {
  if (pointCount <= 0) return undefined
  if (pointCount <= 8) return pointCount
  if (pointCount <= 16) return 8
  return 6
}

export const escapeTooltipText = (value: string) =>
  value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')

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

export const buildChartTheme = (): ChartThemeState => ({
  mode: detectThemeMode(),
  primary: readCssVar('--color-accent-primary', '#0071E3'),
  secondary: readCssVar('--color-accent-secondary', '#2997FF'),
  tertiary: readCssVar('--color-info', '#5AC8FA'),
  quaternary: readCssVar('--color-warning', '#FF9F0A'),
  success: readCssVar('--color-success', '#5B8A62'),
  info: readCssVar('--color-info', '#7D97B6'),
  warning: readCssVar('--color-warning', '#BC8540'),
  reasoningToken: readCssVar(
    usageTokenCategoryColors.reasoning.cssVar,
    usageTokenCategoryColors.reasoning.fallback,
  ),
  muted: readCssVar('--color-text-muted', '#6E6E73'),
  inputToken: readCssVar(
    usageTokenCategoryColors.input.cssVar,
    usageTokenCategoryColors.input.fallback,
  ),
  outputToken: readCssVar(
    usageTokenCategoryColors.output.cssVar,
    usageTokenCategoryColors.output.fallback,
  ),
  cacheReadToken: readCssVar(
    usageTokenCategoryColors.cacheRead.cssVar,
    usageTokenCategoryColors.cacheRead.fallback,
  ),
  cacheCreationToken: readCssVar(
    usageTokenCategoryColors.cacheCreation.cssVar,
    usageTokenCategoryColors.cacheCreation.fallback,
  ),
  textPrimary: readCssVar('--color-text-primary', '#1D1D1F'),
  textSecondary: readCssVar('--color-text-secondary', '#424245'),
  textMuted: readCssVar('--color-text-muted', '#6E6E73'),
  grid: readCssVar('--color-border-subtle', 'rgb(29 29 31 / 8%)'),
  border: readCssVar('--color-border-default', 'rgb(29 29 31 / 12%)'),
})

export type BuildTrendTooltipInput = {
  context: ApexCustomTooltipContext
  buckets: DisplayUsageTrendBucket[]
  granularity: TrendGranularity
  locale: string
  seriesNames: string[]
  fallbackColor: string
  costLabel: string
  formatTokens: (value: number) => string
  formatCost: (value: number) => string
}

export const buildTrendTooltipHtml = ({
  context,
  buckets,
  granularity,
  locale,
  seriesNames,
  fallbackColor,
  costLabel,
  formatTokens,
  formatCost,
}: BuildTrendTooltipInput) => {
  const index = context.dataPointIndex ?? -1
  const bucket = buckets[index]
  if (!bucket) return ''

  const label = formatTrendTooltipLabel(bucket.startDate, bucket.displayEndDate, granularity, locale)
  const colors = context.w?.globals?.colors ?? []
  const names = context.w?.globals?.seriesNames ?? []
  const rows = (context.series ?? []).map((series, seriesIndex) => {
    const name = escapeTooltipText(names[seriesIndex] ?? seriesNames[seriesIndex] ?? '')
    const value = formatTokens(series[index] ?? 0)
    const color = colors[seriesIndex] ?? fallbackColor
    return `<div class="usage-chart-tooltip__row"><span><i style="background:${color}"></i>${name}</span><strong>${value}</strong></div>`
  }).join('')

  return `<div class="usage-chart-tooltip"><div class="usage-chart-tooltip__title">${escapeTooltipText(label)}</div>${rows}<div class="usage-chart-tooltip__row usage-chart-tooltip__row--cost"><span>${escapeTooltipText(costLabel)}</span><strong>${formatCost(bucket.costUsd)}</strong></div></div>`
}

// ============ 图表静态骨架（模块级冻结常量）============
// 这些片段既不含主题色也不含数据，跨渲染共享同一引用。ApexCharts 内部会深拷贝 options，
// 因此冻结既能表达“静态骨架”的意图，又能拦截意外的原地修改。
const TREND_CHART_BASE = Object.freeze({
  background: 'transparent',
  fontFamily: 'inherit',
  parentHeightOffset: 0,
  redrawOnParentResize: false,
  redrawOnWindowResize: false,
  animations: { enabled: false },
  toolbar: { show: false },
})
const TREND_MARKERS = Object.freeze({ size: 0, hover: { size: 0, sizeOffset: 0 } })
const TREND_STROKE = Object.freeze({ curve: 'smooth' as const, width: 2.2 })
const TREND_FILL = Object.freeze({
  type: 'gradient',
  gradient: { opacityFrom: 0.32, opacityTo: 0.04 },
})
const TREND_DATA_LABELS = Object.freeze({ enabled: false })
const TREND_AXIS_BORDER = Object.freeze({ show: false })

const PIE_CHART_BASE = Object.freeze({
  background: 'transparent',
  fontFamily: 'inherit',
  parentHeightOffset: 0,
  redrawOnParentResize: false,
  redrawOnWindowResize: false,
  animations: { enabled: false },
})
const PIE_LEGEND = Object.freeze({ show: false })
const PIE_DATA_LABEL_STYLE = Object.freeze({ fontSize: '11px', fontWeight: 600 })
const PIE_DATA_LABEL_DROP_SHADOW = Object.freeze({ enabled: false })

// 趋势图 options 工厂：只注入主题色、坐标轴标量（tickAmount/granularity）、系列名与格式化闭包。
// buckets 走 getBuckets 取值器而非快照——tooltip 闭包在渲染时才读当前数据，
// options 因此无需随数据刷新重建（详见 useUsageCharts 的记忆化说明）。
export interface TrendChartOptionsInput {
  theme: ChartThemeState
  locale: string
  granularity: TrendGranularity
  tickAmount: number | undefined
  seriesNames: string[]
  costLabel: string
  getBuckets: () => DisplayUsageTrendBucket[]
  formatTokens: (value: number) => string
  formatCost: (value: number) => string
}

export const buildTrendChartOptions = ({
  theme,
  locale,
  granularity,
  tickAmount,
  seriesNames,
  costLabel,
  getBuckets,
  formatTokens,
  formatCost,
}: TrendChartOptionsInput) => ({
  chart: TREND_CHART_BASE,
  theme: { mode: theme.mode },
  colors: [theme.inputToken, theme.outputToken, theme.cacheReadToken],
  xaxis: {
    type: 'datetime' as const,
    tickAmount,
    labels: {
      style: { colors: theme.textMuted, fontSize: '11px' },
      datetimeUTC: false,
      formatter: (_value: string, timestamp?: number) => {
        if (timestamp == null) return ''
        return formatTrendAxisLabel(timestamp, granularity, locale)
      },
    },
    axisBorder: TREND_AXIS_BORDER,
    axisTicks: { color: theme.grid },
  },
  yaxis: [
    {
      seriesName: seriesNames[0],
      labels: {
        style: { colors: theme.textMuted },
        formatter: (value: number) => formatTokens(value),
      },
    },
    {
      seriesName: seriesNames[1],
      opposite: true,
      showAlways: true,
      labels: {
        style: { colors: theme.textMuted },
        formatter: (value: number) => formatTokens(value),
      },
    },
    {
      seriesName: seriesNames[2],
      show: false,
      labels: {
        style: { colors: theme.textMuted },
        formatter: (value: number) => formatTokens(value),
      },
    },
  ],
  markers: TREND_MARKERS,
  stroke: TREND_STROKE,
  fill: TREND_FILL,
  dataLabels: TREND_DATA_LABELS,
  tooltip: {
    theme: theme.mode,
    shared: true,
    intersect: false,
    custom: (context: ApexCustomTooltipContext) =>
      buildTrendTooltipHtml({
        context,
        buckets: getBuckets(),
        granularity,
        locale,
        seriesNames,
        fallbackColor: theme.primary,
        costLabel,
        formatTokens,
        formatCost,
      }),
    x: {
      // ApexCharts 把 formatter 的 context 类型标成 any，这里收口到本地 ApexFormatterContext。
      formatter: (_value: string, context: ApexFormatterContext) => {
        const bucket = getBuckets()[context?.dataPointIndex ?? -1]
        if (!bucket) return _value
        return formatTrendTooltipLabel(bucket.startDate, bucket.displayEndDate, granularity, locale)
      },
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
})

// 分布饼图 options 工厂：labels/colors 由调用方以“按值记忆化”的方式传入，
// metric 决定数值/费用格式化，totalLabel 承载中心总计文案（随 locale 变化）。
export interface DistributionPieOptionsInput {
  metric: ModelDistributionMetric
  theme: ChartThemeState
  colors: string[]
  labels: string[]
  totalLabel: string
  formatTokens: (value: number) => string
  formatCost: (value: number) => string
}

export const buildDistributionPieOptions = ({
  metric,
  theme,
  colors,
  labels,
  totalLabel,
  formatTokens,
  formatCost,
}: DistributionPieOptionsInput) => ({
  chart: PIE_CHART_BASE,
  theme: { mode: theme.mode },
  colors,
  labels,
  legend: PIE_LEGEND,
  plotOptions: {
    pie: {
      donut: {
        size: '72%',
        labels: {
          show: true,
          name: {
            show: true,
            fontSize: '11px',
            color: theme.textMuted,
            offsetY: -2,
          },
          value: {
            show: true,
            fontSize: '15px',
            fontWeight: 600,
            color: theme.textPrimary,
            formatter: (value: string) =>
              metric === 'tokens' ? formatTokens(Number(value)) : formatCost(Number(value)),
          },
          total: {
            show: true,
            label: totalLabel,
            fontSize: '10px',
            color: theme.textMuted,
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
    style: PIE_DATA_LABEL_STYLE,
    dropShadow: PIE_DATA_LABEL_DROP_SHADOW,
  },
  tooltip: {
    theme: theme.mode,
    y: {
      formatter: (value: number) =>
        metric === 'tokens' ? formatTokens(value) : formatCost(value),
    },
  },
})
