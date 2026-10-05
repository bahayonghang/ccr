// monitoring 域 Query 层（08-22-state-logic-port 批次 5b-ii）。
// 原 useMonitoringFeed 的初始快照拉取（getMonitoringFeed → 失败回退 getRecentEvents）
// 收口为单一 fetcher；高频事件路径（app:monitoring / token-stats）不走本层，
// 由 useMonitoringFeed 的 createEventBatcher 批量 setQueryData 写入同一缓存。
//
// staleTime: Infinity —— 快照只在挂载与显式 refresh（原 loadInitialFeed 语义）时拉取，
// 不参与窗口聚焦等自动 refetch；实时增量全部来自事件批量提交。

import { getMonitoringFeed, getRecentEvents } from '@/api'
import { isTauriRuntime } from '@/utils/tauriRuntime'

export const monitoringKeys = {
  all: ['monitoring'] as const,
  feed: () => [...monitoringKeys.all, 'feed'] as const,
}

/**
 * 初始快照：getMonitoringFeed 失败时回退 legacy event feed（原 loadInitialFeed 的
 * fall-through 语义）；两次都失败时向上抛出，由调用方判定断连。
 * 返回原始条目，归一化（normalizeMonitoringEntry）留在消费方 hook 内。
 */
export async function fetchMonitoringFeedSnapshot(count: number): Promise<unknown[]> {
  if (!isTauriRuntime()) {
    return []
  }

  try {
    return await getMonitoringFeed({ count })
  } catch {
    // 后端迁移期回退到 legacy event feed（原实现注释语义）。
    return await getRecentEvents(count)
  }
}
