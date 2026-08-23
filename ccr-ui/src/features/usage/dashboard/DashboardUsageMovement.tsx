import { memo, useCallback, useMemo, useState } from 'react'
import { Link } from 'react-router'
import { translateWithFallback } from '@/i18n/formatMessage'
import { PillToggleGroup, SIcon, StatTile } from '@/ui'
import type { HomeUsageOverviewResponse } from '@/types/usage'
import type { DashboardUsageMetric } from '@/views/dashboard/dashboardPresentation'
import { useUsageT } from '../translate'
import '../styles/dashboard-usage-movement.css'

interface DashboardUsageMovementProps {
  overview: HomeUsageOverviewResponse | null
  loading: boolean
  error: string | null
  activeDays: number
  onChangeDays: (days: number) => void
  className?: string
}

const DAY_OPTIONS = [7, 30, 90]
const METRIC_OPTIONS: DashboardUsageMetric[] = ['sessions', 'requests', 'tokens']

const metricLabelOf = (metric: DashboardUsageMetric, t: (key: string) => string) => {
  if (metric === 'sessions') return t('dashboard.usage.metricSessions')
  if (metric === 'tokens') return t('dashboard.usage.metricTokens')
  return t('dashboard.usage.metricRequests')
}

const emptyTitleOf = (error: string | null, loading: boolean, t: (key: string) => string) => {
  if (error) return t('dashboard.usage.unavailableTitle')
  if (loading) return t('dashboard.metrics.usagePreparing')
  return t('dashboard.usage.emptyTitle')
}

const emptyDetailOf = (input: {
  error: string | null
  loading: boolean
  emptyReasonDescription: string
  t: (key: string) => string
}) => {
  if (input.error) return input.error
  if (input.loading) return input.t('dashboard.usage.loadingDescription')
  return input.emptyReasonDescription
}

const snapshotCopy = (input: {
  error: string | null
  emptyReason?: string
  emptyReasonDescription: string
  loading: boolean
  t: (key: string) => string
}) => {
  if (input.error) return input.t('dashboard.usage.error')
  if (input.emptyReason) return input.emptyReasonDescription
  if (input.loading) return input.t('dashboard.metrics.usagePreparing')
  return input.t('dashboard.usage.description')
}

const emptyReasonCopy = (reason: string | undefined, t: (key: string) => string) => {
  if (reason === 'no_usage_logs') return t('usageStats.noUsageLogs')
  if (reason === 'no_session_index') return t('usageStats.noSessionIndex')
  if (reason === 'no_usage_and_sessions') return t('usageStats.noUsageAndSessions')
  return t('dashboard.usage.emptyDescription')
}

function ChartReadout({
  hoveredPoint,
  peak,
  metricLabel,
  peakLabel,
  hoverHint,
}: {
  hoveredPoint: { dateLabel: string; valueLabel: string } | null
  peak: { dateLabel: string; valueLabel: string } | null
  metricLabel: string
  peakLabel: string
  hoverHint: string
}) {
  let body = <span className="dashboard-usage__chart-readout-placeholder">{hoverHint}</span>
  if (hoveredPoint) {
    body = (
      <>
        <span className="dashboard-usage__chart-readout-date">{hoveredPoint.dateLabel}</span>
        <span className="dashboard-usage__chart-readout-value">{hoveredPoint.valueLabel}</span>
        <span className="dashboard-usage__chart-readout-metric">{metricLabel}</span>
      </>
    )
  } else if (peak) {
    body = (
      <>
        <span className="dashboard-usage__chart-readout-metric">{peakLabel}</span>
        <span className="dashboard-usage__chart-readout-value">{peak.valueLabel}</span>
        <span className="dashboard-usage__chart-readout-date">{peak.dateLabel}</span>
      </>
    )
  }
  return (
    <span className="dashboard-usage__chart-readout" data-visible={hoveredPoint ? 'true' : 'false'} aria-live="polite">
      {body}
    </span>
  )
}

const UsageBar = memo(function UsageBar({
  pointKey,
  height,
  title,
  active,
  onHover,
}: {
  pointKey: string
  height: number
  title: string
  active: boolean
  onHover: (key: string | null) => void
}) {
  const handleEnter = useCallback(() => onHover(pointKey), [onHover, pointKey])
  const handleLeave = useCallback(() => onHover(null), [onHover])

  return (
    <button
      type="button"
      className="dashboard-usage-bar"
      data-active={active ? 'true' : 'false'}
      style={{ height: `${height}%` }}
      title={title}
      aria-label={title}
      onMouseEnter={handleEnter}
      onFocus={handleEnter}
      onBlur={handleLeave}
    />
  )
})

export function DashboardUsageMovement({
  overview,
  loading,
  error,
  activeDays,
  onChangeDays,
  className,
}: DashboardUsageMovementProps) {
  const t = useUsageT()
  const [selectedMetric, setSelectedMetric] = useState<DashboardUsageMetric>('requests')
  const [hoveredKey, setHoveredKey] = useState<string | null>(null)

  const formatCompact = (value?: number) => {
    if (typeof value !== 'number') return '…'
    return new Intl.NumberFormat(undefined, { notation: 'compact', maximumFractionDigits: 1 }).format(value)
  }

  const formatDateTime = (value?: string) => {
    if (!value) return t('dashboard.usage.lastUpdatedNever')
    const date = new Date(value)
    if (Number.isNaN(date.getTime())) return t('dashboard.usage.lastUpdatedNever')
    return new Intl.DateTimeFormat(undefined, {
      month: 'short',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    }).format(date)
  }

  const formatDateLabel = (value: string) => {
    const date = new Date(value)
    if (Number.isNaN(date.getTime())) return value
    return new Intl.DateTimeFormat(undefined, { month: 'short', day: '2-digit' }).format(date)
  }

  const isInitialLoading = loading && !overview
  const emptyReason = overview?.empty_reason
  const emptyReasonDescription = emptyReasonCopy(emptyReason ?? undefined, t)

  const snapshotDescription = snapshotCopy({
    error,
    emptyReason: emptyReason ?? undefined,
    emptyReasonDescription,
    loading: isInitialLoading,
    t,
  })

  const summaryItems = [
    { label: t('dashboard.usage.metricSessions'), value: formatCompact(overview?.summary.total_sessions) },
    { label: t('dashboard.usage.metricRequests'), value: formatCompact(overview?.summary.total_requests) },
    { label: t('dashboard.usage.metricTokens'), value: formatCompact(overview?.summary.total_tokens) },
    { label: t('dashboard.usage.metricPlatforms'), value: formatCompact(overview?.summary.platforms) },
  ]

  const chartPoints = useMemo(() => {
    const series = overview?.series ?? []
    const values = series.map((item) =>
      item.claude[selectedMetric]
      + item.codex[selectedMetric]
      + item.antigravity[selectedMetric]
      + (item.opencode?.[selectedMetric] ?? 0),
    )
    const max = Math.max(1, ...values)
    return series.map((item, index) => {
      const value = values[index] ?? 0
      return {
        key: item.date,
        dateLabel: formatDateLabel(item.date),
        value,
        valueLabel: formatCompact(value),
        height: Math.max(6, Math.round((value / max) * 100)),
        title: `${formatDateLabel(item.date)} · ${formatCompact(value)} ${metricLabelOf(selectedMetric, t)}`,
      }
    })
  }, [overview, selectedMetric, t])

  const clearHover = useCallback(() => setHoveredKey(null), [])
  const hoveredPoint = chartPoints.find((point) => point.key === hoveredKey) ?? null
  const peakPoint = chartPoints.reduce<(typeof chartPoints)[number] | null>((best, point) => {
    if (!best || point.value > best.value) return point
    return best
  }, null)
  const peak = peakPoint && peakPoint.value > 0 ? peakPoint : null
  const hasSeries = chartPoints.length > 0
  const hasMeaningfulSeries = chartPoints.some((point) => point.value > 0)

  return (
    <section
      className={['dashboard-usage', className].filter(Boolean).join(' ')}
      data-dashboard-usage-movement
    >
      <header className="dashboard-usage__header">
        <div className="dashboard-usage__lede">
          <h2 className="dashboard-usage__title">{t('dashboard.usage.title')}</h2>
          <p className="dashboard-usage__description">{snapshotDescription}</p>
        </div>
        <PillToggleGroup
          options={DAY_OPTIONS.map((days) => ({ value: days, label: t(`dashboard.usage.range${days}`) }))}
          value={activeDays}
          onValueChange={onChangeDays}
          ariaLabel={t('dashboard.usage.rangeLabel')}
        />
      </header>
      <div className="dashboard-usage__body">
        <div className="dashboard-usage__summary">
          {summaryItems.map((item) => (
            <StatTile key={item.label} label={item.label} value={item.value} tone="neutral" />
          ))}
        </div>
        <div className="dashboard-usage__chartArea">
          <PillToggleGroup
            className="dashboard-usage__metric"
            options={METRIC_OPTIONS.map((metric) => ({ value: metric, label: metricLabelOf(metric, t) }))}
            value={selectedMetric}
            onValueChange={setSelectedMetric}
            ariaLabel={t('dashboard.usage.metricSelectLabel')}
          />
          {hasSeries ? (
            <div
              className={['dashboard-usage__chart', !hasMeaningfulSeries ? 'dashboard-usage__chart--ghost' : '']
                .filter(Boolean)
                .join(' ')}
              data-dashboard-usage-bars
              onMouseLeave={clearHover}
            >
              <ChartReadout
                hoveredPoint={hoveredPoint}
                peak={peak}
                metricLabel={metricLabelOf(selectedMetric, t)}
                peakLabel={t('dashboard.usage.peakLabel')}
                hoverHint={t('dashboard.usage.hoverHint')}
              />
              <div className="dashboard-usage__chart-grid" aria-hidden="true">
                <span className="dashboard-usage__chart-grid-line dashboard-usage__chart-grid-line--top" />
                <span className="dashboard-usage__chart-grid-line dashboard-usage__chart-grid-line--bottom" />
              </div>
              <div className="dashboard-usage__chart-bars">
                {chartPoints.map((point) => (
                  <UsageBar
                    key={point.key}
                    pointKey={point.key}
                    height={point.height}
                    title={point.title}
                    active={hoveredKey === point.key}
                    onHover={setHoveredKey}
                  />
                ))}
              </div>
            </div>
          ) : (
            <div className="dashboard-usage__empty">
              <span className="dashboard-usage__empty-icon">
                <SIcon name="BarChart3" size="w-5 h-5" />
              </span>
              <div>
                <h3>{emptyTitleOf(error, isInitialLoading, t)}</h3>
                <p>{emptyDetailOf({ error, loading: isInitialLoading, emptyReasonDescription, t })}</p>
              </div>
            </div>
          )}
        </div>
      </div>
      <footer className="dashboard-usage__footer">
        <span className="dashboard-usage__last">
          {translateWithFallback(t, 'dashboard.usage.lastUpdated', 'Updated {time}', {
            time: formatDateTime(overview?.last_updated),
          })}
        </span>
        <Link to="/usage" className="dashboard-usage__report-link">
          {t('dashboard.usage.fullReport')}
          <SIcon name="ArrowRight" size="w-4 h-4" />
        </Link>
      </footer>
    </section>
  )
}
