import { listen, type UnlistenFn } from '@tauri-apps/api/event'

/** listen() 取消协议：cleanup 已跑过时，迟到的 unlisten 立即调用。 */
export interface ListenBag {
  disposed: boolean
  unlistens: UnlistenFn[]
}

export const createListenBag = (): ListenBag => ({
  disposed: false,
  unlistens: [],
})

export const trackListen = <T>(
  event: string,
  handler: (payload: T) => void,
  bag: ListenBag,
): void => {
  const pending = listen<T>(event, (tauriEvent) => {
    if (bag.disposed) return
    handler(tauriEvent.payload)
  })
  void pending.then((unlisten) => {
    if (bag.disposed) {
      unlisten()
      return
    }
    bag.unlistens.push(unlisten)
  })
}

export const disposeListens = (bag: ListenBag): void => {
  bag.disposed = true
  const pending = bag.unlistens.splice(0)
  pending.forEach((unlisten) => unlisten())
}
