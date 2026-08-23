import type { CodexDashboardOverview, CodexDashboardUsageSummary } from '@/api'
import type { TranslateFunction } from '@/utils/tf'

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

export interface CodexDashboardActionItem {
  title: string
  description: string
  to: string
  icon: string
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

type Translate = TranslateFunction

export const formatTokens = (tokens: number): string => {
  if (tokens >= 1_000_000) return `${(tokens / 1_000_000).toFixed(1)}M`
  if (tokens >= 1_000) return `${(tokens / 1_000).toFixed(1)}K`
  return String(tokens)
}

export const formatDashboardDateTime = (value: string | null | undefined, t: Translate): string => {
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

const statusOf = (tone: CodexDashboardTone, t: Translate) => {
  if (tone === 'success') return t('codex.dashboard.statusLabels.ready')
  if (tone === 'danger') return t('codex.dashboard.statusLabels.blocked')
  return t('codex.dashboard.statusLabels.attention')
}

const profileToneOf = (total: number, current: string | null | undefined): CodexDashboardTone => {
  if (total === 0) return 'danger'
  if (!current) return 'warning'
  return 'success'
}

const configToneOf = (model?: string | null, approval?: string | null, sandbox?: string | null): CodexDashboardTone => {
  if (!model || !approval || !sandbox) return 'warning'
  return 'success'
}

const authDetailOf = (data: CodexDashboardOverview, formatDateTime: (value?: string | null) => string, t: Translate) => {
  if (!data.auth.logged_in) return t('codex.dashboard.readiness.auth.missing')
  if (data.auth.current?.last_refresh) {
    return t('codex.dashboard.readiness.auth.refreshed', { time: formatDateTime(data.auth.current.last_refresh) })
  }
  return t('codex.dashboard.readiness.auth.ready')
}

const usageCopy = (input: {
  usageSummary: CodexDashboardUsageSummary | null
  usageLoading: boolean
  formatDateTime: (value?: string | null) => string
  t: Translate
}) => {
  const { usageSummary, usageLoading, formatDateTime, t } = input
  const status = usageSummary
    ? t(`codex.dashboard.usageFreshness.${usageSummary.freshness}`)
    : usageLoading
      ? t('codex.dashboard.statusLabels.checking')
      : t('codex.dashboard.statusLabels.idle')
  const detail = usageSummary?.last_activity_at
    ? t('codex.dashboard.readiness.usage.activity', { time: formatDateTime(usageSummary.last_activity_at) })
    : usageLoading
      ? t('codex.dashboard.readiness.usage.loadingDetail')
      : t('codex.dashboard.readiness.usage.emptyDetail')
  return { status, detail }
}

export const buildReadinessItems = (input: {
  overview: CodexDashboardOverview
  usageSummary: CodexDashboardUsageSummary | null
  usageLoading: boolean
  currentAccountLabel: string
  currentProfileLabel: string
  formatDateTime: (value?: string | null) => string
  t: Translate
}): CodexDashboardHealthItem[] => {
  const { overview: data, usageSummary, usageLoading, currentAccountLabel, currentProfileLabel, formatDateTime, t } = input
  const authTone: CodexDashboardTone = data.auth.logged_in ? 'success' : 'danger'
  const profileTone = profileToneOf(data.profiles.total, data.profiles.current_profile)
  const configTone = configToneOf(data.config.model, data.config.approval_policy, data.config.sandbox_mode)
  const usage = usageCopy({ usageSummary, usageLoading, formatDateTime, t })
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
      value: currentAccountLabel,
      detail: authDetailOf(data, formatDateTime, t),
      statusLabel: statusOf(authTone, t),
      tone: authTone,
      icon: 'ShieldCheck',
      to: '/codex/auth',
    },
    {
      key: 'profiles',
      title: t('codex.dashboard.readiness.profiles.title'),
      value: currentProfileLabel,
      detail: t('codex.dashboard.readiness.profiles.detail', { total: data.profiles.total, enabled: data.profiles.enabled_total }),
      statusLabel: statusOf(profileTone, t),
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
      statusLabel: statusOf(configTone, t),
      tone: configTone,
      icon: 'SlidersHorizontal',
      to: '/codex/settings',
    },
    {
      key: 'usage',
      title: t('codex.dashboard.readiness.usage.title'),
      value: usageSummary?.freshness_description || t('codex.dashboard.usage.loading'),
      detail: usage.detail,
      statusLabel: usage.status,
      tone: usageToneMap[usageSummary?.freshness ?? 'empty'],
      icon: 'BarChart3',
      to: '/usage',
    },
  ]
}

export const buildNextActions = (input: {
  overview: CodexDashboardOverview
  t: Translate
}): CodexDashboardActionItem[] => {
  const { overview: data, t } = input
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
  if (actions.length === 0) {
    const ready = data.inventory.sessions_total > 0
    actions.push({
      title: ready ? t('codex.dashboard.actions.openSessions.title') : t('codex.dashboard.actions.ready.title'),
      description: ready
        ? t('codex.dashboard.actions.openSessions.description')
        : t('codex.dashboard.actions.ready.description'),
      to: '/codex/sessions',
      icon: ready ? 'MessagesSquare' : 'Sparkles',
      tone: 'success',
    })
  }
  return actions.slice(0, 3)
}

export const buildCompactInventory = (input: {
  overview: CodexDashboardOverview
  t: Translate
}): CodexDashboardInventoryItem[] => {
  const { overview: data, t } = input
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
}
