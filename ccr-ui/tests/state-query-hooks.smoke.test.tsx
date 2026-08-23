import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'

// Query 层 hook 测试（08-22-state-logic-port 批次 2 / AC8 的 Query 部分）。
// mock `@/api` wrapper（不打真实 IPC），断言 key 工厂形态与 queryFn 透传。

vi.mock('@/api', () => ({
  getUsageSummaryV2: vi.fn().mockResolvedValue({ totalCostUsd: 1.5 }),
  getUsageCapabilitiesV2: vi.fn().mockResolvedValue({ supported: true }),
  listConfigs: vi.fn().mockResolvedValue({ configs: [] }),
  listCommands: vi.fn().mockResolvedValue([]),
  executeCommand: vi.fn().mockResolvedValue({ ok: true }),
  claudeObserver: {
    getInsight: vi.fn().mockResolvedValue({ range: 'today' }),
    subscriptionGet: vi.fn().mockResolvedValue({ mode: 'none' }),
    subscriptionSet: vi.fn().mockResolvedValue({ mode: 'pro' }),
    dailyTrend: vi.fn().mockResolvedValue([]),
    costBreakdown: vi.fn().mockResolvedValue([]),
    cacheStats: vi.fn().mockResolvedValue({}),
    topSessions: vi.fn().mockResolvedValue([]),
    toolHeatmap: vi.fn().mockResolvedValue([]),
    topTools: vi.fn().mockResolvedValue([]),
  },
}))

import { getUsageSummaryV2, listConfigs, listCommands } from '@/api'
import { useClaudeObserverInsight, useSetClaudeObserverSubscription } from '@/features/claude/queries'
import { useCommands } from '@/features/commands/queries'
import { useConfigsList } from '@/features/configs/queries'
import { useUsageSummary, usageKeys } from '@/features/usage/queries'

const createWrapper = () => {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  )
  return { client, Wrapper }
}

describe('state-logic-port Query 层（批次 2）', () => {
  it('useUsageSummary：key 工厂含平台与时间维度，queryFn 透传参数', async () => {
    const { Wrapper } = createWrapper()
    const { result } = renderHook(
      () => useUsageSummary('codex', '2026-01-01', '2026-01-31'),
      { wrapper: Wrapper },
    )
    await waitFor(() => expect(result.current.isSuccess).toBe(true))
    expect(getUsageSummaryV2).toHaveBeenCalledWith('codex', '2026-01-01', '2026-01-31')
    expect(result.current.data).toEqual({ totalCostUsd: 1.5 })
  })

  it('usageKeys：范围 key 携带全部查询参数（失效可精确表达）', () => {
    expect(usageKeys.summary('codex', 'a', 'b')).toEqual(['usage', 'summary', 'codex', 'a', 'b'])
    expect(usageKeys.summary()).toEqual(['usage', 'summary', null, null, null])
    expect(usageKeys.all).toEqual(['usage'])
  })

  it('useConfigsList / useCommands：各自 wrapper 被调用且成功', async () => {
    const { Wrapper } = createWrapper()
    const configs = renderHook(() => useConfigsList(), { wrapper: Wrapper })
    const commands = renderHook(() => useCommands(), { wrapper: Wrapper })
    await waitFor(() => expect(configs.result.current.isSuccess).toBe(true))
    await waitFor(() => expect(commands.result.current.isSuccess).toBe(true))
    expect(listConfigs).toHaveBeenCalledTimes(1)
    expect(listCommands).toHaveBeenCalledTimes(1)
  })

  it('useClaudeObserverInsight 与订阅 mutation：参数透传', async () => {
    const { Wrapper } = createWrapper()
    const insight = renderHook(() => useClaudeObserverInsight('today'), { wrapper: Wrapper })
    await waitFor(() => expect(insight.result.current.isSuccess).toBe(true))

    const mutation = renderHook(() => useSetClaudeObserverSubscription(), { wrapper: Wrapper })
    const settled = await mutation.result.current.mutateAsync({ mode: 'pro', plan: 'x', monthlyUsd: 100 })
    expect(settled).toEqual({ mode: 'pro' })
    await waitFor(() => expect(mutation.result.current.isSuccess).toBe(true))
  })
})
