import { ref, watch, onMounted, onBeforeUnmount, getCurrentInstance, type Ref, type WatchSource } from 'vue'

export interface UsePolledDataOptions {
  /** 轮询间隔（毫秒） */
  intervalMs: number
  /** 页面隐藏时暂停轮询（默认 true） */
  pauseWhenHidden?: boolean
  /** 自定义暂停条件（Ref<boolean> 或返回 boolean 的函数） */
  pauseWhen?: WatchSource<boolean> | (() => boolean)
  /** 是否立即执行一次（默认 true） */
  immediate?: boolean
  /** 错误回调 */
  onError?: (error: Error) => void
}

export interface UsePolledDataReturn<T> {
  /** 轮询数据 */
  data: Ref<T | null>
  /** 是否正在加载 */
  loading: Ref<boolean>
  /** 最近一次错误 */
  error: Ref<Error | null>
  /** 手动刷新 */
  refresh: () => Promise<void>
  /** 暂停轮询 */
  pause: () => void
  /** 恢复轮询 */
  resume: () => void
  /** 轮询是否活跃 */
  isActive: Ref<boolean>
}

export function usePolledData<T>(
  fetcher: () => Promise<T>,
  options: UsePolledDataOptions
): UsePolledDataReturn<T> {
  const {
    intervalMs,
    pauseWhenHidden = true,
    pauseWhen,
    immediate = true,
    onError,
  } = options

  const data = ref<T | null>(null) as Ref<T | null>
  const loading = ref(false)
  const error = ref<Error | null>(null)
  const isActive = ref(false)

  let timer: ReturnType<typeof setInterval> | null = null
  let inFlight = false

  const shouldPause = (): boolean => {
    if (pauseWhenHidden && typeof document !== 'undefined' && document.hidden) {
      return true
    }
    if (pauseWhen !== undefined) {
      if (typeof pauseWhen === 'function') {
        return pauseWhen()
      }
      // WatchSource — treat as a Ref-like with .value
      const src = pauseWhen as { value: boolean }
      if (src && typeof src === 'object' && 'value' in src) {
        return src.value
      }
    }
    return false
  }

  const doFetch = async (): Promise<void> => {
    if (inFlight) return
    inFlight = true
    loading.value = true
    error.value = null
    try {
      data.value = await fetcher()
    } catch (err) {
      const e = err instanceof Error ? err : new Error(String(err))
      error.value = e
      onError?.(e)
    } finally {
      loading.value = false
      inFlight = false
    }
  }

  const startTimer = (): void => {
    if (timer) return
    timer = setInterval(() => {
      if (!shouldPause()) {
        void doFetch()
      }
    }, intervalMs)
  }

  const stopTimer = (): void => {
    if (timer) {
      clearInterval(timer)
      timer = null
    }
  }

  const pause = (): void => {
    isActive.value = false
    stopTimer()
  }

  const resume = (): void => {
    if (shouldPause()) return
    isActive.value = true
    void doFetch()
    startTimer()
  }

  const refresh = async (): Promise<void> => {
    await doFetch()
  }

  const handleVisibilityChange = (): void => {
    if (shouldPause()) {
      stopTimer()
    } else if (isActive.value) {
      void doFetch()
      startTimer()
    }
  }

  // Watch pauseWhen if it is a reactive source (Ref)
  if (pauseWhen !== undefined && typeof pauseWhen !== 'function') {
    watch(pauseWhen as WatchSource<boolean>, (paused) => {
      if (paused) {
        stopTimer()
      } else if (isActive.value) {
        void doFetch()
        startTimer()
      }
    })
  }

  const instance = getCurrentInstance()

  if (instance) {
    onMounted(() => {
      if (pauseWhenHidden && typeof document !== 'undefined') {
        document.addEventListener('visibilitychange', handleVisibilityChange)
      }
      if (immediate) {
        isActive.value = true
        if (!shouldPause()) {
          void doFetch()
          startTimer()
        }
      }
    })

    onBeforeUnmount(() => {
      stopTimer()
      if (pauseWhenHidden && typeof document !== 'undefined') {
        document.removeEventListener('visibilitychange', handleVisibilityChange)
      }
    })
  } else {
    // Called outside component context — auto-start if immediate
    if (immediate) {
      isActive.value = true
      if (pauseWhenHidden && typeof document !== 'undefined') {
        document.addEventListener('visibilitychange', handleVisibilityChange)
      }
      if (!shouldPause()) {
        void doFetch()
        startTimer()
      }
    }
  }

  return { data, loading, error, refresh, pause, resume, isActive }
}
