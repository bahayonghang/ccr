import type { HomeInsightsResponse } from '@/types/generated/usage/HomeInsightsResponse'
import { isMappedSource, OTHER_COLOR, sourceColor } from './insightsSources'

export * from './insightsCalendar'

// 首页 Insights 展示逻辑（纯函数）：状态判定、总量、7 天对比、streak 文案、
// 分布、榜单。组件只渲染这里的结果，不再自行排序或推算。

type Insights = HomeInsightsResponse

export const EM_DASH = '—'
const MINUS = '−'

export interface InsightsFlags {
  emptyStore: boolean
  hasRequests: boolean
  /** 会话未索引但有用量：显示说明行。 */
  usageOnly: boolean
}

export const insightsFlags = (data: Insights): InsightsFlags => ({
  emptyStore: data.totals.requests === 0 && data.totals.sessions === 0,
  hasRequests: data.totals.requests > 0,
  usageOnly: !data.sessions_indexed && data.totals.requests > 0,
})


export type TotalsKey = 'sessions' | 'tokens' | 'requests' | 'agents' | 'projects' | 'activeDays'

export interface TotalsItem {
  key: TotalsKey
  /** null → 显示 `—` */
  value: number | null
  /** 会话未索引：`—` 带说明 title */
  unindexed: boolean
}

/** 会话诚实：未索引时一律 null（显示 `—`），不显示 0。 */
export const sessionValue = (value: number, indexed: boolean) => (indexed ? value : null)

export const buildTotals = (data: Insights): TotalsItem[] => {
  const { totals } = data
  const usage = (value: number) => (totals.requests > 0 ? value : null)
  // 已索引的 0 是真实值，显示 0；只有未索引或空库才显示 `—`。
  const sessions =
    data.sessions_indexed && (totals.sessions > 0 || totals.requests > 0) ? totals.sessions : null
  return [
    { key: 'sessions', value: sessions, unindexed: !data.sessions_indexed },
    { key: 'tokens', value: usage(totals.tokens), unindexed: false },
    { key: 'requests', value: usage(totals.requests), unindexed: false },
    { key: 'agents', value: usage(totals.agents), unindexed: false },
    { key: 'projects', value: usage(totals.projects), unindexed: false },
    { key: 'activeDays', value: usage(totals.active_days), unindexed: false },
  ]
}

export type InsightsChange =
  | { kind: 'percent'; value: number }
  | { kind: 'days'; value: number }
  | { kind: 'flat' }
  | { kind: 'new' }
  | { kind: 'none' }

export const percentChange = (last: number | null, previous: number | null): InsightsChange => {
  if (last === null || previous === null) return { kind: 'none' }
  if (previous === 0) return last > 0 ? { kind: 'new' } : { kind: 'none' }
  if (last === previous) return { kind: 'flat' }
  return { kind: 'percent', value: (last - previous) / previous }
}

export const dayChange = (last: number | null, previous: number | null): InsightsChange => {
  if (last === null || previous === null) return { kind: 'none' }
  if (last === previous) return { kind: 'flat' }
  return { kind: 'days', value: last - previous }
}

const signed = (value: number, text: string) => `${value > 0 ? '+' : MINUS}${text}`

/** `+21.7%` / `−17.6%` / `+2 天` / `持平` / `新增` / `—`。 */
export const formatChange = (
  change: InsightsChange,
  t: (key: string, values?: Record<string, string>) => string
) => {
  switch (change.kind) {
    case 'percent':
      return signed(change.value, `${(Math.abs(change.value) * 100).toFixed(1)}%`)
    case 'days':
      return t('dashboard.insights.compare.dayDelta', {
        value: signed(change.value, String(Math.abs(change.value))),
      })
    case 'flat':
      return t('dashboard.insights.compare.flat')
    case 'new':
      return t('dashboard.insights.compare.added')
    default:
      return EM_DASH
  }
}

export type CompareKey = 'sessions' | 'requests' | 'activeDays'

export interface CompareRow {
  key: CompareKey
  last: number | null
  previous: number | null
  change: InsightsChange
}

export const buildWeekCompare = (data: Insights): CompareRow[] => {
  const indexed = data.sessions_indexed
  const usage = (value: number) => (data.totals.requests > 0 ? value : null)
  const sessionsLast = sessionValue(data.last_7_days.sessions, indexed)
  const sessionsPrev = sessionValue(data.previous_7_days.sessions, indexed)
  const requestsLast = usage(data.last_7_days.requests)
  const requestsPrev = usage(data.previous_7_days.requests)
  const daysLast = usage(data.last_7_days.active_days)
  const daysPrev = usage(data.previous_7_days.active_days)
  return [
    {
      key: 'sessions',
      last: sessionsLast,
      previous: sessionsPrev,
      change: percentChange(sessionsLast, sessionsPrev),
    },
    {
      key: 'requests',
      last: requestsLast,
      previous: requestsPrev,
      change: percentChange(requestsLast, requestsPrev),
    },
    {
      key: 'activeDays',
      last: daysLast,
      previous: daysPrev,
      change: dayChange(daysLast, daysPrev),
    },
  ]
}

export interface CaptionPart {
  key: string
  values: Record<string, string | number>
}

/** 热力图说明行：当前连续、最长连续、最忙日；无请求时为空（改显示导入提示）。 */
export const buildStreakCaption = (data: Insights): CaptionPart[] => {
  if (data.totals.requests === 0) return []
  const parts: CaptionPart[] = [
    { key: 'dashboard.insights.heatmap.currentStreak', values: { count: data.current_streak } },
    { key: 'dashboard.insights.heatmap.longestStreak', values: { count: data.longest_streak } },
  ]
  if (data.busiest_day) {
    parts.push({
      key: 'dashboard.insights.heatmap.busiestDay',
      values: { date: data.busiest_day.date, count: data.busiest_day.requests },
    })
  }
  return parts
}

/** 最大值下标；全零（或空）返回 null。同值取靠前者。 */
export const peakIndex = (values: number[]) => {
  let peak: number | null = null
  values.forEach((value, index) => {
    if (value > 0 && (peak === null || value > values[peak])) peak = index
  })
  return peak
}

export type DistributionView = 'hour' | 'weekday' | 'month'

export const WEEKDAY_KEYS = [
  'dashboard.insights.weekdays.mon',
  'dashboard.insights.weekdays.tue',
  'dashboard.insights.weekdays.wed',
  'dashboard.insights.weekdays.thu',
  'dashboard.insights.weekdays.fri',
  'dashboard.insights.weekdays.sat',
  'dashboard.insights.weekdays.sun',
] as const

export const MONTH_KEYS = [
  'dashboard.insights.months.m1',
  'dashboard.insights.months.m2',
  'dashboard.insights.months.m3',
  'dashboard.insights.months.m4',
  'dashboard.insights.months.m5',
  'dashboard.insights.months.m6',
  'dashboard.insights.months.m7',
  'dashboard.insights.months.m8',
  'dashboard.insights.months.m9',
  'dashboard.insights.months.m10',
  'dashboard.insights.months.m11',
  'dashboard.insights.months.m12',
] as const

export interface DistributionBar {
  /** 小时为数字文本；星期/月为 i18n 键 */
  label: string
  translate: boolean
  value: number
  peak: boolean
}

const distributionValues = (data: Insights, view: DistributionView) => {
  if (view === 'weekday') return data.weekday
  if (view === 'month') return data.monthly
  return data.hourly
}

const distributionLabel = (view: DistributionView, index: number) => {
  if (view === 'weekday') return WEEKDAY_KEYS[index] ?? String(index)
  if (view === 'month') return MONTH_KEYS[index] ?? String(index)
  return String(index)
}

export const buildDistribution = (data: Insights, view: DistributionView): DistributionBar[] => {
  const values = distributionValues(data, view)
  const peak = peakIndex(values)
  return values.map((value, index) => ({
    label: distributionLabel(view, index),
    translate: view !== 'hour',
    value,
    peak: index === peak,
  }))
}

/** 最忙时段（小时下标）；全零时 null，不显示该说明。 */
export const busiestHour = (data: Insights) => peakIndex(data.hourly)

export type LeaderDimension = 'agents' | 'projects' | 'models'
export type LeaderMetric = 'requests' | 'tokens' | 'sessions'

export const LEADERBOARD_ROWS = 8

export const defaultLeaderMetric = (data: Insights): LeaderMetric =>
  data.totals.requests === 0 && data.sessions_indexed && data.totals.sessions > 0
    ? 'sessions'
    : 'requests'

export const sessionsMetricAvailable = (dimension: LeaderDimension, indexed: boolean) =>
  dimension === 'agents' && indexed

/** 会话度量只对 Agents 且已索引时可用，否则回退请求数。 */
export const resolveLeaderMetric = (
  dimension: LeaderDimension,
  metric: LeaderMetric,
  indexed: boolean
): LeaderMetric =>
  metric === 'sessions' && !sessionsMetricAvailable(dimension, indexed) ? 'requests' : metric

export interface LeaderRow {
  key: string
  label: string
  value: number
  /** 0–1，相对首行 */
  ratio: number
  /** Agent 行为平台色；项目/模型为 null（中性色条） */
  color: string | null
  unmapped: boolean
}

interface LeaderSource {
  key: string
  label: string
  requests: number
  tokens: number
  sessions: number
  unmapped: boolean
  color: string | null
}

const leaderSources = (
  data: Insights,
  dimension: LeaderDimension,
  resolveLabel: (source: string) => string
): LeaderSource[] => {
  if (dimension === 'agents') {
    return data.agents.map((agent) => {
      const unmapped = agent.unmapped || !isMappedSource(agent.source)
      return {
        key: agent.source,
        label: unmapped ? agent.source : resolveLabel(agent.source),
        requests: agent.requests,
        tokens: agent.tokens,
        sessions: agent.sessions,
        unmapped,
        color: unmapped ? OTHER_COLOR : sourceColor(agent.source),
      }
    })
  }
  const rows = dimension === 'projects' ? data.projects : data.models
  return rows.map((row) => ({
    ...row,
    label: row.label || row.key,
    sessions: 0,
    unmapped: false,
    color: null,
  }))
}

export const buildLeaderboard = (
  data: Insights,
  selection: { dimension: LeaderDimension; metric: LeaderMetric },
  resolveLabel: (source: string) => string
): LeaderRow[] => {
  const metric = resolveLeaderMetric(selection.dimension, selection.metric, data.sessions_indexed)
  const rows = leaderSources(data, selection.dimension, resolveLabel)
    .filter((row) => row[metric] > 0)
    .sort(
      (a, b) =>
        Number(a.unmapped) - Number(b.unmapped) ||
        b[metric] - a[metric] ||
        a.key.localeCompare(b.key)
    )
    .slice(0, LEADERBOARD_ROWS)
  const max = Math.max(0, ...rows.map((row) => row[metric]))
  return rows.map((row) => ({
    key: row.key,
    label: row.label,
    value: row[metric],
    ratio: max > 0 ? row[metric] / max : 0,
    color: row.color,
    unmapped: row.unmapped,
  }))
}

/** 完整值（千分位），跟随运行时默认 locale，与 compactLabel 一致。 */
const FULL_NUMBER_FORMAT = new Intl.NumberFormat()
export const fullNumber = (value: number) => FULL_NUMBER_FORMAT.format(value)
