import type { CliVersionEntry, GrokActivationDto, GrokAuthModeDto, GrokDashboardOverview } from '@/types'
import type { TranslateFunction } from '@/utils/tf'

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

export const GROK_DOCS_URL = 'https://docs.x.ai/'

export const GROK_HOME_COMMANDS = [
  'ccr grok profile list',
  'ccr grok profile switch <name>',
  'ccr grok profile off',
  'ccr grok profile init',
]

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

export const deriveVersionState = (
  entry: CliVersionEntry | null | undefined,
  fetching: boolean,
  t: TranslateFunction,
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
  const version = entry.version
    ? entry.version.startsWith('v') ? entry.version : `v${entry.version}`
    : t('grok.states.version.installed')
  return { versionStatus: 'ok', versionLabel: version }
}

export const activationLabelOf = (activation: GrokActivationDto | undefined, t: TranslateFunction) =>
  activation ? t(ACTIVATION_LABEL_KEYS[activation]) : t('grok.states.unknown')

export const authModeLabelOf = (mode: GrokAuthModeDto | undefined, t: TranslateFunction) =>
  mode ? t(AUTH_MODE_LABEL_KEYS[mode]) : t('grok.states.notSet')

const statusLabelFor = (tone: GrokDashboardTone, t: TranslateFunction) => {
  if (tone === 'success') return t('grok.dashboard.statusLabels.ready')
  if (tone === 'danger') return t('grok.dashboard.statusLabels.blocked')
  return t('grok.dashboard.statusLabels.attention')
}

export const buildReadinessItems = (input: {
  data: GrokDashboardOverview
  version: { versionStatus: GrokVersionStatus; versionLabel: string; versionTone: GrokDashboardTone }
  labels: { currentProfile: string; activation: string }
  t: TranslateFunction
}): GrokReadinessItem[] => {
  const { data, version, labels, t } = input
  const profileTone: GrokDashboardTone =
    data.activation === 'drifted' || data.activation === 'unsafe_missing_entry_state'
      ? 'danger'
      : data.profiles_total === 0 || data.activation !== 'active'
        ? 'warning'
        : 'success'
  const configTone: GrokDashboardTone =
    data.activation === 'drifted' || data.activation === 'unsafe_missing_entry_state'
      ? 'danger'
      : data.config_exists
        ? 'success'
        : 'warning'
  const installationDetail =
    version.versionStatus === 'loading'
      ? t('grok.dashboard.readiness.installation.checking')
      : version.versionStatus === 'ok'
        ? t('grok.dashboard.readiness.installation.installed')
        : version.versionStatus === 'not_installed'
          ? t('grok.dashboard.readiness.installation.notInstalled')
          : version.versionStatus === 'timeout'
            ? t('grok.dashboard.readiness.installation.timeout')
            : t('grok.dashboard.readiness.installation.error')

  return [
    {
      key: 'installation',
      title: t('grok.dashboard.readiness.installation.title'),
      value: version.versionLabel,
      detail: installationDetail,
      statusLabel:
        version.versionStatus === 'loading'
          ? t('grok.dashboard.statusLabels.checking')
          : statusLabelFor(version.versionTone, t),
      tone: version.versionTone,
      icon: 'Package',
    },
    {
      key: 'profiles',
      title: t('grok.dashboard.readiness.profiles.title'),
      value: t('grok.dashboard.readiness.profiles.value', { count: data.profiles_total }),
      detail:
        data.profiles_total === 0
          ? t('grok.dashboard.readiness.profiles.empty')
          : t('grok.dashboard.readiness.profiles.detail', {
              enabled: data.profiles_enabled,
              current: labels.currentProfile,
            }),
      statusLabel: statusLabelFor(profileTone, t),
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
            state: labels.activation,
          })
        : t('grok.dashboard.readiness.config.noPath', { state: labels.activation }),
      statusLabel: statusLabelFor(configTone, t),
      tone: configTone,
      icon: 'SlidersHorizontal',
    },
  ]
}

export const buildNextActions = (
  data: GrokDashboardOverview,
  versionStatus: GrokVersionStatus,
  t: TranslateFunction,
): GrokActionItem[] => {
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
}

export const buildManagementItems = (
  data: GrokDashboardOverview,
  t: TranslateFunction,
): GrokManagementItem[] => [
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

export const activationWarningOf = (
  overview: GrokDashboardOverview | null,
  t: TranslateFunction,
): { label: string; tone: 'warning' | 'danger' } | null => {
  if (!overview) return null
  const name = overview.activation_name || t('grok.states.unknown')
  if (overview.activation === 'drifted') {
    return { label: t('grok.states.activationWarning.drifted', { name }), tone: 'warning' }
  }
  if (overview.activation === 'unsafe_missing_entry_state') {
    return { label: t('grok.states.activationWarning.unsafeMissingEntryState', { name }), tone: 'danger' }
  }
  return null
}

export const versionToneOf = (status: GrokVersionStatus): GrokDashboardTone => {
  if (status === 'ok') return 'success'
  if (status === 'timeout' || status === 'loading') return 'warning'
  return 'danger'
}

export const localOnlyState = (input: {
  envType?: string
  overview?: { status: string; envType?: string } | null
}): { localOnly: boolean; envType: string | null } => {
  if (input.envType && input.envType !== 'local') return { localOnly: true, envType: input.envType }
  if (input.overview?.status === 'unsupported_environment') {
    return { localOnly: true, envType: input.overview.envType ?? null }
  }
  return { localOnly: false, envType: null }
}

export const currentProfileLabelOf = (
  overview: GrokDashboardOverview | null,
  t: TranslateFunction,
): string => overview?.current_profile || overview?.activation_name || t('grok.states.notSet')

export const queryErrorMessage = (input: {
  envError: unknown
  overviewError: unknown
  hasOverview: boolean
  format: (error: unknown) => string
}): { loadError: string | null; refreshError: string | null } => {
  if (input.envError) return { loadError: input.format(input.envError), refreshError: null }
  if (!input.hasOverview && input.overviewError) {
    return { loadError: input.format(input.overviewError), refreshError: null }
  }
  if (input.hasOverview && input.overviewError) {
    return { loadError: null, refreshError: input.format(input.overviewError) }
  }
  return { loadError: null, refreshError: null }
}
