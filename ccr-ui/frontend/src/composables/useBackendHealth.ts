import { ref, onMounted, onBeforeUnmount } from 'vue'
import { getBackendHealth } from '@/api/client'
import { isTauriEnvironment } from '@/api'

export type BackendHealthStatus = 'unsupported' | 'unknown' | 'checking' | 'ok' | 'error'

const status = ref<BackendHealthStatus>('unknown')
const errorMessage = ref<string | null>(null)
const lastCheckedAt = ref<Date | null>(null)

let timer: ReturnType<typeof setInterval> | null = null
let subscribers = 0
let inFlight = false

const checkHealth = async () => {
  if (!isTauriEnvironment()) {
    status.value = 'unsupported'
    return
  }

  if (inFlight) return
  inFlight = true
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
    inFlight = false
  }
}

const startPolling = (intervalMs: number) => {
  if (timer) return
  timer = setInterval(() => {
    void checkHealth()
  }, intervalMs)
}

const stopPolling = () => {
  if (!timer) return
  clearInterval(timer)
  timer = null
}

export const useBackendHealth = (options?: { auto?: boolean; intervalMs?: number }) => {
  const auto = options?.auto ?? true
  const intervalMs = options?.intervalMs ?? 15000

  onMounted(() => {
    if (!auto) return
    subscribers += 1
    if (subscribers === 1) {
      void checkHealth()
      startPolling(intervalMs)
    }
  })

  onBeforeUnmount(() => {
    if (!auto) return
    subscribers = Math.max(0, subscribers - 1)
    if (subscribers === 0) {
      stopPolling()
    }
  })

  return {
    status,
    errorMessage,
    lastCheckedAt,
    checkHealth,
  }
}
