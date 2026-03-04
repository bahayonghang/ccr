import { ref, onMounted, onBeforeUnmount } from 'vue'

export type BackendHealthStatus = 'unsupported' | 'unknown' | 'checking' | 'ok' | 'error'

const status = ref<BackendHealthStatus>('unknown')
const errorMessage = ref<string | null>(null)
const lastCheckedAt = ref<Date | null>(null)

/**
 * 原生 Tauri 模式下，后端内嵌在应用中，始终可用。
 * 健康检查直接返回 ok，无需 HTTP 轮询。
 */
const checkHealth = async () => {
  // 在 Tauri 原生模式下，后端是内嵌的，始终可用
  status.value = 'ok'
  errorMessage.value = null
  lastCheckedAt.value = new Date()
}

let subscribers = 0
let pollerTimer: ReturnType<typeof setInterval> | null = null

export const useBackendHealth = (options?: { auto?: boolean; intervalMs?: number }) => {
  const auto = options?.auto ?? true

  onMounted(() => {
    if (!auto) return
    subscribers += 1
    if (subscribers === 1) {
      // 立即检查一次
      checkHealth()
    }
  })

  onBeforeUnmount(() => {
    if (!auto) return
    subscribers = Math.max(0, subscribers - 1)
    if (subscribers === 0 && pollerTimer) {
      clearInterval(pollerTimer)
      pollerTimer = null
    }
  })

  return {
    status,
    errorMessage,
    lastCheckedAt,
    checkHealth,
  }
}
