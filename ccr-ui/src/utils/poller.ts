// 框架无关轮询核心（08-22-state-logic-port 批次 5）。
// 自 `composables/usePolledData.ts`（Vue composable）抽出：`stores/usage.ts` 在
// Pinia store 内（React 之外）以同一语义调用，需在视图迁移完成前保持可用。
// React 接线见 `composables/usePolledData.ts`。
//
// 语义与原实现逐点对应：
// - 相同 key 的轮询请求共用同一个 in-flight Promise（模块级去重表）；
// - intervalMs 为函数时走递归 setTimeout（每轮按最新间隔重新调度），
//   为数字时走 setInterval；
// - pauseWhenHidden 经 document visibilitychange 监听暂停/恢复；
// - pauseWhen 为纯函数，每个 tick 求值；源变化的即时反应由宿主（hook effect
//   或外部代码）调用 `onPauseChange` 承担（原 Vue watch 分支）。

export interface PollerOptions {
  /** 去重 key；相同 key 的轮询请求共用同一个 in-flight Promise */
  key?: string
  /** 轮询间隔（毫秒）；传函数则每轮重新求值，用于健康/异常自适应退避 */
  intervalMs: number | (() => number)
  /** 页面隐藏时暂停轮询（默认 true） */
  pauseWhenHidden?: boolean
  /** 自定义暂停条件（每 tick 求值的纯函数） */
  pauseWhen?: () => boolean
  /** start() 时是否立即执行一次（默认 true） */
  immediate?: boolean
  /** 页面重新可见时的附加回调 */
  onVisibilityResume?: () => void | Promise<void>
  /** 错误回调 */
  onError?: (error: Error) => void
}

export interface Poller<T> {
  /** 轮询数据 */
  getData(): T | null
  /** 是否正在加载 */
  isLoading(): boolean
  /** 最近一次错误 */
  getError(): Error | null
  /** 轮询是否活跃 */
  isActive(): boolean
  /** 手动刷新 */
  refresh(): Promise<void>
  /** 暂停轮询 */
  pause(): void
  /** 恢复轮询；默认恢复时立即刷新，可通过 immediate=false 只重启定时器 */
  resume(options?: { immediate?: boolean }): void
  /** pauseWhen 源变化时的即时反应：暂停即停表；恢复且活跃则立即拉取并重启定时器 */
  onPauseChange(paused: boolean): void
  /** 宿主挂载接线：绑定 visibility 监听；immediate 时立即执行并启动定时器 */
  start(): void
  /** 宿主卸载清理：停表 + 解绑 visibility 监听 */
  stop(): void
}

const pollerInflightMap = new Map<string, Promise<unknown>>()

export function createPoller<T>(fetcher: () => Promise<T>, options: PollerOptions): Poller<T> {
  const {
    key,
    intervalMs,
    pauseWhenHidden = true,
    pauseWhen,
    immediate = true,
    onVisibilityResume,
    onError,
  } = options

  let data: T | null = null
  let loading = false
  let error: Error | null = null
  let isActive = false

  let timer: ReturnType<typeof setInterval> | null = null
  let inFlight = false
  let visibilityListenerAttached = false
  // 间隔为函数时走递归 setTimeout（每轮重新求值），需用 clearTimeout 取消
  const dynamicInterval = typeof intervalMs === 'function'
  const resolveInterval = (): number =>
    typeof intervalMs === 'function' ? intervalMs() : intervalMs

  const shouldPause = (): boolean => {
    if (pauseWhenHidden && typeof document !== 'undefined' && document.hidden) {
      return true
    }
    if (pauseWhen !== undefined) {
      return pauseWhen()
    }
    return false
  }

  const doFetch = async (): Promise<void> => {
    if (inFlight) return
    inFlight = true
    loading = true
    error = null
    try {
      const existing = key ? (pollerInflightMap.get(key) as Promise<T> | undefined) : undefined
      const sharedPromise = existing ?? fetcher()
      if (key && !existing) {
        pollerInflightMap.set(key, sharedPromise as Promise<unknown>)
      }
      data = await sharedPromise
    } catch (err) {
      const e = err instanceof Error ? err : new Error(String(err))
      error = e
      onError?.(e)
    } finally {
      if (key) {
        pollerInflightMap.delete(key)
      }
      loading = false
      inFlight = false
    }
  }

  const startTimer = (): void => {
    if (timer) return
    if (dynamicInterval) {
      // 递归 setTimeout：每轮按最新间隔重新调度，停止时由 stopTimer 走 clearTimeout
      const tick = (): void => {
        timer = setTimeout(() => {
          if (!shouldPause()) {
            void doFetch()
          }
          tick()
        }, resolveInterval())
      }
      tick()
      return
    }
    timer = setInterval(() => {
      if (!shouldPause()) {
        void doFetch()
      }
    }, resolveInterval())
  }

  const stopTimer = (): void => {
    if (timer) {
      if (dynamicInterval) {
        clearTimeout(timer)
      } else {
        clearInterval(timer)
      }
      timer = null
    }
  }

  const attachVisibilityListener = (): void => {
    if (!pauseWhenHidden || typeof document === 'undefined' || visibilityListenerAttached) {
      return
    }
    document.addEventListener('visibilitychange', handleVisibilityChange)
    visibilityListenerAttached = true
  }

  const detachVisibilityListener = (): void => {
    if (!pauseWhenHidden || typeof document === 'undefined' || !visibilityListenerAttached) {
      return
    }
    document.removeEventListener('visibilitychange', handleVisibilityChange)
    visibilityListenerAttached = false
  }

  const pause = (): void => {
    isActive = false
    stopTimer()
    detachVisibilityListener()
  }

  const resume = (resumeOptions: { immediate?: boolean } = {}): void => {
    attachVisibilityListener()
    if (shouldPause()) return
    isActive = true
    if (resumeOptions.immediate ?? true) {
      void doFetch()
    }
    startTimer()
  }

  const refresh = async (): Promise<void> => {
    await doFetch()
  }

  const handleVisibilityChange = (): void => {
    if (shouldPause()) {
      stopTimer()
    } else if (isActive) {
      void doFetch()
      void onVisibilityResume?.()
      startTimer()
    }
  }

  const activateIfImmediate = (): void => {
    attachVisibilityListener()
    if (immediate) {
      isActive = true
      if (!shouldPause()) {
        void doFetch()
        startTimer()
      }
    }
  }

  return {
    getData: () => data,
    isLoading: () => loading,
    getError: () => error,
    isActive: () => isActive,
    refresh,
    pause,
    resume,
    onPauseChange(paused: boolean): void {
      // 原 watch(pauseWhen) 分支：暂停只停表（isActive 保持），恢复且活跃时立即拉取
      if (paused) {
        stopTimer()
      } else if (isActive) {
        void doFetch()
        startTimer()
      }
    },
    start: activateIfImmediate,
    stop(): void {
      stopTimer()
      detachVisibilityListener()
    },
  }
}
