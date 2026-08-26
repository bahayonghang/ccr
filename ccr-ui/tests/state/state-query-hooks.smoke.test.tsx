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
  codexListModelProviders: vi.fn().mockResolvedValue({ providers: [] }),
  codexSaveModelProvider: vi.fn().mockResolvedValue({ ok: true }),
  codexDeleteModelProvider: vi.fn().mockResolvedValue({ ok: true }),
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

import {
  codexDeleteModelProvider,
  codexListModelProviders,
  codexSaveModelProvider,
} from '@/api'
import { useCodexProviders } from '@/composables/useCodexProviders'
import type { TranslateFunction } from '@/utils/tf'

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

describe('state-logic-port Query 层（批次 5b-ii 补测：mutation 成功路径）', () => {
  it('useCodexProviders：save 成功后 invalidate 并 refetch 列表，错误态保持为空', async () => {
    vi.mocked(codexListModelProviders).mockClear()
    interface ConfirmDialogOptions {
      title: string
      message: string
      confirmText: string
      type: string
      action: () => Promise<void>
    }
    const openConfirmDialog = vi.fn<(options: ConfirmDialogOptions) => void>()
    const { result } = renderHook(
      () =>
        useCodexProviders({
          t: ((key: string) => key) as TranslateFunction,
          openConfirmDialog,
          setActiveManagerTab: vi.fn(),
        }),
      { wrapper: createWrapper().Wrapper },
    )
    await waitFor(() => expect(result.current.providerLoading).toBe(false))
    expect(codexListModelProviders).toHaveBeenCalledTimes(1)

    result.current.providerFormApi.setValue('name', ' My Provider ')
    result.current.providerFormApi.setValue('baseUrl', 'https://api.example.com')
    await result.current.handleSaveProvider()

    // 保存载荷经 trim 净化后透传给 wrapper（第二参为 react-query 注入的上下文）
    const [savePayload] = vi.mocked(codexSaveModelProvider).mock.calls[0]
    expect(savePayload).toMatchObject({ name: 'My Provider', baseUrl: 'https://api.example.com' })
    await waitFor(() => expect(result.current.providerLoading).toBe(false))
    // invalidate 触发一次 refetch，reloadProviders 再显式拉一次：挂载之外共两次
    expect(codexListModelProviders).toHaveBeenCalledTimes(3)
    expect(result.current.providerError).toBeNull()
    // 表单成功后复位
    expect(result.current.providerForm.name).toBe('')
    expect(openConfirmDialog).not.toHaveBeenCalled()

    // delete 走确认弹窗编排；action 内完成删除与 refetch
    result.current.requestDeleteProvider({
      id: 'p1',
      name: 'p1',
      base_url: 'https://api.example.com',
      api_keys: [],
    } as never)
    expect(openConfirmDialog).toHaveBeenCalledTimes(1)
    const dialog = openConfirmDialog.mock.calls[0][0]
    await dialog.action()
    expect(vi.mocked(codexDeleteModelProvider).mock.calls[0][0]).toBe('p1')
    await waitFor(() => expect(result.current.providerLoading).toBe(false))
    // delete 同样 invalidate + reload：再拉两次
    expect(codexListModelProviders).toHaveBeenCalledTimes(5)
  })
})
