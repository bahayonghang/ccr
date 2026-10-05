export const HIGH_FREQUENCY_FLUSH_INTERVAL_MS = 250

export interface EventBatcher<T> {
  push: (item: T) => void
  dispose: () => void
  commit: () => void
}

/** 高频事件缓冲：累积后定时批量提交。 */
export function createEventBatcher<T>(
  flush: (batch: T[]) => void,
  intervalMs = HIGH_FREQUENCY_FLUSH_INTERVAL_MS,
): EventBatcher<T> {
  let buffer: T[] = []
  let timer: ReturnType<typeof setInterval> | null = null

  const commit = () => {
    if (buffer.length === 0) return
    const batch = buffer
    buffer = []
    flush(batch)
  }

  const push = (item: T) => {
    buffer.push(item)
    if (timer !== null) return
    timer = setInterval(() => {
      commit()
      if (timer !== null && buffer.length === 0) {
        clearInterval(timer)
        timer = null
      }
    }, intervalMs)
  }

  const dispose = () => {
    commit()
    if (timer === null) return
    clearInterval(timer)
    timer = null
  }

  return { push, dispose, commit }
}
