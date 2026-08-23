import { useCallback, useMemo, useRef } from 'react'
import { useQuery } from '@tanstack/react-query'
import { getCodexDashboardOverview, getCodexDashboardUsageSummary } from '@/api'
import { getCliVersion } from '@/api/runtime/system'
import type { CliVersionEntry } from '@/types'
import { getErrorMessage } from '@/utils/errorHandler'
import { perfMark, perfMeasure } from '@/utils/perfTelemetry'
import type { TranslateFunction } from '@/utils/tf'
import {
  buildCompactInventory,
  buildNextActions,
  buildReadinessItems,
  formatDashboardDateTime,
  formatTokens,
  type CodexDashboardActionItem,
} from './dashboard-model'
import { CODEX_DASHBOARD_STALE_TIME, CODEX_VERSION_STALE_TIME, codexKeys } from './queries'

const measureAsync = async <T>(scope: string, action: () => Promise<T>): Promise<T> => {
  const token = `${scope}:${Date.now()}:${Math.random().toString(16).slice(2)}`
  const startMark = `${token}:start`
  const endMark = `${token}:end`
  perfMark(startMark)
  try {
    return await action()
  } finally {
    perfMark(endMark)
    perfMeasure(scope, startMark, endMark)
  }
}

const deriveVersionState = (
  entry: CliVersionEntry | null | undefined,
  fetching: boolean,
  t: TranslateFunction,
) => {
  if (!entry) {
    return fetching
      ? { versionStatus: 'loading' as const, versionLabel: '...' }
      : { versionStatus: 'error' as const, versionLabel: t('codex.status.retryVersionCheck') }
  }
  if (entry.status === 'timeout') {
    return { versionStatus: 'timeout' as const, versionLabel: t('codex.status.checkingVersion') }
  }
  if (entry.status === 'error') {
    return { versionStatus: 'error' as const, versionLabel: t('codex.status.retryVersionCheck') }
  }
  if (entry.status === 'not_installed' || !entry.installed) {
    return { versionStatus: 'not_installed' as const, versionLabel: t('codex.status.notInstalled') }
  }
  return {
    versionStatus: 'ok' as const,
    versionLabel: entry.version ? `v${entry.version}` : t('codex.status.installed'),
  }
}

export function useCodexDashboard(t: TranslateFunction) {
  const overviewForceRef = useRef(false)
  const usageForceRef = useRef(false)
  const versionForceRef = useRef(false)

  const overviewQuery = useQuery({
    queryKey: codexKeys.dashboard.overview(),
    queryFn: () =>
      measureAsync('codex:overview-fetch', async () => {
        const force = overviewForceRef.current
        overviewForceRef.current = false
        return getCodexDashboardOverview({ force })
      }),
    staleTime: CODEX_DASHBOARD_STALE_TIME,
  })

  const usageQuery = useQuery({
    queryKey: codexKeys.dashboard.usageSummary(),
    queryFn: () =>
      measureAsync('codex:usage-summary-fetch', async () => {
        const force = usageForceRef.current
        usageForceRef.current = false
        return getCodexDashboardUsageSummary({ force })
      }),
    staleTime: CODEX_DASHBOARD_STALE_TIME,
  })

  const versionQuery = useQuery({
    queryKey: codexKeys.dashboard.version(),
    queryFn: () =>
      measureAsync('codex:version-fetch', () => {
        const force = versionForceRef.current
        versionForceRef.current = false
        return getCliVersion({ tool: 'codex', timeoutMs: 1_500, force })
      }),
    staleTime: CODEX_VERSION_STALE_TIME,
  })

  const overview = overviewQuery.data?.auth ? overviewQuery.data : null
  const usageSummary = usageQuery.data?.all_time ? usageQuery.data : null
  const formatDateTime = useCallback(
    (value?: string | null) => formatDashboardDateTime(value, t),
    [t],
  )
  const currentAccountLabel = useMemo(() => {
    const current = overview?.auth?.current
    return current?.name || current?.email || current?.account_id || t('codex.status.notSet')
  }, [overview, t])
  const currentProfileLabel = useMemo(
    () => overview?.profiles?.current_profile || t('codex.status.notSet'),
    [overview, t],
  )
  const { versionStatus, versionLabel } = useMemo(
    () => deriveVersionState(versionQuery.data, versionQuery.isFetching, t),
    [t, versionQuery.data, versionQuery.isFetching],
  )
  const readinessItems = useMemo(() => {
    if (!overview) return []
    return buildReadinessItems({
      overview,
      usageSummary,
      usageLoading: usageQuery.isFetching,
      currentAccountLabel,
      currentProfileLabel,
      formatDateTime,
      t,
    })
  }, [currentAccountLabel, currentProfileLabel, formatDateTime, overview, t, usageQuery.isFetching, usageSummary])
  const nextActions = useMemo(
    () => (overview ? buildNextActions({ overview, t }) : []),
    [overview, t],
  )
  const compactInventory = useMemo(
    () => (overview ? buildCompactInventory({ overview, t }) : []),
    [overview, t],
  )
  const primaryAction = useMemo<CodexDashboardActionItem>(
    () =>
      nextActions[0] ?? {
        title: t('codex.dashboard.actions.refresh.title'),
        description: t('codex.dashboard.actions.refresh.description'),
        to: '/codex/auth',
        icon: 'RefreshCw',
        tone: 'neutral',
      },
    [nextActions, t],
  )

  const refresh = useCallback(
    async (force = false) => {
      if (force) {
        overviewForceRef.current = true
        usageForceRef.current = true
        versionForceRef.current = true
      }
      const tasks: Array<Promise<unknown>> = []
      if (force || overviewQuery.isStale) tasks.push(overviewQuery.refetch())
      if (force || usageQuery.isStale) tasks.push(usageQuery.refetch())
      if (force || versionQuery.isStale) tasks.push(versionQuery.refetch())
      if (tasks.length === 0) return
      await Promise.allSettled(tasks)
    },
    [overviewQuery, usageQuery, versionQuery],
  )

  return {
    overview,
    usageSummary,
    loading: overviewQuery.isFetching || usageQuery.isFetching || versionQuery.isFetching,
    overviewLoading: overviewQuery.isFetching,
    usageLoading: usageQuery.isFetching,
    error: overviewQuery.error
      ? getErrorMessage(overviewQuery.error)
      : usageQuery.error
        ? getErrorMessage(usageQuery.error)
        : null,
    usageError: usageQuery.error ? getErrorMessage(usageQuery.error) : null,
    versionLabel,
    versionStatus,
    currentAccountLabel,
    currentProfileLabel,
    usageTotalRequests: usageSummary?.all_time?.total_requests ?? '—',
    usageTotalTokens: usageSummary?.all_time
      ? formatTokens(usageSummary.all_time.total_input_tokens + usageSummary.all_time.total_output_tokens)
      : '—',
    readinessItems,
    nextActions,
    primaryAction,
    compactInventory,
    formatDateTime,
    refresh,
  }
}
