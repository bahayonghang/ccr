import type { HomeOverviewSeriesItem } from '@/types/usage'
import { useUsageSummary } from '../queries'

export type HomeDateWindow = {
  startDate: string
  endDate: string
}

export type CostMetricQueryState = {
  mounted: boolean
  isLoading: boolean
  isError: boolean
  totalCostUsd: number | null | undefined
}

const COST_UNAVAILABLE = '—'
const COST_ZERO = '$0.00'

const pad2 = (value: number) => `${value}`.padStart(2, '0')

/** 按本地日历格式化为 YYYY-MM-DD，避免 toISOString() 的 UTC 偏移。 */
export function formatHomeLocalDate(date: Date): string {
  return `${date.getFullYear()}-${pad2(date.getMonth() + 1)}-${pad2(date.getDate())}`
}

/** 与后端 local_usage_date_window 对齐：end = 本地今天，start = end - (days - 1)。 */
export function homeDateWindow(days: number, now: Date = new Date()): HomeDateWindow {
  const safeDays = Math.max(1, Math.floor(days))
  const end = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const start = new Date(end)
  start.setDate(end.getDate() - (safeDays - 1))
  return {
    startDate: formatHomeLocalDate(start),
    endDate: formatHomeLocalDate(end),
  }
}

export function formatCostUsd(value: number): string {
  return `$${value.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`
}

/** 成本三态：不可用 —；真实 0 为 $0.00；正数为格式化金额。 */
export function formatCostMetric(state: CostMetricQueryState): string {
  if (!state.mounted || state.isLoading || state.isError) return COST_UNAVAILABLE
  if (state.totalCostUsd == null || !Number.isFinite(state.totalCostUsd)) return COST_UNAVAILABLE
  if (state.totalCostUsd === 0) return COST_ZERO
  return formatCostUsd(state.totalCostUsd)
}

interface DashboardCostMetricProps {
  days: number
}

/** 首页唯一调用 useUsageSummary 的成本格；由父组件在空闲后条件挂载。 */
export function DashboardCostMetric({ days }: DashboardCostMetricProps) {
  const { startDate, endDate } = homeDateWindow(days)
  const query = useUsageSummary(undefined, startDate, endDate)
  const text = formatCostMetric({
    mounted: true,
    isLoading: query.isPending,
    isError: query.isError,
    totalCostUsd: query.data?.total_cost_usd,
  })

  return (
    <span
      data-dashboard-cost-metric
      data-cost-state={costStateOf(text)}
      aria-live="polite"
    >
      {text}
    </span>
  )
}

function costStateOf(text: string): 'unavailable' | 'zero' | 'value' {
  if (text === COST_UNAVAILABLE) return 'unavailable'
  if (text === COST_ZERO) return 'zero'
  return 'value'
}

export const USAGE_STACK_PLATFORMS = ['claude', 'codex', 'antigravity', 'opencode'] as const
export type UsageStackPlatform = (typeof USAGE_STACK_PLATFORMS)[number]
export type StackedUsageSegment = { platform: UsageStackPlatform; requests: number; heightPercent: number }
export type StackedUsageBar = { date: string; total: number; heightPercent: number; segments: StackedUsageSegment[] }
export type StackedUsageChart = { maxDailyTotal: number; empty: boolean; legend: UsageStackPlatform[]; bars: StackedUsageBar[] }

const PLATFORM_LABEL_KEY: Record<UsageStackPlatform, string> = {
  claude: 'dashboard.usage.platformClaude',
  codex: 'dashboard.usage.platformCodex',
  antigravity: 'dashboard.usage.platformAntigravity',
  opencode: 'dashboard.usage.platformOpenCode',
}

const platformTotal = (item: HomeOverviewSeriesItem) =>
  USAGE_STACK_PLATFORMS.reduce((sum, platform) => sum + item[platform].requests, 0)

const toStackedBar = (item: HomeOverviewSeriesItem, total: number, maxDailyTotal: number): StackedUsageBar => ({
  date: item.date,
  total,
  heightPercent: (total / maxDailyTotal) * 100,
  segments: USAGE_STACK_PLATFORMS
    .map((platform) => ({
      platform,
      requests: item[platform].requests,
      heightPercent: total > 0 ? (item[platform].requests / total) * 100 : 0,
    }))
    .filter((segment) => segment.requests > 0),
})

/** 按天按平台派生堆叠柱；maxDailyTotal === 0 走空态。 */
export function deriveStackedUsageBars(series: HomeOverviewSeriesItem[]): StackedUsageChart {
  const dayTotals = series.map(platformTotal)
  const maxDailyTotal = dayTotals.reduce((max, value) => Math.max(max, value), 0)
  if (maxDailyTotal === 0) {
    return { maxDailyTotal: 0, empty: true, legend: [], bars: [] }
  }
  return {
    maxDailyTotal,
    empty: false,
    legend: USAGE_STACK_PLATFORMS.filter((platform) => series.some((item) => item[platform].requests > 0)),
    bars: series.map((item, index) => toStackedBar(item, dayTotals[index] ?? 0, maxDailyTotal)),
  }
}

export const platformLabelKey = (platform: UsageStackPlatform) => PLATFORM_LABEL_KEY[platform]

export const emptyTitleOf = (error: string | null, loading: boolean, t: (key: string) => string) => {
  if (error) return t('dashboard.usage.unavailableTitle')
  if (loading) return t('dashboard.metrics.usagePreparing')
  return t('dashboard.usage.emptyTitle')
}

export const emptyDetailOf = (input: {
  error: string | null
  loading: boolean
  reason?: string
  t: (key: string) => string
}) => {
  if (input.error) return input.error
  if (input.loading) return input.t('dashboard.usage.loadingDescription')
  if (input.reason === 'no_usage_logs') return input.t('usageStats.noUsageLogs')
  if (input.reason === 'no_session_index') return input.t('usageStats.noSessionIndex')
  if (input.reason === 'no_usage_and_sessions') return input.t('usageStats.noUsageAndSessions')
  return input.t('dashboard.usage.emptyDescription')
}

export const compactLabel = (error: string | null, value?: number) => {
  if (error) return '—'
  if (typeof value !== 'number') return '—'
  return new Intl.NumberFormat(undefined, { notation: 'compact', maximumFractionDigits: 1 }).format(value)
}

export const movementStateOf = (error: string | null, loading: boolean, showChart: boolean) => {
  if (error) return 'error'
  if (loading) return 'loading'
  if (showChart) return 'ready'
  return 'empty'
}
