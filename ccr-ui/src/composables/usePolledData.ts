import { useCallback, useEffect, useRef, useState } from 'react'
import { createPoller, type Poller } from '@/utils/poller'

// 轮询 composable 的 React 迁移（08-22-state-logic-port 批次 5）。
// 框架无关核心在 `utils/poller.ts`（createPoller）；本文件是薄 React 接线：
// useState 镜像 data/loading/error/isActive，useRef 持有 poller 单例，
// effect 承担原 onMounted/onBeforeUnmount（启动/停表+解绑监听）。
//
// 签名变化（相对 Vue 版，消费方均为待迁移 .vue 视图）：
// - 返回对象中的 Ref<T> 读取改为普通值（React 状态）；
// - pauseWhen 由 `WatchSource<boolean> | (() => boolean)` 改为
//   `boolean | (() => boolean)`：布尔源经 effect 调用 poller.onPauseChange
//   即时反应（原 watch 分支）；函数源由核心每 tick 求值。

export interface UsePolledDataOptions {
  /** 去重 key；相同 key 的轮询请求共用同一个 in-flight Promise */
  key?: string
  /** 轮询间隔（毫秒）；传函数则每轮重新求值，用于健康/异常自适应退避 */
  intervalMs: number | (() => number)
  /** 页面隐藏时暂停轮询（默认 true） */
  pauseWhenHidden?: boolean
  /** 自定义暂停条件：响应式布尔值或每 tick 求值的纯函数 */
  pauseWhen?: boolean | (() => boolean)
  /** 是否立即执行一次（默认 true） */
  immediate?: boolean
  /** 页面重新可见时的附加回调 */
  onVisibilityResume?: () => void | Promise<void>
  /** 错误回调 */
  onError?: (error: Error) => void
}

export interface UsePolledDataReturn<T> {
  /** 轮询数据 */
  data: T | null
  /** 是否正在加载 */
  loading: boolean
  /** 最近一次错误 */
  error: Error | null
  /** 手动刷新 */
  refresh: () => Promise<void>
  /** 暂停轮询 */
  pause: () => void
  /** 恢复轮询；默认恢复时立即刷新，可通过 immediate=false 只重启定时器 */
  resume: (options?: { immediate?: boolean }) => void
  /** 轮询是否活跃 */
  isActive: boolean
}

export function usePolledData<T>(
  fetcher: () => Promise<T>,
  options: UsePolledDataOptions
): UsePolledDataReturn<T> {
  const { pauseWhen } = options

  const [data, setData] = useState<T | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<Error | null>(null)
  const [isActive, setIsActive] = useState(false)

  // options/fetcher 经 ref 惰性读取：poller 只创建一次（原 setup 时读一次 options）
  const fetcherRef = useRef(fetcher)
  fetcherRef.current = fetcher
  const optionsRef = useRef(options)
  optionsRef.current = options

  const pollerRef = useRef<Poller<T> | null>(null)
  if (!pollerRef.current) {
    const initial = optionsRef.current
    pollerRef.current = createPoller<T>(
      async () => {
        // 状态镜像由包装 fetcher 维护；错误归一化后抛回核心走 onError 分支
        setLoading(true)
        setError(null)
        try {
          const result = await fetcherRef.current()
          setData(result)
          return result
        } catch (err) {
          throw err instanceof Error ? err : new Error(String(err))
        } finally {
          setLoading(false)
        }
      },
      {
        key: initial.key,
        intervalMs: initial.intervalMs,
        pauseWhenHidden: initial.pauseWhenHidden,
        pauseWhen: typeof initial.pauseWhen === 'function' ? initial.pauseWhen : undefined,
        immediate: initial.immediate,
        onVisibilityResume: initial.onVisibilityResume,
        onError: (e) => {
          setError(e)
          optionsRef.current.onError?.(e)
        },
      },
    )
  }
  const poller = pollerRef.current

  // 挂载启动 / 卸载清理（原 onMounted/onBeforeUnmount：停表 + 解绑 visibility 监听）；
  // StrictMode 双挂载下 start/stop 幂等（监听去重 + 定时器判空重建）
  useEffect(() => {
    poller.start()
    setIsActive(poller.isActive())
    return () => {
      poller.stop()
    }
  }, [poller])

  // 原 watch(pauseWhen as WatchSource<boolean>)（无选项）：布尔源变化的即时反应；
  // isActive 不受该分支影响（与原实现一致，仅显式 pause 会置 false）
  useEffect(() => {
    if (typeof pauseWhen === 'boolean') {
      poller.onPauseChange(pauseWhen)
    }
  }, [pauseWhen, poller])

  const refresh = useCallback(() => poller.refresh(), [poller])

  const pause = useCallback(() => {
    poller.pause()
    setIsActive(poller.isActive())
  }, [poller])

  const resume = useCallback((resumeOptions?: { immediate?: boolean }) => {
    poller.resume(resumeOptions)
    setIsActive(poller.isActive())
  }, [poller])

  return { data, loading, error, refresh, pause, resume, isActive }
}
