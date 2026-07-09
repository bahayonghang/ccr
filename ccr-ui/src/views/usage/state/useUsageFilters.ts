import { computed, type Ref } from 'vue'
import { useUsageStore } from '@/stores/usage'
import type { UsagePlatform } from '@/types/usage'
import {
  coerceLegacyDaysToUsageRangePreset,
  getLocalDateRangeWindow,
  getUsageRangePresetImportDays,
  type UsageRangePreset,
} from '../dateWindow'
import { buildSelectedPlatformLabel, buildSelectedWindowLabel } from '../usageOverviewInsights'

type UsageStore = ReturnType<typeof useUsageStore>
type TranslateDashboardText = (
  key: string,
  values: Record<string, number | string> | undefined,
  fallback: string
) => string

const getTimeRange = getLocalDateRangeWindow

export const useUsageFilters = (params: {
  store: UsageStore
  translateDashboardText: TranslateDashboardText
  activeTab: Ref<string>
  selectedPlatform: Ref<string>
  selectedRange: Ref<UsageRangePreset>
  loadLogs: (direction?: 'reset' | 'next' | 'prev' | 'same') => Promise<void>
}) => {
  const { store, translateDashboardText, activeTab, selectedPlatform, selectedRange, loadLogs } =
    params

  const tabKeys = ['overview', 'tokens', 'cost', 'providers', 'models', 'projects', 'logs'] as const

  const selectedDays = computed({
    get: () => getUsageRangePresetImportDays(selectedRange.value),
    set: (value: number) => {
      selectedRange.value = coerceLegacyDaysToUsageRangePreset(value)
    },
  })

  const onFilterChange = () => {
    const { start, end } = getTimeRange(selectedRange.value)
    store.setFilters({
      platform: (selectedPlatform.value || undefined) as UsagePlatform | undefined,
      start,
      end,
    })

    if (activeTab.value === 'logs') {
      void loadLogs('reset')
    }
  }

  const selectedPlatformLabel = computed(() =>
    buildSelectedPlatformLabel(selectedPlatform.value, translateDashboardText)
  )

  const selectedWindowLabel = computed(() =>
    buildSelectedWindowLabel(selectedRange.value, translateDashboardText)
  )

  return {
    activeTab,
    selectedPlatform,
    selectedRange,
    tabKeys,
    selectedDays,
    selectedPlatformLabel,
    selectedWindowLabel,
    onFilterChange,
  }
}
