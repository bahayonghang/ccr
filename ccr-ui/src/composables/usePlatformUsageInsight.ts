import { useCallback, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { getUsageDashboardV2 } from '@/api'
import { usageKeys } from '@/features/usage/queries'
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
  platform: PlatformUsageId
  days?: number
  enabled?: boolean
  labels?: Partial<PlatformUsageInsightLabels>
  tone?: PlatformUsageTone
}

const resolveErrorMessage = (error: unknown) =>
  error instanceof Error ? error.message : String(error || 'Usage insight unavailable')

// 平台用量洞察卡的 React 迁移（08-22-state-logic-port 批次 5，服务端数据 → Query）。
//
// 签名变化（消费方均为待迁移 .vue 视图）：MaybeRef<...> 参数改为普通值；
// 返回对象中的 Ref<T> 改为普通值。
//
// watch/onMounted 映射（composable-classification.md §2 登记）：
// 原 onMounted(refresh) 与 watch([platform, days, enabled])（无 immediate/deep/flush
// 选项）由 Query 承担——挂载自动拉取覆盖 onMounted；key（platform+日期窗口）与
// enabled 变化自动重拉覆盖 watch。原 requestId 竞态防护由 Query 单飞请求承担。

export const usePlatformUsageInsight = ({
  platform,
  days = 30,
  enabled = true,
  labels,
  tone = 'neutral',
}: UsePlatformUsageInsightOptions) => {
  const dateWindow = useMemo(() => getLocalDateWindow(days), [days])
  const resolvedLabels = useMemo(() => buildPlatformUsageLabels(labels), [labels])

  // 原实现挂载与参数变化时总是重新拉取、无 TTL → staleTime 0
  const query = useQuery({
    queryKey: usageKeys.insightDashboard(platform, dateWindow.start, dateWindow.end),
    queryFn: () => getUsageDashboardV2(platform, dateWindow.start, dateWindow.end, 0, false),
    enabled,
    staleTime: 0,
  })

  const presentation = useMemo(
    () => buildPlatformUsageInsight({
      data: query.data ?? null,
      labels: resolvedLabels,
      tone,
    }),
    [query.data, resolvedLabels, tone]
  )

  const { refetch } = query
  const refresh = useCallback(async () => {
    await refetch()
  }, [refetch])

  return {
    loading: query.isFetching,
    error: query.error ? resolveErrorMessage(query.error) : null,
    dashboard: (query.data ?? null) as UsageDashboardResponse | null,
    dateWindow,
    presentation,
    refresh,
  }
}
