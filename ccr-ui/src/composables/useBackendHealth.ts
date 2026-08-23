import { useCallback } from 'react'
import { useQuery } from '@tanstack/react-query'
import { isTauriEnvironment } from '@/api/runtime/environment'
import { healthCheck } from '@/api/runtime/system'
import {
  BACKEND_HEALTH_POLL_DEGRADED_MS,
  BACKEND_HEALTH_POLL_OK_MS,
  isBackendHealthPayloadHealthy,
  systemKeys,
  type BackendHealthPayload,
} from '@/features/system/queries'
import { getErrorMessage } from '@/utils/errorHandler'

export type BackendHealthStatus = 'unsupported' | 'unknown' | 'checking' | 'ok' | 'error'

const DEGRADED_MESSAGE = 'Backend health check reported degraded status.'

// 原生 Tauri 模式下，后端内嵌在应用中，始终可用；web 模式返回 unsupported。
//
// 批次 5 迁移说明（原模块级单例 usePolledData 轮询 → TanStack Query）：
// - 共享单例语义由 Query 缓存承担（同一 queryKey 全局一份）。
// - 自适应退避（健康 5min / 未知异常 30s）映射为 refetchInterval 函数。
// - 签名变化：原「消费者生命周期驱动 resume/pause」改为挂载后自动探测并进入
//   轮询节奏（Query 默认行为）；resume() 触发一次立即探测，pause() 为兼容
//   保留的空操作（消费方均为待迁移的 .vue 视图，无存活调用方）。
// - lastCheckedAt 成功取 dataUpdatedAt、失败取 errorUpdatedAt（原实现两者都记录）。

/** staleTime 取 30s（异常态的最短探测周期）；稳态新鲜度由 refetchInterval 主导。 */
const BACKEND_HEALTH_STALE_TIME = 30_000

export const useBackendHealth = () => {
  const isTauri = isTauriEnvironment()

  const query = useQuery({
    queryKey: systemKeys.backendHealth(),
    queryFn: () => healthCheck<BackendHealthPayload>(),
    enabled: isTauri,
    staleTime: BACKEND_HEALTH_STALE_TIME,
    refetchInterval: (q) =>
      isBackendHealthPayloadHealthy(q.state.data)
        ? BACKEND_HEALTH_POLL_OK_MS
        : BACKEND_HEALTH_POLL_DEGRADED_MS,
  })

  let status: BackendHealthStatus
  if (!isTauri) {
    status = 'unsupported'
  } else if (query.error) {
    status = 'error'
  } else if (query.data) {
    status = isBackendHealthPayloadHealthy(query.data) ? 'ok' : 'error'
  } else {
    status = query.fetchStatus === 'fetching' ? 'checking' : 'unknown'
  }

  let errorMessage: string | null = null
  if (query.error) {
    errorMessage = getErrorMessage(query.error)
  } else if (query.data && status === 'error') {
    errorMessage = DEGRADED_MESSAGE
  }

  const checkedAtMs = query.dataUpdatedAt || query.errorUpdatedAt
  const lastCheckedAt = checkedAtMs > 0 ? new Date(checkedAtMs) : null

  const refetch = query.refetch

  const checkHealth = useCallback(() => refetch(), [refetch])
  const resume = useCallback(() => void refetch(), [refetch])


  return {
    status,
    errorMessage,
    lastCheckedAt,
    checkHealth,
    /** 立即探测一次（原语义：恢复时立即探测并进入轮询节奏；轮询现常驻）。 */
    resume,
    /** 兼容保留：Query 挂载期自动轮询，无需显式暂停。 */
    pause: () => {},
  }
}
