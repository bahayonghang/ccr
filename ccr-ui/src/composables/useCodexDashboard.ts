import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  getCodexDashboardOverview,
  getCodexDashboardUsageSummary,
  type CodexDashboardOverview,
  type CodexDashboardUsageSummary,
} from '@/api'
import { getCliVersion } from '@/api/runtime/system'
import type { CliVersionEntry } from '@/types'
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

const DASHBOARD_TTL_MS = 30_000
const VERSION_TTL_MS = 60_000

let sharedOverview: CodexDashboardOverview | null = null
let sharedUsageSummary: CodexDashboardUsageSummary | null = null
let sharedVersionEntry: CliVersionEntry | null = null
let sharedOverviewLoadedAt = 0
let sharedUsageLoadedAt = 0
let sharedVersionLoadedAt = 0
let overviewInflight: Promise<CodexDashboardOverview> | null = null
let usageInflight: Promise<CodexDashboardUsageSummary> | null = null
let versionInflight: Promise<CliVersionEntry> | null = null

const nowMs = () => Date.now()

const isCacheFresh = (loadedAt: number, ttlMs: number) => (
  loadedAt > 0 && nowMs() - loadedAt < ttlMs
)

const measureAsync = async <T>(scope: string, action: () => Promise<T>): Promise<T> => {
  const token = `${scope}:${nowMs()}:${Math.random().toString(16).slice(2)}`
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

export function useCodexDashboard() {
  const { t } = useI18n()

  const overview = ref<CodexDashboardOverview | null>(sharedOverview)
  const usageSummary = ref<CodexDashboardUsageSummary | null>(sharedUsageSummary)
  const overviewLoading = ref(false)
  const usageLoading = ref(false)
  const versionLoading = ref(false)
  const overviewError = ref<string | null>(null)
  const usageError = ref<string | null>(null)
  const versionLabel = ref('...')
  const versionStatus = ref<'loading' | 'ok' | 'timeout' | 'error' | 'not_installed'>('loading')

  const syncCachedState = () => {
    overview.value = sharedOverview
    usageSummary.value = sharedUsageSummary
    if (sharedVersionEntry) {
      applyVersionEntry(sharedVersionEntry)
    }
  }

  const formatTokens = (tokens: number): string => {
    if (tokens >= 1_000_000) return `${(tokens / 1_000_000).toFixed(1)}M`
    if (tokens >= 1_000) return `${(tokens / 1_000).toFixed(1)}K`
    return String(tokens)
  }

  const formatDateTime = (value?: string | null): string => {
    if (!value) return t('common.notAvailable')

    const date = new Date(value)
    if (Number.isNaN(date.getTime())) return value

    return new Intl.DateTimeFormat('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    }).format(date)
  }

  const applyVersionEntry = (entry?: CliVersionEntry | null) => {
    if (!entry) {
      versionStatus.value = 'error'
      versionLabel.value = t('codex.status.retryVersionCheck')
      return
    }

    if (entry.status === 'timeout') {
      versionStatus.value = 'timeout'
      versionLabel.value = t('codex.status.checkingVersion')
      return
    }

    if (entry.status === 'error') {
      versionStatus.value = 'error'
      versionLabel.value = t('codex.status.retryVersionCheck')
      return
    }

    if (entry.status === 'not_installed' || !entry.installed) {
      versionStatus.value = 'not_installed'
      versionLabel.value = t('codex.status.notInstalled')
      return
    }

    versionStatus.value = 'ok'
    versionLabel.value = entry.version ? `v${entry.version}` : t('codex.status.installed')
  }

  const loadOverview = async (force = false) => {
    if (!force && sharedOverview && isCacheFresh(sharedOverviewLoadedAt, DASHBOARD_TTL_MS)) {
      return sharedOverview
    }

    if (!force && overviewInflight) {
      return overviewInflight
    }

    overviewInflight = measureAsync('codex:overview-fetch', () => (
      getCodexDashboardOverview<CodexDashboardOverview>({ force })
    ))
      .then((result) => {
        sharedOverview = result
        sharedOverviewLoadedAt = nowMs()
        return result
      })
      .finally(() => {
        overviewInflight = null
      })

    return overviewInflight
  }

  const loadUsageSummary = async (force = false) => {
    if (!force && sharedUsageSummary && isCacheFresh(sharedUsageLoadedAt, DASHBOARD_TTL_MS)) {
      return sharedUsageSummary
    }

    if (!force && usageInflight) {
      return usageInflight
    }

    usageInflight = measureAsync('codex:usage-summary-fetch', () => (
      getCodexDashboardUsageSummary<CodexDashboardUsageSummary>({ force })
    ))
      .then((result) => {
        sharedUsageSummary = result
        sharedUsageLoadedAt = nowMs()
        return result
      })
      .finally(() => {
        usageInflight = null
      })

    return usageInflight
  }

  const loadVersion = async (force = false) => {
    if (!force && sharedVersionEntry && isCacheFresh(sharedVersionLoadedAt, VERSION_TTL_MS)) {
      return sharedVersionEntry
    }

    if (!force && versionInflight) {
      return versionInflight
    }

    versionInflight = measureAsync('codex:version-fetch', () => (
      getCliVersion<CliVersionEntry>({
        tool: 'codex',
        timeoutMs: 1_500,
        force,
      })
    ))
      .then((entry) => {
        sharedVersionEntry = entry
        sharedVersionLoadedAt = nowMs()
        return entry
      })
      .finally(() => {
        versionInflight = null
      })

    return versionInflight
  }

  const refresh = async (force = false) => {
    syncCachedState()
    overviewError.value = null
    usageError.value = null

    const tasks: Array<Promise<void>> = []

    if (force || !sharedOverview || !isCacheFresh(sharedOverviewLoadedAt, DASHBOARD_TTL_MS)) {
      overviewLoading.value = true
      tasks.push(
        loadOverview(force)
          .then((result) => {
            overview.value = result
          })
          .catch((reason: unknown) => {
            overviewError.value = reason instanceof Error ? reason.message : String(reason)
          })
          .finally(() => {
            overviewLoading.value = false
          }),
      )
    }

    if (force || !sharedUsageSummary || !isCacheFresh(sharedUsageLoadedAt, DASHBOARD_TTL_MS)) {
      usageLoading.value = true
      tasks.push(
        loadUsageSummary(force)
          .then((result) => {
            usageSummary.value = result
          })
          .catch((reason: unknown) => {
            usageError.value = reason instanceof Error ? reason.message : String(reason)
          })
          .finally(() => {
            usageLoading.value = false
          }),
      )
    }

    if (force || !sharedVersionEntry || !isCacheFresh(sharedVersionLoadedAt, VERSION_TTL_MS)) {
      versionLoading.value = true
      tasks.push(
        loadVersion(force)
          .then((entry) => {
            applyVersionEntry(entry)
          })
          .catch(() => {
            applyVersionEntry(undefined)
          })
          .finally(() => {
            versionLoading.value = false
          }),
      )
    } else if (sharedVersionEntry) {
      applyVersionEntry(sharedVersionEntry)
    }

    if (tasks.length === 0) {
      return
    }

    await Promise.allSettled(tasks)
  }

  const loading = computed(() => (
    overviewLoading.value || usageLoading.value || versionLoading.value
  ))

  const error = computed(() => overviewError.value ?? usageError.value)

  const currentAccountLabel = computed(() => {
    const current = overview.value?.auth.current
    return current?.name || current?.email || current?.account_id || t('codex.status.notSet')
  })

  const currentProfileLabel = computed(() => {
    return overview.value?.profiles.current_profile || t('codex.status.notSet')
  })

  const usageTotalRequests = computed(() => {
    return usageSummary.value?.all_time.total_requests ?? '—'
  })

  const usageTotalTokens = computed(() => {
    const usage = usageSummary.value?.all_time
    if (!usage) return '—'
    return formatTokens(usage.total_input_tokens + usage.total_output_tokens)
  })

  const readinessItems = computed<CodexDashboardReadinessItem[]>(() => {
    const data = overview.value
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

    return [
      {
        key: 'auth',
        title: t('codex.dashboard.readiness.auth.title'),
        value: currentAccountLabel.value,
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
        value: currentProfileLabel.value,
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
        value: usageSummary.value?.freshness_description || t('codex.dashboard.usage.loading'),
        detail: usageSummary.value?.last_activity_at
          ? t('codex.dashboard.readiness.usage.activity', { time: formatDateTime(usageSummary.value.last_activity_at) })
          : usageLoading.value
            ? t('codex.dashboard.readiness.usage.loadingDetail')
            : t('codex.dashboard.readiness.usage.emptyDetail'),
        statusLabel: usageSummary.value
          ? t(`codex.dashboard.usageFreshness.${usageSummary.value.freshness}`)
          : usageLoading.value
            ? t('codex.dashboard.statusLabels.checking')
            : t('codex.dashboard.statusLabels.idle'),
        tone: usageToneMap[usageSummary.value?.freshness ?? 'empty'],
        icon: 'BarChart3',
        to: '/usage',
      },
    ]
  })

  const healthItems = readinessItems

  const nextActions = computed<CodexDashboardActionItem[]>(() => {
    const data = overview.value
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
  })

  const primaryAction = computed<CodexDashboardActionItem>(() => (
    nextActions.value[0] ?? {
      title: t('codex.dashboard.actions.refresh.title'),
      description: t('codex.dashboard.actions.refresh.description'),
      to: '/codex/auth',
      icon: 'RefreshCw',
      tone: 'neutral',
    }
  ))

  const compactInventory = computed<CodexDashboardInventoryItem[]>(() => {
    const data = overview.value
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
  })

  const managementLinks = computed<CodexDashboardLinkItem[]>(() => {
    const data = overview.value
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
  })

  syncCachedState()

  return {
    overview,
    usageSummary,
    loading,
    overviewLoading,
    usageLoading,
    versionLoading,
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
