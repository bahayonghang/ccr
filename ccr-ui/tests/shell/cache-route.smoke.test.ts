import { beforeEach, describe, expect, it } from 'vitest'
import { queryClient } from '@/shell/queryClient'
import { useCommandsStreamStore } from '@/features/commands/stores'
import { COMMAND_OUTPUT_LINES_PER_CHANNEL, emptyCommandJob, mergeCommandSnapshot } from '@/features/commands/commandJobState'
import { useConfigsViewStore } from '@/features/configs/stores'
import { useGrokViewStore } from '@/features/grok/stores'
import { useUsageViewStore } from '@/features/usage/stores'
import { readInnerScroll, restoreInnerScroll, saveInnerScroll } from '@/shell/innerScroll'

describe('cache-route store R/W（AC4）', () => {
  beforeEach(() => {
    useGrokViewStore.setState(useGrokViewStore.getInitialState())
    useConfigsViewStore.setState(useConfigsViewStore.getInitialState())
    useUsageViewStore.setState(useUsageViewStore.getInitialState())
    useCommandsStreamStore.setState(useCommandsStreamStore.getInitialState())
  })

  it('dashboard 数据由 QueryClient 承担，无需额外视图 store', () => {
    expect(queryClient.getDefaultOptions().queries?.staleTime).toBe(30_000)
    const gcTime = queryClient.getDefaultOptions().queries?.gcTime
    expect(typeof gcTime).toBe('number')
    expect(gcTime).toBeGreaterThan(0)
    expect(gcTime).toBeLessThanOrEqual(120_000)
  })

  it('grok 选中态写入后可取回', () => {
    useGrokViewStore.getState().setSelectedProfileName('work')
    expect(useGrokViewStore.getState().selectedProfileName).toBe('work')
  })

  it('commands retains one bounded snapshot instead of a separate client output buffer', () => {
    const store = useCommandsStreamStore.getState()
    useCommandsStreamStore.setState({ job: mergeCommandSnapshot(emptyCommandJob(), {
      job_id: 'job-cap', command: 'status', args: [], status: 'queued', started_at: '',
      finished_at: null, duration_ms: null, exit_code: null, stdout_lines: [], stderr_lines: [],
      system_lines: [], truncated: false, dropped_lines: 0, error: null,
    }, false) })
    store.receiveDeltas([{ job_id: 'job-cap', seq: 0, channel: 'stdout', lines: ['first'], dropped_count: 0 }])
    expect(useCommandsStreamStore.getState().job.snapshot?.stdout_lines).toEqual(['first'])
    const overflow = Array.from({ length: COMMAND_OUTPUT_LINES_PER_CHANNEL + 5 }, (_, index) => String(index))
    store.receiveDeltas([{ job_id: 'job-cap', seq: 1, channel: 'stdout', lines: overflow, dropped_count: 0 }])
    const snapshot = useCommandsStreamStore.getState().job.snapshot
    expect(snapshot?.stdout_lines).toHaveLength(COMMAND_OUTPUT_LINES_PER_CHANNEL)
    expect(snapshot?.stdout_lines[0]).toBe('5')
    expect(snapshot?.truncated).toBe(true)
    expect(snapshot?.dropped_lines).toBe(6)
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
