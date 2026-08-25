import type { IconName } from '@/config/icons'
import type { CliVersionEntry, SystemInfo } from '@/types'
import type {
  HomeOverviewPlatformStats,
  HomeOverviewSeriesItem,
  HomeUsageOverviewResponse,
  UsageSourceHealthState,
} from '@/types/usage'

/** 看板信号条目：只消费 channel / level，避免展示层依赖 monitoring composable。 */
export interface DashboardLogEntry {
  channel: string
  level: string
}

export type DashboardTone = 'neutral' | 'success' | 'warning' | 'danger' | 'accent'
export type DashboardActionTone =
  | 'command'
  | 'config'
  | 'sync'
  | 'usage'
  | 'monitoring'
  | 'platform'
  | 'system'

export type DashboardReadinessStatus =
  | 'ready'
  | 'attention'
  | 'warming'
  | 'web-preview'
  | 'unknown'

export type DashboardBackendStatus = 'unsupported' | 'unknown' | 'checking' | 'ok' | 'error'
export type DashboardPlatformMode = 'cli' | 'managed'
export type DashboardPlatformState = 'ready' | 'scanning' | 'attention' | 'managed'
export type DashboardUsageMetric = 'sessions' | 'requests' | 'tokens'

export interface DashboardPlatformSource {
  title: string
  desc: string
  path: string
  icon: IconName
  iconClass: string
  platformKey: string
  usageKey?: 'claude' | 'codex' | 'gemini' | 'opencode'
  role: string
  mode: DashboardPlatformMode
  isRuntimeCli: boolean
}

export interface DashboardMetricValue {
  labelKey: string
  value?: string
  valueKey?: string
}

export type DashboardPlatformUsageKey = NonNullable<DashboardPlatformSource['usageKey']>
export type DashboardTrackingHealth = UsageSourceHealthState

export interface DashboardPlatformRow extends DashboardPlatformSource {
  state: DashboardPlatformState
  stateKey: string
  version?: string
  versionKey?: string
  metrics: DashboardMetricValue[]
  sparkline?: number[]
  trackingHealth?: DashboardTrackingHealth
}

export interface DashboardStatusMetric {
  id: string
  labelKey: string
  value?: string
  valueKey?: string
  hint?: string
  hintKey?: string
  tone: DashboardTone
}

export interface DashboardReadinessReason {
  key: string
  ok: boolean
}

export interface DashboardReadiness {
  status: DashboardReadinessStatus
  tone: DashboardTone
  labelKey: string
  titleKey: string
  descriptionKey: string
  reasons: DashboardReadinessReason[]
}

export interface DashboardAction {
  id: string
  titleKey: string
  descKey: string
  path: string
  icon: IconName
  tone: DashboardActionTone
  priority: number
  detail?: string
}

export interface DashboardSignalCounts {
  total: number
  warnings: number
  errors: number
}

export interface DashboardPresentationInput {
  backendStatus: DashboardBackendStatus
  isNativeRuntime: boolean
  systemInfo: SystemInfo | null
  cliVersions: Map<string, CliVersionEntry>
  cliVersionsLoaded: boolean
  platforms: DashboardPlatformSource[]
  overview: HomeUsageOverviewResponse | null
  usageLoading: boolean
  usageError: string | null
  logs: DashboardLogEntry[]
}

export interface DashboardPresentation {
  readiness: DashboardReadiness
  actions: DashboardAction[]
  statusMetrics: DashboardStatusMetric[]
  platformRows: DashboardPlatformRow[]
  signalCounts: DashboardSignalCounts
  installedCliCount: number
  runtimeCliCount: number
  /** 桌面运行时下 CLI 探测已完成但一个都未安装：视为首次使用，行动队列改渲染引导态 */
  isFirstRun: boolean
}

const DASHBOARD_DEFAULT_ACTIONS: DashboardAction[] = [
  {
    id: 'command-runner',
    titleKey: 'dashboard.actions.commandRunnerTitle',
    descKey: 'dashboard.actions.commandRunnerDesc',
    path: '/commands',
    icon: 'Terminal',
    tone: 'command',
    priority: 80,
  },
  {
    id: 'config-manager',
    titleKey: 'dashboard.actions.configManagerTitle',
    descKey: 'dashboard.actions.configManagerDesc',
    path: '/configs',
    icon: 'Settings',
    tone: 'config',
    priority: 90,
  },
  {
    id: 'cloud-sync',
    titleKey: 'dashboard.actions.cloudSyncTitle',
    descKey: 'dashboard.actions.cloudSyncDesc',
    path: '/sync',
    icon: 'Cloud',
    tone: 'sync',
    priority: 100,
  },
  {
    id: 'usage-stats',
    titleKey: 'dashboard.actions.usageStatsTitle',
    descKey: 'dashboard.actions.usageStatsDesc',
    path: '/usage',
    icon: 'Activity',
    tone: 'usage',
    priority: 110,
  },
]

const formatCompact = (value?: number) => {
  if (typeof value !== 'number' || Number.isNaN(value)) return '…'
  return new Intl.NumberFormat(undefined, {
    notation: 'compact',
    maximumFractionDigits: 1,
  }).format(value)
}

const formatPercent = (value?: number) => {
  if (typeof value !== 'number' || Number.isNaN(value)) return '…'
  return `${value.toFixed(1)}%`
}

const formatFixed = (value?: number) => {
  if (typeof value !== 'number' || Number.isNaN(value)) return '…'
  return value.toFixed(1)
}

const isRuntimeCliInstalled = (
  platform: DashboardPlatformSource,
  cliVersions: Map<string, CliVersionEntry>,
  cliVersionsLoaded: boolean,
) => {
  if (!platform.isRuntimeCli) return false
  const entry = cliVersions.get(platform.platformKey)
  if (!entry && !cliVersionsLoaded) return false

  return Boolean(entry?.installed && entry.status !== 'error' && entry.status !== 'timeout' && entry.status !== 'not_installed')
}

const getPlatformState = (
  platform: DashboardPlatformSource,
  cliVersions: Map<string, CliVersionEntry>,
  cliVersionsLoaded: boolean,
): DashboardPlatformState => {
  if (platform.mode === 'managed') return 'managed'

  const entry = cliVersions.get(platform.platformKey)
  if (!entry) return cliVersionsLoaded ? 'attention' : 'scanning'
  if (entry.status === 'timeout') return 'scanning'
  if (entry.status === 'error' || entry.status === 'not_installed' || !entry.installed) return 'attention'
  return 'ready'
}

const getPlatformStateKey = (state: DashboardPlatformState) => {
  switch (state) {
    case 'managed':
      return 'dashboard.platforms.stateManaged'
    case 'scanning':
      return 'dashboard.platforms.stateScanning'
    case 'attention':
      return 'dashboard.platforms.stateAttention'
    default:
      return 'dashboard.platforms.stateReady'
  }
}

const getVersionValue = (
  platform: DashboardPlatformSource,
  cliVersions: Map<string, CliVersionEntry>,
): Pick<DashboardPlatformRow, 'version' | 'versionKey'> => {
  if (platform.mode === 'managed') {
    return { versionKey: 'dashboard.platforms.managedLabel' }
  }

  const entry = cliVersions.get(platform.platformKey)
  if (!entry || entry.status === 'timeout' || entry.status === 'error') {
    return { versionKey: 'dashboard.platforms.stateScanning' }
  }
  if (entry.status === 'not_installed' || !entry.installed) {
    return { versionKey: 'dashboard.platforms.notInstalled' }
  }
  return { version: entry.version ? `v${entry.version}` : undefined, versionKey: entry.version ? undefined : 'common.installed' }
}

const getPlatformMetric = (
  stats: HomeOverviewPlatformStats | undefined,
  metric: DashboardUsageMetric,
): Pick<DashboardMetricValue, 'value' | 'valueKey'> => {
  if (!stats) return { valueKey: 'dashboard.platforms.untracked' }
  return { value: formatCompact(stats[metric]) }
}

const USAGE_KEY_TO_SERIES_FIELD = {
  claude: 'claude',
  codex: 'codex',
  gemini: 'antigravity',
  opencode: 'opencode',
} as const satisfies Record<DashboardPlatformUsageKey, keyof Omit<HomeOverviewSeriesItem, 'date'>>

const USAGE_KEY_TO_SOURCE_ID: Record<DashboardPlatformUsageKey, string> = {
  claude: 'claude',
  codex: 'codex',
  gemini: 'antigravity',
  opencode: 'opencode',
}

const isPlatformUsageKey = (value: string | undefined): value is DashboardPlatformUsageKey =>
  value === 'claude' || value === 'codex' || value === 'gemini' || value === 'opencode'

const getPlatformStats = (
  platform: DashboardPlatformSource,
  overview: HomeUsageOverviewResponse | null,
): HomeOverviewPlatformStats | undefined => {
  if (!platform.usageKey || !overview?.by_platform) return undefined
  const direct = overview.by_platform[platform.usageKey]
  if (direct) return direct
  if (!isPlatformUsageKey(platform.usageKey)) return undefined
  return overview.by_platform[USAGE_KEY_TO_SOURCE_ID[platform.usageKey]]
}

const buildSparkline = (
  usageKey: DashboardPlatformSource['usageKey'],
  series: HomeOverviewSeriesItem[] | undefined,
): number[] | undefined => {
  if (!usageKey || !isPlatformUsageKey(usageKey) || !series?.length) return undefined
  const field = USAGE_KEY_TO_SERIES_FIELD[usageKey]
  return series.map((item) => item[field].requests)
}

const resolveTrackingHealth = (
  usageKey: DashboardPlatformSource['usageKey'],
  overview: HomeUsageOverviewResponse | null,
): DashboardTrackingHealth | undefined => {
  if (!usageKey || !isPlatformUsageKey(usageKey)) return undefined
  const sourceHealth = overview?.archive.source_health
  if (!sourceHealth?.length) return undefined
  const sourceId = USAGE_KEY_TO_SOURCE_ID[usageKey]
  const hit = sourceHealth.find((entry) => entry.source === sourceId || entry.source === usageKey)
  return hit?.state
}

const buildPlatformRows = (input: DashboardPresentationInput): DashboardPlatformRow[] => {
  return input.platforms.map((platform) => {
    const state = getPlatformState(platform, input.cliVersions, input.cliVersionsLoaded)
    const trackingHealth = resolveTrackingHealth(platform.usageKey, input.overview)
    // missing 平台即便 series 被补成全零，也不能把 0 当真实用量展示。
    const stats = trackingHealth === 'missing' ? undefined : getPlatformStats(platform, input.overview)

    return {
      ...platform,
      state,
      stateKey: getPlatformStateKey(state),
      ...getVersionValue(platform, input.cliVersions),
      metrics: [
        {
          labelKey: 'dashboard.platforms.metrics.requests',
          ...getPlatformMetric(stats, 'requests'),
        },
        {
          labelKey: 'dashboard.platforms.metrics.sessions',
          ...getPlatformMetric(stats, 'sessions'),
        },
        {
          labelKey: 'dashboard.platforms.metrics.tokens',
          ...getPlatformMetric(stats, 'tokens'),
        },
      ],
      // missing 时 series 仍可能是全零；不把零数组写进 sparkline，避免下游误当真实用量。
      sparkline:
        trackingHealth === 'missing'
          ? undefined
          : buildSparkline(platform.usageKey, input.overview?.series),
      trackingHealth,
    }
  })
}

// 前端 UI 日志与 tracing 桥接的 runtime 诊断只归入事件流展示，
// 不参与阻塞叙事 / 红色 tile / 行动队列的驱动。
const DIAGNOSTIC_CHANNELS = new Set(['frontend', 'runtime'])
const isCoreSignal = (entry: DashboardLogEntry) => !DIAGNOSTIC_CHANNELS.has(entry.channel)

const countSignals = (logs: DashboardLogEntry[]): DashboardSignalCounts => {
  const coreLogs = logs.filter(isCoreSignal)
  const errors = coreLogs.filter((entry) => entry.level === 'error').length
  const warnings = coreLogs.filter((entry) => entry.level === 'warn').length

  return {
    total: coreLogs.length,
    warnings,
    errors,
  }
}

const getUsageReasonKey = (input: DashboardPresentationInput) => {
  if (input.usageError) return 'dashboard.readiness.reasons.usageError'
  if (input.usageLoading) return 'dashboard.readiness.reasons.usageLoading'
  if (!input.overview) return 'dashboard.readiness.reasons.usageLoading'
  if (input.overview.empty_reason) return 'dashboard.readiness.reasons.usageEmpty'
  if (input.overview.bootstrap?.needs_session_index || input.overview.bootstrap?.needs_usage_import || !input.overview.bootstrap?.is_warm) {
    return 'dashboard.readiness.reasons.usageWarmup'
  }
  return 'dashboard.readiness.reasons.usageReady'
}

const buildReadiness = (
  input: DashboardPresentationInput,
  platformRows: DashboardPlatformRow[],
  signalCounts: DashboardSignalCounts,
): DashboardReadiness => {
  const runtimeRows = platformRows.filter((platform) => platform.isRuntimeCli)
  const missingRuntimeRows = runtimeRows.filter((platform) => platform.state === 'attention')
  const scanningRuntimeRows = runtimeRows.filter((platform) => platform.state === 'scanning')
  const usageReasonKey = getUsageReasonKey(input)
  const reasons: DashboardReadinessReason[] = [
    input.backendStatus === 'unsupported'
      ? { key: 'dashboard.readiness.reasons.backendUnsupported', ok: false }
      : input.backendStatus === 'error'
        ? { key: 'dashboard.readiness.reasons.backendError', ok: false }
        : input.backendStatus === 'ok'
          ? { key: 'dashboard.readiness.reasons.backendOk', ok: true }
          : { key: 'dashboard.readiness.reasons.backendChecking', ok: false },
    missingRuntimeRows.length > 0
      ? { key: 'dashboard.readiness.reasons.cliMissing', ok: false }
      : scanningRuntimeRows.length > 0
        ? { key: 'dashboard.readiness.reasons.cliScanning', ok: false }
        : { key: 'dashboard.readiness.reasons.allCliReady', ok: true },
    { key: usageReasonKey, ok: usageReasonKey === 'dashboard.readiness.reasons.usageReady' },
    signalCounts.errors > 0
      ? { key: 'dashboard.readiness.reasons.signalsError', ok: false }
      : signalCounts.warnings > 0
        ? { key: 'dashboard.readiness.reasons.signalsWarn', ok: false }
        : { key: 'dashboard.readiness.reasons.signalsQuiet', ok: true },
  ]

  if (!input.isNativeRuntime || input.backendStatus === 'unsupported') {
    return {
      status: 'web-preview',
      tone: 'accent',
      labelKey: 'dashboard.readiness.webPreviewLabel',
      titleKey: 'dashboard.readiness.webPreviewTitle',
      descriptionKey: 'dashboard.readiness.webPreviewDescription',
      reasons,
    }
  }

  if (input.backendStatus === 'error'
    || input.usageError
    || signalCounts.errors > 0
    || missingRuntimeRows.length > 0
    || input.overview?.empty_reason
    || input.overview?.bootstrap?.needs_session_index
    || input.overview?.bootstrap?.needs_usage_import
  ) {
    return {
      status: 'attention',
      tone: 'warning',
      labelKey: 'dashboard.readiness.attentionLabel',
      titleKey: 'dashboard.readiness.attentionTitle',
      descriptionKey: 'dashboard.readiness.attentionDescription',
      reasons,
    }
  }

  if (input.backendStatus === 'checking'
    || input.backendStatus === 'unknown'
    || scanningRuntimeRows.length > 0
    || input.usageLoading
    || !input.overview
    || input.overview.bootstrap?.is_warm === false
  ) {
    return {
      status: 'warming',
      tone: 'neutral',
      labelKey: 'dashboard.readiness.warmingLabel',
      titleKey: 'dashboard.readiness.warmingTitle',
      descriptionKey: 'dashboard.readiness.warmingDescription',
      reasons,
    }
  }

  if (signalCounts.warnings > 0) {
    return {
      status: 'attention',
      tone: 'warning',
      labelKey: 'dashboard.readiness.attentionLabel',
      titleKey: 'dashboard.readiness.attentionTitle',
      descriptionKey: 'dashboard.readiness.attentionDescription',
      reasons,
    }
  }

  return {
    status: 'ready',
    tone: 'success',
    labelKey: 'dashboard.readiness.readyLabel',
    titleKey: 'dashboard.readiness.readyTitle',
    descriptionKey: 'dashboard.readiness.readyDescription',
    reasons,
  }
}

const addUniqueAction = (actions: DashboardAction[], action: DashboardAction) => {
  if (actions.some((candidate) => candidate.id === action.id)) return
  actions.push(action)
}

const buildActions = (
  input: DashboardPresentationInput,
  platformRows: DashboardPlatformRow[],
  signalCounts: DashboardSignalCounts,
): DashboardAction[] => {
  const actions: DashboardAction[] = []
  const missingCliRows = platformRows.filter((platform) => platform.isRuntimeCli && platform.state === 'attention')

  if (!input.isNativeRuntime || input.backendStatus === 'unsupported') {
    addUniqueAction(actions, {
      id: 'web-preview-boundary',
      titleKey: 'dashboard.actions.webPreviewTitle',
      descKey: 'dashboard.actions.webPreviewDesc',
      path: '/usage',
      icon: 'Monitor',
      tone: 'system',
      priority: 10,
    })
  }

  if (input.backendStatus === 'error') {
    addUniqueAction(actions, {
      id: 'backend-health',
      titleKey: 'dashboard.actions.backendTitle',
      descKey: 'dashboard.actions.backendDesc',
      path: '/monitoring',
      icon: 'AlertTriangle',
      tone: 'monitoring',
      priority: 20,
    })
  }

  if (missingCliRows.length > 0) {
    addUniqueAction(actions, {
      id: 'install-runtime-cli',
      titleKey: 'dashboard.actions.installCliTitle',
      descKey: 'dashboard.actions.installCliDesc',
      path: missingCliRows[0]?.path ?? '/configs',
      icon: missingCliRows[0]?.icon ?? 'Terminal',
      tone: 'platform',
      priority: 30,
      detail: missingCliRows.map((platform) => platform.title).join(' · '),
    })
  }

  if (input.usageError
    || input.usageLoading
    || !input.overview
    || input.overview.empty_reason
    || input.overview.bootstrap?.needs_session_index
    || input.overview.bootstrap?.needs_usage_import
  ) {
    addUniqueAction(actions, {
      id: 'open-usage',
      titleKey: 'dashboard.actions.openUsageTitle',
      descKey: 'dashboard.actions.openUsageDesc',
      path: '/usage',
      icon: 'Activity',
      tone: 'usage',
      priority: 40,
    })
  }

  if (signalCounts.errors > 0 || signalCounts.warnings > 0) {
    addUniqueAction(actions, {
      id: 'open-monitoring',
      titleKey: 'dashboard.actions.openMonitoringTitle',
      descKey: 'dashboard.actions.openMonitoringDesc',
      path: '/monitoring',
      icon: 'AlertCircle',
      tone: 'monitoring',
      priority: 50,
      detail: `${signalCounts.errors} / ${signalCounts.warnings}`,
    })
  }

  for (const action of DASHBOARD_DEFAULT_ACTIONS) {
    addUniqueAction(actions, action)
  }

  return actions
    .sort((left, right) => left.priority - right.priority)
    .slice(0, 4)
}

const getBackendMetric = (status: DashboardBackendStatus): DashboardStatusMetric => {
  switch (status) {
    case 'unsupported':
      return {
        id: 'backend',
        labelKey: 'dashboard.metrics.backend',
        valueKey: 'dashboard.metrics.backendUnsupported',
        hintKey: 'dashboard.readiness.reasons.backendUnsupported',
        tone: 'accent',
      }
    case 'checking':
      return {
        id: 'backend',
        labelKey: 'dashboard.metrics.backend',
        valueKey: 'dashboard.metrics.backendChecking',
        hintKey: 'dashboard.readiness.reasons.backendChecking',
        tone: 'neutral',
      }
    case 'error':
      return {
        id: 'backend',
        labelKey: 'dashboard.metrics.backend',
        valueKey: 'dashboard.metrics.backendError',
        hintKey: 'dashboard.readiness.reasons.backendError',
        tone: 'danger',
      }
    case 'ok':
      return {
        id: 'backend',
        labelKey: 'dashboard.metrics.backend',
        valueKey: 'dashboard.metrics.backendReady',
        hintKey: 'dashboard.readiness.reasons.backendOk',
        tone: 'success',
      }
    default:
      return {
        id: 'backend',
        labelKey: 'dashboard.metrics.backend',
        valueKey: 'dashboard.metrics.backendUnknown',
        hintKey: 'dashboard.readiness.reasons.backendChecking',
        tone: 'neutral',
      }
  }
}

const buildStatusMetrics = (
  input: DashboardPresentationInput,
  signalCounts: DashboardSignalCounts,
  installedCliCount: number,
  runtimeCliCount: number,
  missingCliTitles: string[],
): DashboardStatusMetric[] => {
  const systemMetric: DashboardStatusMetric = input.systemInfo
    ? {
        id: 'system',
        labelKey: 'dashboard.metrics.system',
        value: `${formatPercent(input.systemInfo.cpu_usage)} / ${formatPercent(input.systemInfo.memory_usage_percent)}`,
        hint: `${input.systemInfo.hostname || 'host'} · ${formatFixed(input.systemInfo.used_memory_gb)} / ${formatFixed(input.systemInfo.total_memory_gb)} GB`,
        tone: input.systemInfo.cpu_usage >= 90 || input.systemInfo.memory_usage_percent >= 92
          ? 'danger'
          : input.systemInfo.cpu_usage >= 70 || input.systemInfo.memory_usage_percent >= 78
            ? 'warning'
            : 'neutral',
      }
    : {
        id: 'system',
        labelKey: 'dashboard.metrics.system',
        value: '…',
        hintKey: 'dashboard.metrics.systemPending',
        tone: 'neutral',
      }

  const usageMetric: DashboardStatusMetric = input.usageError
    ? {
        id: 'usage',
        labelKey: 'dashboard.metrics.usage',
        valueKey: 'dashboard.metrics.usageUnavailable',
        hint: input.usageError,
        tone: 'warning',
      }
    : input.usageLoading || !input.overview
      ? {
          id: 'usage',
          labelKey: 'dashboard.metrics.usage',
          valueKey: 'dashboard.metrics.usagePreparing',
          hintKey: 'dashboard.readiness.reasons.usageLoading',
          tone: 'neutral',
        }
      : input.overview.empty_reason
        ? {
            id: 'usage',
            labelKey: 'dashboard.metrics.usage',
            valueKey: 'dashboard.metrics.usageMissing',
            hintKey: 'dashboard.readiness.reasons.usageEmpty',
            tone: 'warning',
          }
        : {
            id: 'usage',
            labelKey: 'dashboard.metrics.usage',
            value: formatCompact(input.overview.summary?.total_requests),
            hintKey: 'dashboard.readiness.reasons.usageReady',
            tone: 'accent',
          }

  const signalMetric: DashboardStatusMetric = signalCounts.errors > 0
    ? {
        id: 'signals',
        labelKey: 'dashboard.metrics.signals',
        value: `${signalCounts.errors}/${signalCounts.warnings}`,
        hintKey: 'dashboard.metrics.signalsError',
        tone: 'danger',
      }
    : signalCounts.warnings > 0
      ? {
          id: 'signals',
          labelKey: 'dashboard.metrics.signals',
          value: `${signalCounts.errors}/${signalCounts.warnings}`,
          hintKey: 'dashboard.metrics.signalsWarn',
          tone: 'warning',
        }
      : {
          id: 'signals',
          labelKey: 'dashboard.metrics.signals',
          value: String(signalCounts.total),
          hintKey: 'dashboard.metrics.signalsQuiet',
          tone: 'neutral',
        }

  return [
    systemMetric,
    getBackendMetric(input.backendStatus),
    {
      id: 'cli',
      labelKey: 'dashboard.metrics.cli',
      value: `${installedCliCount}/${runtimeCliCount}`,
      hint: missingCliTitles.length > 0 ? missingCliTitles.join(' · ') : undefined,
      hintKey: missingCliTitles.length > 0 ? undefined : 'dashboard.readiness.reasons.allCliReady',
      tone: runtimeCliCount === 0
        ? 'neutral'
        : installedCliCount === runtimeCliCount
          ? 'success'
          : 'warning',
    },
    usageMetric,
    signalMetric,
  ]
}

export const buildDashboardPresentation = (input: DashboardPresentationInput): DashboardPresentation => {
  const signalCounts = countSignals(input.logs)
  const platformRows = buildPlatformRows(input)
  const runtimeRows = input.platforms.filter((platform) => platform.isRuntimeCli)
  const runtimeCliCount = runtimeRows.length
  const installedCliCount = runtimeRows.filter((platform) => (
    isRuntimeCliInstalled(platform, input.cliVersions, input.cliVersionsLoaded)
  )).length
  const missingCliTitles = platformRows
    .filter((platform) => platform.isRuntimeCli && platform.state === 'attention')
    .map((platform) => platform.title)

  return {
    readiness: buildReadiness(input, platformRows, signalCounts),
    actions: buildActions(input, platformRows, signalCounts),
    statusMetrics: buildStatusMetrics(input, signalCounts, installedCliCount, runtimeCliCount, missingCliTitles),
    platformRows,
    signalCounts,
    installedCliCount,
    runtimeCliCount,
    // installedCliCount 只统计 CLI 档平台，纯 OpenCode（managed 档）用户会永远是 0；
    // 再叠加用量归档判断，避免把已有真实用量的用户误判成首次使用。
    isFirstRun: input.isNativeRuntime
      && input.cliVersionsLoaded
      && !input.usageLoading
      && installedCliCount === 0
      && (!input.overview || input.overview.summary?.total_requests === 0),
  }
}

