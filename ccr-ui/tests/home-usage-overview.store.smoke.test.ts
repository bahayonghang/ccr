import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  makeArchiveDiagnostics,
  makeSessionIndexJobSnapshot,
  makeSnapshotProjection,
  makeUsageImportJobSnapshot,
} from './helpers/usageFixtures'

const eventListeners = new Map<string, (event: { payload: unknown }) => void>()

const listenMock = vi.fn(
  async (eventName: string, callback: (event: { payload: unknown }) => void) => {
    eventListeners.set(eventName, callback)
    return () => {
      eventListeners.delete(eventName)
    }
  }
)

const tauriRuntimeMock = vi.fn(() => true)

vi.mock('@tauri-apps/api/event', () => ({
  listen: listenMock,
}))

vi.mock('@/utils/tauriRuntime', () => ({
  isTauriRuntime: tauriRuntimeMock,
}))

vi.mock('@/utils/logger', () => ({
  logger: {
    error: vi.fn(),
  },
}))

vi.mock('@/api', () => ({
  ensureSessionIndexV2: vi.fn(),
  getUsageCapabilitiesV2: vi.fn(),
  getHomeUsageOverviewV2: vi.fn(),
  getSessionIndexJobStatusV2: vi.fn(),
  getUsageImportJobStatusV2: vi.fn(),
  startUsageImportJobV2: vi.fn(),
}))

const flushPromises = async () => {
  await Promise.resolve()
  await Promise.resolve()
  await Promise.resolve()
}

const createSupportedCapabilities = () => ({
  cli_available: true,
  cli_version: 'llmusage 0.0.0-test',
  root_dir: 'C:/Users/test/.llmusage',
  db_path: 'C:/Users/test/.llmusage/llmusage.db',
  db_exists: true,
  db_readable: true,
  schema_version: 10,
  features: {
    home_overview: { supported: true, reason: null, detail: null },
    sync_json_events: { supported: true, reason: null, detail: null },
  },
})

const createOverview = (
  bootstrap?: Partial<{
    usage_job_id: string | null
    needs_usage_import: boolean
    needs_session_index: boolean
    is_warm: boolean
    usage_import_attempted: boolean
    usage_imported_records: number
    session_reindex_attempted: boolean
    indexed_sessions: number
    session_job_id: string | null
  }>
) => ({
  summary: {
    total_sessions: 0,
    total_requests: 0,
    total_tokens: 0,
    active_days: 0,
    platforms: 0,
  },
  by_platform: {
    claude: { sessions: 0, requests: 0, tokens: 0 },
    codex: { sessions: 0, requests: 0, tokens: 0 },
    gemini: { sessions: 0, requests: 0, tokens: 0 },
    opencode: { sessions: 0, requests: 0, tokens: 0 },
  },
  series: [],
  archive: makeArchiveDiagnostics({
    archive_root: 'C:/Users/test/.ccr/analytics/usage.db',
    live_sources: 0,
    missing_sources: 0,
    deleted_sources: 0,
    archived_sessions: 0,
    recent_completed_at: null,
    history_completed_at: null,
  }),
  snapshot: makeSnapshotProjection(),
  bootstrap: {
    usage_import_attempted: false,
    usage_imported_records: 0,
    session_reindex_attempted: false,
    indexed_sessions: 0,
    usage_job_id: null,
    session_job_id: null,
    needs_usage_import: false,
    needs_session_index: false,
    is_warm: true,
    ...bootstrap,
  },
  empty_reason: 'no_usage_and_sessions' as const,
  last_updated: '2026-04-08T00:00:00Z',
})

describe('home usage overview store smoke', () => {
  beforeEach(async () => {
    setActivePinia(createPinia())
    eventListeners.clear()
    listenMock.mockClear()
    tauriRuntimeMock.mockReturnValue(true)
    vi.useRealTimers()

    const api = await import('@/api')
    vi.mocked(api.getUsageCapabilitiesV2).mockResolvedValue(createSupportedCapabilities())
  })

  afterEach(async () => {
    const { useHomeUsageOverviewStore } = await import('@/stores/homeUsageOverview')
    const store = useHomeUsageOverviewStore()
    await store.teardown()
    vi.resetAllMocks()
  })

  it('reuses cached overview data for the same day window', async () => {
    const api = await import('@/api')
    vi.mocked(api.getHomeUsageOverviewV2).mockResolvedValue(createOverview())

    const { useHomeUsageOverviewStore } = await import('@/stores/homeUsageOverview')
    const store = useHomeUsageOverviewStore()

    await store.loadOverview(30)
    await store.loadOverview(30)

    expect(api.getHomeUsageOverviewV2).toHaveBeenCalledTimes(1)
  })

  it('starts warmup work when the overview reports missing usage logs and session index', async () => {
    const api = await import('@/api')
    vi.mocked(api.getHomeUsageOverviewV2).mockResolvedValue(
      createOverview({
        needs_usage_import: true,
        needs_session_index: true,
        is_warm: false,
      })
    )
    vi.mocked(api.startUsageImportJobV2).mockResolvedValue({
      job_id: 'usage-import-home',
      snapshot: makeUsageImportJobSnapshot({
        job_id: 'usage-import-home',
        status: 'running',
        stage: 'importing_recent',
        platform_scope: 'all',
        recent_window_days: 30,
        files_total: 0,
        files_scanned: 0,
        files_imported: 0,
        records_imported: 0,
        records_skipped: 0,
        history_cursor_hit: false,
        live_sources: 0,
        missing_sources: 0,
        deleted_sources: 0,
        started_at: '2026-04-08T00:00:00Z',
        updated_at: '2026-04-08T00:00:00Z',
        recent_ready_at: null,
        finished_at: null,
        current_file: null,
        warnings: [],
        error: null,
        results: [],
        summary: null,
      }),
    })
    vi.mocked(api.getUsageImportJobStatusV2).mockResolvedValue(
      makeUsageImportJobSnapshot({
        job_id: 'usage-import-home',
        status: 'running',
        stage: 'importing_recent',
        platform_scope: 'all',
        recent_window_days: 30,
        files_total: 0,
        files_scanned: 0,
        files_imported: 0,
        records_imported: 0,
        records_skipped: 0,
        history_cursor_hit: false,
        live_sources: 0,
        missing_sources: 0,
        deleted_sources: 0,
        started_at: '2026-04-08T00:00:00Z',
        updated_at: '2026-04-08T00:00:00Z',
        recent_ready_at: null,
        finished_at: null,
        current_file: null,
        warnings: [],
        error: null,
        results: [],
        summary: null,
      })
    )
    vi.mocked(api.ensureSessionIndexV2).mockResolvedValue({
      job_id: 'session-index-home',
      snapshot: makeSessionIndexJobSnapshot({
        job_id: 'session-index-home',
        status: 'running',
        stage: 'indexing',
        platforms_total: 4,
        platforms_completed: 0,
        files_total: 12,
        files_scanned: 0,
        sessions_added: 0,
        sessions_updated: 0,
        errors: 0,
        started_at: '2026-04-08T00:00:00Z',
        updated_at: '2026-04-08T00:00:00Z',
        finished_at: null,
        current_platform: 'claude',
        warnings: [],
        error: null,
      }),
    })
    vi.mocked(api.getSessionIndexJobStatusV2).mockResolvedValue(
      makeSessionIndexJobSnapshot({
        job_id: 'session-index-home',
        status: 'running',
        stage: 'indexing',
        platforms_total: 4,
        platforms_completed: 0,
        files_total: 12,
        files_scanned: 0,
        sessions_added: 0,
        sessions_updated: 0,
        errors: 0,
        started_at: '2026-04-08T00:00:00Z',
        updated_at: '2026-04-08T00:00:00Z',
        finished_at: null,
        current_platform: 'claude',
        warnings: [],
        error: null,
      })
    )

    const { useHomeUsageOverviewStore } = await import('@/stores/homeUsageOverview')
    const store = useHomeUsageOverviewStore()

    await store.loadOverview(30)
    await flushPromises()

    expect(api.getHomeUsageOverviewV2).toHaveBeenCalledTimes(1)
    expect(api.startUsageImportJobV2).toHaveBeenCalledWith(undefined, 30, undefined)
    expect(api.ensureSessionIndexV2).toHaveBeenCalledTimes(1)
    expect(api.getSessionIndexJobStatusV2).toHaveBeenCalledWith('session-index-home')
    expect(listenMock).toHaveBeenCalled()
    expect(store.currentSessionJob?.current_platform).toBe('claude')
  })

  it('refreshes the cached overview when the active usage import job reaches recent_ready', async () => {
    const api = await import('@/api')
    vi.mocked(api.getHomeUsageOverviewV2)
      .mockResolvedValueOnce(
        createOverview({
          usage_job_id: 'usage-import-home',
          needs_usage_import: true,
          is_warm: false,
        })
      )
      .mockResolvedValueOnce(
        createOverview({
          usage_job_id: null,
          needs_usage_import: false,
          is_warm: true,
        })
      )
    vi.mocked(api.getUsageImportJobStatusV2).mockResolvedValue(
      makeUsageImportJobSnapshot({
        job_id: 'usage-import-home',
        status: 'running',
        stage: 'importing_recent',
        platform_scope: 'all',
        recent_window_days: 30,
        files_total: 0,
        files_scanned: 0,
        files_imported: 0,
        records_imported: 0,
        records_skipped: 0,
        history_cursor_hit: false,
        live_sources: 0,
        missing_sources: 0,
        deleted_sources: 0,
        started_at: '2026-04-08T00:00:00Z',
        updated_at: '2026-04-08T00:00:00Z',
        recent_ready_at: null,
        finished_at: null,
        current_file: null,
        warnings: [],
        error: null,
        results: [],
        summary: null,
      })
    )

    const { useHomeUsageOverviewStore } = await import('@/stores/homeUsageOverview')
    const store = useHomeUsageOverviewStore()
    await store.loadOverview(30)

    eventListeners.get('usage:job-recent-ready')?.({
      payload: {
        job_id: 'usage-import-home',
        status: 'recent_ready',
        stage: 'importing_history',
        platform_scope: 'all',
        recent_window_days: 30,
        files_total: 4,
        files_scanned: 2,
        files_imported: 2,
        records_imported: 24,
        records_skipped: 0,
        history_cursor_hit: true,
        live_sources: 1,
        missing_sources: 0,
        deleted_sources: 0,
        started_at: '2026-04-08T00:00:00Z',
        updated_at: '2026-04-08T00:00:01Z',
        recent_ready_at: '2026-04-08T00:00:01Z',
        finished_at: null,
        current_file: null,
        warnings: [],
        error: null,
        results: [],
        summary: null,
      },
    })
    await flushPromises()

    expect(api.getHomeUsageOverviewV2).toHaveBeenCalledTimes(2)
  })
})
