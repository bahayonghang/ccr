// system 域 Query 层（08-22-state-logic-port 批次 5）。
// 原 `composables/useBackendHealth.ts` 的模块级单例轮询迁入 Query 缓存：
// 多消费者共享同一 queryKey；健康判定为纯函数，供消费侧派生状态与退避间隔。

export const systemKeys = {
  all: ['system'] as const,
  backendHealth: () => [...systemKeys.all, 'backend-health'] as const,
}

/** `healthCheck` 的 payload 形态（原 useBackendHealth 内联接口）。 */
export interface BackendHealthPayload {
  status?: string
  database?: boolean
}

/** 健康态轮询周期：5min（原 intervalMs 函数的 ok 分支）。 */
export const BACKEND_HEALTH_POLL_OK_MS = 300_000

/** 未知/异常态轮询周期：30s（原 intervalMs 函数的非 ok 分支）。 */
export const BACKEND_HEALTH_POLL_DEGRADED_MS = 30_000

/** 健康判定（原 checkHealth 内联逻辑）：status=healthy 且 database 未显式为 false。 */
export function isBackendHealthPayloadHealthy(
  payload: BackendHealthPayload | undefined | null
): boolean {
  return payload?.status === 'healthy' && payload.database !== false
}
