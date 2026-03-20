import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { getCliVersions, getCodexDashboardSummary } from '@/api'
import type { CodexDashboardSummary } from '@/api'
import type { CliVersionEntry, CliVersionsResponse } from '@/types'

export type CodexDashboardTone = 'success' | 'warning' | 'danger' | 'neutral'

export interface CodexDashboardHealthItem {
  key: string
  title: string
  value: string
  detail: string
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

export interface CodexDashboardLinkItem {
  title: string
  description: string
  to: string
  icon: string
  badge: string
  tone: CodexDashboardTone
}

const DASHBOARD_TTL_MS = 30_000
const VERSION_TTL_MS = 60_000

export function useCodexDashboard() {
  const { t } = useI18n()

  const summary = ref<CodexDashboardSummary | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const versionLabel = ref('...')
  const versionStatus = ref<'loading' | 'ok' | 'timeout' | 'error' | 'not_installed'>('loading')
  const lastDashboardLoadedAt = ref(0)
  const lastVersionLoadedAt = ref(0)

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
      minute: '2-digit'
    }).format(date)
  }

  const applyVersionEntry = (entry?: CliVersionEntry) => {
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

  const loadSummary = async (force = false) => {
    if (!force && summary.value && Date.now() - lastDashboardLoadedAt.value < DASHBOARD_TTL_MS) {
      return summary.value
    }

    const result = await getCodexDashboardSummary<CodexDashboardSummary>()
    summary.value = result
    lastDashboardLoadedAt.value = Date.now()
    return result
  }

  const loadVersion = async (force = false) => {
    if (!force && Date.now() - lastVersionLoadedAt.value < VERSION_TTL_MS) {
      return
    }

    const versions = await getCliVersions<CliVersionsResponse>({ mode: 'fast', timeoutMs: 3500, parallelism: 4 })
    const codex = versions.versions.find((item: CliVersionEntry) => item.platform === 'codex')
    applyVersionEntry(codex)
    lastVersionLoadedAt.value = Date.now()
  }

  const refresh = async (force = false) => {
    loading.value = true
    error.value = null

    const [summaryResult, versionResult] = await Promise.allSettled([
      loadSummary(force),
      loadVersion(force)
    ])

    if (summaryResult.status === 'rejected') {
      error.value = summaryResult.reason instanceof Error ? summaryResult.reason.message : String(summaryResult.reason)
    }

    if (versionResult.status === 'rejected') {
      applyVersionEntry(undefined)
    }

    loading.value = false
  }

  const currentAccountLabel = computed(() => {
    const current = summary.value?.auth.current
    return current?.name || current?.email || current?.account_id || t('codex.status.notSet')
  })

  const currentProfileLabel = computed(() => {
    return summary.value?.profiles.current_profile || t('codex.status.notSet')
  })

  const usageTotalTokens = computed(() => {
    const usage = summary.value?.usage.all_time
    if (!usage) return '0'
    return formatTokens(usage.total_input_tokens + usage.total_output_tokens)
  })

  const healthItems = computed<CodexDashboardHealthItem[]>(() => {
    const data = summary.value
    if (!data) return []

    const authTone: CodexDashboardTone = !data.auth.logged_in
      ? 'danger'
      : data.auth.current?.is_expired
        ? 'warning'
        : 'success'

    const profileTone: CodexDashboardTone = data.profiles.total === 0
      ? 'danger'
      : !data.profiles.current_profile
        ? 'warning'
        : 'success'

    const configTone: CodexDashboardTone = !data.config.model
      ? 'warning'
      : !data.config.approval_policy || !data.config.sandbox_mode
        ? 'warning'
        : 'success'

    const usageToneMap: Record<CodexDashboardSummary['usage']['freshness'], CodexDashboardTone> = {
      fresh: 'success',
      stale: 'warning',
      old: 'danger',
      empty: 'neutral'
    }

    return [
      {
        key: 'auth',
        title: '当前账号',
        value: currentAccountLabel.value,
        detail: data.auth.logged_in
          ? (data.auth.current?.freshness_description || '已登录，可继续使用')
          : '尚未登录 Codex 账号',
        tone: authTone,
        icon: 'ShieldCheck',
        to: '/codex/auth'
      },
      {
        key: 'profiles',
        title: '当前 Profile',
        value: currentProfileLabel.value,
        detail: `共 ${data.profiles.total} 个，启用 ${data.profiles.enabled_total} 个`,
        tone: profileTone,
        icon: 'Settings2',
        to: '/codex/profiles'
      },
      {
        key: 'config',
        title: '模型与权限',
        value: data.config.model || '未配置模型',
        detail: `${data.config.approval_policy || '未设置审批'} · ${data.config.sandbox_mode || '未设置沙箱'}`,
        tone: configTone,
        icon: 'SlidersHorizontal',
        to: '/codex/settings'
      },
      {
        key: 'usage',
        title: '用量新鲜度',
        value: data.usage.freshness_description,
        detail: data.usage.last_activity_at
          ? `最近活动 ${formatDateTime(data.usage.last_activity_at)}`
          : '还没有可用的使用记录',
        tone: usageToneMap[data.usage.freshness],
        icon: 'BarChart3',
        to: '/usage'
      }
    ]
  })

  const nextActions = computed<CodexDashboardActionItem[]>(() => {
    const data = summary.value
    if (!data) return []

    const actions: CodexDashboardActionItem[] = []

    if (!data.auth.logged_in) {
      actions.push({
        title: '先完成账号登录',
        description: '进入 Auth 页面登录或切换到可用账号，避免后续流程卡住。',
        to: '/codex/auth',
        icon: 'LogIn',
        tone: 'danger'
      })
    }

    if (data.profiles.total === 0) {
      actions.push({
        title: '创建首个 Profile',
        description: '先准备一个可切换的 Profile，把模型、鉴权和策略固定下来。',
        to: '/codex/profiles',
        icon: 'Plus',
        tone: 'warning'
      })
    } else if (!data.profiles.current_profile) {
      actions.push({
        title: '指定当前 Profile',
        description: '已有配置但未激活当前 Profile，建议先切换到默认工作配置。',
        to: '/codex/profiles',
        icon: 'ArrowRightLeft',
        tone: 'warning'
      })
    }

    if (!data.config.model || !data.config.approval_policy || !data.config.sandbox_mode) {
      actions.push({
        title: '补齐 CLI 安全设置',
        description: '检查模型、审批策略和沙箱模式，确保日常工作流可直接使用。',
        to: '/codex/settings',
        icon: 'Lock',
        tone: 'warning'
      })
    }

    if (data.inventory.mcp_servers_total === 0) {
      actions.push({
        title: '接入 MCP 能力',
        description: '如果你要把 Codex 接到本地工具链，现在可以添加第一个 MCP 服务器。',
        to: '/codex/mcp',
        icon: 'Server',
        tone: 'neutral'
      })
    }

    if (actions.length === 0) {
      actions.push({
        title: '工作流已经就绪',
        description: '账号、Profile 和核心配置都可用，可以直接开始日常使用或微调扩展能力。',
        to: '/codex/settings',
        icon: 'Sparkles',
        tone: 'success'
      })
    }

    return actions.slice(0, 3)
  })

  const managementLinks = computed<CodexDashboardLinkItem[]>(() => {
    const data = summary.value
    if (!data) return []

    return [
      {
        title: 'Auth 与账号',
        description: '查看当前会话、保存账号和过期状态。',
        to: '/codex/auth',
        icon: 'KeyRound',
        badge: `${data.auth.saved_accounts_total} 个账号`,
        tone: data.auth.logged_in ? 'success' : 'danger'
      },
      {
        title: 'Profiles',
        description: '管理默认工作配置，快速切换不同模型与策略。',
        to: '/codex/profiles',
        icon: 'Folders',
        badge: `${data.profiles.total} 个 Profile`,
        tone: data.profiles.total > 0 ? 'success' : 'warning'
      },
      {
        title: 'CLI Settings',
        description: '补齐模型、审批、沙箱和推理相关设置。',
        to: '/codex/settings',
        icon: 'SlidersHorizontal',
        badge: data.config.model || '未设置模型',
        tone: data.config.model ? 'neutral' : 'warning'
      },
      {
        title: 'MCP 服务器',
        description: '维护工具接入能力，把工作流扩展到本地或远端环境。',
        to: '/codex/mcp',
        icon: 'Server',
        badge: `${data.inventory.mcp_servers_total} 个服务`,
        tone: data.inventory.mcp_servers_total > 0 ? 'neutral' : 'warning'
      },
      {
        title: 'Slash Commands',
        description: '查看命令能力入口；当前平台不支持时也能快速确认状态。',
        to: '/codex/slash-commands',
        icon: 'Command',
        badge: `${data.inventory.slash_commands_total} 条命令`,
        tone: data.inventory.slash_commands_total > 0 ? 'neutral' : 'neutral'
      }
    ]
  })

  return {
    summary,
    loading,
    error,
    versionLabel,
    versionStatus,
    currentAccountLabel,
    currentProfileLabel,
    usageTotalTokens,
    healthItems,
    nextActions,
    managementLinks,
    formatTokens,
    formatDateTime,
    refresh
  }
}
