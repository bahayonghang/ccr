import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { grokApi } from '@/api'
import { getCurrentEnvironment } from '@/api/runtime/environment'
import { getCliVersion } from '@/api/runtime/system'
import type { EnvironmentInfo } from '@/types/generated/environment/EnvironmentInfo'
import type {
  CliVersionEntry,
  GrokActivationDto,
  GrokAuthModeDto,
  GrokDashboardCommandResponse,
  GrokDashboardOverview,
} from '@/types'
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

type OverviewLoadResult =
  | { status: 'ok'; data: GrokDashboardOverview }
  | { status: 'unsupported_environment'; envType: string }

const OVERVIEW_TTL_MS = 30_000
const VERSION_TTL_MS = 60_000
const GROK_DOCS_URL = 'https://docs.x.ai/'

let sharedOverview: GrokDashboardOverview | null = null
let sharedVersionEntry: CliVersionEntry | null = null
let sharedOverviewLoadedAt = 0
let sharedVersionLoadedAt = 0
let sharedLocalEnvironmentId: string | null = null
let overviewInflight: Promise<OverviewLoadResult> | null = null
let versionInflight: Promise<CliVersionEntry> | null = null
let environmentInflight: Promise<EnvironmentInfo> | null = null

const nowMs = () => Date.now()
const isCacheFresh = (loadedAt: number, ttlMs: number) => (
  loadedAt > 0 && nowMs() - loadedAt < ttlMs
)

const toOverview = (
  response: Extract<GrokDashboardCommandResponse, { status: 'ok' }>,
): GrokDashboardOverview => ({
  activation: response.activation,
  activation_name: response.activation_name,
  current_profile: response.current_profile,
  auth_mode: response.auth_mode,
  profiles_total: response.profiles_total,
  profiles_enabled: response.profiles_enabled,
  config_exists: response.config_exists,
  config_path_display: response.config_path_display,
})

const loadEnvironment = (): Promise<EnvironmentInfo> => {
  if (environmentInflight) return environmentInflight

  environmentInflight = getCurrentEnvironment().finally(() => {
    environmentInflight = null
  })
  return environmentInflight
}

const loadOverview = (force = false): Promise<OverviewLoadResult> => {
  if (overviewInflight) return overviewInflight

  if (!force && sharedOverview && isCacheFresh(sharedOverviewLoadedAt, OVERVIEW_TTL_MS)) {
    return Promise.resolve({ status: 'ok', data: sharedOverview })
  }

  overviewInflight = grokApi.getGrokDashboardOverview()
    .then((response): OverviewLoadResult => {
      if (response.status === 'unsupported_environment') {
        return { status: response.status, envType: response.env_type }
      }

      const data = toOverview(response)
      sharedOverview = data
      sharedOverviewLoadedAt = nowMs()
      return { status: 'ok', data }
    })
    .finally(() => {
      overviewInflight = null
    })

  return overviewInflight
}

const loadVersion = (force = false): Promise<CliVersionEntry> => {
  if (versionInflight) return versionInflight

  if (!force && sharedVersionEntry && isCacheFresh(sharedVersionLoadedAt, VERSION_TTL_MS)) {
    return Promise.resolve(sharedVersionEntry)
  }

  versionInflight = getCliVersion({
    tool: 'grok',
    timeoutMs: 1_500,
    force,
  })
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

const resetSharedCaches = () => {
  sharedOverview = null
  sharedVersionEntry = null
  sharedOverviewLoadedAt = 0
  sharedVersionLoadedAt = 0
}

export function useGrokDashboard() {
  const { t } = useI18n()

  const overview = ref<GrokDashboardOverview | null>(sharedOverview)
  const environmentLoading = ref(false)
  const overviewLoading = ref(false)
  const versionLoading = ref(false)
  const loadError = ref<string | null>(null)
  const refreshError = ref<string | null>(null)
  const localOnly = ref(false)
  const localOnlyEnvType = ref<string | null>(null)
  const versionStatus = ref<GrokVersionStatus>('loading')
  const versionLabel = ref(t('grok.states.version.checking'))

  const applyVersionEntry = (entry?: CliVersionEntry | null) => {
    if (!entry) {
      versionStatus.value = 'error'
      versionLabel.value = t('grok.states.version.error')
      return
    }

    if (entry.status === 'timeout') {
      versionStatus.value = 'timeout'
      versionLabel.value = t('grok.states.version.timeout')
      return
    }

    if (entry.status === 'error') {
      versionStatus.value = 'error'
      versionLabel.value = t('grok.states.version.error')
      return
    }

    if (entry.status === 'not_installed' || !entry.installed) {
      versionStatus.value = 'not_installed'
      versionLabel.value = t('grok.states.version.notInstalled')
      return
    }

    if (entry.status !== 'ok') {
      versionStatus.value = 'error'
      versionLabel.value = t('grok.states.version.error')
      return
    }

    versionStatus.value = 'ok'
    versionLabel.value = entry.version
      ? entry.version.startsWith('v') ? entry.version : `v${entry.version}`
      : t('grok.states.version.installed')
  }

  const syncCachedState = () => {
    overview.value = sharedOverview
    if (sharedVersionEntry) applyVersionEntry(sharedVersionEntry)
  }

  const setDashboardError = (reason: unknown) => {
    const message = getErrorMessage(reason)
    if (overview.value || sharedOverview) {
      refreshError.value = message
    } else {
      loadError.value = message
    }
  }

  const refresh = async (force = false): Promise<void> => {
    loadError.value = null
    refreshError.value = null
    environmentLoading.value = true

    let environment: EnvironmentInfo
    try {
      environment = await loadEnvironment()
    } catch (error) {
      overview.value = null
      loadError.value = getErrorMessage(error)
      environmentLoading.value = false
      return
    }
    environmentLoading.value = false

    if (environment.env_type !== 'local') {
      localOnly.value = true
      localOnlyEnvType.value = environment.env_type
      overview.value = null
      return
    }

    localOnly.value = false
    localOnlyEnvType.value = null

    if (sharedLocalEnvironmentId && sharedLocalEnvironmentId !== environment.id) {
      resetSharedCaches()
    }
    sharedLocalEnvironmentId = environment.id
    syncCachedState()

    const needsOverview = force
      || !sharedOverview
      || !isCacheFresh(sharedOverviewLoadedAt, OVERVIEW_TTL_MS)

    if (needsOverview) {
      overviewLoading.value = true
      try {
        const result = await loadOverview(force)
        if (result.status === 'unsupported_environment') {
          resetSharedCaches()
          localOnly.value = true
          localOnlyEnvType.value = result.envType
          overview.value = null
          return
        }
        overview.value = result.data
      } catch (error) {
        setDashboardError(error)
      } finally {
        overviewLoading.value = false
      }
    }

    if (!overview.value) return

    const needsVersion = force
      || !sharedVersionEntry
      || !isCacheFresh(sharedVersionLoadedAt, VERSION_TTL_MS)

    if (needsVersion) {
      versionLoading.value = true
      try {
        applyVersionEntry(await loadVersion(force))
      } catch (error) {
        if (sharedVersionEntry) {
          applyVersionEntry(sharedVersionEntry)
          refreshError.value = getErrorMessage(error)
        } else {
          applyVersionEntry(undefined)
        }
      } finally {
        versionLoading.value = false
      }
    } else if (sharedVersionEntry) {
      applyVersionEntry(sharedVersionEntry)
    }
  }

  const loading = computed(() => (
    environmentLoading.value || overviewLoading.value || versionLoading.value
  ))

  const initialLoading = computed(() => (
    !localOnly.value
    && !overview.value
    && (environmentLoading.value || overviewLoading.value)
  ))

  const currentProfileLabel = computed(() => (
    overview.value?.current_profile
    || overview.value?.activation_name
    || t('grok.states.notSet')
  ))

  const activationLabel = computed(() => {
    const state = overview.value?.activation
    const keys: Record<GrokActivationDto, string> = {
      inactive: 'grok.states.activation.inactive',
      active: 'grok.states.activation.active',
      drifted: 'grok.states.activation.drifted',
      unsafe_missing_entry_state: 'grok.states.activation.unsafeMissingEntryState',
    }
    return state ? t(keys[state]) : t('grok.states.unknown')
  })

  const authModeLabel = computed(() => {
    const mode = overview.value?.auth_mode
    const keys: Record<GrokAuthModeDto, string> = {
      inline_api_key: 'grok.states.authMode.inlineApiKey',
      env_key: 'grok.states.authMode.envKey',
      session: 'grok.states.authMode.session',
    }
    return mode ? t(keys[mode]) : t('grok.states.notSet')
  })

  const activationWarning = computed<GrokActivationWarning | null>(() => {
    const data = overview.value
    if (!data) return null
    const name = data.activation_name || t('grok.states.unknown')

    if (data.activation === 'drifted') {
      return {
        label: t('grok.states.activationWarning.drifted', { name }),
        tone: 'warning',
        icon: 'AlertTriangle',
      }
    }

    if (data.activation === 'unsafe_missing_entry_state') {
      return {
        label: t('grok.states.activationWarning.unsafeMissingEntryState', { name }),
        tone: 'danger',
        icon: 'AlertCircle',
      }
    }

    return null
  })

  const versionTone = computed<GrokDashboardTone>(() => {
    if (versionStatus.value === 'ok') return 'success'
    if (versionStatus.value === 'timeout' || versionStatus.value === 'loading') return 'warning'
    return 'danger'
  })

  const readinessItems = computed<GrokReadinessItem[]>(() => {
    const data = overview.value
    if (!data) return []

    const installationDetail = versionStatus.value === 'loading'
      ? t('grok.dashboard.readiness.installation.checking')
      : versionStatus.value === 'ok'
        ? t('grok.dashboard.readiness.installation.installed')
        : versionStatus.value === 'not_installed'
          ? t('grok.dashboard.readiness.installation.notInstalled')
          : versionStatus.value === 'timeout'
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

    const statusLabel = (tone: GrokDashboardTone) => {
      if (tone === 'success') return t('grok.dashboard.statusLabels.ready')
      if (tone === 'danger') return t('grok.dashboard.statusLabels.blocked')
      return t('grok.dashboard.statusLabels.attention')
    }

    return [
      {
        key: 'installation',
        title: t('grok.dashboard.readiness.installation.title'),
        value: versionLabel.value,
        detail: installationDetail,
        statusLabel: versionStatus.value === 'loading'
          ? t('grok.dashboard.statusLabels.checking')
          : statusLabel(versionTone.value),
        tone: versionTone.value,
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
              current: currentProfileLabel.value,
            }),
        statusLabel: statusLabel(profileTone),
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
              state: activationLabel.value,
            })
          : t('grok.dashboard.readiness.config.noPath', { state: activationLabel.value }),
        statusLabel: statusLabel(configTone),
        tone: configTone,
        icon: 'SlidersHorizontal',
      },
    ]
  })

  const nextActions = computed<GrokActionItem[]>(() => {
    const data = overview.value
    if (!data) return []

    const actions: GrokActionItem[] = []

    if (versionStatus.value === 'not_installed') {
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
  })

  const primaryAction = computed<GrokActionItem>(() => (
    nextActions.value[0] ?? {
      key: 'open-settings',
      title: t('grok.dashboard.actions.openSettings.title'),
      description: t('grok.dashboard.actions.openSettings.description'),
      to: '/grok/settings',
      icon: 'SlidersHorizontal',
      tone: 'neutral',
    }
  ))

  const managementItems = computed<GrokManagementItem[]>(() => {
    const data = overview.value
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
  })

  syncCachedState()

  return {
    overview,
    loading,
    initialLoading,
    overviewLoading,
    versionLoading,
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
