import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { translateWithFallback } from '@/i18n/formatMessage'
import { PageHeader, PageShell, PillToggleGroup, SIcon } from '@/ui'
import { useMonitoringLocale, useMonitoringT } from './locale'
import {
  formatCompactNumber,
  formatCostUsd,
  formatDateTime,
  formatTime,
  formatWholeNumber,
  getLevelClass,
  healthStatusClassOf,
  healthStatusLabelOf,
  healthStatusOf,
} from './monitoring-format'
import { MonitoringLogRow } from './MonitoringLogRow'
import type { UsageMetricCard } from './MonitoringUsageCards'
import { MonitoringUsageCards } from './MonitoringUsageCards'
import type { MonitoringEntry, MonitoringLevel } from './monitoring-types'
import { useMonitoringFeed } from './useMonitoringFeed'
import { useMonitoringUsage } from './useMonitoringUsage'

const LEVELS: MonitoringLevel[] = ['error', 'warn', 'info', 'debug']

export function MonitoringView() {
  const t = useMonitoringT()
  const locale = useMonitoringLocale()
  const { isConnected, logs, clearLogs, refresh } = useMonitoringFeed()
  const usage = useMonitoringUsage(t)
  const [filterLevel, setFilterLevel] = useState<'all' | MonitoringLevel>('all')
  const logContainer = useRef<HTMLElement | null>(null)
  const setLogContainer = useCallback((node: HTMLDivElement | null) => {
    logContainer.current = node
  }, [])

  const filteredLogs = useMemo(() => {
    if (filterLevel === 'all') return logs
    return logs.filter((log) => log.level === filterLevel)
  }, [filterLevel, logs])

  const levelCounts = useMemo(() => {
    const counts: Record<MonitoringLevel, number> = { error: 0, warn: 0, info: 0, debug: 0 }
    for (const log of logs) counts[log.level] += 1
    return counts
  }, [logs])

  const recentIssueEvents = useMemo(
    () => logs.filter((log) => log.level === 'error' || log.level === 'warn').slice(-4).reverse(),
    [logs],
  )
  const latestUsageEvent = useMemo(
    () => [...logs].reverse().find((log) => log.channel === 'usage' || log.eventType.includes('usage')) ?? null,
    [logs],
  )

  const healthStatus = healthStatusOf(levelCounts, logs.length)
  const levelToggleOptions = useMemo(
    () => [
      { value: 'all' as const, label: t('monitoring.allLevels') },
      ...LEVELS.map((level) => ({ value: level, label: `${t(`monitoring.levels.${level}`)} ${levelCounts[level]}` })),
    ],
    [levelCounts, t],
  )

  const usageMetricValue = useCallback(
    (value: number | null | undefined, formatter: (value: number) => string) => {
      if (usage.usageLoading) return '…'
      if (usage.usageStatus !== 'ready' || !usage.usageSummary || value == null) return '—'
      return formatter(value)
    },
    [usage.usageLoading, usage.usageStatus, usage.usageSummary],
  )
  const usageMetricDetail = useCallback(
    (detail: string) => {
      if (usage.usageLoading) return t('monitoring.usageLoadingDetail')
      if (usage.usageStatus !== 'ready' || !usage.usageSummary) return t('monitoring.usageMetricUnavailable')
      return detail
    },
    [t, usage.usageLoading, usage.usageStatus, usage.usageSummary],
  )

  const usageMetricCards = useMemo<UsageMetricCard[]>(() => {
    const summary = usage.usageSummary
    const inputOutputDetail = summary
      ? t('monitoring.inputOutputDetail', {
          input: formatCompactNumber(locale, summary.total_input_tokens),
          output: formatCompactNumber(locale, summary.total_output_tokens),
        })
      : ''
    const compact = (value: number) => formatCompactNumber(locale, value)
    return [
      { id: 'requests', label: t('monitoring.totalRequests'), value: usageMetricValue(summary?.total_requests, (v) => formatWholeNumber(locale, v)), detail: usageMetricDetail(t('monitoring.requestsDetail')), icon: 'Activity' },
      { id: 'tokens', label: t('monitoring.totalTokens'), value: usageMetricValue(summary?.total_tokens, compact), detail: usageMetricDetail(inputOutputDetail), icon: 'Layers' },
      { id: 'input-output', label: t('monitoring.inputOutputTokens'), value: usageMetricValue(summary ? summary.total_input_tokens + summary.total_output_tokens : null, compact), detail: usageMetricDetail(t('monitoring.cacheDetail', { cache: formatCompactNumber(locale, summary?.total_cache_read_tokens ?? 0) })), icon: 'ArrowLeftRight' },
      { id: 'cost', label: t('monitoring.estimatedCost'), value: usageMetricValue(summary?.total_cost_usd, formatCostUsd), detail: usageMetricDetail(usage.usageUpdatedAt ? t('monitoring.lastUpdated', { time: formatDateTime(locale, usage.usageUpdatedAt) }) : t('monitoring.notUpdated')), icon: 'Wallet' },
    ]
  }, [locale, t, usage.usageSummary, usage.usageUpdatedAt, usageMetricDetail, usageMetricValue])

  const usageStatusLabel = usage.usageLoading
    ? t('monitoring.usageLoading')
    : usage.usageStatus === 'ready'
      ? t('monitoring.usageReady')
      : usage.usageStatus === 'unavailable'
        ? t('monitoring.usageUnavailable')
        : t('monitoring.usageIdle')

  const refreshMonitoring = useCallback(async () => {
    await Promise.all([refresh(), usage.loadUsageSummary()])
  }, [refresh, usage])
  const handleRefresh = useCallback(() => {
    void refreshMonitoring()
  }, [refreshMonitoring])

  const scrollToBottom = useCallback(() => {
    const container = logContainer.current
    if (!container) return
    container.scrollTop = container.scrollHeight
  }, [])

  useEffect(() => {
    const container = logContainer.current
    if (!container) return
    const nearBottom = container.scrollHeight - container.scrollTop - container.clientHeight < 24
    if (nearBottom) scrollToBottom()
  }, [filteredLogs.length, scrollToBottom])

  useEffect(() => {
    scrollToBottom()
  }, [filterLevel, scrollToBottom])

  const connectionClass = isConnected
    ? 'border-accent-success/30 bg-accent-success/8 text-accent-success'
    : 'border-accent-danger/30 bg-accent-danger/8 text-accent-danger'

  return (
    <PageShell
      className="min-w-0"
      header={
        <PageHeader
          title={t('monitoring.title')}
          description={t('monitoring.subtitle')}
          status={
            <div data-testid="monitoring-connection-status" className={`inline-flex items-center gap-2 rounded-full border px-3 py-1.5 text-xs font-medium ${connectionClass}`}>
              <span className={`h-2 w-2 rounded-full ${isConnected ? 'bg-accent-success' : 'bg-accent-danger'}`} />
              {isConnected ? t('monitoring.connected') : t('monitoring.disconnected')}
            </div>
          }
          actions={
            <>
              <button type="button" className="inline-flex items-center gap-2 rounded-lg border border-border-default/55 bg-bg-surface px-3 py-2 text-xs font-medium text-text-secondary hover:border-accent-secondary/30 hover:text-text-primary disabled:cursor-not-allowed disabled:opacity-60" disabled={usage.usageLoading} onClick={handleRefresh}>
                <SIcon name="RefreshCw" size="w-3.5 h-3.5" className={usage.usageLoading ? 'animate-spin' : ''} />
                {t('monitoring.refresh')}
              </button>
              <button type="button" className="inline-flex items-center gap-2 rounded-lg border border-border-default/55 bg-bg-surface px-3 py-2 text-xs font-medium text-text-secondary hover:border-accent-danger/25 hover:text-accent-danger" onClick={clearLogs}>
                <SIcon name="Trash2" size="w-3.5 h-3.5" />
                {t('monitoring.clearView')}
              </button>
            </>
          }
        />
      }
    >
      <div className="grid min-w-0 gap-4 xl:grid-cols-[minmax(20rem,22.5rem)_minmax(0,1fr)]">
        <aside className="min-w-0 space-y-4">
          <section className="rounded-xl border border-border-default/55 bg-bg-elevated p-4">
            <div className="flex items-start justify-between gap-4">
              <div>
                <p className="text-xs font-medium text-text-muted">{t('monitoring.usageEyebrow')}</p>
                <h2 className="mt-1 text-lg font-semibold text-text-primary">{t('monitoring.usageTitle')}</h2>
              </div>
              <span className={`rounded-full border px-2.5 py-1 text-[0.6875rem] font-medium ${usage.usageStatus === 'ready' ? 'border-accent-success/30 bg-accent-success/10 text-accent-success' : usage.usageStatus === 'unavailable' ? 'border-accent-warning/30 bg-accent-warning/10 text-accent-warning' : 'border-border-default/45 bg-bg-elevated text-text-secondary'}`}>
                {usageStatusLabel}
              </span>
            </div>
            {usage.usageStatus === 'unavailable' ? (
              <div data-testid="monitoring-usage-unavailable" className="mt-4 rounded-2xl border border-accent-warning/25 bg-accent-warning/8 px-4 py-3 text-sm text-text-secondary">
                <p className="font-medium text-text-primary">{t('monitoring.usageUnavailable')}</p>
                <p className="mt-1 text-xs leading-5 text-text-muted">{usage.usageUnavailableDetail}</p>
              </div>
            ) : null}
            <MonitoringUsageCards cards={usageMetricCards} />
          </section>

          <section className="rounded-xl border border-border-default/55 bg-bg-elevated p-4">
            <div className="flex items-center justify-between gap-3">
              <div>
                <p className="text-xs font-medium text-text-muted">{t('monitoring.healthEyebrow')}</p>
                <h2 className="mt-1 text-lg font-semibold text-text-primary">{t('monitoring.healthTitle')}</h2>
              </div>
              <span className={`rounded-full border px-2.5 py-1 text-[0.6875rem] font-medium ${healthStatusClassOf(healthStatus)}`}>
                {healthStatusLabelOf(healthStatus, t)}
              </span>
            </div>
            <div className="mt-3">
              <PillToggleGroup options={levelToggleOptions} value={filterLevel} onValueChange={setFilterLevel} />
            </div>
            <IssuePreview latest={latestUsageEvent} issues={recentIssueEvents} locale={locale} t={t} />
          </section>
        </aside>

        <section className="min-w-0 overflow-hidden rounded-xl border border-border-default/55 bg-bg-elevated">
          <div className="flex flex-col gap-4 border-b border-border-default/45 p-4 lg:flex-row lg:items-center lg:justify-between">
            <div>
              <p className="text-xs font-medium text-text-muted">{t('monitoring.logsEyebrow')}</p>
              <h2 className="mt-1 flex items-center gap-2 text-lg font-semibold text-text-primary">
                <SIcon name="Terminal" size="w-4 h-4" className="text-text-muted" />
                {t('monitoring.realTimeLogs')}
              </h2>
            </div>
            <span data-testid="monitoring-filtered-count" className="rounded-full border border-border-default/45 bg-bg-elevated px-3 py-1.5 text-xs font-medium text-text-secondary">
              {translateWithFallback(t, 'monitoring.filteredCount', '{filtered} / {count} events', { filtered: filteredLogs.length, count: logs.length })}
            </span>
          </div>
          <div className="overflow-hidden p-3">
            <div className="w-full min-w-[40rem] rounded-2xl border border-border-default/45 bg-bg-elevated font-mono text-xs">
              <div className="grid grid-cols-[4.5rem_3.875rem_5.875rem_5.875rem_minmax(0,1fr)] gap-2 border-b border-border-default/45 px-3 py-2 text-[0.6875rem] font-medium text-text-muted">
                <span>{t('monitoring.columnTime')}</span>
                <span>{t('monitoring.columnLevel')}</span>
                <span>{t('monitoring.columnChannel')}</span>
                <span>{t('monitoring.columnSource')}</span>
                <span>{t('monitoring.columnMessage')}</span>
              </div>
              <div ref={setLogContainer} className="max-h-[38.75rem] min-h-[27.5rem] overflow-y-auto">
                {logs.length === 0 ? <EmptyLogs icon="Monitor" title={t('monitoring.noLogs')} detail={t('monitoring.waitingForLogs')} /> : null}
                {logs.length > 0 && filteredLogs.length === 0 ? <EmptyLogs icon="Filter" title={t('monitoring.noFilteredLogs')} detail={t('monitoring.adjustFilter')} /> : null}
                {filteredLogs.map((log) => (
                  <MonitoringLogRow key={log.id} log={log} locale={locale} />
                ))}
              </div>
            </div>
          </div>
        </section>
      </div>
    </PageShell>
  )
}

function EmptyLogs({ icon, title, detail }: { icon: string; title: string; detail: string }) {
  return (
    <div className="flex min-h-[27.5rem] flex-col items-center justify-center px-6 text-center text-text-muted">
      <SIcon name={icon} size="w-10 h-10" className="mb-3 opacity-35" />
      <p className="text-sm font-medium text-text-secondary">{title}</p>
      <p className="mt-1 max-w-sm text-xs leading-5">{detail}</p>
    </div>
  )
}

function IssuePreview({
  latest,
  issues,
  locale,
  t,
}: {
  latest: MonitoringEntry | null
  issues: MonitoringEntry[]
  locale: string
  t: (key: string) => string
}) {
  return (
    <>
      <div className="mt-3 rounded-2xl border border-border-default/45 bg-bg-elevated p-3">
        <p className="text-xs font-medium text-text-muted">{t('monitoring.recentUsageImport')}</p>
        {latest ? (
          <div className="mt-3 space-y-1">
            <div className="flex items-center gap-2 text-xs text-text-muted">
              <span className={`rounded-full px-2 py-0.5 font-semibold uppercase ${getLevelClass(latest.level)}`}>{latest.level}</span>
              <span>{formatTime(locale, latest.timestamp)}</span>
              <span>{latest.source}</span>
            </div>
            <p className="line-clamp-2 text-sm leading-5 text-text-secondary">{latest.message}</p>
          </div>
        ) : (
          <p className="mt-3 text-sm text-text-muted">{t('monitoring.noUsageImportEvent')}</p>
        )}
      </div>
      <div className="mt-3 rounded-2xl border border-border-default/45 bg-bg-elevated p-3">
        <p className="text-xs font-medium text-text-muted">{t('monitoring.recentIssues')}</p>
        {issues.length > 0 ? (
          <div className="mt-3 space-y-2">
            {issues.map((event) => (
              <div key={event.id} className="rounded-xl border border-border-default/35 bg-bg-base px-3 py-2">
                <div className="flex items-center gap-2 text-[0.6875rem] text-text-muted">
                  <span className={`rounded-full px-2 py-0.5 font-semibold uppercase ${getLevelClass(event.level)}`}>{event.level}</span>
                  <span>{formatTime(locale, event.timestamp)}</span>
                  <span className="truncate">{event.channel}</span>
                </div>
                <p className="mt-1 line-clamp-2 text-xs leading-5 text-text-secondary">{event.message}</p>
              </div>
            ))}
          </div>
        ) : (
          <p className="mt-3 text-sm text-text-muted">{t('monitoring.noRecentIssues')}</p>
        )}
      </div>
    </>
  )
}
