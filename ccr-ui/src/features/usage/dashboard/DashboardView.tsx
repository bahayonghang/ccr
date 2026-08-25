import { useCallback, useEffect, useMemo, useState } from 'react'
import { Link } from 'react-router'
import { getCliVersions, getSystemInfo } from '@/api'
import { getErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'
import { perfMark, shouldLogPerfTelemetry } from '@/utils/perfTelemetry'
import { readPrefersReducedMotion } from '@/utils/reducedMotion'
import { scheduleWhenIdle } from '@/utils/scheduling'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import { PageHeader, SIcon } from '@/ui'
import type { CliVersionEntry, CliVersionsResponse, SystemInfo } from '@/types'
import {
  buildDashboardPresentation,
  type DashboardBackendStatus,
  type DashboardPlatformSource,
} from '@/views/dashboard/dashboardPresentation'
import { useHomeUsageOverview } from '../queries'
import { hydrateUsageLocale, useUsageT } from '../translate'
import { DashboardNextActions } from './DashboardNextActions'
import { DashboardPlatformMatrix } from './DashboardPlatformMatrix'
import { DashboardSignalStream } from './DashboardSignalStream'
import { DashboardUsageMovement } from './DashboardUsageMovement'
import { useDashboardSignals } from './useDashboardSignals'
import '../styles/dashboard-view.css'

const CLI_PLATFORM_ALIASES: Record<string, string> = {
  claude: 'claude-code',
  'claude-code': 'claude-code',
  codex: 'codex',
  gemini: 'antigravity',
  agy: 'antigravity',
  antigravity: 'antigravity',
  'antigravity-cli': 'antigravity',
  'gemini-cli': 'antigravity',
  opencode: 'opencode',
  'open-code': 'opencode',
}

const normalizeDashboardCliPlatform = (platform: string) =>
  CLI_PLATFORM_ALIASES[platform.trim().toLowerCase()] ?? null

const stripTrailingPeriod = (text: string) => text.replace(/[。.]$/, '')

export function DashboardView() {
  const t = useUsageT()
  const [activeDays, setActiveDays] = useState(7)
  const overviewQuery = useHomeUsageOverview(activeDays)
  const { logs, pause, resume } = useDashboardSignals(24)
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null)
  const [systemInfoError, setSystemInfoError] = useState<string | null>(null)
  const [cliVersions, setCliVersions] = useState<Map<string, CliVersionEntry>>(new Map())
  const [cliVersionsLoaded, setCliVersionsLoaded] = useState(false)
  const isNativeRuntime = isTauriRuntime()

  const applyCliVersions = useCallback((entries: CliVersionEntry[]) => {
    const normalized = new Map<string, CliVersionEntry>()
    entries.forEach((entry) => {
      const platformKey = normalizeDashboardCliPlatform(entry.platform)
      if (!platformKey) return
      normalized.set(platformKey, { ...entry, platform: platformKey })
    })
    setCliVersions(normalized)
    setCliVersionsLoaded(true)
    perfMark('dashboard:cli-badges-updated')
  }, [])

  useEffect(() => {
    void hydrateUsageLocale()
    const cancel = scheduleWhenIdle(() => {
      if (!isNativeRuntime) {
        perfMark('dashboard:web-preview-ready')
        setCliVersionsLoaded(true)
        return
      }
      void getSystemInfo()
        .then((info) => {
          setSystemInfo(info)
          setSystemInfoError(null)
          perfMark('dashboard:system-ready')
        })
        .catch((caught: unknown) => {
          setSystemInfoError(getErrorMessage(caught))
          logger.error('[DashboardView] failed to load system info', caught)
        })
      void getCliVersions({ mode: 'fast', timeoutMs: 3500, parallelism: 4 })
        .then((versions: CliVersionsResponse) => applyCliVersions(versions.entries))
        .catch((caught: unknown) => {
          setCliVersionsLoaded(true)
          logger.error('[DashboardView] failed to load CLI versions', caught)
        })
    }, { timeout: 1400, fallbackDelay: 280 })

    return () => {
      cancel()
      pause()
    }
  }, [applyCliVersions, isNativeRuntime, pause])

  useEffect(() => {
    resume()
  }, [resume])

  const loadUsageOverview = useCallback((days: number) => {
    setActiveDays(days)
  }, [])

  const backendStatus: DashboardBackendStatus = !isNativeRuntime
    ? 'unsupported'
    : systemInfoError
      ? 'error'
      : systemInfo
        ? 'ok'
        : 'checking'

  const platforms = useMemo<DashboardPlatformSource[]>(() => [
    {
      title: t('dashboard.platforms.claudeTitle'),
      desc: t('dashboard.platforms.claudeDesc'),
      path: '/claude-code',
      icon: 'Code2',
      iconClass: 'text-platform-claude',
      platformKey: 'claude-code',
      usageKey: 'claude',
      role: t('dashboard.platforms.roleCoreCli'),
      mode: 'cli',
      isRuntimeCli: true,
    },
    {
      title: t('dashboard.platforms.codexTitle'),
      desc: t('dashboard.platforms.codexDesc'),
      path: '/codex',
      icon: 'Settings',
      iconClass: 'text-platform-codex',
      platformKey: 'codex',
      usageKey: 'codex',
      role: t('dashboard.platforms.roleCoreCli'),
      mode: 'cli',
      isRuntimeCli: true,
    },
    {
      title: t('dashboard.platforms.antigravityTitle'),
      desc: t('dashboard.platforms.antigravityDesc'),
      path: '/antigravity',
      icon: 'Sparkles',
      iconClass: 'text-platform-gemini',
      platformKey: 'antigravity',
      usageKey: 'gemini',
      role: t('dashboard.platforms.roleCoreCli'),
      mode: 'cli',
      isRuntimeCli: true,
    },
    {
      title: t('dashboard.platforms.opencodeTitle'),
      desc: t('dashboard.platforms.opencodeDesc'),
      path: '/opencode',
      icon: 'TerminalSquare',
      iconClass: 'text-[color:var(--color-platform-opencode)]',
      platformKey: 'opencode',
      usageKey: 'opencode',
      role: t('dashboard.platforms.roleManaged'),
      mode: 'managed',
      isRuntimeCli: false,
    },
  ], [t])

  const overview = isNativeRuntime ? (overviewQuery.data ?? null) : null
  const usageLoading = Boolean(isNativeRuntime && overviewQuery.isLoading)
  const usageError = isNativeRuntime && overviewQuery.error
    ? getErrorMessage(overviewQuery.error)
    : null

  const dashboardPresentation = useMemo(() => buildDashboardPresentation({
    backendStatus,
    isNativeRuntime,
    systemInfo,
    cliVersions,
    cliVersionsLoaded,
    platforms,
    overview,
    usageLoading,
    usageError,
    logs,
  }), [
    backendStatus,
    cliVersions,
    cliVersionsLoaded,
    isNativeRuntime,
    logs,
    overview,
    platforms,
    systemInfo,
    usageError,
    usageLoading,
  ])

  const issueCount = dashboardPresentation.readiness.reasons.filter((reason) => !reason.ok).length
  const primaryAction = dashboardPresentation.actions[0]

  const scrollToReadiness = useCallback(() => {
    const target = document.querySelector('[data-dashboard-readiness]')
    const reduceMotion = readPrefersReducedMotion()
    target?.scrollIntoView({ behavior: reduceMotion ? 'auto' : 'smooth', block: 'center' })
  }, [])

  useEffect(() => {
    if (!shouldLogPerfTelemetry() || typeof performance === 'undefined') return undefined
    const timer = window.setTimeout(() => {
      logger.info('[Perf]', { scope: 'dashboard' })
    }, 4500)
    return () => window.clearTimeout(timer)
  }, [])

  return (
    <main className="dashboard-view">
      <div className="dashboard-workbench">
        <section className="dashboard-header" data-dashboard-hero>
          <PageHeader
            title={t('dashboard.title')}
            eyebrow={t('dashboard.eyebrow')}
            description={t(dashboardPresentation.readiness.descriptionKey)}
            status={(
              <div className="dashboard-header__readiness">
                <h2 className="dashboard-header__readiness-title">
                  {t(dashboardPresentation.readiness.titleKey)}
                </h2>
                <button
                  type="button"
                  className="dashboard-header__badge"
                  data-status={dashboardPresentation.readiness.status}
                  aria-label={t('dashboard.readiness.label')}
                  onClick={scrollToReadiness}
                >
                  <span className="dashboard-header__badge-dot" aria-hidden="true" />
                  {t(dashboardPresentation.readiness.labelKey)}
                  {issueCount > 0 ? (
                    <span className="dashboard-header__badge-count">{issueCount}</span>
                  ) : null}
                </button>
              </div>
            )}
            actions={(
              <>
                {primaryAction ? (
                  <Link to={primaryAction.path} className="dashboard-header__action">
                    {t(primaryAction.titleKey)}
                  </Link>
                ) : null}
                <Link to="/usage" className="dashboard-header__all">
                  {t('usage.dashboard.allPlatforms')}
                  <SIcon name="ArrowRight" size="w-4 h-4" />
                </Link>
              </>
            )}
          />
          <ul className="dashboard-header__reasons" data-dashboard-readiness>
            {dashboardPresentation.readiness.reasons.map((reason) => (
              <li
                key={reason.key}
                className="dashboard-header__reason"
                data-ok={reason.ok}
              >
                <SIcon
                  name={reason.ok ? 'Check' : 'AlertTriangle'}
                  size="w-3.5 h-3.5"
                  className="dashboard-header__reason-icon"
                />
                {stripTrailingPeriod(t(reason.key))}
              </li>
            ))}
          </ul>
        </section>
        <DashboardPlatformMatrix
          rows={dashboardPresentation.platformRows}
          installedCliCount={dashboardPresentation.installedCliCount}
          runtimeCliCount={dashboardPresentation.runtimeCliCount}
        />
        <section className="dashboard-lower">
          <DashboardUsageMovement
            overview={overview}
            loading={usageLoading}
            error={usageError}
            activeDays={activeDays}
            onChangeDays={loadUsageOverview}
          />
          <div className="dashboard-rail">
            <DashboardNextActions
              actions={dashboardPresentation.actions}
              showOnboarding={dashboardPresentation.isFirstRun}
            />
            <DashboardSignalStream
              entries={logs}
              limit={6}
            />
          </div>
        </section>
      </div>
    </main>
  )
}
