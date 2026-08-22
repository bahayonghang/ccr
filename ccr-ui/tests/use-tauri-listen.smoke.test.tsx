import { StrictMode } from 'react'
import { act, render } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { useTauriListen } from '@/shell/useTauriListen'

// listen() mock：返回延迟 resolve 的 deferred，unlisten 仅在 resolve 后可获取。
// 同步 resolve 的 mock 无法暴露「卸载先于 resolve」的泄漏形态（08-22-state-logic-port AC5）。
const state = vi.hoisted(() => {
  const pending: Array<{ resolve: (fn: () => void) => void }> = []
  const unlisten = vi.fn()
  return { pending, unlisten }
})

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(
    () =>
      new Promise<() => void>((resolve) => {
        state.pending.push({ resolve })
      }),
  ),
}))

const nextDeferred = (): { resolve: (fn: () => void) => void } => {
  const deferred = state.pending.shift()
  expect(deferred, 'listen() 应已被调用并挂起新的 deferred').toBeTruthy()
  return deferred as { resolve: (fn: () => void) => void }
}

/** 订阅挂载点：事件回调为空操作，仅验证订阅生命周期 */
function SubscribeHarness(): null {
  useTauriListen('app-log', () => {})
  return null
}

const renderStrict = (): ReturnType<typeof render> =>
  render(
    <StrictMode>
      <SubscribeHarness />
    </StrictMode>,
  )

describe('StrictMode 下 Tauri 订阅不翻倍（TPR-05）', () => {
  it('resolve 先于卸载：每个 mount/unmount 对恰好一次订阅与一次解绑', async () => {
    const { unmount } = renderStrict()

    // StrictMode 双调用 effect：两次订阅请求，但同一时刻至多一条活跃订阅
    expect(state.pending).toHaveLength(2)
    const first = nextDeferred()
    const second = nextDeferred()

    // 第一条订阅的 effect 已被 StrictMode 清理 → resolve 后立即补发解绑
    await act(async () => {
      first.resolve(state.unlisten)
    })
    expect(state.unlisten).toHaveBeenCalledTimes(1)

    // 第二条订阅保持活跃 → 不解绑
    await act(async () => {
      second.resolve(state.unlisten)
    })
    expect(state.unlisten).toHaveBeenCalledTimes(1)

    // 卸载 → 恰好补上第二次解绑：每条订阅配对一次解绑，活跃订阅数归零
    unmount()
    expect(state.unlisten).toHaveBeenCalledTimes(2)
  })

  it('resolve 发生在卸载之后：resolve 后立即补发解绑，无泄漏订阅', async () => {
    const { unmount } = renderStrict()

    // 卸载先于任何 resolve：此时没有任何 unlisten 可调用
    unmount()
    expect(state.unlisten).toHaveBeenCalledTimes(0)

    // 卸载后两条挂起订阅才 resolve → 各自立即补发一次解绑
    while (state.pending.length > 0) {
      const deferred = nextDeferred()
      await act(async () => {
        deferred.resolve(state.unlisten)
      })
    }
    expect(state.pending).toHaveLength(0)
    // 双挂载产生的每条订阅都恰好解绑一次：活跃订阅归零
    expect(state.unlisten).toHaveBeenCalledTimes(2)
  })
})
