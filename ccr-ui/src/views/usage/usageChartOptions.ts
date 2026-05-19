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
  textPrimary: string
  textSecondary: string
  textMuted: string
  grid: string
  border: string
}

export type ModelDistributionMetric = 'cost' | 'tokens'

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
