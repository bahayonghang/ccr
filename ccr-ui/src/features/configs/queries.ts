import { useQuery } from '@tanstack/react-query'
import { getHistory, getUsageByProviderV2, listConfigs } from '@/api'

// configs 域 Query 层（08-22-state-logic-port 批次 2）。
// 原 `stores/configs.ts`（Options-API）的 5 分钟缓存由 staleTime 等效替代；
// `current_config` 选中态入 Zustand（同目录 stores.ts，批次 4）。

export const configsKeys = {
  all: ['configs'] as const,
  list: () => [...configsKeys.all, 'list'] as const,
  history: () => [...configsKeys.all, 'history'] as const,
  providerUsage: () => [...configsKeys.all, 'provider-usage'] as const,
}

/** staleTime 取值记录（批次 2）：5min，等效原 store 的 isCacheValid 窗口。 */
const CONFIGS_STALE_TIME = 300_000

export function useConfigsList() {
  return useQuery({
    queryKey: configsKeys.list(),
    queryFn: () => listConfigs(),
    staleTime: CONFIGS_STALE_TIME,
  })
}

export function useConfigsHistory(enabled: boolean) {
  return useQuery({
    queryKey: configsKeys.history(),
    queryFn: () => getHistory(),
    enabled,
    staleTime: CONFIGS_STALE_TIME,
  })
}

export function useProviderUsage() {
  return useQuery({
    queryKey: configsKeys.providerUsage(),
    queryFn: async () => {
      const startDate = new Date(Date.now() - 30 * 86400000).toISOString().slice(0, 10)
      const breakdown = await getUsageByProviderV2(undefined, startDate)
      const usage: Record<string, number> = {}
      for (const item of breakdown) {
        const key = item.provider ?? 'unknown'
        usage[key] = (usage[key] ?? 0) + item.request_count
      }
      return usage
    },
    staleTime: CONFIGS_STALE_TIME,
  })
}
