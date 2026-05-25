import type {
  UsageArchiveDiagnostics,
  UsageFreshnessProjection,
  UsageFreshnessState,
  UsageReadinessState,
  UsageSnapshotProjection,
  UsageSourceHealth,
  UsageSourceHealthState,
} from '@/types/usage'
import {
  formatDateTime,
  type UsageDashboardTranslate,
} from './usageOverviewInsights'

export type UsageOpsTone = 'success' | 'warning' | 'danger' | 'info' | 'muted'
export type UsageOpsAction = 'import' | 'diagnostics' | 'none'

export interface UsageOpsHealthItem {
  id: string
  label: string
  value: string
  detail: string
  tone: UsageOpsTone
}

export interface UsageOpsSourceItem {
  id: string
  label: string
  statusLabel: string
  detail: string
  tone: UsageOpsTone
}

export interface UsageOpsAlert {
  id: string
  tone: UsageOpsTone
  title: string
  details: string[]
}

export interface UsageOpsCockpitPresentation {
  state: UsageReadinessState
  tone: UsageOpsTone
  eyebrow: string
  title: string
  detail: string
  summaryLine: string
  primaryAction: UsageOpsAction
  primaryActionLabel: string | null
  secondaryActionLabel: string | null
  healthItems: UsageOpsHealthItem[]
  sourceItems: UsageOpsSourceItem[]
  alerts: UsageOpsAlert[]
}

const readinessTone: Record<UsageReadinessState, UsageOpsTone> = {
  ready: 'success',
  syncing: 'info',
  stale: 'warning',
  degraded: 'warning',
  needs_import: 'danger',
  needs_session_index: 'warning',
  empty: 'muted',
}

const readinessFallbacks: Record<UsageReadinessState, { label: string; title: string; detail: string }> = {
  ready: {
    label: 'Ready',
    title: 'Usage data is ready',
    detail: 'The local read model is fresh enough for cost and token decisions.',
  },
  syncing: {
    label: 'Syncing',
    title: 'Usage import is running',
    detail: 'CCR is updating the local usage snapshot; current analytics may refresh in the background.',
  },
  stale: {
    label: 'Stale',
    title: 'Usage data is stale',
    detail: 'Refresh the local usage archive before making budget or quota decisions.',
  },
  degraded: {
    label: 'Degraded',
    title: 'Some usage sources need attention',
    detail: 'One or more usage sources are missing, deleted, or older than the freshness window.',
  },
  needs_import: {
    label: 'Needs import',
    title: 'Import usage before analyzing',
    detail: 'No imported usage source is available for the current scope yet.',
  },
  needs_session_index: {
    label: 'Needs index',
    title: 'Session index is missing',
    detail: 'Raw sessions exist, but the local session archive has not been indexed yet.',
  },
  empty: {
    label: 'Empty',
    title: 'No usage records in this window',
    detail: 'The source is reachable, but the selected scope currently has no usage rows.',
  },
}

const freshnessTone: Record<UsageFreshnessState, UsageOpsTone> = {
  fresh: 'success',
  stale: 'warning',
  missing: 'danger',
}

const sourceTone: Record<UsageSourceHealthState, UsageOpsTone> = {
  live: 'success',
  degraded: 'warning',
  missing: 'danger',
}

const sourceRank: Record<UsageSourceHealthState, number> = {
  degraded: 0,
  missing: 1,
  live: 2,
}

const sourceLabels: Record<string, string> = {
  claude: 'Claude',
  codex: 'Codex',
  gemini: 'Antigravity',
  opencode: 'OpenCode',
  unknown: 'Unknown',
}

const translateWithFallback = (
  translate: UsageDashboardTranslate,
  key: string,
  fallback: string,
  values?: Record<string, number | string>,
) => translate(key, values, fallback)

const formatAge = (
  seconds: number | null | undefined,
  translate: UsageDashboardTranslate,
) => {
  if (seconds == null) {
    return translateWithFallback(translate, 'usage.dashboard.ops.age.unknown', 'age unknown')
  }

  if (seconds < 60) {
    return translateWithFallback(translate, 'usage.dashboard.ops.age.seconds', '{count}s ago', {
      count: seconds,
    })
  }

  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) {
    return translateWithFallback(translate, 'usage.dashboard.ops.age.minutes', '{count}m ago', {
      count: minutes,
    })
  }

  const hours = Math.floor(minutes / 60)
  if (hours < 48) {
    return translateWithFallback(translate, 'usage.dashboard.ops.age.hours', '{count}h ago', {
      count: hours,
    })
  }

  return translateWithFallback(translate, 'usage.dashboard.ops.age.days', '{count}d ago', {
    count: Math.floor(hours / 24),
  })
}

const formatOptionalDate = (
  value: string | null | undefined,
  locale: string,
  translate: UsageDashboardTranslate,
) => value
  ? formatDateTime(value, locale)
  : translateWithFallback(translate, 'usage.dashboard.ops.missingTimestamp', 'No completed import yet')

const readinessAction = (
  state: UsageReadinessState,
  nextAction: string | null | undefined,
): UsageOpsAction => {
  if (nextAction === 'import_usage' || nextAction === 'refresh_usage') return 'import'
  if (nextAction === 'index_sessions' || nextAction === 'inspect_sources') return 'diagnostics'
  if (state === 'needs_import' || state === 'stale') return 'import'
  if (state === 'syncing' || state === 'degraded' || state === 'needs_session_index') {
    return 'diagnostics'
  }
  return 'none'
}

const actionLabel = (
  action: UsageOpsAction,
  state: UsageReadinessState,
  nextAction: string | null | undefined,
  translate: UsageDashboardTranslate,
) => {
  if (action === 'none') return null

  if (action === 'diagnostics') {
    return translateWithFallback(
      translate,
      nextAction === 'index_sessions'
        ? 'usage.dashboard.ops.actions.index_sessions'
        : 'usage.dashboard.ops.actions.diagnostics',
      nextAction === 'index_sessions' ? 'Open diagnostics' : 'Inspect diagnostics',
    )
  }

  return translateWithFallback(
    translate,
    nextAction === 'refresh_usage' || state === 'stale'
      ? 'usage.dashboard.ops.actions.refresh_usage'
      : 'usage.dashboard.ops.actions.import_usage',
    nextAction === 'refresh_usage' || state === 'stale' ? 'Refresh usage' : 'Import usage',
  )
}

const sourceLabel = (source: string) => sourceLabels[source] ?? source

const buildSourceItems = ({
  locale,
  sourceHealth,
  translate,
}: {
  locale: string
  sourceHealth: UsageSourceHealth[]
  translate: UsageDashboardTranslate
}): UsageOpsSourceItem[] => [...sourceHealth]
  .sort((left, right) =>
    sourceRank[left.state] - sourceRank[right.state] ||
    right.live_sources - left.live_sources ||
    left.source.localeCompare(right.source)
  )
  .slice(0, 4)
  .map((source) => {
    const latest = source.freshness.latest_completed_at
      ?? source.history_completed_at
      ?? source.recent_completed_at

    return {
      id: source.source,
      label: sourceLabel(source.source),
      statusLabel: translateWithFallback(
        translate,
        `usage.dashboard.ops.sourceStates.${source.state}`,
        source.state,
      ),
      detail: [
        `L ${source.live_sources.toLocaleString()} · M ${source.missing_sources.toLocaleString()} · D ${source.deleted_sources.toLocaleString()}`,
        latest
          ? formatDateTime(latest, locale)
          : translateWithFallback(translate, 'usage.dashboard.ops.missingTimestampShort', 'no sync'),
      ].join(' · '),
      tone: sourceTone[source.state],
    }
  })

const buildAlerts = ({
  importDetails,
  importJobBanner,
  importJobWarnings,
  unsupportedSyncMessage,
  warningMessage,
  translate,
}: {
  importDetails: string[]
  importJobBanner: string | null
  importJobWarnings: string[]
  unsupportedSyncMessage: string | null
  warningMessage: string | null
  translate: UsageDashboardTranslate
}): UsageOpsAlert[] => {
  const alerts: UsageOpsAlert[] = []

  if (importJobBanner) {
    alerts.push({
      id: 'active-import',
      tone: 'info',
      title: importJobBanner,
      details: importJobWarnings.slice(0, 4),
    })
  }

  if (unsupportedSyncMessage) {
    alerts.push({
      id: 'sync-unsupported',
      tone: 'warning',
      title: unsupportedSyncMessage,
      details: importDetails.slice(0, 4),
    })
  } else if (warningMessage) {
    alerts.push({
      id: 'warning',
      tone: 'warning',
      title: warningMessage,
      details: importDetails.slice(0, 4),
    })
  } else if (importDetails.length > 0) {
    alerts.push({
      id: 'import-details',
      tone: 'warning',
      title: translateWithFallback(
        translate,
        'usage.dashboard.ops.alerts.importDetails',
        'Import details need review',
      ),
      details: importDetails.slice(0, 4),
    })
  }

  return alerts
}

export const buildUsageOpsCockpit = ({
  archive,
  importDetails,
  importing,
  importJobBanner,
  importJobWarnings,
  lastUpdatedAt,
  loading,
  locale,
  selectedPlatformLabel,
  selectedWindowLabel,
  snapshot,
  translate,
  unsupportedSyncMessage,
  warningMessage,
}: {
  archive: UsageArchiveDiagnostics | null
  importDetails: string[]
  importing: boolean
  importJobBanner: string | null
  importJobWarnings: string[]
  lastUpdatedAt: string | null
  loading: boolean
  locale: string
  selectedPlatformLabel: string
  selectedWindowLabel: string
  snapshot: UsageSnapshotProjection | null
  translate: UsageDashboardTranslate
  unsupportedSyncMessage: string | null
  warningMessage: string | null
}): UsageOpsCockpitPresentation => {
  const readiness = snapshot?.readiness ?? archive?.readiness ?? null
  const freshness: UsageFreshnessProjection | null = snapshot?.freshness ?? archive?.freshness ?? null
  const sourceHealth = snapshot?.source_health ?? archive?.source_health ?? []
  const state: UsageReadinessState = importing || loading
    ? 'syncing'
    : readiness?.state ?? 'empty'
  const stateCopy = readinessFallbacks[state]
  const tone = readinessTone[state]
  const primaryAction = readinessAction(state, readiness?.next_action)
  const primaryActionLabel = actionLabel(primaryAction, state, readiness?.next_action, translate)
  const secondaryActionLabel = primaryAction === 'diagnostics'
    ? null
    : translateWithFallback(
        translate,
        'usage.dashboard.ops.actions.diagnostics',
        'Inspect diagnostics',
      )
  const latestCompletedAt = freshness?.latest_completed_at
    ?? readiness?.recent_completed_at
    ?? archive?.history_completed_at
    ?? archive?.recent_completed_at

  const sourceValue = archive
    ? `L ${archive.live_sources.toLocaleString()} · M ${archive.missing_sources.toLocaleString()} · D ${archive.deleted_sources.toLocaleString()}`
    : '—'
  const generatedAt = snapshot?.generated_at ?? lastUpdatedAt
  const cacheTtlSeconds = snapshot?.cache_ttl_seconds ?? 30
  const dimensions = snapshot?.drilldown.dimensions ?? []

  return {
    state,
    tone,
    eyebrow: translateWithFallback(
      translate,
      'usage.dashboard.ops.eyebrow',
      'Usage operations cockpit',
    ),
    title: translateWithFallback(
      translate,
      `usage.dashboard.ops.states.${state}.title`,
      stateCopy.title,
    ),
    detail: translateWithFallback(
      translate,
      `usage.dashboard.ops.states.${state}.detail`,
      stateCopy.detail,
    ),
    summaryLine: `${selectedPlatformLabel} · ${selectedWindowLabel}`,
    primaryAction,
    primaryActionLabel,
    secondaryActionLabel,
    healthItems: [
      {
        id: 'readiness',
        label: translateWithFallback(translate, 'usage.dashboard.ops.health.readiness', 'Readiness'),
        value: translateWithFallback(
          translate,
          `usage.dashboard.ops.states.${state}.label`,
          stateCopy.label,
        ),
        detail: readiness?.next_action
          ? translateWithFallback(
              translate,
              'usage.dashboard.ops.health.nextAction',
              'Next action: {action}',
              {
                action: readiness.next_action,
              },
            )
          : translateWithFallback(translate, 'usage.dashboard.ops.health.noAction', 'No blocking action'),
        tone,
      },
      {
        id: 'freshness',
        label: translateWithFallback(translate, 'usage.dashboard.ops.health.freshness', 'Freshness'),
        value: freshness
          ? translateWithFallback(
              translate,
              `usage.dashboard.ops.freshness.${freshness.state}`,
              freshness.state,
            )
          : translateWithFallback(translate, 'usage.dashboard.ops.freshness.missing', 'missing'),
        detail: [
          formatOptionalDate(latestCompletedAt, locale, translate),
          formatAge(freshness?.age_seconds, translate),
        ].join(' · '),
        tone: freshness ? freshnessTone[freshness.state] : 'danger',
      },
      {
        id: 'source-health',
        label: translateWithFallback(translate, 'usage.dashboard.ops.health.sourceHealth', 'Source health'),
        value: sourceValue,
        detail: translateWithFallback(
          translate,
          'usage.dashboard.ops.health.archivedSessions',
          '{count} archived sessions',
          {
            count: archive?.archived_sessions ?? 0,
          },
        ),
        tone: archive
          ? archive.missing_sources > 0 || archive.deleted_sources > 0
            ? 'warning'
            : archive.live_sources > 0
              ? 'success'
              : 'danger'
          : 'muted',
      },
      {
        id: 'snapshot-cache',
        label: translateWithFallback(translate, 'usage.dashboard.ops.health.snapshotCache', 'Snapshot cache'),
        value: translateWithFallback(translate, 'usage.dashboard.ops.health.cacheTtl', '{count}s TTL', {
          count: cacheTtlSeconds,
        }),
        detail: generatedAt
          ? translateWithFallback(
              translate,
              'usage.dashboard.ops.health.generatedAt',
              'Generated {time}',
              {
                time: formatDateTime(generatedAt, locale),
              },
            )
          : translateWithFallback(translate, 'usage.dashboard.ops.health.notGenerated', 'Not generated yet'),
        tone: snapshot ? 'info' : 'muted',
      },
      {
        id: 'scope',
        label: translateWithFallback(translate, 'usage.dashboard.ops.health.scope', 'Scope'),
        value: selectedPlatformLabel,
        detail: selectedWindowLabel,
        tone: 'info',
      },
      {
        id: 'drilldown',
        label: translateWithFallback(translate, 'usage.dashboard.ops.health.drilldown', 'Drilldown'),
        value: translateWithFallback(
          translate,
          'usage.dashboard.ops.health.dimensions',
          '{count} dimensions',
          {
            count: dimensions.length,
          },
        ),
        detail: dimensions.length > 0
          ? dimensions.slice(0, 4).join(' · ')
          : translateWithFallback(translate, 'usage.dashboard.ops.health.noDrilldown', 'No dimensions yet'),
        tone: dimensions.length > 0 ? 'success' : 'muted',
      },
    ],
    sourceItems: buildSourceItems({
      locale,
      sourceHealth,
      translate,
    }),
    alerts: buildAlerts({
      importDetails,
      importJobBanner,
      importJobWarnings,
      unsupportedSyncMessage,
      warningMessage,
      translate,
    }),
  }
}
