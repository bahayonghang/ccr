import { describe, expect, it } from 'vitest'
import {
  BALANCE_REFRESH_MIN_INTERVAL_MS,
  runPerKeySequential,
  shouldSkipBalanceRefresh,
} from '@/views/checkin/composables/balanceRefreshQueue'

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))

describe('balance refresh queue smoke', () => {
  it('runs same-key tasks sequentially and caps global concurrency', async () => {
    let active = 0
    let maxActive = 0
    const activePerKey = new Map<string, number>()
    const maxActivePerKey = new Map<string, number>()

    const makeTask = (key: string, value: string) => ({
      key,
      run: async () => {
        active += 1
        maxActive = Math.max(maxActive, active)
        const keyActive = (activePerKey.get(key) ?? 0) + 1
        activePerKey.set(key, keyActive)
        maxActivePerKey.set(key, Math.max(maxActivePerKey.get(key) ?? 0, keyActive))
        await delay(5)
        activePerKey.set(key, (activePerKey.get(key) ?? 1) - 1)
        active -= 1
        return value
      },
    })

    // 20 个任务分布在 8 个 origin（模拟 20 账号批量刷余额）
    const tasks = Array.from({ length: 20 }, (_, i) =>
      makeTask(`https://site-${i % 8}.example.com`, `result-${i}`)
    )

    const results = await runPerKeySequential(tasks, 5)

    expect(results).toHaveLength(20)
    // 结果顺序与入参一致
    results.forEach((result, index) => {
      expect(result.status).toBe('fulfilled')
      if (result.status === 'fulfilled') {
        expect(result.value).toBe(`result-${index}`)
      }
    })
    // 全局并发不超上限
    expect(maxActive).toBeLessThanOrEqual(5)
    expect(maxActive).toBeGreaterThan(1)
    // 同 origin 严格串行
    for (const [, keyMax] of maxActivePerKey) {
      expect(keyMax).toBe(1)
    }
  })

  it('captures rejections without aborting other tasks (allSettled semantics)', async () => {
    const tasks = [
      { key: 'a', run: async () => 'ok-1' },
      {
        key: 'a',
        run: async () => {
          throw new Error('boom')
        },
      },
      { key: 'b', run: async () => 'ok-2' },
    ]

    const results = await runPerKeySequential(tasks, 2)

    expect(results[0]).toMatchObject({ status: 'fulfilled', value: 'ok-1' })
    expect(results[1].status).toBe('rejected')
    if (results[1].status === 'rejected') {
      expect((results[1].reason as Error).message).toBe('boom')
    }
    expect(results[2]).toMatchObject({ status: 'fulfilled', value: 'ok-2' })
  })

  it('skips accounts refreshed within the min interval and keeps stale ones', () => {
    const now = Date.parse('2026-06-11T08:00:00.000Z')

    const justRefreshed = new Date(now - 10_000).toISOString()
    const refreshedLongAgo = new Date(now - BALANCE_REFRESH_MIN_INTERVAL_MS - 1).toISOString()

    expect(shouldSkipBalanceRefresh(justRefreshed, now)).toBe(true)
    expect(shouldSkipBalanceRefresh(refreshedLongAgo, now)).toBe(false)
    // 缺失或非法时间不跳过
    expect(shouldSkipBalanceRefresh(undefined, now)).toBe(false)
    expect(shouldSkipBalanceRefresh('not-a-date', now)).toBe(false)
  })
})
