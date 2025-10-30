import { ref, onUnmounted } from 'vue'

/**
 * SSE (Server-Sent Events) 流式读取
 * @param url SSE 端点 URL
 */
export async function* sse(url: string): AsyncGenerator<string> {
  const response = await fetch(url, {
    headers: {
      Accept: 'text/event-stream'
    }
  })

  if (!response.ok) {
    throw new Error(`SSE connection failed: ${response.status} ${response.statusText}`)
  }

  if (!response.body) {
    throw new Error('Response body is null')
  }

  const reader = response.body.getReader()
  const decoder = new TextDecoder()

  try {
    while (true) {
      const { value, done } = await reader.read()
      if (done) break

      const chunk = decoder.decode(value, { stream: true })
      yield chunk
    }
  } finally {
    reader.releaseLock()
  }
}

/**
 * 使用流式输出的 composable
 * @param url 流式端点 URL
 * @param maxLines 最大行数限制，默认 2000
 */
export function useStream(url: string, maxLines = 2000) {
  const lines = ref<string[]>([])
  const isStreaming = ref(false)
  const error = ref<string | null>(null)
  const isComplete = ref(false)

  let abortController: AbortController | null = null

  /**
   * 开始读取流
   */
  const start = async () => {
    if (isStreaming.value) {
      console.warn('Stream is already running')
      return
    }

    isStreaming.value = true
    error.value = null
    isComplete.value = false
    lines.value = []

    abortController = new AbortController()

    try {
      const response = await fetch(url, {
        headers: {
          Accept: 'text/event-stream'
        },
        signal: abortController.signal
      })

      if (!response.ok) {
        throw new Error(`Stream failed: ${response.status} ${response.statusText}`)
      }

      if (!response.body) {
        throw new Error('Response body is null')
      }

      const reader = response.body.getReader()
      const decoder = new TextDecoder()

      while (true) {
        const { value, done } = await reader.read()
        
        if (done) {
          isComplete.value = true
          break
        }

        const chunk = decoder.decode(value, { stream: true })
        
        // 按行分割
        const newLines = chunk.split('\n').filter(line => line.trim())
        
        for (const line of newLines) {
          lines.value.push(line)
          
          // 限制行数，移除最旧的行
          if (lines.value.length > maxLines) {
            lines.value.splice(0, lines.value.length - maxLines)
          }
        }
      }
    } catch (err: any) {
      if (err.name !== 'AbortError') {
        error.value = err.message || 'Stream error'
        console.error('[Stream]', err)
      }
    } finally {
      isStreaming.value = false
      abortController = null
    }
  }

  /**
   * 停止流式读取
   */
  const stop = () => {
    if (abortController) {
      abortController.abort()
      abortController = null
    }
    isStreaming.value = false
  }

  /**
   * 清除内容
   */
  const clear = () => {
    lines.value = []
    error.value = null
    isComplete.value = false
  }

  // 组件卸载时自动停止
  onUnmounted(() => {
    stop()
  })

  return {
    lines,
    isStreaming,
    isComplete,
    error,
    start,
    stop,
    clear
  }
}

/**
 * 使用简单的文本流（非 SSE）
 * 适用于普通的 chunked 响应
 */
export function useTextStream(url: string, maxLines = 2000) {
  const lines = ref<string[]>([])
  const isStreaming = ref(false)
  const error = ref<string | null>(null)

  let abortController: AbortController | null = null

  const start = async () => {
    isStreaming.value = true
    error.value = null
    lines.value = []

    abortController = new AbortController()

    try {
      for await (const chunk of sse(url)) {
        const newLines = chunk.split('\n').filter(line => line.trim())
        
        for (const line of newLines) {
          lines.value.push(line)
          if (lines.value.length > maxLines) {
            lines.value.shift()
          }
        }
      }
    } catch (err: any) {
      if (err.name !== 'AbortError') {
        error.value = err.message || 'Stream error'
      }
    } finally {
      isStreaming.value = false
    }
  }

  const stop = () => {
    if (abortController) {
      abortController.abort()
    }
    isStreaming.value = false
  }

  const clear = () => {
    lines.value = []
    error.value = null
  }

  onUnmounted(() => {
    stop()
  })

  return {
    lines,
    isStreaming,
    error,
    start,
    stop,
    clear
  }
}
