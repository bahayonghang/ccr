import type { HomeInsightsResponse } from '@/types/generated/usage/HomeInsightsResponse'
import {
  INSIGHTS_OTHER_KEY,
  INSIGHTS_STACK_ORDER,
  isMappedSource,
  sourceRank,
} from './insightsSources'

// 热力图与周趋势的网格构建（纯函数）。日期一律按 UTC 零点解析 `YYYY-MM-DD`，
// 只做日历运算，不涉及时区换算（后端已按报告时区给出本地日期）。

export const INSIGHTS_WEEKS = 53
export const INSIGHTS_TOP_SOURCES = 4
const DAY_MS = 86_400_000

const parseDate = (value: string) => Date.parse(`${value}T00:00:00Z`)

const formatDate = (ms: number) => new Date(ms).toISOString().slice(0, 10)

export const addDays = (value: string, days: number) => formatDate(parseDate(value) + days * DAY_MS)

export type HeatLevel = 0 | 1 | 2 | 3 | 4

export interface HeatCell {
  date: string
  requests: number
  level: HeatLevel
  /** 晚于 as_of 的格：图表不渲染，表格留空。 */
  future: boolean
}

export interface HeatWeek {
  weekStart: string
  cells: HeatCell[]
  total: number
}

export interface HeatMonthLabel {
  column: number
  /** 1–12 */
  month: number
}

export interface HeatmapModel {
  weeks: HeatWeek[]
  months: HeatMonthLabel[]
  activeDays: number
}

/** 线性插值分位（与常见统计软件默认一致）；空数组返回 0。 */
const quantile = (sorted: number[], q: number) => {
  if (sorted.length === 0) return 0
  const position = (sorted.length - 1) * q
  const lower = Math.floor(position)
  const upper = Math.min(lower + 1, sorted.length - 1)
  return sorted[lower] + (sorted[upper] - sorted[lower]) * (position - lower)
}

export const heatThresholds = (values: number[]) => {
  const sorted = values.filter((value) => value > 0).sort((a, b) => a - b)
  return [quantile(sorted, 0.25), quantile(sorted, 0.5), quantile(sorted, 0.75)] as const
}

export const heatLevel = (
  requests: number,
  thresholds: readonly [number, number, number]
): HeatLevel => {
  if (requests <= 0) return 0
  if (requests <= thresholds[0]) return 1
  if (requests <= thresholds[1]) return 2
  if (requests <= thresholds[2]) return 3
  return 4
}

const monthOf = (date: string) => Number(date.slice(5, 7))

const buildMonthLabels = (weeks: HeatWeek[]): HeatMonthLabel[] => {
  const labels: HeatMonthLabel[] = []
  weeks.forEach((week, column) => {
    const month = monthOf(week.weekStart)
    const previous = column > 0 ? monthOf(weeks[column - 1].weekStart) : null
    if (month !== previous) labels.push({ column, month })
  })
  return labels
}

export const buildHeatmap = (data: HomeInsightsResponse): HeatmapModel => {
  const byDate = new Map(data.daily.map((day) => [day.date, day.requests]))
  const thresholds = heatThresholds(data.daily.map((day) => day.requests))
  const asOf = parseDate(data.as_of)
  const weeks = Array.from({ length: INSIGHTS_WEEKS }, (_, column): HeatWeek => {
    const weekStart = addDays(data.trend_start, column * 7)
    const cells = Array.from({ length: 7 }, (__, row): HeatCell => {
      const date = addDays(weekStart, row)
      const requests = byDate.get(date) ?? 0
      return {
        date,
        requests,
        level: heatLevel(requests, thresholds),
        future: parseDate(date) > asOf,
      }
    })
    return { weekStart, cells, total: cells.reduce((sum, cell) => sum + cell.requests, 0) }
  })
  return {
    weeks,
    months: buildMonthLabels(weeks),
    activeDays: data.daily.filter((day) => day.requests > 0).length,
  }
}

export interface HeatTableRow {
  weekStart: string
  /** 周一至周日；晚于 as_of 为 null。 */
  days: Array<number | null>
  total: number
}

/** 热力图表格视图：最新周在前。 */
export const buildHeatTable = (model: HeatmapModel): HeatTableRow[] =>
  [...model.weeks].reverse().map((week) => ({
    weekStart: week.weekStart,
    days: week.cells.map((cell) => (cell.future ? null : cell.requests)),
    total: week.total,
  }))

export interface TrendSeriesMeta {
  /** 来源键或 INSIGHTS_OTHER_KEY */
  key: string
  total: number
}

export interface TrendWeek {
  weekStart: string
  /** 与 series 同序 */
  values: number[]
  total: number
}

export interface TrendModel {
  series: TrendSeriesMeta[]
  weeks: TrendWeek[]
  max: number
}

const sumWeekly = (weekly: number[]) => weekly.reduce((sum, value) => sum + value, 0)

/** 前 4 来源：只取已登记来源，按窗口总量降序，同量按 SourceKind::ALL 顺序。 */
export const pickTopSources = (trend: HomeInsightsResponse['trend']) =>
  trend
    .filter((series) => isMappedSource(series.source) && sumWeekly(series.weekly) > 0)
    .map((series) => ({ source: series.source, total: sumWeekly(series.weekly) }))
    .sort((a, b) => b.total - a.total || sourceRank(a.source) - sourceRank(b.source))
    .slice(0, INSIGHTS_TOP_SOURCES)
    .map((entry) => entry.source)

const weeklyAt = (weekly: number[], index: number) => weekly[index] ?? 0

export const buildTrend = (data: HomeInsightsResponse): TrendModel => {
  const top = new Set(pickTopSources(data.trend))
  const topSeries = INSIGHTS_STACK_ORDER.filter((source) => top.has(source))
    .map((source) => data.trend.find((series) => series.source === source))
    .filter((series) => series !== undefined)
  const others = data.trend.filter((series) => !top.has(series.source))
  const otherWeekly = Array.from({ length: INSIGHTS_WEEKS }, (_, index) =>
    others.reduce((sum, series) => sum + weeklyAt(series.weekly, index), 0)
  )
  const columns = topSeries.map((series) => ({ key: series.source, weekly: series.weekly }))
  if (sumWeekly(otherWeekly) > 0) columns.push({ key: INSIGHTS_OTHER_KEY, weekly: otherWeekly })
  const weeks = Array.from({ length: INSIGHTS_WEEKS }, (_, index): TrendWeek => {
    const values = columns.map((column) => weeklyAt(column.weekly, index))
    return {
      weekStart: addDays(data.trend_start, index * 7),
      values,
      total: values.reduce((sum, value) => sum + value, 0),
    }
  })
  return {
    series: columns.map((column) => ({ key: column.key, total: sumWeekly(column.weekly) })),
    weeks,
    max: Math.max(0, ...weeks.map((week) => week.total)),
  }
}

/** 周趋势表格视图：最新周在前。 */
export const buildTrendTable = (model: TrendModel): TrendWeek[] => [...model.weeks].reverse()
