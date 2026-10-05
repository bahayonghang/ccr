import { useEffect, useRef } from 'react'
import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'

/**
 * StrictMode 安全的 Tauri 事件订阅。
 *
 * 订阅的建立与解绑保持在组件生命周期内；卸载若先于 listen() resolve，
 * 则在 resolve 后立即补发一次 unlisten。两条路径由同一个 called 标志收敛，
 * 保证每个订阅恰好配对一次解绑——StrictMode 双挂载下活跃订阅数不翻倍。
 *
 * 本实现是后续子任务的订阅写法参照（父任务 design.md §4：事件回调中做
 * queryClient.invalidateQueries / setQueryData，不在回调外持有数据）。
 */
export function useTauriListen<T>(event: string, onEvent: (payload: T) => void): void {
  // 用 ref 持有最新回调，避免回调身份变化触发重新订阅。
  const handlerRef = useRef(onEvent)
  handlerRef.current = onEvent

  useEffect(() => {
    let disposed = false
    let called = false
    let unlistenFn: UnlistenFn | undefined

    const unlistenOnce = (fn: UnlistenFn): void => {
      if (called) {
        return
      }
      called = true
      fn()
    }

    const subscription = listen<T>(event, (tauriEvent) => {
      handlerRef.current(tauriEvent.payload)
    })

    void subscription.then((fn) => {
      if (disposed) {
        // resolve 发生在卸载之后：立即补发解绑。
        unlistenOnce(fn)
        return
      }
      unlistenFn = fn
    })

    return () => {
      disposed = true
      if (unlistenFn) {
        unlistenOnce(unlistenFn)
      }
    }
  }, [event])
}
