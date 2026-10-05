import { useMemo } from 'react'
import { getErrorMessage } from '@/utils/errorHandler'
import type { TranslateFunction } from '@/utils/tf'
import {
  activationLabelOf,
  activationWarningOf,
  authModeLabelOf,
  currentProfileLabelOf,
  buildManagementItems,
  buildNextActions,
  buildReadinessItems,
  deriveVersionState,
  localOnlyState,
  queryErrorMessage,
  versionToneOf,
  type GrokActionItem,
} from './home/grokHomeModel'
import { useGrokHomeQueries } from './useGrokHomeQueries'

export function useGrokHome(t: TranslateFunction) {
  const queries = useGrokHomeQueries()
  const { environment, environmentQuery, overviewQuery, overviewResult, overview, versionQuery, refresh } = queries

  const gated = localOnlyState({
    envType: environment?.env_type,
    overview: overviewResult,
  })
  const localOnly = gated.localOnly
  const localOnlyEnvType = gated.envType

  const loading = environmentQuery.isFetching || overviewQuery.isFetching || versionQuery.isFetching
  const initialLoading = !localOnly && !overview && (environmentQuery.isFetching || overviewQuery.isFetching)
  const errors = queryErrorMessage({
    envError: environmentQuery.error,
    overviewError: overviewQuery.error,
    hasOverview: Boolean(overview),
    format: getErrorMessage,
  })
  const loadError = errors.loadError
  const refreshError = errors.refreshError

  const currentProfileLabel = currentProfileLabelOf(overview, t)
  const activationLabel = activationLabelOf(overview?.activation, t)
  const authModeLabel = authModeLabelOf(overview?.auth_mode ?? undefined, t)
  const version = deriveVersionState(versionQuery.data, versionQuery.isFetching, t)
  const versionTone = versionToneOf(version.versionStatus)

  const activationWarning = useMemo(() => activationWarningOf(overview, t), [overview, t])

  const readinessItems = useMemo(
    () =>
      overview
        ? buildReadinessItems({
            data: overview,
            version: { ...version, versionTone },
            labels: { currentProfile: currentProfileLabel, activation: activationLabel },
            t,
          })
        : [],
    [activationLabel, currentProfileLabel, overview, t, version, versionTone],
  )
  const nextActions = useMemo(
    () => (overview ? buildNextActions(overview, version.versionStatus, t) : []),
    [overview, t, version.versionStatus],
  )
  const primaryAction: GrokActionItem = nextActions[0] ?? {
    key: 'open-settings',
    title: t('grok.dashboard.actions.openSettings.title'),
    description: t('grok.dashboard.actions.openSettings.description'),
    to: '/grok/settings',
    icon: 'SlidersHorizontal',
    tone: 'neutral',
  }
  const managementItems = useMemo(
    () => (overview ? buildManagementItems(overview, t) : []),
    [overview, t],
  )

  return {
    overview,
    loading,
    initialLoading,
    loadError,
    refreshError,
    localOnly,
    localOnlyEnvType,
    versionLabel: version.versionLabel,
    currentProfileLabel,
    authModeLabel,
    activationWarning,
    readinessItems,
    nextActions,
    primaryAction,
    managementItems,
    refresh,
  }
}
