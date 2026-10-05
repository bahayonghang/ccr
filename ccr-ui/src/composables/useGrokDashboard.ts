import { useCallback, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import type { EnvironmentInfo } from '@/types/generated/environment/EnvironmentInfo'
import type {
  CliVersionEntry,
  GrokActivationDto,
  GrokAuthModeDto,
} from '@/types'
import {
  GROK_OVERVIEW_STALE_TIME,
  GROK_VERSION_STALE_TIME,
  fetchGrokEnvironment,
  fetchGrokOverview,
  fetchGrokVersion,
  grokKeys,
  type GrokOverviewLoadResult,
} from '@/features/grok/queries'
import { getErrorMessage } from '@/utils/errorHandler'

export type GrokDashboardTone = 'success' | 'warning' | 'danger' | 'neutral'
export type GrokVersionStatus = 'loading' | 'ok' | 'timeout' | 'error' | 'not_installed'

export interface GrokReadinessItem {
  key: 'installation' | 'profiles' | 'config'
  title: string
  value: string
  detail: string
  statusLabel: string
  tone: GrokDashboardTone
  icon: string
}

export interface GrokActionItem {
  key: string
  title: string
  description: string
  to: string
  icon: string
  tone: GrokDashboardTone
  external?: boolean
}

export interface GrokManagementItem {
  key: 'profiles' | 'settings'
  title: string
  description: string
  badge: string
  to: string
  icon: string
  tone: GrokDashboardTone
}

interface GrokActivationWarning {
  label: string
  tone: 'warning' | 'danger'
  icon: string
}

// Grok 仪表盘的 React 迁移（08-22-state-logic-port 批次 5，服务端数据 → Query）。
// 原模块级 TTL 缓存 / in-flight 去重由 Query 承担：overview staleTime 30s、
// version 60s、environment 0（原每次 refresh 都拉取）。environment id 进入
// overview/version 的 key，换环境落到新缓存条目（等价原 resetSharedCaches）。
//
// 签名变化（消费方均为待迁移 .vue 视图）：
// - i18n 由调用方传入 t；返回对象中的 Ref<T> 改为普通值；
// - loadError/refreshError 由原命令式分支改为派生值：尚无 overview 数据时的错误
//   记入 loadError，已有数据时的错误记入 refreshError（与原 setDashboardError 判定一致）。

type Translate = (key: string, params?: Record<string, unknown>) => string

interface UseGrokDashboardOptions {
  /** i18n translation function. */
  t: Translate
}

const GROK_DOCS_URL = 'https://docs.x.ai/'

const ACTIVATION_LABEL_KEYS: Record<GrokActivationDto, string> = {
  inactive: 'grok.states.activation.inactive',
  active: 'grok.states.activation.active',
  drifted: 'grok.states.activation.drifted',
  unsafe_missing_entry_state: 'grok.states.activation.unsafeMissingEntryState',
}

const AUTH_MODE_LABEL_KEYS: Record<GrokAuthModeDto, string> = {
  inline_api_key: 'grok.states.authMode.inlineApiKey',
  env_key: 'grok.states.authMode.envKey',
  session: 'grok.states.authMode.session',
}

/** 原 applyVersionEntry 的纯函数化；entry 缺失时按是否在拉取区分 checking/error。 */
const deriveVersionState = (
  entry: CliVersionEntry | null | undefined,
  fetching: boolean,
  t: Translate,
): { versionStatus: GrokVersionStatus; versionLabel: string } => {
  if (!entry) {
    return fetching
      ? { versionStatus: 'loading', versionLabel: t('grok.states.version.checking') }
      : { versionStatus: 'error', versionLabel: t('grok.states.version.error') }
  }

  if (entry.status === 'timeout') {
    return { versionStatus: 'timeout', versionLabel: t('grok.states.version.timeout') }
  }

  if (entry.status === 'error') {
    return { versionStatus: 'error', versionLabel: t('grok.states.version.error') }
  }

  if (entry.status === 'not_installed' || !entry.installed) {
    return { versionStatus: 'not_installed', versionLabel: t('grok.states.version.notInstalled') }
  }

  if (entry.status !== 'ok') {
    return { versionStatus: 'error', versionLabel: t('grok.states.version.error') }
  }

  return {
    versionStatus: 'ok',
    versionLabel: entry.version
      ? entry.version.startsWith('v') ? entry.version : `v${entry.version}`
      : t('grok.states.version.installed'),
  }
}

export function useGrokDashboard({ t }: UseGrokDashboardOptions) {
  const environmentQuery = useQuery({
    queryKey: grokKeys.environment(),
    queryFn: fetchGrokEnvironment,
    staleTime: 0,
  })

  const environment = (environmentQuery.data ?? null) as EnvironmentInfo | null
  const environmentId = environment?.id ?? null

  const overviewQuery = useQuery({
    queryKey: grokKeys.overview(environmentId),
    queryFn: fetchGrokOverview,
    // 原 refresh 流程：仅 local 环境继续拉取 overview
    enabled: environmentQuery.isSuccess && environment?.env_type === 'local',
    staleTime: GROK_OVERVIEW_STALE_TIME,
  })

  const overviewResult = (overviewQuery.data ?? null) as GrokOverviewLoadResult | null
  const overview = overviewResult?.status === 'ok' ? overviewResult.data : null

  const versionQuery = useQuery({
    queryKey: grokKeys.version(environmentId),
    queryFn: fetchGrokVersion,
    // 原 refresh 流程：overview 就绪后才探测版本
    enabled: overview !== null,
    staleTime: GROK_VERSION_STALE_TIME,
  })

  // 原流程的 localOnly 判定：非 local 环境，或 overview 返回 unsupported_environment
  const localOnly = useMemo(() => {
    if (environment && environment.env_type !== 'local') return true
    return overviewResult?.status === 'unsupported_environment'
  }, [environment, overviewResult])

  const localOnlyEnvType = useMemo(() => {
    if (environment && environment.env_type !== 'local') return environment.env_type
    if (overviewResult?.status === 'unsupported_environment') return overviewResult.envType
    return null
  }, [environment, overviewResult])

  const loading = useMemo(
    () => environmentQuery.isFetching || overviewQuery.isFetching || versionQuery.isFetching,
    [environmentQuery.isFetching, overviewQuery.isFetching, versionQuery.isFetching]
  )

  const initialLoading = useMemo(
    () => !localOnly && !overview && (environmentQuery.isFetching || overviewQuery.isFetching),
    [environmentQuery.isFetching, localOnly, overview, overviewQuery.isFetching]
  )

  // 错误派生：environment 失败必为 loadError；overview/version 失败按是否有数据分流
  const loadError = useMemo(() => {
    if (environmentQuery.error) return getErrorMessage(environmentQuery.error)
    if (!overview) {
      if (overviewQuery.error) return getErrorMessage(overviewQuery.error)
      if (versionQuery.error) return getErrorMessage(versionQuery.error)
    }
    return null
  }, [environmentQuery.error, overview, overviewQuery.error, versionQuery.error])

  const refreshError = useMemo(() => {
    if (!overview) return null
    if (overviewQuery.error) return getErrorMessage(overviewQuery.error)
    // 版本失败但已有版本缓存：保留旧展示并记录刷新错误（原 catch 分支）
    if (versionQuery.error && versionQuery.data) return getErrorMessage(versionQuery.error)
    return null
  }, [overview, overviewQuery.error, versionQuery.data, versionQuery.error])

  const currentProfileLabel = useMemo(
    () => overview?.current_profile
      || overview?.activation_name
      || t('grok.states.notSet'),
    [overview, t]
  )

  const activationLabel = useMemo(
    () => (overview?.activation
      ? t(ACTIVATION_LABEL_KEYS[overview.activation])
      : t('grok.states.unknown')),
    [overview, t]
  )

  const authModeLabel = useMemo(
    () => (overview?.auth_mode
      ? t(AUTH_MODE_LABEL_KEYS[overview.auth_mode])
      : t('grok.states.notSet')),
    [overview, t]
  )

  const activationWarning = useMemo<GrokActivationWarning | null>(() => {
    if (!overview) return null
    const name = overview.activation_name || t('grok.states.unknown')

    if (overview.activation === 'drifted') {
      return {
        label: t('grok.states.activationWarning.drifted', { name }),
        tone: 'warning',
        icon: 'AlertTriangle',
      }
    }

    if (overview.activation === 'unsafe_missing_entry_state') {
      return {
        label: t('grok.states.activationWarning.unsafeMissingEntryState', { name }),
        tone: 'danger',
        icon: 'AlertCircle',
      }
    }

    return null
  }, [overview, t])

  const { versionStatus, versionLabel } = useMemo(
    () => deriveVersionState(versionQuery.data, versionQuery.isFetching, t),
    [t, versionQuery.data, versionQuery.isFetching]
  )

  const versionTone = useMemo<GrokDashboardTone>(() => {
    if (versionStatus === 'ok') return 'success'
    if (versionStatus === 'timeout' || versionStatus === 'loading') return 'warning'
    return 'danger'
  }, [versionStatus])

  const readinessItems = useMemo<GrokReadinessItem[]>(() => {
    const data = overview
    if (!data) return []

    const installationDetail = versionStatus === 'loading'
      ? t('grok.dashboard.readiness.installation.checking')
      : versionStatus === 'ok'
        ? t('grok.dashboard.readiness.installation.installed')
        : versionStatus === 'not_installed'
          ? t('grok.dashboard.readiness.installation.notInstalled')
          : versionStatus === 'timeout'
            ? t('grok.dashboard.readiness.installation.timeout')
            : t('grok.dashboard.readiness.installation.error')

    const profileTone: GrokDashboardTone = data.activation === 'drifted'
      || data.activation === 'unsafe_missing_entry_state'
      ? 'danger'
      : data.profiles_total === 0 || data.activation !== 'active'
        ? 'warning'
        : 'success'

    const configTone: GrokDashboardTone = data.activation === 'drifted'
      || data.activation === 'unsafe_missing_entry_state'
      ? 'danger'
      : data.config_exists
        ? 'success'
        : 'warning'

    const statusLabelFor = (tone: GrokDashboardTone) => {
      if (tone === 'success') return t('grok.dashboard.statusLabels.ready')
      if (tone === 'danger') return t('grok.dashboard.statusLabels.blocked')
      return t('grok.dashboard.statusLabels.attention')
    }

    return [
      {
        key: 'installation',
        title: t('grok.dashboard.readiness.installation.title'),
        value: versionLabel,
        detail: installationDetail,
        statusLabel: versionStatus === 'loading'
          ? t('grok.dashboard.statusLabels.checking')
          : statusLabelFor(versionTone),
        tone: versionTone,
        icon: 'Package',
      },
      {
        key: 'profiles',
        title: t('grok.dashboard.readiness.profiles.title'),
        value: t('grok.dashboard.readiness.profiles.value', { count: data.profiles_total }),
        detail: data.profiles_total === 0
          ? t('grok.dashboard.readiness.profiles.empty')
          : t('grok.dashboard.readiness.profiles.detail', {
              enabled: data.profiles_enabled,
              current: currentProfileLabel,
            }),
        statusLabel: statusLabelFor(profileTone),
        tone: profileTone,
        icon: 'Folders',
      },
      {
        key: 'config',
        title: t('grok.dashboard.readiness.config.title'),
        value: data.config_exists
          ? t('grok.dashboard.readiness.config.exists')
          : t('grok.dashboard.readiness.config.missing'),
        detail: data.config_path_display
          ? t('grok.dashboard.readiness.config.path', {
              path: data.config_path_display,
              state: activationLabel,
            })
          : t('grok.dashboard.readiness.config.noPath', { state: activationLabel }),
        statusLabel: statusLabelFor(configTone),
        tone: configTone,
        icon: 'SlidersHorizontal',
      },
    ]
  }, [activationLabel, currentProfileLabel, overview, t, versionLabel, versionStatus, versionTone])

  // actions 为 memo 内本地累积数组（mutation-rewrite.md 判定：本地临时，无需改写）
  const nextActions = useMemo<GrokActionItem[]>(() => {
    const data = overview
    if (!data) return []

    const actions: GrokActionItem[] = []

    if (versionStatus === 'not_installed') {
      actions.push({
        key: 'install',
        title: t('grok.dashboard.actions.install.title'),
        description: t('grok.dashboard.actions.install.description'),
        to: GROK_DOCS_URL,
        icon: 'ExternalLink',
        tone: 'danger',
        external: true,
      })
    }

    if (data.profiles_total === 0) {
      actions.push({
        key: 'create-profile',
        title: t('grok.dashboard.actions.createProfile.title'),
        description: t('grok.dashboard.actions.createProfile.description'),
        to: '/grok/profiles',
        icon: 'Plus',
        tone: 'warning',
      })
    } else if (data.activation === 'drifted') {
      actions.push({
        key: 'repair-drift',
        title: t('grok.dashboard.actions.repairDrift.title'),
        description: t('grok.dashboard.actions.repairDrift.description'),
        to: '/grok/profiles',
        icon: 'AlertTriangle',
        tone: 'danger',
      })
    } else if (data.activation === 'unsafe_missing_entry_state') {
      actions.push({
        key: 'inspect-unsafe',
        title: t('grok.dashboard.actions.inspectUnsafe.title'),
        description: t('grok.dashboard.actions.inspectUnsafe.description'),
        to: '/grok/profiles',
        icon: 'AlertCircle',
        tone: 'danger',
      })
    } else if (data.activation !== 'active') {
      actions.push({
        key: 'activate-profile',
        title: t('grok.dashboard.actions.activateProfile.title'),
        description: t('grok.dashboard.actions.activateProfile.description'),
        to: '/grok/profiles',
        icon: 'ArrowRightLeft',
        tone: 'warning',
      })
    }

    if (actions.length === 0) {
      actions.push({
        key: 'open-settings',
        title: t('grok.dashboard.actions.openSettings.title'),
        description: t('grok.dashboard.actions.openSettings.description'),
        to: '/grok/settings',
        icon: 'SlidersHorizontal',
        tone: 'success',
      })
    }

    return actions.slice(0, 3)
  }, [overview, t, versionStatus])

  const primaryAction = useMemo<GrokActionItem>(() => (
    nextActions[0] ?? {
      key: 'open-settings',
      title: t('grok.dashboard.actions.openSettings.title'),
      description: t('grok.dashboard.actions.openSettings.description'),
      to: '/grok/settings',
      icon: 'SlidersHorizontal',
      tone: 'neutral',
    }
  ), [nextActions, t])

  const managementItems = useMemo<GrokManagementItem[]>(() => {
    const data = overview
    if (!data) return []

    return [
      {
        key: 'profiles',
        title: t('grok.dashboard.management.profiles.title'),
        description: t('grok.dashboard.management.profiles.description'),
        badge: t('grok.dashboard.management.profiles.badge', { count: data.profiles_total }),
        to: '/grok/profiles',
        icon: 'Folders',
        tone: data.profiles_total > 0 ? 'success' : 'warning',
      },
      {
        key: 'settings',
        title: t('grok.dashboard.management.settings.title'),
        description: t('grok.dashboard.management.settings.description'),
        badge: data.config_exists
          ? t('grok.dashboard.management.settings.configured')
          : t('grok.dashboard.management.settings.missing'),
        to: '/grok/settings',
        icon: 'SlidersHorizontal',
        tone: data.config_exists ? 'success' : 'warning',
      },
    ]
  }, [overview, t])

  /** 原 refresh(force)：重拉环境 → 按需重拉 overview / version。enabled 门控保持
   * 「非 local 不拉 overview、无 overview 不拉 version」的原流程；force 越过 TTL。 */
  const refresh = useCallback(async (force = false): Promise<void> => {
    await environmentQuery.refetch()
    if (environment?.env_type !== 'local') return

    const tasks: Array<Promise<unknown>> = []
    if (force || overviewQuery.isStale) tasks.push(overviewQuery.refetch())
    if (overview && (force || versionQuery.isStale)) tasks.push(versionQuery.refetch())
    await Promise.allSettled(tasks)
  }, [environment, environmentQuery, overview, overviewQuery, versionQuery])

  return {
    overview,
    loading,
    initialLoading,
    overviewLoading: overviewQuery.isFetching,
    versionLoading: versionQuery.isFetching,
    loadError,
    refreshError,
    localOnly,
    localOnlyEnvType,
    versionStatus,
    versionLabel,
    versionTone,
    currentProfileLabel,
    activationLabel,
    authModeLabel,
    activationWarning,
    readinessItems,
    nextActions,
    primaryAction,
    managementItems,
    refresh,
  }
}
