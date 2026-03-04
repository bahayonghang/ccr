import { ref } from 'vue'

/**
 * API 错误接口
 */
export interface ApiError {
  code: number
  message: string
  details?: unknown
}

/**
 * 带重试的请求封装
 * @param fn 请求函数
 * @param retries 重试次数
 * @param delay 重试延迟（毫秒）
 */
export async function withRetry<T>(
  fn: () => Promise<T>,
  retries = 3,
  delay = 1000,
  signal?: AbortSignal
): Promise<T> {
  try {
    return await fn()
  } catch (error) {
    if (signal?.aborted) {
      throw error // 已取消，不再重试
    }
    if (retries > 0) {
      await new Promise((resolve, reject) => {
        const timer = setTimeout(resolve, delay)
        signal?.addEventListener('abort', () => {
          clearTimeout(timer)
          reject(new DOMException('Aborted', 'AbortError'))
        }, { once: true })
      })
      return withRetry(fn, retries - 1, delay * 2, signal) // 指数退避
    }
    throw error
  }
}

/**
 * useApiRequest composable
 * 提供响应式的加载状态管理（与 Tauri invoke 兼容）
 */
export function useApiRequest<T>() {
  const loading = ref(false)
  const error = ref<ApiError | null>(null)
  const data = ref<T | null>(null)

  const execute = async (fn: () => Promise<T>) => {
    loading.value = true
    error.value = null

    try {
      data.value = await fn()
      return data.value
    } catch (err) {
      error.value = err as ApiError
      throw err
    } finally {
      loading.value = false
    }
  }

  return {
    loading,
    error,
    data,
    execute
  }
}
