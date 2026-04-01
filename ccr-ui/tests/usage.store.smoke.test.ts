import { setActivePinia, createPinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ImportAllUsageResponse, UsageSummary } from '@/types/usage'

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => () => undefined),
}))

vi.mock('@/utils/tauriRuntime', () => ({
  isTauriRuntime: () => false,
}))

vi.mock('@/api', () => ({
  getUsageByModelV2: vi.fn().mockResolvedValue([]),
  getUsageByProjectV2: vi.fn().mockResolvedValue([]),
  getUsageDashboardV2: vi.fn().mockResolvedValue({
    summary: {
      total_requests: 0,
      total_input_tokens: 0,
      total_output_tokens: 0,
      total_cache_read_tokens: 0,
      total_cost_usd: 0,
      cache_efficiency: 0
    },
    trends: [],
    model_stats: [],
    project_stats: []
  }),
  getUsageHeatmapV2: vi.fn().mockResolvedValue({ data: {} }),
  getUsageLogsV2: vi.fn().mockResolvedValue({
    records: [],
    total: 0,
    page: 1,
    page_size: 50
  }),
  getUsageSummaryV2: vi.fn().mockResolvedValue({
    total_requests: 0,
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

describe('usage store smoke', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetModules()
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
      total_input_tokens: 120,
      total_output_tokens: 45,
      total_cache_read_tokens: 30,
      total_cost_usd: 1.5,
      cache_efficiency: 0.2
    }
    store.summary = summary

    expect(store.totalTokens).toBe(165)
    expect(store.hasUsageData).toBe(true)
    expect(store.hasNoUsageData).toBe(false)
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

    expect(api.startUsageImportJobV2).toHaveBeenCalledWith(undefined, 90)
    expect(store.currentImportJob?.job_id).toBe('usage-import-1')
    expect(store.importing).toBe(true)
    expect(store.isBootstrapping).toBe(false)
  })
})
