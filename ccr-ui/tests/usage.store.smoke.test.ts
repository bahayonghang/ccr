import { setActivePinia, createPinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ImportAllUsageResponse, UsageSummary } from '@/types/usage'

const tauriRuntimeMock = vi.fn(() => false)
const eventListeners = new Map<string, (event: { payload: unknown }) => void>()
const listenMock = vi.fn(async (eventName: string, callback: (event: { payload: unknown }) => void) => {
  eventListeners.set(eventName, callback)
  return () => {
    eventListeners.delete(eventName)
  }
})

vi.mock('@tauri-apps/api/event', () => ({
  listen: listenMock,
}))

vi.mock('@/utils/tauriRuntime', () => ({
  isTauriRuntime: tauriRuntimeMock,
}))

vi.mock('@/api', () => ({
  getUsageCapabilitiesV2: vi.fn().mockResolvedValue(createSupportedCapabilities()),
  getUsageByModelV2: vi.fn().mockResolvedValue([]),
  getUsageByProjectV2: vi.fn().mockResolvedValue([]),
  getUsageDashboardV2: vi.fn().mockResolvedValue({
    summary: {
      total_requests: 0,
      total_tokens: 0,
      total_input_tokens: 0,
      total_output_tokens: 0,
      total_cache_read_tokens: 0,
      total_cost_usd: 0,
      cache_efficiency: 0
    },
    trends: [],
    model_stats: [],
    project_stats: [],
    archive: {
      archive_root: 'C:/Users/test/.ccr/analytics/usage.db',
      live_sources: 0,
      missing_sources: 0,
      deleted_sources: 0,
      archived_sessions: 0,
      recent_completed_at: null,
      history_completed_at: null,
    }
  }),
  getUsageHeatmapV2: vi.fn().mockResolvedValue({ data: {} }),
  getUsageLogsV2: vi.fn().mockResolvedValue({
    records: [],
    total: 0,
    page: 1,
    page_size: 50,
    next_cursor: null,
    mode: 'cursor'
  }),
  getUsageSummaryV2: vi.fn().mockResolvedValue({
    total_requests: 0,
    total_tokens: 0,
    total_input_tokens: 0,
    total_output_tokens: 0,
    total_cache_read_tokens: 0,
    total_cost_usd: 0,
    cache_efficiency: 0
  }),
  getUsageImportJobStatusV2: vi.fn(),
  getUsageTrendsV2: vi.fn().mockResolvedValue([]),
  importAllUsageV2: vi.fn().mockResolvedValue({
    results: [],
    summary: {
      success_count: 0,
      failure_count: 0,
      imported_records: 0,
      processed_files: 0,
      has_partial: false
    }
  }),
  importUsageV2: vi.fn(),
  startUsageImportJobV2: vi.fn(),
}))

const createSupportedCapabilities = () => ({
  cli_available: true,
  cli_version: 'llmusage 0.0.0-test',
  root_dir: 'C:/Users/test/.llmusage',
  db_path: 'C:/Users/test/.llmusage/llmusage.db',
  db_exists: true,
  db_readable: true,
  schema_version: 10,
  features: {
    overview: { supported: true, reason: null, detail: null },
    sync_json_events: { supported: true, reason: null, detail: null },
  },
})

const flushPromises = async () => {
  await Promise.resolve()
  await Promise.resolve()
  await Promise.resolve()
}

describe('usage store smoke', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetModules()
    tauriRuntimeMock.mockReturnValue(false)
    eventListeners.clear()
    listenMock.mockClear()
    vi.useRealTimers()
  })

  it('keeps stable default and computed state', async () => {
    const { useUsageStore } = await import('@/stores/usage')
    const store = useUsageStore()

    expect(store.summary).toBeNull()
    expect(store.totalTokens).toBe(0)
    expect(store.hasUsageData).toBe(false)
    expect(store.hasNoUsageData).toBe(false)

    store.loading = false
    expect(store.hasNoUsageData).toBe(true)

    const summary: UsageSummary = {
      total_requests: 3,
      total_tokens: 195,
      total_input_tokens: 120,
      total_output_tokens: 45,
      total_cache_read_tokens: 30,
      total_cost_usd: 1.5,
      cache_efficiency: 0.2
    }
    store.summary = summary

    expect(store.totalTokens).toBe(195)
    expect(store.hasUsageData).toBe(true)
    expect(store.hasNoUsageData).toBe(false)
  })



  it('uses backend total_tokens instead of deriving input plus output', async () => {
    const { useUsageStore } = await import('@/stores/usage')
    const store = useUsageStore()

    store.summary = {
      total_requests: 1,
      total_tokens: 195,
      total_input_tokens: 120,
      total_output_tokens: 45,
      total_cache_read_tokens: 30,
      total_cost_usd: 0.1,
      cache_efficiency: 0.2,
    }

    expect(store.totalTokens).toBe(195)
    expect(store.totalTokens).not.toBe(165)
  })

  it('normalizes single-platform import results into store summary state', async () => {
    const api = await import('@/api')
    vi.mocked(api.importUsageV2).mockResolvedValue({
      platform: 'codex',
      files_processed: 2,
      records_imported: 12,
      records_skipped: 1,
      duration_ms: 18,
      completed: true,
      error: null
    })

    const { useUsageStore } = await import('@/stores/usage')
    const store = useUsageStore()
    const result = await store.triggerImport('codex')

    expect(api.importUsageV2).toHaveBeenCalledWith('codex')
    expect((result as ImportAllUsageResponse).summary).toEqual({
      success_count: 1,
      failure_count: 0,
      imported_records: 12,
      processed_files: 2,
      has_partial: false
    })
    expect(store.lastImportSummary).toEqual((result as ImportAllUsageResponse).summary)
    expect(store.lastImportResults).toHaveLength(1)
    expect(store.error).toBeNull()
  })

  it('normalizes opencode import results into store summary state', async () => {
    const api = await import('@/api')
    vi.mocked(api.importUsageV2).mockResolvedValue({
      platform: 'opencode',
      files_processed: 1,
      records_imported: 7,
      records_skipped: 2,
      duration_ms: 11,
      completed: true,
      error: null
    })

    const { useUsageStore } = await import('@/stores/usage')
    const store = useUsageStore()
    const result = await store.triggerImport('opencode')

    expect(api.importUsageV2).toHaveBeenCalledWith('opencode')
    expect((result as ImportAllUsageResponse).summary).toEqual({
      success_count: 1,
      failure_count: 0,
      imported_records: 7,
      processed_files: 1,
      has_partial: false
    })
    expect(store.lastImportResults[0]?.platform).toBe('opencode')
    expect(store.error).toBeNull()
  })

  it('treats a missing optional OpenCode database as an absent source, not an import error', async () => {
    const api = await import('@/api')
    vi.mocked(api.importUsageV2).mockResolvedValue({
      platform: 'opencode',
      files_processed: 0,
      records_imported: 0,
      records_skipped: 0,
      duration_ms: 0,
      completed: false,
      error: 'OpenCode SQLite DB 缺失',
      // 模拟新契约：后端通过 typed `is_optional_absent` 标记 OpenCode optional 缺失，
      // 前端 normalize 据此把 completed→true / error→null。
      is_optional_absent: true,
    })

    const { useUsageStore } = await import('@/stores/usage')
    const store = useUsageStore()
    const result = await store.triggerImport('opencode')

    expect((result as ImportAllUsageResponse).summary).toEqual({
      success_count: 1,
      failure_count: 0,
      imported_records: 0,
      processed_files: 0,
      has_partial: false
    })
    expect(store.lastImportResults[0]).toMatchObject({
      platform: 'opencode',
      completed: true,
      error: null
    })
    expect(store.warning).toBeNull()
    expect(store.error).toBeNull()
  })


  it('starts a background usage import job and stores the active snapshot', async () => {
    const api = await import('@/api')
    vi.mocked(api.startUsageImportJobV2).mockResolvedValue({
      job_id: 'usage-import-1',
      snapshot: {
        job_id: 'usage-import-1',
        status: 'running',
        stage: 'importing_recent',
        platform_scope: 'all',
        recent_window_days: 90,
        files_total: 12,
        files_scanned: 1,
        files_imported: 1,
        records_imported: 24,
        records_skipped: 0,
        history_cursor_hit: false,
        live_sources: 0,
        missing_sources: 0,
        deleted_sources: 0,
        started_at: '2026-04-01T00:00:00Z',
        updated_at: '2026-04-01T00:00:01Z',
        recent_ready_at: null,
        finished_at: null,
        current_file: '/tmp/usage.jsonl',
        warnings: [],
        error: null,
        results: [],
        summary: null,
      },
    })

    const { useUsageStore } = await import('@/stores/usage')
    const store = useUsageStore()
    await store.startImportJob({ recentDays: 90, reason: 'manual' })

    expect(api.startUsageImportJobV2).toHaveBeenCalledWith(undefined, 90, undefined)
    expect(store.currentImportJob?.job_id).toBe('usage-import-1')
    expect(store.importing).toBe(true)
    expect(store.isBootstrapping).toBe(false)
  })

  it('passes resetSources when codex repair is requested', async () => {
    const api = await import('@/api')
    vi.mocked(api.startUsageImportJobV2).mockResolvedValue({
      job_id: 'usage-import-repair',
      snapshot: {
        job_id: 'usage-import-repair',
        status: 'running',
        stage: 'importing_recent',
        platform_scope: 'codex',
        recent_window_days: 30,
        files_total: 1,
        files_scanned: 0,
        files_imported: 0,
        records_imported: 0,
        records_skipped: 0,
        history_cursor_hit: false,
        live_sources: 0,
        missing_sources: 0,
        deleted_sources: 0,
        started_at: '2026-04-01T00:00:00Z',
        updated_at: '2026-04-01T00:00:00Z',
        recent_ready_at: null,
        finished_at: null,
        current_file: null,
        warnings: [],
        error: null,
        results: [],
        summary: null,
      },
    })

    const { useUsageStore } = await import('@/stores/usage')
    const store = useUsageStore()
    await store.startImportJob({
      platform: 'codex',
      recentDays: 30,
      reason: 'manual',
      resetSources: true,
    })

    expect(api.startUsageImportJobV2).toHaveBeenCalledWith('codex', 30, true)
  })

  it('normalizes missing OpenCode DB from background job snapshots', async () => {
    const api = await import('@/api')
    vi.mocked(api.startUsageImportJobV2).mockResolvedValue({
      job_id: 'usage-import-opencode',
      snapshot: {
        job_id: 'usage-import-opencode',
        status: 'finished',
        stage: 'finished',
        platform_scope: 'opencode',
        recent_window_days: 30,
        files_total: 1,
        files_scanned: 1,
        files_imported: 0,
        records_imported: 0,
        records_skipped: 0,
        history_cursor_hit: false,
        live_sources: 0,
        missing_sources: 0,
        deleted_sources: 0,
        started_at: '2026-04-01T00:00:00Z',
        updated_at: '2026-04-01T00:00:00Z',
        recent_ready_at: '2026-04-01T00:00:00Z',
        finished_at: '2026-04-01T00:00:00Z',
        current_file: null,
        // 后端 push_warning 之前已过滤 absent，warnings 不应含 absent 文案；
        // job-level error 也不再嗅探 absent，保持空。
        warnings: [],
        error: null,
        results: [{
          platform: 'opencode',
          files_processed: 0,
          records_imported: 0,
          records_skipped: 0,
          duration_ms: 0,
          completed: false,
          error: 'OpenCode SQLite DB 缺失',
          // 新契约：后端通过 typed 字段标记 absent，前端不再嗅探 error 字符串。
          is_optional_absent: true,
        }],
        summary: {
          success_count: 0,
          failure_count: 1,
          imported_records: 0,
          processed_files: 0,
          has_partial: true,
        },
      },
    })

    const { useUsageStore } = await import('@/stores/usage')
    const store = useUsageStore()
    await store.startImportJob({ platform: 'opencode', recentDays: 30, reason: 'manual' })

    expect(store.lastImportSummary).toEqual({
      success_count: 1,
      failure_count: 0,
      imported_records: 0,
      processed_files: 0,
      has_partial: false,
    })
    expect(store.lastImportResults[0]).toMatchObject({
      platform: 'opencode',
      completed: true,
      error: null,
    })
    expect(store.currentImportJob?.warnings).toEqual([])
    expect(store.currentImportJob?.error).toBeNull()
    expect(store.error).toBeNull()
    expect(store.warning).toBeNull()
  })

  it('uses cursor paging when fetching diagnostics logs', async () => {
    const api = await import('@/api')
    const { useUsageStore } = await import('@/stores/usage')
    const store = useUsageStore()

    store.platform = 'codex'
    store.timeRange = { start: '2026-03-01', end: '2026-03-31' }
    store.logsModelFilter = 'unknown'

    await store.fetchLogs('reset')

    expect(api.getUsageLogsV2).toHaveBeenCalledWith(expect.objectContaining({
      platform: 'codex',
      model: 'unknown',
      start_date: '2026-03-01',
      end_date: '2026-03-31',
      page: 1,
      page_size: 50,
      include_total: true,
      cursor: undefined,
      mode: 'cursor',
    }))
  })

  it('uses next_cursor instead of recounting when loading the next logs page', async () => {
    const api = await import('@/api')
    vi.mocked(api.getUsageLogsV2)
      .mockResolvedValueOnce({
        records: [{ id: 'row-1' }],
        total: 120,
        page: 1,
        page_size: 50,
        next_cursor: 'cursor-2',
        mode: 'cursor',
      })
      .mockResolvedValueOnce({
        records: [{ id: 'row-2' }],
        total: null,
        page: 2,
        page_size: 50,
        next_cursor: 'cursor-3',
        mode: 'cursor',
      })

    const { useUsageStore } = await import('@/stores/usage')
    const store = useUsageStore()
    store.platform = 'codex'

    await store.fetchLogs('reset')
    await store.fetchLogs('next')

    expect(api.getUsageLogsV2).toHaveBeenNthCalledWith(1, expect.objectContaining({
      page: 1,
      include_total: true,
      cursor: undefined,
      mode: 'cursor',
    }))
    expect(api.getUsageLogsV2).toHaveBeenNthCalledWith(2, expect.objectContaining({
      page: 2,
      include_total: false,
      cursor: 'cursor-2',
      mode: 'cursor',
    }))
    expect(store.logs?.total).toBe(120)
    expect(store.logsPage).toBe(2)
  })

  it('progressively refreshes the dashboard during import without flipping loading state', async () => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2026-04-01T00:00:00Z'))
    tauriRuntimeMock.mockReturnValue(true)

    const api = await import('@/api')
    vi.mocked(api.startUsageImportJobV2).mockResolvedValue({
      job_id: 'usage-import-2',
      snapshot: {
        job_id: 'usage-import-2',
        status: 'running',
        stage: 'importing_recent',
        platform_scope: 'all',
        recent_window_days: 90,
        files_total: 12,
        files_scanned: 0,
        files_imported: 0,
        records_imported: 0,
        records_skipped: 0,
        history_cursor_hit: false,
        live_sources: 0,
        missing_sources: 0,
        deleted_sources: 0,
        started_at: '2026-04-01T00:00:00Z',
        updated_at: '2026-04-01T00:00:00Z',
        recent_ready_at: null,
        finished_at: null,
        current_file: null,
        warnings: [],
        error: null,
        results: [],
        summary: null,
      },
    })
    vi.mocked(api.getUsageImportJobStatusV2).mockResolvedValue({
      job_id: 'usage-import-2',
      status: 'running',
      stage: 'importing_recent',
      platform_scope: 'all',
      recent_window_days: 90,
      files_total: 12,
      files_scanned: 0,
      files_imported: 0,
      records_imported: 0,
      records_skipped: 0,
      history_cursor_hit: false,
      live_sources: 0,
      missing_sources: 0,
      deleted_sources: 0,
      started_at: '2026-04-01T00:00:00Z',
      updated_at: '2026-04-01T00:00:00Z',
      recent_ready_at: null,
      finished_at: null,
      current_file: null,
      warnings: [],
      error: null,
      results: [],
      summary: null,
    })

    const { useUsageStore } = await import('@/stores/usage')
    const store = useUsageStore()
    store.loading = false

    await store.startImportJob({ recentDays: 90, reason: 'manual' })
    expect(listenMock).toHaveBeenCalled()
    expect(api.getUsageDashboardV2).not.toHaveBeenCalled()

    const progress = eventListeners.get('usage:job-progress')
    progress?.({
      payload: {
        job_id: 'usage-import-2',
        status: 'running',
        stage: 'importing_recent',
        platform_scope: 'all',
        recent_window_days: 90,
        files_total: 12,
        files_scanned: 1,
        files_imported: 1,
        records_imported: 24,
        records_skipped: 0,
        history_cursor_hit: false,
        live_sources: 0,
        missing_sources: 0,
        deleted_sources: 0,
        started_at: '2026-04-01T00:00:00Z',
        updated_at: '2026-04-01T00:00:01Z',
        recent_ready_at: null,
        finished_at: null,
        current_file: '/tmp/usage-1.jsonl',
        warnings: [],
        error: null,
        results: [],
        summary: null,
      },
    })
    await flushPromises()

    expect(api.getUsageDashboardV2).toHaveBeenCalledTimes(1)
    expect(store.loading).toBe(false)

    progress?.({
      payload: {
        job_id: 'usage-import-2',
        status: 'running',
        stage: 'importing_recent',
        platform_scope: 'all',
        recent_window_days: 90,
        files_total: 12,
        files_scanned: 2,
        files_imported: 2,
        records_imported: 48,
        records_skipped: 0,
        history_cursor_hit: false,
        live_sources: 0,
        missing_sources: 0,
        deleted_sources: 0,
        started_at: '2026-04-01T00:00:00Z',
        updated_at: '2026-04-01T00:00:01Z',
        recent_ready_at: null,
        finished_at: null,
        current_file: '/tmp/usage-2.jsonl',
        warnings: [],
        error: null,
        results: [],
        summary: null,
      },
    })
    await flushPromises()

    expect(api.getUsageDashboardV2).toHaveBeenCalledTimes(1)

    const finished = eventListeners.get('usage:job-finished')
    finished?.({
      payload: {
        job_id: 'usage-import-2',
        status: 'finished',
        stage: 'finished',
        platform_scope: 'all',
        recent_window_days: 90,
        files_total: 12,
        files_scanned: 12,
        files_imported: 12,
        records_imported: 96,
        records_skipped: 0,
        history_cursor_hit: false,
        live_sources: 0,
        missing_sources: 0,
        deleted_sources: 0,
        started_at: '2026-04-01T00:00:00Z',
        updated_at: '2026-04-01T00:00:05Z',
        recent_ready_at: '2026-04-01T00:00:04Z',
        finished_at: '2026-04-01T00:00:05Z',
        current_file: null,
        warnings: [],
        error: null,
        results: [],
        summary: {
          success_count: 1,
          failure_count: 0,
          imported_records: 96,
          processed_files: 12,
          has_partial: false,
        },
      },
    })
    await flushPromises()

    expect(api.getUsageDashboardV2).toHaveBeenCalledTimes(2)
  })
})
