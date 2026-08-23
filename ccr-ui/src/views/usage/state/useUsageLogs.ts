import { computed, type Ref } from 'vue'
import type { Composer } from 'vue-i18n'
import { useUsageStore } from '@/stores/usage'
import type { UsagePlatform } from '@/types/usage'
import { getUsageRangePresetImportDays, type UsageRangePreset } from '../dateWindow'
import { buildUsageDiagnosticsSummary, type UsageDiagnosticsSummary } from '../usageDiagnostics'

type UsageStore = ReturnType<typeof useUsageStore>
// ReturnType<typeof useI18n> 解析为 any（vue-i18n Composer 泛型默认 Record<string, any>）；
// 显式实例化 Composer，让 t/locale 的类型可用（no-unsafe-* 依赖真实类型）。
type I18nComposer = Composer<Record<string, unknown>, Record<string, unknown>, Record<string, unknown>, string>

export const useUsageLogs = (params: {
  store: UsageStore
  t: I18nComposer['t']
  locale: I18nComposer['locale']
  selectedPlatform: Ref<string>
  selectedRange: Ref<UsageRangePreset>
  logModelFilter: Ref<string>
}) => {
  const { store, t, locale, selectedPlatform, selectedRange, logModelFilter } = params

  const loadLogs = async (direction: 'reset' | 'next' | 'prev' | 'same' = 'reset') => {
    store.setLogsModelFilter(logModelFilter.value || undefined)
    await store.fetchLogs(direction)
  }

  const updateLogModelFilter = (value: string) => {
    logModelFilter.value = value
  }

  const logsRecords = computed(() => store.logs?.records ?? [])
  const unknownModelStat = computed(
    () => store.modelStats.find((item) => item.model === 'unknown') ?? null
  )
  const logsTotalCount = computed(() => store.logs?.total ?? logsRecords.value.length)
  const diagnosticsSummary = computed<UsageDiagnosticsSummary>(() =>
    buildUsageDiagnosticsSummary({
      selectedPlatform: selectedPlatform.value as UsagePlatform | '',
      summary: store.summary,
      logsRecords: logsRecords.value,
      logsTotalCount: logsTotalCount.value,
      unknownModelStat: unknownModelStat.value,
      archive: store.archive,
      locale: locale.value,
      messages: {
        noRecentRecord: t('usage.dashboard.diagnostics.noRecentRecord'),
        rawLogsHint: t('usage.dashboard.diagnostics.rawLogsHint'),
        repairNeeded: t('usage.dashboard.diagnostics.repairNeeded'),
        codexRepairHint: (unknown) => t('usage.dashboard.diagnostics.codexRepairHint', { unknown }),
        healthy: t('usage.dashboard.diagnostics.healthy'),
      },
    })
  )
  const diagnosticsEmptyMessage = computed(() =>
    logModelFilter.value
      ? t('usage.dashboard.diagnostics.filteredNoResults')
      : t('usage.dashboard.logs.noLogs')
  )
  const diagnosticsEmptyDetail = computed(() =>
    logModelFilter.value
      ? t('usage.dashboard.diagnostics.filteredNoResultsHint')
      : t('usage.dashboard.diagnostics.emptyHint')
  )
  const repairCodexButtonLabel = computed(() =>
    store.importing
      ? t('usage.dashboard.diagnostics.repairingCodex')
      : t('usage.dashboard.diagnostics.repairCodex')
  )
  const repairCodexLogs = async () => {
    if (selectedPlatform.value !== 'codex' || store.importing) return

    await store.startImportJob({
      platform: 'codex',
      reason: 'manual',
      recentDays: getUsageRangePresetImportDays(selectedRange.value),
      resetSources: true,
    })
  }

  return {
    logModelFilter,
    loadLogs,
    updateLogModelFilter,
    logsRecords,
    diagnosticsSummary,
    diagnosticsEmptyMessage,
    diagnosticsEmptyDetail,
    repairCodexButtonLabel,
    repairCodexLogs,
  }
}
