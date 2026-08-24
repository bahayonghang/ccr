import type { DailyPoint, HeatmapCell } from '@/types/claudeObserver'
import { formatTokens } from '@/features/claude/observer/formatters'
import { infoScaleColor, readObserverChartTheme } from '@/features/claude/observer/chartTheme'

const WEEK_LABELS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']
const WEEK_ORDER = [1, 2, 3, 4, 5, 6, 0]

interface ChartMotion {
  enabled: boolean
}

export function dailyCostSeries(daily: DailyPoint[]) {
  return [
    {
      name: 'USD',
      data: daily.map((point) => ({ x: point.date, y: Number(point.cost_usd.toFixed(4)) })),
    },
  ]
}

export function dailyCostOptions(motion: ChartMotion) {
  const theme = readObserverChartTheme()
  return {
    chart: {
      background: 'transparent',
      toolbar: { show: false },
      fontFamily: 'inherit',
      parentHeightOffset: 0,
      redrawOnParentResize: false,
      redrawOnWindowResize: false,
      animations: { enabled: motion.enabled, speed: 220, easing: 'easeinout' },
    },
    theme: { mode: theme.mode },
    colors: [theme.primary],
    stroke: { curve: 'smooth', width: 2 },
    fill: {
      type: 'gradient',
      gradient: { shadeIntensity: 1, opacityFrom: 0.32, opacityTo: 0.05, stops: [0, 100] },
    },
    dataLabels: { enabled: false },
    grid: { borderColor: theme.grid, strokeDashArray: 3, padding: { left: 12, right: 12 } },
    xaxis: {
      type: 'datetime' as const,
      labels: { style: { colors: theme.textMuted, fontSize: '0.6875rem' }, datetimeUTC: false },
      axisBorder: { show: false },
      axisTicks: { color: theme.grid },
    },
    yaxis: {
      labels: {
        style: { colors: theme.textMuted, fontSize: '0.6875rem' },
        formatter: (value: number) => `$${value.toFixed(2)}`,
      },
    },
    tooltip: {
      theme: theme.mode,
      x: { format: 'yyyy-MM-dd' },
      y: { formatter: (value: number) => `$${value.toFixed(4)}` },
    },
    legend: { show: false },
  }
}

export function tokenStackSeries(daily: DailyPoint[]) {
  return [
    { name: 'Cache read', data: daily.map((point) => ({ x: point.date, y: point.cache_read_tokens })) },
    { name: 'Cache write', data: daily.map((point) => ({ x: point.date, y: point.cache_write_tokens })) },
    { name: 'Input', data: daily.map((point) => ({ x: point.date, y: point.input_tokens })) },
    { name: 'Output', data: daily.map((point) => ({ x: point.date, y: point.output_tokens })) },
  ]
}

export function tokenStackOptions(motion: ChartMotion) {
  const theme = readObserverChartTheme()
  return {
    chart: {
      type: 'bar' as const,
      stacked: true,
      background: 'transparent',
      toolbar: { show: false },
      fontFamily: 'inherit',
      parentHeightOffset: 0,
      redrawOnParentResize: false,
      redrawOnWindowResize: false,
      animations: { enabled: motion.enabled, speed: 220, easing: 'easeinout' },
    },
    theme: { mode: theme.mode },
    colors: [theme.primary, theme.secondary, theme.tertiary, theme.info],
    plotOptions: { bar: { columnWidth: '55%', borderRadius: 2 } },
    dataLabels: { enabled: false },
    stroke: { width: 0 },
    grid: { borderColor: theme.grid, strokeDashArray: 3, padding: { left: 12, right: 12 } },
    xaxis: {
      type: 'datetime' as const,
      labels: { style: { colors: theme.textMuted, fontSize: '0.6875rem' }, datetimeUTC: false },
      axisBorder: { show: false },
      axisTicks: { color: theme.grid },
    },
    yaxis: {
      labels: {
        style: { colors: theme.textMuted, fontSize: '0.6875rem' },
        formatter: (value: number) => formatTokens(value),
      },
    },
    tooltip: {
      theme: theme.mode,
      x: { format: 'yyyy-MM-dd' },
      y: { formatter: (value: number) => formatTokens(value) },
    },
    legend: {
      position: 'bottom' as const,
      labels: { colors: theme.textSecondary },
    },
  }
}

export function heatmapSeries(cells: HeatmapCell[]) {
  const matrix: Record<number, Record<number, number>> = {}
  for (const cell of cells) {
    if (!matrix[cell.dow]) matrix[cell.dow] = {}
    matrix[cell.dow][cell.hour] = (matrix[cell.dow][cell.hour] ?? 0) + cell.count
  }
  return WEEK_ORDER.map((dow) => ({
    name: WEEK_LABELS[dow],
    data: Array.from({ length: 24 }, (_, hour) => ({
      x: `${hour.toString().padStart(2, '0')}`,
      y: matrix[dow]?.[hour] ?? 0,
    })),
  }))
}

export function heatmapOptions(motion: ChartMotion) {
  const theme = readObserverChartTheme()
  return {
    chart: {
      type: 'heatmap' as const,
      background: 'transparent',
      toolbar: { show: false },
      fontFamily: 'inherit',
      parentHeightOffset: 0,
      redrawOnParentResize: false,
      redrawOnWindowResize: false,
      animations: { enabled: motion.enabled, speed: 220 },
    },
    theme: { mode: theme.mode },
    dataLabels: { enabled: false },
    plotOptions: {
      heatmap: {
        radius: 3,
        enableShades: false,
        colorScale: {
          ranges: [
            { from: 0, to: 0, color: infoScaleColor('0.08'), name: '0' },
            { from: 1, to: 4, color: infoScaleColor('0.28'), name: '1-4' },
            { from: 5, to: 14, color: infoScaleColor('0.5'), name: '5-14' },
            { from: 15, to: 49, color: infoScaleColor('0.72'), name: '15-49' },
            { from: 50, to: 100000, color: infoScaleColor('0.92'), name: '50+' },
          ],
        },
      },
    },
    grid: { borderColor: theme.grid, padding: { left: 4, right: 4 } },
    xaxis: {
      type: 'category' as const,
      labels: { style: { colors: theme.textMuted, fontSize: '0.625rem' } },
      axisBorder: { show: false },
      axisTicks: { show: false },
    },
    yaxis: { labels: { style: { colors: theme.textMuted, fontSize: '0.6875rem' } } },
    tooltip: { theme: theme.mode },
    legend: { show: false },
  }
}
