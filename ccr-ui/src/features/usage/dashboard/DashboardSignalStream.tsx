import { useMemo, useState } from 'react'
import { Link } from 'react-router'
import { PillToggleGroup, SIcon } from '@/ui'
import { useUsageT } from '../translate'
import type { DashboardSignalEntry } from './useDashboardSignals'
import '../styles/dashboard-signal-stream.css'

interface DashboardSignalStreamProps {
  entries: DashboardSignalEntry[]
  limit?: number
  className?: string
}

type FilterId = 'all' | 'warn' | 'error'
type AggregatedEntry = DashboardSignalEntry & { count: number }

const matchesFilter = (entry: DashboardSignalEntry, id: FilterId) => {
  if (id === 'all') return true
  if (id === 'warn') return entry.level === 'warn' || entry.level === 'error'
  return entry.level === 'error'
}

export function DashboardSignalStream({
  entries,
  limit = 6,
  className,
}: DashboardSignalStreamProps) {
  const t = useUsageT()
  const [filter, setFilter] = useState<FilterId>('all')

  const aggregatedEntries = useMemo<AggregatedEntry[]>(() => {
    const sorted = [...entries].sort(
      (left, right) => new Date(right.timestamp).getTime() - new Date(left.timestamp).getTime(),
    )
    const result: AggregatedEntry[] = []
    sorted.forEach((entry) => {
      const last = result[result.length - 1]
      if (last && last.message === entry.message && last.channel === entry.channel && last.level === entry.level) {
        last.count += 1
        return
      }
      result.push({ ...entry, count: 1 })
    })
    return result
  }, [entries])

  const visibleEntries = aggregatedEntries.filter((entry) => matchesFilter(entry, filter)).slice(0, limit)
  const filterOptions = [
    { value: 'all' as const, label: `${t('dashboard.signals.filterAll')} ${aggregatedEntries.length}` },
    { value: 'warn' as const, label: `${t('dashboard.signals.filterWarn')} ${aggregatedEntries.filter((entry) => matchesFilter(entry, 'warn')).length}` },
    { value: 'error' as const, label: `${t('dashboard.signals.filterError')} ${aggregatedEntries.filter((entry) => matchesFilter(entry, 'error')).length}` },
  ]

  const formatTime = (timestamp: string) => {
    const date = new Date(timestamp)
    if (Number.isNaN(date.getTime())) return t('dashboard.signals.unknownTime')
    return new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit' }).format(date)
  }

  return (
    <section
      className={['dashboard-signals', className].filter(Boolean).join(' ')}
      data-dashboard-signals
    >
      <header className="dashboard-signals__header">
        <div className="dashboard-signals__lede">
          <p className="dashboard-signals__eyebrow">{t('dashboard.signals.eyebrow')}</p>
          <h2 className="dashboard-signals__title">{t('dashboard.signals.title')}</h2>
        </div>
        <PillToggleGroup
          options={filterOptions}
          value={filter}
          onValueChange={setFilter}
          ariaLabel={t('dashboard.signals.title')}
        />
      </header>
      {visibleEntries.length > 0 ? (
        <ol className="dashboard-signals__list">
          {visibleEntries.map((entry) => (
            <li key={entry.id} className="dashboard-signal" data-level={entry.level}>
              <span className="dashboard-signal__time">{formatTime(entry.timestamp)}</span>
              <span className="dashboard-signal__dot" aria-label={entry.level} />
              <span className="dashboard-signal__channel">{entry.channel}</span>
              <span className="dashboard-signal__message-group">
                <p className="dashboard-signal__message" title={entry.message}>
                  {entry.message}
                </p>
                {entry.count > 1 ? (
                  <span className="dashboard-signal__count">×{entry.count}</span>
                ) : null}
              </span>
            </li>
          ))}
        </ol>
      ) : (
        <div className="dashboard-signals__empty">
          <span className="dashboard-signals__empty-icon">
            <SIcon name="History" size="w-5 h-5" />
          </span>
          <div>
            <h3>{t('dashboard.signals.emptyTitle')}</h3>
            <p>{t('dashboard.signals.emptyDescription')}</p>
          </div>
          <Link to="/monitoring" className="dashboard-signals__empty-cta">
            {t('dashboard.signals.viewAll')}
            <SIcon name="ArrowRight" size="w-4 h-4" />
          </Link>
        </div>
      )}
      {visibleEntries.length > 0 ? (
        <footer className="dashboard-signals__footer">
          <Link to="/monitoring">
            {t('dashboard.signals.openMonitoring')}
            <SIcon name="ArrowRight" size="w-4 h-4" />
          </Link>
        </footer>
      ) : null}
    </section>
  )
}
