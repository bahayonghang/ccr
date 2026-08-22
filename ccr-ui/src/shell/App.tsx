import { useCallback, useState } from 'react'
import { systemApi } from '@/api'
import { useTauriListen } from './useTauriListen'

type VersionInfo = Awaited<ReturnType<typeof systemApi.getVersion>>

/** 阶段 1 最小页面使用硬编码中文文案；i18n Provider 由 08-22-i18n-port 在本层之外补入。 */
function BackendVersionCard() {
  const [result, setResult] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  const fetchVersion = useCallback(async (): Promise<void> => {
    setLoading(true)
    setError(null)
    try {
      const info: VersionInfo = await systemApi.getVersion()
      setResult(JSON.stringify(info))
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause))
    } finally {
      setLoading(false)
    }
  }, [])

  return (
    <section>
      <h2>后端 IPC 自检</h2>
      <p>点击按钮调用 src/api 下已有的 check_version wrapper，并显示后端返回值。</p>
      <button type="button" onClick={() => void fetchVersion()} disabled={loading}>
        {loading ? '查询中…' : '调用 check_version'}
      </button>
      {result !== null && (
        <pre data-testid="ipc-result">{result}</pre>
      )}
      {error !== null && <p role="alert">IPC 调用失败：{error}</p>}
    </section>
  )
}

interface AppLogPayload {
  message?: string
}

/** 带 listen() 订阅的示例组件：StrictMode 双挂载下活跃订阅数不翻倍。 */
function EventSubscriptionCard() {
  const [count, setCount] = useState(0)
  const [lastMessage, setLastMessage] = useState<string | null>(null)

  const onEvent = useCallback((payload: AppLogPayload) => {
    setCount((current) => current + 1)
    setLastMessage(typeof payload.message === 'string' ? payload.message : JSON.stringify(payload))
  }, [])

  useTauriListen<AppLogPayload>('app-log', onEvent)

  return (
    <section>
      <h2>Tauri 事件订阅示例</h2>
      <p data-testid="event-count">已收到 app-log 事件：{count} 条</p>
      {lastMessage !== null && <p data-testid="event-last">最近一条：{lastMessage}</p>}
    </section>
  )
}

export function App() {
  return (
    <main>
      <h1>CCR UI — React 基座</h1>
      <p>阶段 1 最小页面（react-foundation 批次 1）。路由、状态、i18n 由后续子任务填充。</p>
      <BackendVersionCard />
      <EventSubscriptionCard />
    </main>
  )
}


