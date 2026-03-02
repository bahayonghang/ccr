/**
 * API Client Core - Axios 实例和拦截器配置
 *
 * 这是所有 API 模块的基础，提供统一的 axios 实例
 * 支持 Web 和 Tauri 两种运行环境
 */

import axios, { type AxiosInstance } from 'axios'
import { logger } from '@/utils/logger'

// ═══════════════════════════════════════════════════════════
// 🔍 环境检测 (Environment Detection)
// ═══════════════════════════════════════════════════════════

/** 检测是否在 Tauri 桌面应用环境中运行 */
export const isTauriEnvironment = (): boolean => {
  return typeof window !== 'undefined' && '__TAURI__' in window
}

/** 根据运行环境解析 API 基础 URL */
export const resolveApiBaseUrl = (): string => {
  if (isTauriEnvironment()) {
    const port = import.meta.env.VITE_TAURI_BACKEND_PORT || '38081'
    return `http://127.0.0.1:${port}/api`
  }
  return '/api'
}

/** 健康检查（等待后端启动） */
export const getBackendHealth = async (): Promise<void> => {
  const baseUrl = resolveApiBaseUrl()
  const rootUrl = baseUrl.endsWith('/api') ? baseUrl.slice(0, -4) : baseUrl
  const timeout = isTauriEnvironment() ? 20000 : 4000
  await axios.get(`${rootUrl}/health`, { timeout })
}

// ═══════════════════════════════════════════════════════════
// 🔧 Axios 实例 (Axios Instance)
// ═══════════════════════════════════════════════════════════

/**
 * 创建配置好的 axios 实例
 */
function createApiClient(): AxiosInstance {
  const instance = axios.create({
    baseURL: resolveApiBaseUrl(),
    timeout: 15000, // 15秒默认超时
    headers: {
      'Content-Type': 'application/json',
    },
  })

  // 请求拦截器
  instance.interceptors.request.use(
    (config) => {
      if (import.meta.env.DEV) {
        logger.debug(`[API] ${config.method?.toUpperCase()} ${config.url}`)
      }
      return config
    },
    (error) => {
      logger.error('[API] Request error', error)
      return Promise.reject(error)
    }
  )

  // 响应拦截器：统一处理 ApiResponse 格式的响应
  instance.interceptors.response.use(
    (response) => {
      if (import.meta.env.DEV) {
        logger.debug('API Response:', response.data)
      }

      // 如果响应是 ApiResponse 格式（包含 success 字段）
      if (response.data && typeof response.data.success === 'boolean') {
        // success=false 时统一抛出错误
        if (!response.data.success) {
          const errorMessage = response.data.message || 'Unknown API error'
          logger.error('[API] API returned error', errorMessage)
          return Promise.reject(new Error(errorMessage))
        }
      }

      return response
    },
    (error) => {
      // 统一处理网络错误和 HTTP 错误
      let errorMessage = 'Network error or server unreachable'

      if (error.response) {
        // 服务器返回了错误状态码
        const status = error.response.status
        const data = error.response.data

        if (typeof data === 'string' && data.trim()) {
          errorMessage = data
        } else if (data && typeof data.message === 'string') {
          errorMessage = data.message
        } else if (data && typeof data.error === 'string') {
          errorMessage = data.error
        } else {
          errorMessage = `HTTP ${status}: ${error.message}`
        }
      } else if (error.request) {
        // 请求已发送但没有收到响应
        errorMessage = 'No response from server'
      } else {
        // 请求配置出错
        errorMessage = error.message
      }

      logger.error('[API] Response error', errorMessage)
      return Promise.reject(new Error(errorMessage))
    }
  )

  return instance
}

/**
 * 共享的 axios 实例
 * 所有 API 模块都应该使用这个实例
 */
export const api = createApiClient()

export default api
