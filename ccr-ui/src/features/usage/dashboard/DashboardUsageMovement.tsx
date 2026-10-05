import { memo, useCallback, useEffect, useMemo, useState, type CSSProperties, type KeyboardEvent } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { Link } from 'react-router'
import { createTf } from '@/utils/tf'
import { scheduleWhenIdle } from '@/utils/scheduling'
import { PillToggleGroup, SIcon } from '@/ui'
import type { HomeUsageOverviewResponse } from '@/types/usage'
import type { DashboardSessionIndexState } from '@/views/dashboard/dashboardPresentation'
import { homeUsageKeys } from '../queries'
import { useUsageT } from '../translate'
import {
  compactLabel,
  deriveStackedUsageBars,
  emptyDetailOf,
  emptyTitleOf,
  movementStateOf,
  platformLabelKey,
  type StackedUsageBar,
  type StackedUsageChart,
  type StackedUsageSegment,
  type UsageStackPlatform,
} from './DashboardCostMetric'
import { UsageMetricsRow } from './DashboardUsageMetricsRow'
import '../styles/dashboard-usage-movement.css'

interface DashboardUsageMovementProps {
  overview: HomeUsageOverviewResponse | null
  loading: boolean
  error: string | null
  activeDays: number
  onChangeDays: (days: number) => void
  sessionIndexState?: DashboardSessionIndexState
  className?: string
}

const DAY_OPTIONS = [7, 30, 90] as const
const SKELETON_BAR_KEYS = ['sk1', 'sk2', 'sk3', 'sk4', 'sk5', 'sk6', 'sk7'] as const
const formatAxisDate = (value: string) => {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value)
  if (!match) return value
  return `${Number(match[2])}/${Number(match[3])}`
}
const formatDateTime = (value: string | undefined, neverLabel: string) => {
  if (!value) return neverLabel
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return neverLabel
  return new Intl.DateTimeFormat(undefined, { month: 'short', day: '2-digit', hour: '2-digit', minute: '2-digit' }).format(date)
}
const barTitleOf = (dateLabel: string, bar: StackedUsageBar, labels: Record<UsageStackPlatform, string>) => {
  if (bar.segments.length === 0) return `${dateLabel}: 0`
  return `${dateLabel}: ${bar.segments.map((segment) => `${labels[segment.platform]} ${segment.requests}`).join(', ')}`
}

const handleRangeKeyDown = (
  event: KeyboardEvent<HTMLDivElement>,
  activeDays: number,
  onChangeDays: (days: number) => void,
) => {
  const index = DAY_OPTIONS.indexOf(activeDays as (typeof DAY_OPTIONS)[number])
  if (index < 0) return
  if (event.key === 'ArrowRight' || event.key === 'ArrowDown') {
    const next = DAY_OPTIONS[index + 1]
    if (next == null) return
    event.preventDefault()
    onChangeDays(next)
    return
  }
  if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') {
    const next = DAY_OPTIONS[index - 1]
    if (next == null) return
    event.preventDefault()
    onChangeDays(next)
  }
}

const UsageStackBar = memo(function UsageStackBar(props: {
  date: string
  title: string
  heightPercent: number
  segments: StackedUsageSegment[]
}) {
  return (
    <div
      className="dashboard-usage-stack"
      data-date={props.date}
      title={props.title}
      style={{ '--stack-height': `${props.heightPercent}%` } as CSSProperties}
    >
      {props.segments.map((segment) => (
        <span
          key={segment.platform}
          className="dashboard-usage-segment"
          data-platform={segment.platform}
          style={{ '--segment-height': `${segment.heightPercent}%` } as CSSProperties}
        />
      ))}
    </div>
  )
})

function UsageStatusPanel(props: {
  kind: 'error' | 'empty'
  title: string
  detail: string
  retryLabel?: string
  onRetry?: () => void
}) {
  const icon = props.kind === 'error' ? 'AlertTriangle' : 'BarChart3'
  return (
    <div className={props.kind === 'error' ? 'dashboard-usage__error' : 'dashboard-usage__empty'}>
      <span className={props.kind === 'error' ? 'dashboard-usage__error-icon' : 'dashboard-usage__empty-icon'}>
        <SIcon name={icon} size="w-5 h-5" />
      </span>
      <div>
        <h3>{props.title}</h3>
        <p>{props.detail}</p>
        {props.onRetry ? (
          <button type="button" className="dashboard-usage__retry" onClick={props.onRetry}>{props.retryLabel}</button>
        ) : null}
      </div>
    </div>
  )
}

function UsageChartPanel(props: { ariaLabel: string; bars: StackedUsageBar[]; labels: Record<UsageStackPlatform, string> }) {
  return (
    <div className="dashboard-usage__chart" role="img" aria-label={props.ariaLabel} data-dashboard-usage-bars>
      <div className="dashboard-usage__grid" aria-hidden="true">
        <span className="dashboard-usage__grid-line dashboard-usage__grid-line--high" />
        <span className="dashboard-usage__grid-line dashboard-usage__grid-line--mid" />
        <span className="dashboard-usage__grid-line dashboard-usage__grid-line--low" />
      </div>
      {props.bars.map((bar) => (
        <UsageStackBar
          key={bar.date}
          date={bar.date}
          title={barTitleOf(formatAxisDate(bar.date), bar, props.labels)}
          heightPercent={bar.heightPercent}
          segments={bar.segments}
        />
      ))}
    </div>
  )
}

function UsageSkeleton() {
  return (
    <div className="dashboard-usage__chart" data-dashboard-usage-skeleton>
      <div className="dashboard-usage__grid" aria-hidden="true">
        <span className="dashboard-usage__grid-line dashboard-usage__grid-line--high" />
        <span className="dashboard-usage__grid-line dashboard-usage__grid-line--mid" />
        <span className="dashboard-usage__grid-line dashboard-usage__grid-line--low" />
      </div>
      {SKELETON_BAR_KEYS.map((key) => (
        <div key={key} className="dashboard-usage-stack dashboard-usage-stack--skeleton" />
      ))}
    </div>
  )
}

function UsageMainPanel(props: {
  loading: boolean
  error: string | null
  chart: StackedUsageChart
  title: string
  detail: string
  retryLabel: string
  onRetry: () => void
  ariaLabel: string
  labels: Record<UsageStackPlatform, string>
}) {
  if (props.loading) return <UsageSkeleton />
  if (props.error) {
    return (
      <UsageStatusPanel
        kind="error"
        title={props.title}
        detail={props.detail}
        retryLabel={props.retryLabel}
        onRetry={props.onRetry}
      />
    )
  }
  if (props.chart.empty) return <UsageStatusPanel kind="empty" title={props.title} detail={props.detail} />
  return <UsageChartPanel ariaLabel={props.ariaLabel} bars={props.chart.bars} labels={props.labels} />
}

function UsageFooter(props: { showChart: boolean; bars: StackedUsageBar[]; reportLabel: string }) {
  return (
    <footer className="dashboard-usage__footer">
      {props.showChart ? (
        <div className="dashboard-usage__axis" aria-hidden="true">
          {props.bars.map((bar) => (
            <span key={bar.date} className="dashboard-usage__tick">{formatAxisDate(bar.date)}</span>
          ))}
        </div>
      ) : <span />}
      <Link to="/usage" className="dashboard-usage__report-link">
        {props.reportLabel}
        <SIcon name="ArrowRight" size="w-4 h-4" />
      </Link>
    </footer>
  )
}

type MovementViewProps = DashboardUsageMovementProps & {
  t: (key: string) => string
  tf: (key: string, fallback: string, values?: Record<string, string | number>) => string
  costReady: boolean
  chart: StackedUsageChart
  dayOptions: Array<{ value: number; label: string }>
  platformLabels: Record<UsageStackPlatform, string>
  onRangeKeyDown: (event: KeyboardEvent<HTMLDivElement>) => void
  onRetry: () => void
}

function UsageMovementView(props: MovementViewProps) {
  const loading = Boolean(props.loading && !props.overview)
  const showChart = Boolean(!loading && !props.error && !props.chart.empty)
  const requests = compactLabel(props.error, props.overview?.summary.total_requests)
  const title = emptyTitleOf(props.error, loading, props.t)
  const detail = emptyDetailOf({
    error: props.error,
    loading,
    reason: props.overview?.empty_reason ?? undefined,
    t: props.t,
  })
  return (
    <section
      className={['dashboard-usage', props.className].filter(Boolean).join(' ')}
      data-dashboard-usage-movement
      data-count={showChart ? props.chart.bars.length : 0}
      data-state={movementStateOf(props.error, loading, showChart)}
    >
      <header className="dashboard-usage__header">
        <div className="dashboard-usage__lede">
          <h2 className="dashboard-usage__title">{props.t('dashboard.usage.title')}</h2>
          <p className="dashboard-usage__description">
            {props.t('dashboard.usage.description')}
            {' · '}
            {props.tf('dashboard.usage.lastUpdated', 'Updated {time}', {
              time: formatDateTime(props.overview?.last_updated, props.t('dashboard.usage.lastUpdatedNever')),
            })}
          </p>
        </div>
        <div className="dashboard-usage__range" onKeyDown={props.onRangeKeyDown}>
          <PillToggleGroup
            options={props.dayOptions}
            value={props.activeDays}
            onValueChange={props.onChangeDays}
            ariaLabel={props.t('dashboard.usage.rangeLabel')}
          />
        </div>
      </header>
      <UsageMetricsRow
        loading={loading}
        showLegend={showChart}
        requests={requests}
        tokens={compactLabel(props.error, props.overview?.summary.total_tokens)}
        sessions={compactLabel(props.error, props.overview?.summary.total_sessions)}
        costLabel={props.t('dashboard.usage.metricCost')}
        requestLabel={props.t('dashboard.usage.metricRequests')}
        tokenLabel={props.t('dashboard.usage.metricTokens')}
        sessionLabel={props.t('dashboard.usage.metricSessions')}
        sessionStateLabel={props.t(props.sessionIndexState === 'indexing'
          ? 'dashboard.usage.sessionsIndexing'
          : 'dashboard.usage.sessionsUnindexed')}
        sessionStateHint={props.t('dashboard.usage.sessionsUnindexedHint')}
        sessionIndexState={props.sessionIndexState ?? null}
        days={props.activeDays}
        costReady={props.costReady}
        legend={props.chart.legend}
        labels={props.platformLabels}
      />
      <UsageMainPanel
        loading={loading}
        error={props.error}
        chart={props.chart}
        title={title}
        detail={detail}
        retryLabel={props.t('common.retry')}
        onRetry={props.onRetry}
        ariaLabel={props.tf('dashboard.usage.chartAria', '{days}-day window, {requests} requests, platforms: {platforms}', {
          days: props.activeDays,
          requests,
          platforms: props.chart.legend.map((platform) => props.platformLabels[platform]).join(', '),
        })}
        labels={props.platformLabels}
      />
      <UsageFooter showChart={showChart} bars={props.chart.bars} reportLabel={props.t('dashboard.usage.fullReport')} />
    </section>
  )
}

export function DashboardUsageMovement({
  overview,
  loading,
  error,
  activeDays,
  onChangeDays,
  sessionIndexState,
  className,
}: DashboardUsageMovementProps) {
  const t = useUsageT()
  const tf = useMemo(() => createTf(t), [t])
  const queryClient = useQueryClient()
  const [costReady, setCostReady] = useState(false)
  const chart = useMemo(() => deriveStackedUsageBars(overview?.series ?? []), [overview])
  useEffect(() => scheduleWhenIdle(() => setCostReady(true)), [])
  const dayOptions = useMemo(
    () => DAY_OPTIONS.map((days) => ({ value: days, label: t(`dashboard.usage.range${days}`) })),
    [t],
  )
  const platformLabels = useMemo(() => ({
    claude: t(platformLabelKey('claude')),
    codex: t(platformLabelKey('codex')),
    antigravity: t(platformLabelKey('antigravity')),
    opencode: t(platformLabelKey('opencode')),
  }), [t])
  const onRangeKeyDown = useCallback(
    (event: KeyboardEvent<HTMLDivElement>) => handleRangeKeyDown(event, activeDays, onChangeDays),
    [activeDays, onChangeDays],
  )
  const onRetry = useCallback(() => {
    onChangeDays(activeDays)
    void queryClient.invalidateQueries({ queryKey: homeUsageKeys.overview(activeDays) })
  }, [activeDays, onChangeDays, queryClient])

  return (
    <UsageMovementView
      overview={overview}
      loading={loading}
      error={error}
      activeDays={activeDays}
      onChangeDays={onChangeDays}
      sessionIndexState={sessionIndexState}
      className={className}
      t={t}
      tf={tf}
      costReady={costReady}
      chart={chart}
      dayOptions={dayOptions}
      platformLabels={platformLabels}
      onRangeKeyDown={onRangeKeyDown}
      onRetry={onRetry}
    />
  )
}
