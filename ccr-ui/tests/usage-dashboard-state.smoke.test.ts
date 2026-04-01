import { createApp, defineComponent, h, nextTick, reactive } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const usageStore = reactive({
  summary: null,
  trends: [],
  modelStats: [],
  projectStats: [],
  logs: null as null | {
    records: unknown[]
    total?: number | null
    page?: number
    page_size?: number
    next_cursor?: string | null
    mode?: string
  },
  logsLoading: false,
  logsModelFilter: undefined as string | undefined,
  lastImportResults: [] as Array<{ platform: string; error?: string }>,
  warning: '',
  currentImportJob: null as null | { warnings: string[]; status: string; files_total: number; files_scanned: number; records_imported: number },
  hasNoUsageData: false,
  isBootstrapping: false,
  importing: false,
  lastImportSummary: null as null | { processed_files: number; imported_records: number },
  initializeDashboard: vi.fn(async () => undefined),
  startAutoRefresh: vi.fn(),
  stopAutoRefresh: vi.fn(),
  setFilters: vi.fn(),
  startImportJob: vi.fn(async () => undefined),
  triggerImport: vi.fn(async () => undefined),
  fetchLogs: vi.fn(),
})

let tauriRuntime = false

vi.mock('@/stores/usage', () => ({
  useUsageStore: () => usageStore,
}))

vi.mock('@/utils/tauriRuntime', () => ({
  isTauriRuntime: () => tauriRuntime,
}))

vi.mock('vue-i18n', async () => {
  const { ref } = await import('vue')
  return {
    useI18n: () => ({
      t: (key: string) => key,
      locale: ref('en-US'),
    }),
  }
})

const mountComposable = async () => {
  const { useUsageDashboardState } = await import('@/views/usage/useUsageDashboardState')
  let state: ReturnType<typeof useUsageDashboardState> | null = null
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      state = useUsageDashboardState()
      return () => h('div')
    },
  }))

  app.mount(el)
  await Promise.resolve()
  await nextTick()
  await nextTick()

  return {
    el,
    state: state!,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

beforeEach(() => {
  tauriRuntime = false
  document.documentElement.setAttribute('data-theme', 'light')
  usageStore.summary = null
  usageStore.trends = []
  usageStore.modelStats = []
  usageStore.projectStats = []
  usageStore.logs = null
  usageStore.logsLoading = false
  usageStore.logsModelFilter = undefined
  usageStore.lastImportResults = []
  usageStore.warning = ''
  usageStore.currentImportJob = null
  usageStore.hasNoUsageData = false
  usageStore.isBootstrapping = false
  usageStore.importing = false
  usageStore.lastImportSummary = null
  usageStore.initializeDashboard.mockClear()
  usageStore.startAutoRefresh.mockClear()
  usageStore.stopAutoRefresh.mockClear()
  usageStore.setFilters.mockClear()
  usageStore.startImportJob.mockClear()
  usageStore.triggerImport.mockClear()
  usageStore.fetchLogs.mockClear()
})

afterEach(() => {
  document.body.innerHTML = ''
  vi.restoreAllMocks()
})

describe('usage dashboard state smoke', () => {
  it('starts store auto refresh without registering its own visibility listener', async () => {
    tauriRuntime = true
    const addEventListenerSpy = vi.spyOn(document, 'addEventListener')
    const { unmount } = await mountComposable()

    try {
      expect(usageStore.initializeDashboard).toHaveBeenCalledTimes(1)
      expect(usageStore.startAutoRefresh).toHaveBeenCalledTimes(1)
      expect(addEventListenerSpy).not.toHaveBeenCalledWith(
        'visibilitychange',
        expect.any(Function),
      )
    } finally {
      unmount()
    }

    expect(usageStore.stopAutoRefresh).toHaveBeenCalledTimes(1)
  })

  it('loads logs when the logs tab becomes active', async () => {
    tauriRuntime = true
    const { state, unmount } = await mountComposable()

    try {
      state.activeTab.value = 'logs'
      await nextTick()

      expect(usageStore.fetchLogs).toHaveBeenCalledWith('reset')
    } finally {
      unmount()
    }
  })

  it('skips desktop initialization in web runtime mode', async () => {
    tauriRuntime = false
    const { state, unmount } = await mountComposable()

    try {
      expect(state.runtimeUnavailable.value).toBe(true)
      expect(usageStore.initializeDashboard).not.toHaveBeenCalled()
      expect(usageStore.startAutoRefresh).not.toHaveBeenCalled()
    } finally {
      unmount()
    }

    expect(usageStore.stopAutoRefresh).toHaveBeenCalled()
  })

  it('starts a background import job with the current recent-days window', async () => {
    tauriRuntime = true
    const { state, unmount } = await mountComposable()

    try {
      state.selectedDays.value = 90
      await state.doImport()

      expect(usageStore.startImportJob).toHaveBeenCalledWith({
        platform: undefined,
        reason: 'manual',
        recentDays: 90,
      })
    } finally {
      unmount()
    }
  })

  it('exposes codex repair diagnostics when codex records look unhealthy', async () => {
    tauriRuntime = true
    usageStore.summary = {
      total_requests: 42,
      total_input_tokens: 42000,
      total_output_tokens: 1800,
      total_cache_read_tokens: 0,
      total_cost_usd: 0,
      cache_efficiency: 0,
    }
    usageStore.modelStats = [
      { model: 'unknown', request_count: 42, total_tokens: 43800, total_cost: 0 },
    ]
    usageStore.logs = {
      total: 42,
      page: 1,
      page_size: 50,
      next_cursor: null,
      mode: 'offset',
      records: [
        {
          id: 'codex-1',
          platform: 'codex',
          project_path: '/tmp/project',
          record_json: '{}',
          recorded_at: '2026-04-01T00:00:00Z',
          source_id: 'source-1',
          model: null,
          input_tokens: 120,
          output_tokens: 24,
          cache_read_tokens: 0,
          cost_usd: 0,
        },
      ],
    }

    const { state, unmount } = await mountComposable()

    try {
      state.selectedPlatform.value = 'codex'
      expect(state.diagnosticsSummary.value.repairRecommended).toBe(true)

      await state.repairCodexLogs()

      expect(usageStore.startImportJob).toHaveBeenCalledWith({
        platform: 'codex',
        reason: 'manual',
        recentDays: 30,
        resetSources: true,
      })
    } finally {
      unmount()
    }
  })

  it('builds dashboard meta and summary cards from the loaded usage data', async () => {
    tauriRuntime = true
    usageStore.summary = {
      total_requests: 120,
      total_input_tokens: 42000,
      total_output_tokens: 18000,
      total_cache_read_tokens: 9000,
      total_cost_usd: 24.5,
      cache_efficiency: 0.375,
    }
    usageStore.modelStats = [
      { model: 'claude-opus', request_count: 72, total_tokens: 30000, total_cost: 18.4 },
      { model: 'gemini-flash', request_count: 48, total_tokens: 30000, total_cost: 6.1 },
    ]
    usageStore.projectStats = [
      { project_path: 'D:/workspace/heavy-project', request_count: 80, total_tokens: 36000, total_cost: 15.6 },
    ]

    const { state, unmount } = await mountComposable()

    try {
      expect(state.summaryCards.value.map((card) => card.id)).toEqual([
        'requests',
        'tokens',
        'cost',
        'cache',
      ])
      expect(state.dashboardMetaItems.value).toHaveLength(4)
      expect(state.topModelRankings.value[0]?.label).toBe('claude-opus')
      expect(state.topProjectRankings.value[0]?.title).toBe('D:/workspace/heavy-project')
    } finally {
      unmount()
    }
  })

  it('keeps chart theme in sync with the document theme', async () => {
    tauriRuntime = true
    document.documentElement.setAttribute('data-theme', 'dark')
    const { state, unmount } = await mountComposable()

    try {
      expect(state.trendOptions.value.theme.mode).toBe('dark')

      document.documentElement.setAttribute('data-theme', 'light')
      await nextTick()
      await nextTick()

      expect(state.trendOptions.value.theme.mode).toBe('light')
    } finally {
      unmount()
    }
  })
})
