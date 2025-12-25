/**
 * API Client Core - Axios 实例和拦截器配置
 * 
 * 这是所有 API 模块的基础，提供统一的 axios 实例
 */

import axios, { type AxiosInstance } from 'axios'

/**
 * 创建配置好的 axios 实例
 */
function createApiClient(): AxiosInstance {
    const instance = axios.create({
        baseURL: '/api',
        timeout: 600000, // 10分钟超时，支持长时间编译更新
        headers: {
            'Content-Type': 'application/json',
        },
    })

    // 请求拦截器
    instance.interceptors.request.use(
        (config) => {
            if (import.meta.env.DEV) {
                console.log(`[API] ${config.method?.toUpperCase()} ${config.url}`)
            }
            return config
        },
        (error) => {
            console.error('[API] Request error:', error)
            return Promise.reject(error)
        }
    )

    // 响应拦截器：统一处理 ApiResponse 格式的响应
    instance.interceptors.response.use(
        (response) => {
            if (import.meta.env.DEV) {
                console.log(`[API] Response:`, response.data)
            }

            // 如果响应是 ApiResponse 格式（包含 success 字段）
            if (response.data && typeof response.data.success === 'boolean') {
                // success=false 时统一抛出错误
                if (!response.data.success) {
                    const errorMessage = response.data.message || 'Unknown API error'
                    console.error('[API] API returned error:', errorMessage)
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

            console.error('[API] Response error:', errorMessage)
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
