import { ref } from 'vue'
import { isTauriEnvironment } from '@/api/runtime/environment'
import { healthCheck } from '@/api/runtime/system'
import { usePolledData } from '@/composables/usePolledData'

export type BackendHealthStatus = 'unsupported' | 'unknown' | 'checking' | 'ok' | 'error'

const status = ref<BackendHealthStatus>('unknown')
const errorMessage = ref<string | null>(null)
const lastCheckedAt = ref<Date | null>(null)
const isTauri = isTauriEnvironment()

interface BackendHealthPayload {
  status?: string
  database?: boolean
}

/**
 * 原生 Tauri 模式下，后端内嵌在应用中，始终可用。
 * web 模式下返回 unsupported；Tauri 模式下走共享轮询器。
 */
const checkHealth = async () => {
  if (!isTauri) {
    status.value = 'unsupported'
    errorMessage.value = null
    lastCheckedAt.value = null
    return
  }

  status.value = 'checking'

  try {
    const result = await healthCheck<BackendHealthPayload>()
    status.value = result.status === 'healthy' && result.database !== false ? 'ok' : 'error'
    errorMessage.value =
      status.value === 'error' ? 'Backend health check reported degraded status.' : null
    lastCheckedAt.value = new Date()
  } catch (error) {
    status.value = 'error'
    errorMessage.value = error instanceof Error ? error.message : String(error)
    lastCheckedAt.value = new Date()
    throw error
  }
}

if (!isTauri) {
  status.value = 'unsupported'
}

const backendHealthPoller = isTauri
  ? usePolledData<BackendHealthPayload>(async () => {
      const result = await healthCheck<BackendHealthPayload>()
      status.value = result.status === 'healthy' && result.database !== false ? 'ok' : 'error'
      errorMessage.value =
        status.value === 'error' ? 'Backend health check reported degraded status.' : null
      lastCheckedAt.value = new Date()
      return result
    }, {
      key: 'backend-health',
      intervalMs: 30_000,
      pauseWhenHidden: true,
      immediate: true,
      onError: (error) => {
        status.value = 'error'
        errorMessage.value = error.message
        lastCheckedAt.value = new Date()
      },
    })
  : null

export const useBackendHealth = (options?: { auto?: boolean; intervalMs?: number }) => {
  if (!isTauri) {
    status.value = 'unsupported'
  } else if (options?.auto === false && status.value === 'unknown') {
    status.value = 'checking'
  } else if (backendHealthPoller?.loading.value) {
    status.value = 'checking'
  }

  return {
    status,
    errorMessage,
    lastCheckedAt,
    checkHealth,
  }
}
