import { computed, type ComputedRef, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useUsageStore } from '@/stores/usage'
import type { UsageFeatureCapability, UsageUnsupportedReason } from '@/types/usage'
import { buildUsageOpsCockpit } from '../usageOpsCockpit'
import { buildDashboardMetaItems, type DashboardMetaItem } from '../usageOverviewInsights'

type UsageStore = ReturnType<typeof useUsageStore>
type I18nComposer = ReturnType<typeof useI18n>
type TranslateDashboardText = (
  key: string,
  values: Record<string, number | string> | undefined,
  fallback: string
) => string

const normalizeUnsupportedReason = (reason: UsageUnsupportedReason | null | undefined) => {
  if (
    reason === 'cli_missing' ||
    reason === 'db_missing' ||
    reason === 'db_unreadable' ||
    reason === 'schema_unsupported' ||
    reason === 'missing_table' ||
    reason === 'missing_column' ||
    reason === 'waiting_for_llmusage'
  ) {
    return reason
  }

  return 'waiting_for_llmusage'
}

export const useUsageMeta = (params: {
  store: UsageStore
  t: I18nComposer['t']
  locale: I18nComposer['locale']
  dashboardReady: ComputedRef<boolean>
  selectedPlatform: Ref<string>
  selectedPlatformLabel: ComputedRef<string>
  selectedWindowLabel: ComputedRef<string>
  translateDashboardText: TranslateDashboardText
}) => {
  const {
    store,
    t,
    locale,
    dashboardReady,
    selectedPlatform,
    selectedPlatformLabel,
    selectedWindowLabel,
    translateDashboardText,
  } = params

  const dashboardMetaItems = computed<DashboardMetaItem[]>(() => {
    if (!dashboardReady.value) return []

    return buildDashboardMetaItems({
      archive: store.archive,
      locale: locale.value,
      modelCount: store.modelStats.length,
      projectCount: store.projectStats.length,
      selectedPlatformLabel: selectedPlatformLabel.value,
      selectedWindowLabel: selectedWindowLabel.value,
      translate: translateDashboardText,
    })
  })

  const importButtonLabel = computed(() => {
    if (store.isBootstrapping) return t('usage.dashboard.bootstrapping')
    if (store.importing) return t('usage.dashboard.importing')
    return t('usage.dashboard.import')
  })

  const dashboardUnsupportedCapability = computed<UsageFeatureCapability | null>(() => {
    const capability = store.dashboardCapability
    if (!capability || capability.supported) return null
    return capability
  })

  const syncUnsupportedCapability = computed<UsageFeatureCapability | null>(() => {
    const capability = store.syncCapability
    if (!capability || capability.supported) return null
    return capability
  })

  const providerCapability = computed<UsageFeatureCapability | null>(
    () => store.usageCapabilities?.features.provider_breakdown ?? null
  )

  const unsupportedStateTitle = computed(() => {
    const reason = normalizeUnsupportedReason(dashboardUnsupportedCapability.value?.reason)
    return translateDashboardText(
      `usage.unsupported.${reason}.title`,
      undefined,
      translateDashboardText(
        'usage.unsupported.waiting_for_llmusage.title',
        undefined,
        'Waiting for llmusage support'
      )
    )
  })

  const unsupportedStateDescription = computed(() => {
    const capability = dashboardUnsupportedCapability.value
    const reason = normalizeUnsupportedReason(capability?.reason)
    const translated = translateDashboardText(
      `usage.unsupported.${reason}.description`,
      undefined,
      translateDashboardText(
        'usage.unsupported.waiting_for_llmusage.description',
        undefined,
        'The installed llmusage runtime does not expose this usage view yet.'
      )
    )

    return capability?.detail ? `${translated} ${capability.detail}` : translated
  })

  const unsupportedSyncMessage = computed(() => {
    const capability = syncUnsupportedCapability.value
    if (!capability) return null

    const reason = normalizeUnsupportedReason(capability.reason)
    const translated = translateDashboardText(
      `usage.unsupported.${reason}.description`,
      undefined,
      translateDashboardText(
        'usage.unsupported.cli_missing.description',
        undefined,
        'Install llmusage and run a sync before usage data can be imported.'
      )
    )

    return capability.detail ? `${translated} ${capability.detail}` : translated
  })

  const importDetails = computed(() =>
    store.lastImportResults
      .filter((result) => result.error)
      .map((result) => `${result.platform}: ${result.error}`)
  )

  const warningMessage = computed(() => store.warning || null)
  const importJobBanner = computed(() => {
    const job = store.currentImportJob
    if (!job || !store.importing) return null

    const totalFiles = Math.max(job.files_total, job.files_scanned)
    const params = {
      scanned: job.files_scanned,
      total: totalFiles,
      records: job.records_imported.toLocaleString(),
    }

    return job.status === 'recent_ready'
      ? t('usage.dashboard.importJobBanner.recent', params)
      : t('usage.dashboard.importJobBanner.running', params)
  })
  const importJobWarnings = computed(() => store.currentImportJob?.warnings ?? [])
  const opsCockpit = computed(() =>
    buildUsageOpsCockpit({
      archive: store.archive,
      importDetails: importDetails.value,
      importing: store.importing,
      importJobBanner: importJobBanner.value,
      importJobWarnings: importJobWarnings.value,
      lastUpdatedAt: store.lastUpdated?.toISOString() ?? null,
      loading: store.loading,
      locale: locale.value,
      selectedPlatformLabel: selectedPlatformLabel.value,
      selectedWindowLabel: selectedWindowLabel.value,
      snapshot: store.snapshot,
      translate: translateDashboardText,
      unsupportedSyncMessage: unsupportedSyncMessage.value,
      warningMessage: warningMessage.value,
    })
  )
  const showEmptyState = computed(() => store.hasNoUsageData)

  const emptyStateTitle = computed(() => {
    if (
      store.lastImportSummary &&
      store.lastImportSummary.processed_files === 0 &&
      store.lastImportSummary.imported_records === 0
    ) {
      return t('usage.dashboard.status.noLogsTitle')
    }
    return t('usage.states.noData')
  })

  const emptyStateDescription = computed(() => {
    if (
      store.lastImportSummary &&
      store.lastImportSummary.processed_files === 0 &&
      store.lastImportSummary.imported_records === 0
    ) {
      return t('usage.dashboard.status.noLogs')
    }
    return t('usage.states.noDataHint', { platform: selectedPlatform.value || 'AI' })
  })

  const providerStats = computed(() => (dashboardReady.value ? store.providerStats : []))

  return {
    dashboardMetaItems,
    importButtonLabel,
    importDetails,
    importJobBanner,
    importJobWarnings,
    unsupportedStateDescription,
    unsupportedStateTitle,
    unsupportedSyncMessage,
    opsCockpit,
    providerCapability,
    providerStats,
    showEmptyState,
    emptyStateTitle,
    emptyStateDescription,
    warningMessage,
  }
}
