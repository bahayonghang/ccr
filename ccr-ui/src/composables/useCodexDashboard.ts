import { useCallback, useMemo, useRef } from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  getCodexDashboardOverview,
  getCodexDashboardUsageSummary,
  type CodexDashboardUsageSummary,
} from '@/api'
import { getCliVersion } from '@/api/runtime/system'
import type { CliVersionEntry } from '@/types'
import {
  CODEX_DASHBOARD_STALE_TIME,
  CODEX_VERSION_STALE_TIME,
  codexKeys,
} from '@/features/codex/queries'
import { getErrorMessage } from '@/utils/errorHandler'
import { perfMark, perfMeasure } from '@/utils/perfTelemetry'

export type CodexDashboardTone = 'success' | 'warning' | 'danger' | 'neutral'

export interface CodexDashboardHealthItem {
  key: string
  title: string
  value: string
  detail: string
  statusLabel: string
  tone: CodexDashboardTone
  icon: string
  to: string
}

export type CodexDashboardReadinessItem = CodexDashboardHealthItem

export interface CodexDashboardActionItem {
  title: string
  description: string
  to: string
  icon: string
  tone: CodexDashboardTone
}

export interface CodexDashboardLinkItem {
  title: string
  description: string
  to: string
  icon: string
  badge: string
  tone: CodexDashboardTone
}

export interface CodexDashboardInventoryItem {
  key: string
  title: string
  value: string
  detail: string
  to: string
  icon: string
  tone: CodexDashboardTone
}

// Codex 仪表盘的 React 迁移（08-22-state-logic-port 批次 5，服务端数据 → Query）。
// 原模块级共享 TTL 缓存 / in-flight 去重由 Query 缓存承担：
// overview/usage staleTime 30s（原 DASHBOARD_TTL_MS）、version 60s（原 VERSION_TTL_MS）。
//
// 签名变化（消费方均为待迁移 .vue 视图）：
// - i18n 由 vue-i18n useI18n 改为参数传入 t（与 shell/hooks 的既有形态一致）；
// - 返回对象中的 Ref<T> 改为普通值（useMemo 派生）；
// - refresh(force) 的 force 经 forceRef 透传给后端 IPC wrapper，非 force 时仅重拉
//   已陈旧（isStale）的切片，等价原 TTL 检查后的按需加载；
// - versionLabel/versionStatus 由 applyVersionEntry 改写为纯函数派生（useMemo），
//   初值保持原字面量：status='loading'、label='...'。

type Translate = (key: string, params?: Record<string, unknown>) => string

interface UseCodexDashboardOptions {
  /** i18n 翻译函数（原 useI18n().t）。 */
  t: Translate
}

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

const formatTokens = (tokens: number): string => {
  if (tokens >= 1_000_000) return `${(tokens / 1_000_000).toFixed(1)}M`
  if (tokens >= 1_000) return `${(tokens / 1_000).toFixed(1)}K`
  return String(tokens)
}

/** 原 applyVersionEntry 的纯函数化；entry 缺失时按是否在拉取区分 loading/error。 */
const deriveVersionState = (
  entry: CliVersionEntry | null | undefined,
  fetching: boolean,
  t: Translate,
): { versionStatus: 'loading' | 'ok' | 'timeout' | 'error' | 'not_installed'; versionLabel: string } => {
  if (!entry) {
    return fetching
      ? { versionStatus: 'loading', versionLabel: '...' }
      : { versionStatus: 'error', versionLabel: t('codex.status.retryVersionCheck') }
  }

  if (entry.status === 'timeout') {
    return { versionStatus: 'timeout', versionLabel: t('codex.status.checkingVersion') }
  }

  if (entry.status === 'error') {
    return { versionStatus: 'error', versionLabel: t('codex.status.retryVersionCheck') }
  }

  if (entry.status === 'not_installed' || !entry.installed) {
    return { versionStatus: 'not_installed', versionLabel: t('codex.status.notInstalled') }
  }

  return {
    versionStatus: 'ok',
    versionLabel: entry.version ? `v${entry.version}` : t('codex.status.installed'),
  }
}

export function useCodexDashboard({ t }: UseCodexDashboardOptions) {

  // force 透传：refetch 重跑 queryFn 时消费一次（原 loadOverview/loadUsageSummary/
  // loadVersion 的 { force } 参数语义）
  const overviewForceRef = useRef(false)
  const usageForceRef = useRef(false)
  const versionForceRef = useRef(false)

  const overviewQuery = useQuery({
    queryKey: codexKeys.dashboard.overview(),
    queryFn: () => measureAsync('codex:overview-fetch', async () => {
      const force = overviewForceRef.current
      overviewForceRef.current = false
      return getCodexDashboardOverview({ force })
    }),
    staleTime: CODEX_DASHBOARD_STALE_TIME,
  })

  const usageQuery = useQuery({
    queryKey: codexKeys.dashboard.usageSummary(),
    queryFn: () => measureAsync('codex:usage-summary-fetch', async () => {
      const force = usageForceRef.current
      usageForceRef.current = false
      return getCodexDashboardUsageSummary({ force })
    }),
    staleTime: CODEX_DASHBOARD_STALE_TIME,
  })

  const versionQuery = useQuery({
    queryKey: codexKeys.dashboard.version(),
    queryFn: () => measureAsync('codex:version-fetch', () => {
      const force = versionForceRef.current
      versionForceRef.current = false
      return getCliVersion({ tool: 'codex', timeoutMs: 1_500, force })
    }),
    staleTime: CODEX_VERSION_STALE_TIME,
  })

  const overview = overviewQuery.data ?? null
  const usageSummary = usageQuery.data ?? null

  const loading = useMemo(
    () => overviewQuery.isFetching || usageQuery.isFetching || versionQuery.isFetching,
    [overviewQuery.isFetching, usageQuery.isFetching, versionQuery.isFetching]
  )

  const overviewError = overviewQuery.error ? getErrorMessage(overviewQuery.error) : null
  const usageError = usageQuery.error ? getErrorMessage(usageQuery.error) : null

  const error = useMemo(
    () => overviewError ?? usageError,
    [overviewError, usageError]
  )

  const formatDateTime = useCallback((value?: string | null): string => {
    if (!value) return t('common.notAvailable')

    const date = new Date(value)
    if (Number.isNaN(date.getTime())) return value

    return new Intl.DateTimeFormat('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    }).format(date)
  }, [t])

  const currentAccountLabel = useMemo(() => {
    const current = overview?.auth.current
    return current?.name || current?.email || current?.account_id || t('codex.status.notSet')
  }, [overview, t])

  const currentProfileLabel = useMemo(
    () => overview?.profiles.current_profile || t('codex.status.notSet'),
    [overview, t]
  )

  const usageTotalRequests = useMemo(
    () => usageSummary?.all_time.total_requests ?? '—',
    [usageSummary]
  )

  const usageTotalTokens = useMemo(() => {
    const usage = usageSummary?.all_time
    if (!usage) return '—'
    return formatTokens(usage.total_input_tokens + usage.total_output_tokens)
  }, [usageSummary])

  const { versionStatus, versionLabel } = useMemo(
    () => deriveVersionState(versionQuery.data, versionQuery.isFetching, t),
    [t, versionQuery.data, versionQuery.isFetching]
  )

  // 原 computed readinessItems：来源 overview/usageSummary/loading 标志 + i18n + 派生标签
  const readinessItems = useMemo<CodexDashboardReadinessItem[]>(() => {
    const data = overview
    if (!data) return []

    const authTone: CodexDashboardTone = !data.auth.logged_in
      ? 'danger'
      : 'success'

    const profileTone: CodexDashboardTone =
      data.profiles.total === 0 ? 'danger' : !data.profiles.current_profile ? 'warning' : 'success'

    const configTone: CodexDashboardTone = !data.config.model
      ? 'warning'
      : !data.config.approval_policy || !data.config.sandbox_mode
        ? 'warning'
        : 'success'

    const usageToneMap: Record<CodexDashboardUsageSummary['freshness'], CodexDashboardTone> = {
      fresh: 'success',
      stale: 'warning',
      old: 'danger',
      empty: 'neutral',
    }

    const usageLoading = usageQuery.isFetching

    return [
      {
        key: 'auth',
        title: t('codex.dashboard.readiness.auth.title'),
        value: currentAccountLabel,
        detail: data.auth.logged_in
          ? data.auth.current?.last_refresh
            ? t('codex.dashboard.readiness.auth.refreshed', { time: formatDateTime(data.auth.current.last_refresh) })
            : t('codex.dashboard.readiness.auth.ready')
          : t('codex.dashboard.readiness.auth.missing'),
        statusLabel: data.auth.logged_in
          ? t('codex.dashboard.statusLabels.ready')
          : t('codex.dashboard.statusLabels.blocked'),
        tone: authTone,
        icon: 'ShieldCheck',
        to: '/codex/auth',
      },
      {
        key: 'profiles',
        title: t('codex.dashboard.readiness.profiles.title'),
        value: currentProfileLabel,
        detail: t('codex.dashboard.readiness.profiles.detail', {
          total: data.profiles.total,
          enabled: data.profiles.enabled_total,
        }),
        statusLabel: profileTone === 'success'
          ? t('codex.dashboard.statusLabels.ready')
          : profileTone === 'danger'
            ? t('codex.dashboard.statusLabels.blocked')
            : t('codex.dashboard.statusLabels.attention'),
        tone: profileTone,
        icon: 'Settings2',
        to: '/codex/profiles',
      },
      {
        key: 'config',
        title: t('codex.dashboard.readiness.config.title'),
        value: data.config.model || t('codex.dashboard.readiness.config.noModel'),
        detail: t('codex.dashboard.readiness.config.detail', {
          approval: data.config.approval_policy || t('codex.dashboard.readiness.config.noApproval'),
          sandbox: data.config.sandbox_mode || t('codex.dashboard.readiness.config.noSandbox'),
        }),
        statusLabel: configTone === 'success'
          ? t('codex.dashboard.statusLabels.ready')
          : t('codex.dashboard.statusLabels.attention'),
        tone: configTone,
        icon: 'SlidersHorizontal',
        to: '/codex/settings',
      },
      {
        key: 'usage',
        title: t('codex.dashboard.readiness.usage.title'),
        value: usageSummary?.freshness_description || t('codex.dashboard.usage.loading'),
        detail: usageSummary?.last_activity_at
          ? t('codex.dashboard.readiness.usage.activity', { time: formatDateTime(usageSummary.last_activity_at) })
          : usageLoading
            ? t('codex.dashboard.readiness.usage.loadingDetail')
            : t('codex.dashboard.readiness.usage.emptyDetail'),
        statusLabel: usageSummary
          ? t(`codex.dashboard.usageFreshness.${usageSummary.freshness}`)
          : usageLoading
            ? t('codex.dashboard.statusLabels.checking')
            : t('codex.dashboard.statusLabels.idle'),
        tone: usageToneMap[usageSummary?.freshness ?? 'empty'],
        icon: 'BarChart3',
        to: '/usage',
      },
    ]
  }, [currentAccountLabel, currentProfileLabel, formatDateTime, overview, t, usageQuery.isFetching, usageSummary])

  const healthItems = readinessItems

  // 原 computed nextActions：actions 为 memo 内本地累积数组（mutation-rewrite.md 判定）
  const nextActions = useMemo<CodexDashboardActionItem[]>(() => {
    const data = overview
    if (!data) return []

    const actions: CodexDashboardActionItem[] = []

    if (!data.auth.logged_in) {
      actions.push({
        title: t('codex.dashboard.actions.login.title'),
        description: t('codex.dashboard.actions.login.description'),
        to: '/codex/auth',
        icon: 'LogIn',
        tone: 'danger',
      })
    }

    if (data.profiles.total === 0) {
      actions.push({
        title: t('codex.dashboard.actions.createProfile.title'),
        description: t('codex.dashboard.actions.createProfile.description'),
        to: '/codex/profiles',
        icon: 'Plus',
        tone: 'warning',
      })
    } else if (!data.profiles.current_profile) {
      actions.push({
        title: t('codex.dashboard.actions.selectProfile.title'),
        description: t('codex.dashboard.actions.selectProfile.description'),
        to: '/codex/profiles',
        icon: 'ArrowRightLeft',
        tone: 'warning',
      })
    }

    if (!data.config.model || !data.config.approval_policy || !data.config.sandbox_mode) {
      actions.push({
        title: t('codex.dashboard.actions.completeSettings.title'),
        description: t('codex.dashboard.actions.completeSettings.description'),
        to: '/codex/settings',
        icon: 'Lock',
        tone: 'warning',
      })
    }

    if (data.inventory.mcp_servers_total === 0) {
      actions.push({
        title: t('codex.dashboard.actions.addMcp.title'),
        description: t('codex.dashboard.actions.addMcp.description'),
        to: '/codex/mcp',
        icon: 'Server',
        tone: 'neutral',
      })
    }

    if (actions.length === 0 && data.inventory.sessions_total > 0) {
      actions.push({
        title: t('codex.dashboard.actions.openSessions.title'),
        description: t('codex.dashboard.actions.openSessions.description'),
        to: '/codex/sessions',
        icon: 'MessagesSquare',
        tone: 'success',
      })
    }

    if (actions.length === 0) {
      actions.push({
        title: t('codex.dashboard.actions.ready.title'),
        description: t('codex.dashboard.actions.ready.description'),
        to: '/codex/sessions',
        icon: 'Sparkles',
        tone: 'success',
      })
    }

    return actions.slice(0, 3)
  }, [overview, t])

  const primaryAction = useMemo<CodexDashboardActionItem>(() => (
    nextActions[0] ?? {
      title: t('codex.dashboard.actions.refresh.title'),
      description: t('codex.dashboard.actions.refresh.description'),
      to: '/codex/auth',
      icon: 'RefreshCw',
      tone: 'neutral',
    }
  ), [nextActions, t])

  const compactInventory = useMemo<CodexDashboardInventoryItem[]>(() => {
    const data = overview
    if (!data) return []

    return [
      {
        key: 'auth',
        title: t('codex.dashboard.management.auth.title'),
        value: String(data.auth.saved_accounts_total),
        detail: t('codex.dashboard.management.auth.badge', { count: data.auth.saved_accounts_total }),
        to: '/codex/auth',
        icon: 'KeyRound',
        tone: data.auth.logged_in ? 'success' : 'danger',
      },
      {
        key: 'profiles',
        title: t('codex.dashboard.management.profiles.title'),
        value: String(data.profiles.total),
        detail: t('codex.dashboard.management.profiles.badge', { count: data.profiles.total }),
        to: '/codex/profiles',
        icon: 'Folders',
        tone: data.profiles.total > 0 ? 'success' : 'warning',
      },
      {
        key: 'settings',
        title: t('codex.dashboard.management.settings.title'),
        value: data.config.model || '—',
        detail: t('codex.dashboard.management.settings.badge', {
          model: data.config.model || t('codex.dashboard.readiness.config.noModel'),
        }),
        to: '/codex/settings',
        icon: 'SlidersHorizontal',
        tone: data.config.model ? 'neutral' : 'warning',
      },
      {
        key: 'mcp',
        title: t('codex.dashboard.management.mcp.title'),
        value: String(data.inventory.mcp_servers_total),
        detail: t('codex.dashboard.management.mcp.badge', { count: data.inventory.mcp_servers_total }),
        to: '/codex/mcp',
        icon: 'Server',
        tone: data.inventory.mcp_servers_total > 0 ? 'neutral' : 'warning',
      },
      {
        key: 'agents',
        title: t('codex.dashboard.management.agents.title'),
        value: String(data.inventory.agents_total),
        detail: t('codex.dashboard.management.agents.badge', { count: data.inventory.agents_total }),
        to: '/codex/agents',
        icon: 'Bot',
        tone: data.inventory.agents_total > 0 ? 'neutral' : 'warning',
      },
      {
        key: 'sessions',
        title: t('codex.dashboard.management.sessions.title'),
        value: String(data.inventory.sessions_total),
        detail: t('codex.dashboard.management.sessions.badge', { count: data.inventory.sessions_total }),
        to: '/codex/sessions',
        icon: 'MessagesSquare',
        tone: data.inventory.sessions_total > 0 ? 'success' : 'neutral',
      },
    ]
  }, [overview, t])

  const managementLinks = useMemo<CodexDashboardLinkItem[]>(() => {
    const data = overview
    if (!data) return []

    return [
      {
        title: t('codex.dashboard.management.auth.title'),
        description: t('codex.dashboard.management.auth.description'),
        to: '/codex/auth',
        icon: 'KeyRound',
        badge: t('codex.dashboard.management.auth.badge', { count: data.auth.saved_accounts_total }),
        tone: data.auth.logged_in ? 'success' : 'danger',
      },
      {
        title: t('codex.dashboard.management.profiles.title'),
        description: t('codex.dashboard.management.profiles.description'),
        to: '/codex/profiles',
        icon: 'Folders',
        badge: t('codex.dashboard.management.profiles.badge', { count: data.profiles.total }),
        tone: data.profiles.total > 0 ? 'success' : 'warning',
      },
      {
        title: t('codex.dashboard.management.settings.title'),
        description: t('codex.dashboard.management.settings.description'),
        to: '/codex/settings',
        icon: 'SlidersHorizontal',
        badge: t('codex.dashboard.management.settings.badge', {
          model: data.config.model || t('codex.dashboard.readiness.config.noModel'),
        }),
        tone: data.config.model ? 'neutral' : 'warning',
      },
      {
        title: t('codex.dashboard.management.mcp.title'),
        description: t('codex.dashboard.management.mcp.description'),
        to: '/codex/mcp',
        icon: 'Server',
        badge: t('codex.dashboard.management.mcp.badge', { count: data.inventory.mcp_servers_total }),
        tone: data.inventory.mcp_servers_total > 0 ? 'neutral' : 'warning',
      },
      {
        title: t('codex.dashboard.management.agents.title'),
        description: t('codex.dashboard.management.agents.description'),
        to: '/codex/agents',
        icon: 'Bot',
        badge: t('codex.dashboard.management.agents.badge', { count: data.inventory.agents_total }),
        tone: data.inventory.agents_total > 0 ? 'neutral' : 'warning',
      },
      {
        title: t('codex.dashboard.management.sessions.title'),
        description: t('codex.dashboard.management.sessions.description'),
        to: '/codex/sessions',
        icon: 'MessagesSquare',
        badge: t('codex.dashboard.management.sessions.badge', { count: data.inventory.sessions_total }),
        tone: data.inventory.sessions_total > 0 ? 'success' : 'neutral',
      },
    ]
  }, [overview, t])

  const refresh = useCallback(async (force = false) => {
    if (force) {
      overviewForceRef.current = true
      usageForceRef.current = true
      versionForceRef.current = true
    }

    // 非 force 仅重拉已陈旧的切片（等价原 isCacheFresh 检查后的按需加载）
    const tasks: Array<Promise<unknown>> = []
    if (force || overviewQuery.isStale) tasks.push(overviewQuery.refetch())
    if (force || usageQuery.isStale) tasks.push(usageQuery.refetch())
    if (force || versionQuery.isStale) tasks.push(versionQuery.refetch())

    if (tasks.length === 0) {
      return
    }

    await Promise.allSettled(tasks)
  }, [overviewQuery, usageQuery, versionQuery])

  return {
    overview,
    usageSummary,
    loading,
    overviewLoading: overviewQuery.isFetching,
    usageLoading: usageQuery.isFetching,
    versionLoading: versionQuery.isFetching,
    error,
    overviewError,
    usageError,
    versionLabel,
    versionStatus,
    currentAccountLabel,
    currentProfileLabel,
    usageTotalRequests,
    usageTotalTokens,
    healthItems,
    readinessItems,
    nextActions,
    primaryAction,
    compactInventory,
    managementLinks,
    formatTokens,
    formatDateTime,
    refresh,
  }
}
