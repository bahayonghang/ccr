type IdleCapableWindow = Window & {
  requestIdleCallback?: (callback: IdleRequestCallback, options?: IdleRequestOptions) => number
  cancelIdleCallback?: (handle: number) => void
}

interface IdleTaskOptions {
  timeout?: number
  fallbackDelay?: number
}

const noop = () => undefined

// 将非关键任务推迟到首帧之后，避免与首屏渲染抢占主线程。
export const scheduleAfterPaint = (task: () => void): (() => void) => {
  if (typeof window === 'undefined') {
    task()
    return noop
  }

  let cancelled = false
  const frameId = window.requestAnimationFrame(() => {
    if (cancelled) return
    task()
  })

  return () => {
    cancelled = true
    window.cancelAnimationFrame(frameId)
  }
}

// 将非关键任务放到浏览器空闲阶段执行，并保留超时兜底。
export const scheduleWhenIdle = (
  task: () => void,
  options: IdleTaskOptions = {},
): (() => void) => {
  if (typeof window === 'undefined') {
    task()
    return noop
  }

  const win = window as IdleCapableWindow
  const timeout = options.timeout ?? 1200
  const fallbackDelay = options.fallbackDelay ?? timeout

  let executed = false
  let idleHandle: number | null = null
  let timeoutHandle: number | null = null

  const runTask = () => {
    if (executed) return
    executed = true

    if (idleHandle !== null && typeof win.cancelIdleCallback === 'function') {
      win.cancelIdleCallback(idleHandle)
    }
    if (timeoutHandle !== null) {
      window.clearTimeout(timeoutHandle)
    }

    task()
  }

  if (typeof win.requestIdleCallback === 'function') {
    idleHandle = win.requestIdleCallback(() => {
      runTask()
    }, { timeout })
  }

  timeoutHandle = window.setTimeout(() => {
    runTask()
  }, fallbackDelay)

  return () => {
    executed = true

    if (idleHandle !== null && typeof win.cancelIdleCallback === 'function') {
      win.cancelIdleCallback(idleHandle)
    }
    if (timeoutHandle !== null) {
      window.clearTimeout(timeoutHandle)
    }
  }
}
