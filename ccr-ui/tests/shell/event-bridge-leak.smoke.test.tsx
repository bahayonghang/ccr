import { StrictMode } from 'react'
import { act, render } from '@testing-library/react'
import type { ReactNode } from 'react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { TAURI_GLOBAL_EVENTS, useTauriEventBridge } from '@/shell/eventBridge'

// 订阅泄漏检测（08-22-state-logic-port AC5，design.md §7 三用例）。
//
const bridge = vi.hoisted(() => {
  const pending: Array<{ resolve: (unlisten: () => void) => void; unlisten: () => void }> = []
  const counters = { listen: 0, unlisten: 0 }
  let immediateResolve = false

  return {
    pending,
    counters,
    setImmediateResolve(value: boolean) {
      immediateResolve = value
    },
    listen() {
      counters.listen++
      const unlisten = () => {
        counters.unlisten++
      }
      if (immediateResolve) {
        return Promise.resolve(unlisten)
      }
      // deferred 模式：计数解绑函数随 deferred 一起交付，测试方 resolve 时必须
      // 回传 entry.unlisten，与真实 listen 的返回契约一致。
      return new Promise<typeof unlisten>((resolve) => {
        pending.push({ resolve, unlisten })
      })
    },
    reset() {
      pending.length = 0
      counters.listen = 0
      counters.unlisten = 0
      immediateResolve = false
    },
  }
})

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => bridge.listen()),
}))

// listenSafe 经 isTauriRuntime() 判定后才走真实 listen mock；置上运行时标记。
beforeEach(() => {
  bridge.reset()
  ;(window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {}
})

afterEach(() => {
  delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__
})

// 三处挂载共用同一客户端形态（重试关闭，保证计数确定）。
const newQueryClient = () =>
  new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })

const createWrapper = () => {
  const client = newQueryClient()
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  )
  return Wrapper
}

/** 订阅挂载点：只验证桥接层的订阅生命周期 */
function BridgeHarness(): null {
  useTauriEventBridge()
  return null
}

describe('eventBridge 订阅泄漏（AC5，design.md §7）', () => {
  it('用例 1：listen 立即 resolve，100 次挂载/卸载后 listen 与 unlisten 计数相等', async () => {
    bridge.setImmediateResolve(true)
    const Wrapper = createWrapper()

    // 首次挂载测出每次挂载建立的订阅数（事件清单 N 项，snapshot-updated 双路失效故为 N+1）
    const probe = render(<BridgeHarness />, { wrapper: Wrapper })
    await act(async () => {})
    probe.unmount()
    const perMount = bridge.counters.listen

    for (let i = 1; i < 100; i++) {
      const view = render(<BridgeHarness />, { wrapper: Wrapper })
      await act(async () => {})
      view.unmount()
    }

    expect(perMount).toBe(TAURI_GLOBAL_EVENTS.length + 1)
    expect(bridge.counters.listen).toBe(perMount * 100)
    expect(bridge.counters.unlisten).toBe(bridge.counters.listen)
    expect(bridge.pending).toHaveLength(0)
  })

  it('用例 2：卸载之后 listen 才 resolve，迟到的 unlisten 仍被调用（取消协议）', async () => {
    const Wrapper = createWrapper()
    const view = render(<BridgeHarness />, { wrapper: Wrapper })

    // 卸载先于任何 resolve：cleanup 同步执行，此刻没有任何 unlisten 可调用
    view.unmount()
    expect(bridge.counters.unlisten).toBe(0)
    expect(bridge.counters.listen).toBeGreaterThan(0)
    expect(bridge.pending).toHaveLength(bridge.counters.listen)

    // 卸载后挂起订阅才 resolve → 取消协议要求每条迟到 unlisten 立即补发。
    // naive「无条件 push 进数组」实现此处计数为 0，用例失败。
    while (bridge.pending.length > 0) {
      const deferred = bridge.pending.shift()
      expect(deferred).toBeTruthy()
      await act(async () => {
        deferred?.resolve(deferred.unlisten)
      })
    }

    expect(bridge.counters.unlisten).toBe(bridge.counters.listen)
  })

  it('用例 3：StrictMode 挂载→卸载→再挂载，延迟 resolve 后计数相等', async () => {
    const view = render(
      <StrictMode>
        <QueryClientProvider client={newQueryClient()}>
          <BridgeHarness />
        </QueryClientProvider>
      </StrictMode>,
    )

    // StrictMode effect 双调用：第一轮订阅请求在 cleanup 中被放弃但尚未 resolve，
    // 第二轮保持活跃——两轮全部挂起
    const total = bridge.counters.listen
    expect(total).toBe((TAURI_GLOBAL_EVENTS.length + 1) * 2)
    expect(bridge.pending).toHaveLength(total)

    // 第一轮（已被 StrictMode 清理）：延迟 resolve 后必须立即补发解绑
    const firstBatch = bridge.pending.splice(0, total / 2)
    for (const deferred of firstBatch) {
      await act(async () => {
        deferred.resolve(deferred.unlisten)
      })
    }
    expect(bridge.counters.unlisten).toBe(total / 2)

    // 卸载后再 resolve 剩余第二轮订阅 → 全部补发解绑
    view.unmount()
    while (bridge.pending.length > 0) {
      const deferred = bridge.pending.shift()
      expect(deferred).toBeTruthy()
      await act(async () => {
        deferred?.resolve(deferred.unlisten)
      })
    }

    expect(bridge.counters.listen).toBe(total)
    expect(bridge.counters.unlisten).toBe(total)
  })
})
