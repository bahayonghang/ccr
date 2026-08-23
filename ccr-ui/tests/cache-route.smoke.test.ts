import { beforeEach, describe, expect, it } from 'vitest'
import { queryClient } from '@/shell/queryClient'
import {
  COMMANDS_STREAM_CAP,
  useCommandsStreamStore,
  type CommandStreamLine,
} from '@/features/commands/stores'
import { useConfigsViewStore } from '@/features/configs/stores'
import { useGrokViewStore } from '@/features/grok/stores'
import { useUsageViewStore } from '@/features/usage/stores'
import { readInnerScroll, restoreInnerScroll, saveInnerScroll } from '@/shell/innerScroll'

const line = (index: number): CommandStreamLine => ({
  channel: 'stdout',
  text: `line-${index}`,
  seq: index,
  jobId: 'job-1',
})

describe('cache-route store R/W（AC4）', () => {
  beforeEach(() => {
    useGrokViewStore.setState(useGrokViewStore.getInitialState())
    useConfigsViewStore.setState(useConfigsViewStore.getInitialState())
    useUsageViewStore.setState(useUsageViewStore.getInitialState())
    useCommandsStreamStore.setState(useCommandsStreamStore.getInitialState())
  })

  it('dashboard 数据由 QueryClient 承担，无需额外视图 store', () => {
    expect(queryClient.getDefaultOptions().queries?.staleTime).toBe(30_000)
  })

  it('grok 选中态写入后可取回', () => {
    useGrokViewStore.getState().setSelectedProfileName('work')
    expect(useGrokViewStore.getState().selectedProfileName).toBe('work')
  })

  it('commands 流式缓冲按 client 追加、卸载不清空、超限截断最旧行', () => {
    const store = useCommandsStreamStore.getState()
    store.setActiveClient('ccr')
    store.appendStreamLines({ lines: [line(1), line(2)] })
    expect(useCommandsStreamStore.getState().linesByClient.ccr).toHaveLength(2)

    const overflow = Array.from({ length: COMMANDS_STREAM_CAP + 5 }, (_, index) => line(index))
    useCommandsStreamStore.setState({ linesByClient: {} })
    store.appendStreamLines({ client: 'ccr', lines: overflow })
    const buffered = useCommandsStreamStore.getState().linesByClient.ccr
    expect(buffered).toHaveLength(COMMANDS_STREAM_CAP)
    expect(buffered[0]?.text).toBe('line-5')
    expect(buffered[buffered.length - 1]?.text).toBe(`line-${COMMANDS_STREAM_CAP + 4}`)
  })

  it('configs 选中态、搜索词与表单草稿按配置 id 读写', () => {
    const store = useConfigsViewStore.getState()
    store.setCurrentConfig('default')
    store.setSearchQuery('prod')
    store.setFormDraft('default', { token: 'x' })
    expect(useConfigsViewStore.getState().currentConfig).toBe('default')
    expect(useConfigsViewStore.getState().searchQuery).toBe('prod')
    expect(useConfigsViewStore.getState().formDrafts.default).toEqual({ token: 'x' })
    const before = useConfigsViewStore.getState()
    store.clearFormDraft('missing')
    expect(useConfigsViewStore.getState()).toBe(before)
    store.clearFormDraft('default')
    expect(useConfigsViewStore.getState().formDrafts.default).toBeUndefined()
  })

  it('usage 筛选条件写入后可取回', () => {
    useUsageViewStore.getState().setPlatform('claude')
    useUsageViewStore.getState().setTimeRange({ start: '2026-01-01', end: '2026-01-31' })
    expect(useUsageViewStore.getState().platform).toBe('claude')
    expect(useUsageViewStore.getState().timeRange).toEqual({
      start: '2026-01-01',
      end: '2026-01-31',
    })
  })

  it('内部滚动：缓存路由恢复，非缓存路由回到顶部', () => {
    saveInnerScroll('/usage', 420)
    const cached = document.createElement('div')
    restoreInnerScroll({ pathname: '/usage', cache: true, element: cached })
    expect(cached.scrollTop).toBe(420)
    expect(readInnerScroll('/usage')).toBe(420)

    const fresh = document.createElement('div')
    fresh.scrollTop = 80
    restoreInnerScroll({ pathname: '/settings', cache: false, element: fresh })
    expect(fresh.scrollTop).toBe(0)
  })
})
