import { computed, onMounted, ref, unref, watch, type MaybeRef } from 'vue'
import { getUsageDashboardV2 } from '@/api'
import type { UsageDashboardResponse } from '@/types/usage'
import type {
  PlatformUsageId,
  PlatformUsageInsightLabels,
  PlatformUsageTone,
} from '@/types/platformUsageInsight'
import { getLocalDateWindow } from '@/views/usage/dateWindow'
import {
  buildPlatformUsageInsight,
  buildPlatformUsageLabels,
} from '@/views/platform-usage/platformUsagePresentation'

export interface UsePlatformUsageInsightOptions {
  platform: MaybeRef<PlatformUsageId>
  days?: MaybeRef<number>
  enabled?: MaybeRef<boolean>
  labels?: MaybeRef<Partial<PlatformUsageInsightLabels>>
  tone?: MaybeRef<PlatformUsageTone>
}

const resolveErrorMessage = (error: unknown) =>
  error instanceof Error ? error.message : String(error || 'Usage insight unavailable')

export const usePlatformUsageInsight = ({
  platform,
  days = 30,
  enabled = true,
  labels,
  tone = 'neutral',
}: UsePlatformUsageInsightOptions) => {
  const loading = ref(false)
  const error = ref<string | null>(null)
  const dashboard = ref<UsageDashboardResponse | null>(null)
  let requestId = 0

  const dateWindow = computed(() => getLocalDateWindow(unref(days)))
  const resolvedLabels = computed(() => buildPlatformUsageLabels(unref(labels)))
  const presentation = computed(() =>
    buildPlatformUsageInsight({
      data: dashboard.value,
      labels: resolvedLabels.value,
      tone: unref(tone),
    }),
  )

  const refresh = async () => {
    if (!unref(enabled)) return

    const currentRequestId = ++requestId
    loading.value = true
    error.value = null

    try {
      const window = dateWindow.value
      const data = await getUsageDashboardV2<UsageDashboardResponse>(
        unref(platform),
        window.start,
        window.end,
        0,
        false,
      )

      if (currentRequestId === requestId) {
        dashboard.value = data
      }
    } catch (caught) {
      if (currentRequestId === requestId) {
        error.value = resolveErrorMessage(caught)
      }
    } finally {
      if (currentRequestId === requestId) {
        loading.value = false
      }
    }
  }

  onMounted(() => {
    void refresh()
  })

  watch(
    () => [unref(platform), unref(days), unref(enabled)] as const,
    ([, , isEnabled]) => {
      if (isEnabled) {
        void refresh()
      }
    },
  )

  return {
    loading,
    error,
    dashboard,
    dateWindow,
    presentation,
    refresh,
  }
}
