import { ref } from 'vue'
import { api } from '@/api/client'
import type { AxiosRequestConfig } from 'axios'

/**
 * API 错误接口
 */
export interface ApiError {
  code: number
  message: string
  details?: any
}

/**
 * 通用 GET 请求
 */
export async function get<T = any>(url: string, config?: AxiosRequestConfig): Promise<T> {
  const response = await api.get<T>(url, config)
  return response.data
}

/**
 * 通用 POST 请求
 */
export async function post<T = any>(
  url: string,
  data?: any,
  config?: AxiosRequestConfig
): Promise<T> {
  const response = await api.post<T>(url, data, config)
  return response.data
}

/**
 * 通用 PUT 请求
 */
export async function put<T = any>(
  url: string,
  data?: any,
  config?: AxiosRequestConfig
): Promise<T> {
  const response = await api.put<T>(url, data, config)
  return response.data
}

/**
 * 通用 DELETE 请求
 */
export async function del<T = any>(url: string, config?: AxiosRequestConfig): Promise<T> {
  const response = await api.delete<T>(url, config)
  return response.data
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
  delay = 1000
): Promise<T> {
  try {
    return await fn()
  } catch (error) {
    if (retries > 0) {
      await new Promise((resolve) => setTimeout(resolve, delay))
      return withRetry(fn, retries - 1, delay * 2) // 指数退避
    }
    throw error
  }
}

/**
 * useApi composable
 * 提供响应式的加载状态管理
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

// Re-export for convenience
export { api as default }