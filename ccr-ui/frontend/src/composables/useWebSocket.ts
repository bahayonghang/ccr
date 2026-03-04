// Tauri 事件监听 composable（替代原 WebSocket 连接）
// 使用 Tauri listen() API 接收后端事件推送

import { ref, onMounted, onUnmounted, type Ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { logger } from '@/utils/logger'

export interface LogMessage {
    id: string
    timestamp: string
    level: 'debug' | 'info' | 'warn' | 'error'
    source: string
    message: string
    metadata?: Record<string, unknown>
}

export interface TokenStats {
    input_tokens: number
    output_tokens: number
    cache_tokens: number
    request_count: number
    estimated_cost_cents: number
    last_updated: string
}

// 保留旧接口以兼容已有代码
export interface WsMessage {
    type: 'Log' | 'TokenStats' | 'ProxyState' | 'Ping' | 'Pong' | 'Error' | 'LogBatch'
    data?: LogMessage | TokenStats | LogMessage[] | { message: string }
}

export interface UseWebSocketOptions {
    url?: string
    reconnectInterval?: number
    maxReconnectAttempts?: number
    onLog?: (log: LogMessage) => void
    onTokenStats?: (stats: TokenStats) => void
    onError?: (error: string) => void
}

/**
 * 使用 Tauri 事件系统替代 WebSocket 连接
 *
 * 后端通过 `app_handle.emit()` 发送事件，前端通过 `listen()` 接收。
 * 保持与旧 useWebSocket 相同的返回接口，确保上层代码无需修改。
 */
export function useWebSocket(options: UseWebSocketOptions = {}) {
    const { onLog, onTokenStats, onError } = options

    const isConnected: Ref<boolean> = ref(true) // Tauri 模式下始终"连接"
    const logs: Ref<LogMessage[]> = ref([])
    const tokenStats: Ref<TokenStats | null> = ref(null)
    const reconnectAttempts = ref(0)
    const isVisible = ref(!document.hidden)

    const unlisteners: UnlistenFn[] = []

    const setupListeners = async () => {
        try {
            // 监听日志事件
            const unLog = await listen<LogMessage>('app-log', (event) => {
                const log = event.payload
                logs.value.push(log)
                if (logs.value.length > 500) {
                    logs.value.shift()
                }
                onLog?.(log)
            })
            unlisteners.push(unLog)

            // 监听 token 统计事件
            const unStats = await listen<TokenStats>('token-stats', (event) => {
                tokenStats.value = event.payload
                onTokenStats?.(tokenStats.value)
            })
            unlisteners.push(unStats)

            // 监听错误事件
            const unErr = await listen<{ message: string }>('app-error', (event) => {
                onError?.(event.payload.message)
            })
            unlisteners.push(unErr)

            // 加载最近事件
            try {
                const recent = await invoke<LogMessage[]>('get_recent_events')
                if (recent && recent.length > 0) {
                    logs.value = recent
                }
            } catch {
                logger.debug('[TauriEvents] No recent events available')
            }

            isConnected.value = true
            logger.debug('[TauriEvents] Event listeners registered')
        } catch (e) {
            logger.error('[TauriEvents] Failed to setup listeners', e)
            isConnected.value = false
        }
    }

    const connect = () => {
        // 兼容旧接口 — Tauri 模式下自动设置监听
        setupListeners()
    }

    const disconnect = () => {
        unlisteners.forEach(fn => fn())
        unlisteners.length = 0
        isConnected.value = false
    }

    const send = (_message: WsMessage) => {
        // Tauri 模式下不需要发送消息到后端（后端内嵌）
        logger.debug('[TauriEvents] send() is no-op in native mode')
    }

    const clearLogs = () => {
        logs.value = []
    }

    onMounted(() => {
        setupListeners()
    })

    onUnmounted(() => {
        disconnect()
    })

    return {
        isConnected,
        logs,
        tokenStats,
        reconnectAttempts,
        isVisible,
        connect,
        disconnect,
        send,
        clearLogs
    }
}
