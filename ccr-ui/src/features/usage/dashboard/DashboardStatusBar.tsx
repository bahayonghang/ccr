import { useEffect, useMemo, useState } from 'react'
import { Link } from 'react-router'
import type { DashboardBackendStatus } from '@/views/dashboard/dashboardPresentation'
import { useUsageT } from '../translate'
import type { DashboardSignalEntry } from './useDashboardSignals'
import '../styles/dashboard-status-bar.css'

interface DashboardStatusBarProps {
  backendStatus: DashboardBackendStatus
  entries: DashboardSignalEntry[]
}

const CLOCK_TICK_MS = 30_000

const COMMAND_HINTS = [
  { index: '01', path: '/', labelKey: 'dashboard.statusBar.cmdOverview' },
  { index: '02', path: '/commands', labelKey: 'dashboard.statusBar.cmdCommands' },
  { index: '03', path: '/sync', labelKey: 'dashboard.statusBar.cmdSync' },
  { index: '04', path: '/usage', labelKey: 'dashboard.statusBar.cmdUsage' },
] as const

const backendStateOf = (status: DashboardBackendStatus) => {
  switch (status) {
    case 'ok':
      return { labelKey: 'dashboard.metrics.backendReady', tone: 'ok' } as const
    case 'checking':
      return { labelKey: 'dashboard.metrics.backendChecking', tone: 'pending' } as const
    case 'error':
      return { labelKey: 'dashboard.metrics.backendError', tone: 'error' } as const
    case 'unsupported':
      return { labelKey: 'dashboard.metrics.backendUnsupported', tone: 'muted' } as const
    default:
      return { labelKey: 'dashboard.metrics.backendUnknown', tone: 'muted' } as const
  }
}

const signalLevelLabelKey = (level: string) => {
  switch (level) {
    case 'error':
      return 'dashboard.signals.levelError'
    case 'warn':
      return 'dashboard.signals.levelWarn'
    case 'debug':
      return 'dashboard.signals.levelDebug'
    default:
      return 'dashboard.signals.levelInfo'
  }
}

const formatClock = (date: Date) =>
  new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit' }).format(date)

/** 底部命令状态条：后端连接、最近事件严重度、路由命令提示、本地时钟。 */
export function DashboardStatusBar({ backendStatus, entries }: DashboardStatusBarProps) {
  const t = useUsageT()
  const [now, setNow] = useState(() => new Date())

  useEffect(() => {
    const timer = window.setInterval(() => setNow(new Date()), CLOCK_TICK_MS)
    return () => window.clearInterval(timer)
  }, [])

  const lastEntry = useMemo(() => {
    let latest: DashboardSignalEntry | null = null
    for (const entry of entries) {
      if (!latest) {
        latest = entry
        continue
      }
      if (new Date(entry.timestamp).getTime() > new Date(latest.timestamp).getTime()) latest = entry
    }
    return latest
  }, [entries])

  const backend = backendStateOf(backendStatus)

  return (
    <footer
      className="dashboard-statusbar"
      data-dashboard-statusbar
      aria-label={t('dashboard.statusBar.ariaLabel')}
    >
      <span className="dashboard-statusbar__item" data-tone={backend.tone}>
        <span className="dashboard-statusbar__label">{t('dashboard.metrics.backend')}</span>
        <span className="dashboard-statusbar__value">{t(backend.labelKey)}</span>
      </span>
      <span className="dashboard-statusbar__item dashboard-statusbar__event" data-level={lastEntry?.level ?? 'none'}>
        {lastEntry ? (
          <>
            <span className="dashboard-statusbar__event-dot" aria-hidden="true" />
            <span className="dashboard-statusbar__label">{t('dashboard.statusBar.lastEvent')}</span>
            <span className="dashboard-statusbar__value">{t(signalLevelLabelKey(lastEntry.level))}</span>
          </>
        ) : (
          <span>{t('dashboard.statusBar.noEvents')}</span>
        )}
      </span>
      <nav className="dashboard-statusbar__commands" aria-label={t('dashboard.statusBar.commandsLabel')}>
        {COMMAND_HINTS.map((hint) => (
          <Link key={hint.path} to={hint.path} className="dashboard-statusbar__command">
            <span className="dashboard-statusbar__command-index" aria-hidden="true">
              {hint.index}
            </span>
            {t(hint.labelKey)}
          </Link>
        ))}
      </nav>
      <span className="dashboard-statusbar__item dashboard-statusbar__clock">
        <time dateTime={now.toISOString()}>{formatClock(now)}</time>
      </span>
    </footer>
  )
}
