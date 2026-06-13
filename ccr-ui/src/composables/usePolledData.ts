import {
  ref,
  watch,
  onMounted,
  onBeforeUnmount,
  getCurrentInstance,
  type Ref,
  type WatchSource,
} from 'vue'

export interface UsePolledDataOptions {
  /** 去重 key；相同 key 的轮询请求共用同一个 in-flight Promise */
  key?: string
  /** 轮询间隔（毫秒）；传函数则每轮重新求值，用于健康/异常自适应退避 */
  intervalMs: number | (() => number)
  /** 页面隐藏时暂停轮询（默认 true） */
  pauseWhenHidden?: boolean
  /** 自定义暂停条件（Ref<boolean> 或返回 boolean 的函数） */
  pauseWhen?: WatchSource<boolean> | (() => boolean)
  /** 是否立即执行一次（默认 true） */
  immediate?: boolean
  /** 页面重新可见时的附加回调 */
  onVisibilityResume?: () => void | Promise<void>
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
  /** 恢复轮询；默认恢复时立即刷新，可通过 immediate=false 只重启定时器 */
  resume: (options?: { immediate?: boolean }) => void
  /** 轮询是否活跃 */
  isActive: Ref<boolean>
}

export function usePolledData<T>(
  fetcher: () => Promise<T>,
  options: UsePolledDataOptions
): UsePolledDataReturn<T> {
  const {
    key,
    intervalMs,
    pauseWhenHidden = true,
    pauseWhen,
    immediate = true,
    onVisibilityResume,
    onError,
  } = options

  const data = ref<T | null>(null) as Ref<T | null>
  const loading = ref(false)
  const error = ref<Error | null>(null)
  const isActive = ref(false)

  let timer: ReturnType<typeof setInterval> | null = null
  let inFlight = false
  let visibilityListenerAttached = false
  // 间隔为函数时走递归 setTimeout（每轮重新求值），需用 clearTimeout 取消
  const dynamicInterval = typeof intervalMs === 'function'
  const resolveInterval = (): number =>
    typeof intervalMs === 'function' ? intervalMs() : intervalMs

  const inFlightByKey = usePolledDataInflightMap

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
      if (key) {
        const sharedPromise = (inFlightByKey.get(key) as Promise<T> | undefined) ?? fetcher()
        if (!inFlightByKey.has(key)) {
          inFlightByKey.set(key, sharedPromise as Promise<unknown>)
        }
        data.value = await sharedPromise
      } else {
        data.value = await fetcher()
      }
    } catch (err) {
      const e = err instanceof Error ? err : new Error(String(err))
      error.value = e
      onError?.(e)
    } finally {
      if (key) {
        inFlightByKey.delete(key)
      }
      loading.value = false
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
    isActive.value = false
    stopTimer()
    detachVisibilityListener()
  }

  const resume = (resumeOptions: { immediate?: boolean } = {}): void => {
    attachVisibilityListener()
    if (shouldPause()) return
    isActive.value = true
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
    } else if (isActive.value) {
      void doFetch()
      void onVisibilityResume?.()
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
      attachVisibilityListener()
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
      detachVisibilityListener()
    })
  } else {
    // Called outside component context — auto-start if immediate
    if (immediate) {
      isActive.value = true
      attachVisibilityListener()
      if (!shouldPause()) {
        void doFetch()
        startTimer()
      }
    }
  }

  return { data, loading, error, refresh, pause, resume, isActive }
}

const usePolledDataInflightMap = new Map<string, Promise<unknown>>()
