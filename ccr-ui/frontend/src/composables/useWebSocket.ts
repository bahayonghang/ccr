// WebSocket 连接管理 composable
// 提供自动重连、消息分发等功能

import { ref, onMounted, onUnmounted, type Ref } from 'vue'

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

export function useWebSocket(options: UseWebSocketOptions = {}) {
    const {
        url = `ws://${window.location.hostname}:38081/ws`,
        reconnectInterval = 3000,
        maxReconnectAttempts = 5,
        onLog,
        onTokenStats,
        onError
    } = options

    const isConnected: Ref<boolean> = ref(false)
    const logs: Ref<LogMessage[]> = ref([])
    const tokenStats: Ref<TokenStats | null> = ref(null)
    const reconnectAttempts = ref(0)

    let ws: WebSocket | null = null
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null

    const connect = () => {
        if (ws?.readyState === WebSocket.OPEN) return

        try {
            ws = new WebSocket(url)

            ws.onopen = () => {
                isConnected.value = true
                reconnectAttempts.value = 0
                console.log('[WebSocket] Connected')
            }

            ws.onmessage = (event) => {
                try {
                    const message: WsMessage = JSON.parse(event.data)
                    handleMessage(message)
                } catch (e) {
                    console.error('[WebSocket] Failed to parse message:', e)
                }
            }

            ws.onclose = () => {
                isConnected.value = false
                console.log('[WebSocket] Disconnected')
                scheduleReconnect()
            }

            ws.onerror = (error) => {
                console.error('[WebSocket] Error:', error)
                onError?.('WebSocket connection error')
            }
        } catch (e) {
            console.error('[WebSocket] Failed to connect:', e)
            scheduleReconnect()
        }
    }

    const handleMessage = (message: WsMessage) => {
        switch (message.type) {
            case 'Log':
                if (message.data) {
                    const log = message.data as LogMessage
                    logs.value.push(log)
                    // Keep max 500 logs
                    if (logs.value.length > 500) {
                        logs.value.shift()
                    }
                    onLog?.(log)
                }
                break

            case 'LogBatch':
                if (message.data && Array.isArray(message.data)) {
                    logs.value = message.data as LogMessage[]
                }
                break

            case 'TokenStats':
                if (message.data) {
                    tokenStats.value = message.data as TokenStats
                    onTokenStats?.(tokenStats.value)
                }
                break

            case 'Error':
                if (message.data && 'message' in message.data) {
                    onError?.(message.data.message)
                }
                break

            case 'Ping':
                // Respond with Pong
                send({ type: 'Pong' })
                break
        }
    }

    const scheduleReconnect = () => {
        if (reconnectAttempts.value >= maxReconnectAttempts) {
            console.log('[WebSocket] Max reconnect attempts reached')
            return
        }

        if (reconnectTimer) {
            clearTimeout(reconnectTimer)
        }

        reconnectTimer = setTimeout(() => {
            reconnectAttempts.value++
            console.log(`[WebSocket] Reconnecting... (attempt ${reconnectAttempts.value})`)
            connect()
        }, reconnectInterval)
    }

    const send = (message: WsMessage) => {
        if (ws?.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify(message))
        }
    }

    const disconnect = () => {
        if (reconnectTimer) {
            clearTimeout(reconnectTimer)
            reconnectTimer = null
        }
        if (ws) {
            ws.close()
            ws = null
        }
        isConnected.value = false
    }

    const clearLogs = () => {
        logs.value = []
    }

    onMounted(() => {
        connect()
    })

    onUnmounted(() => {
        disconnect()
    })

    return {
        isConnected,
        logs,
        tokenStats,
        reconnectAttempts,
        connect,
        disconnect,
        send,
        clearLogs
    }
}
