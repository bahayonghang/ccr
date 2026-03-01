import { ref, onMounted, onBeforeUnmount } from 'vue'
import { getBackendHealth } from '@/api/core'
import { isTauriEnvironment } from '@/api/core'
import { usePolledData } from './usePolledData'

export type BackendHealthStatus = 'unsupported' | 'unknown' | 'checking' | 'ok' | 'error'

const status = ref<BackendHealthStatus>('unknown')
const errorMessage = ref<string | null>(null)
const lastCheckedAt = ref<Date | null>(null)

let subscribers = 0

// 内部轮询实例（单例，在组件外创建以供手动管理）
let pollerStarted = false

const checkHealth = async () => {
  if (!isTauriEnvironment()) {
    status.value = 'unsupported'
    return
  }

  status.value = 'checking'
  errorMessage.value = null

  try {
    await getBackendHealth()
    status.value = 'ok'
  } catch (error) {
    status.value = 'error'
    errorMessage.value = error instanceof Error ? error.message : '无法连接后端'
  } finally {
    lastCheckedAt.value = new Date()
  }
}

// 单例轮询器：在组件外创建，由 subscribers 计数控制生命周期
let pollerInstance: ReturnType<typeof usePolledData> | null = null

const getPoller = (intervalMs: number) => {
  if (!pollerInstance) {
    pollerInstance = usePolledData(
      async () => {
        await checkHealth()
        return true
      },
      {
        intervalMs,
        pauseWhenHidden: true,
        immediate: false,
        onError: () => {
          status.value = 'error'
        },
      }
    )
    pollerStarted = false
  }
  return pollerInstance
}

export const useBackendHealth = (options?: { auto?: boolean; intervalMs?: number }) => {
  const auto = options?.auto ?? true
  const intervalMs = options?.intervalMs ?? 15000

  onMounted(() => {
    if (!auto) return
    subscribers += 1
    if (subscribers === 1) {
      const poller = getPoller(intervalMs)
      if (!pollerStarted) {
        pollerStarted = true
        poller.resume()
      }
    }
  })

  onBeforeUnmount(() => {
    if (!auto) return
    subscribers = Math.max(0, subscribers - 1)
    if (subscribers === 0 && pollerInstance) {
      pollerInstance.pause()
    }
  })

  return {
    status,
    errorMessage,
    lastCheckedAt,
    checkHealth,
  }
}
