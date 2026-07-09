import { ref } from 'vue'
import { getErrorMessage } from '@/utils/errorHandler'
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
    errorMessage.value = getErrorMessage(error)
    lastCheckedAt.value = new Date()
    throw error
  }
}

if (!isTauri) {
  status.value = 'unsupported'
}

// 模块加载不自启动轮询（immediate:false）；由消费者（BackendStatusBanner）按生命周期 resume/pause。
// 健康时退避到 5min，未知/异常时回到 30s，降低稳态下的 SQLite 健康探测频率。
const backendHealthPoller = isTauri
  ? usePolledData<BackendHealthPayload>(
      async () => {
        const result = await healthCheck<BackendHealthPayload>()
        status.value = result.status === 'healthy' && result.database !== false ? 'ok' : 'error'
        errorMessage.value =
          status.value === 'error' ? 'Backend health check reported degraded status.' : null
        lastCheckedAt.value = new Date()
        return result
      },
      {
        key: 'backend-health',
        intervalMs: () => (status.value === 'ok' ? 300_000 : 30_000),
        pauseWhenHidden: true,
        immediate: false,
        onError: (error) => {
          status.value = 'error'
          errorMessage.value = error.message
          lastCheckedAt.value = new Date()
        },
      }
    )
  : null

export const useBackendHealth = () => {
  if (!isTauri) {
    status.value = 'unsupported'
  } else if (backendHealthPoller?.loading.value) {
    status.value = 'checking'
  }

  return {
    status,
    errorMessage,
    lastCheckedAt,
    checkHealth,
    /** 恢复轮询（消费者 onMounted 调用）：立即探测一次并启动定时器 */
    resume: () => backendHealthPoller?.resume(),
    /** 暂停轮询（消费者 onUnmounted 调用） */
    pause: () => backendHealthPoller?.pause(),
  }
}
