import { createApp, defineComponent, h, nextTick, reactive } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

let localeHydrated = false

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

vi.mock('@/i18n', () => ({
  ensureLocaleLoaded: vi.fn(async () => {
    localeHydrated = true
    return 'en-US'
  }),
}))

const translationTemplates: Record<string, string> = {
  'usage.dashboard.allPlatforms': 'All Platforms',
  'usage.platforms.qwen': 'Qwen',
  'usage.dashboard.days7': '7 Days',
  'usage.dashboard.days30': '30 Days',
  'usage.dashboard.days90': '90 Days',
  'usage.dashboard.days365': '365 Days',
  'usage.dashboard.cards.totalRequests': 'Total Requests',
  'usage.dashboard.cards.requestsDetail': '{models} models · {projects} projects',
  'usage.dashboard.cards.totalTokens': 'Total Tokens',
  'usage.dashboard.cards.tokensDetail': '{input} in · {output} out',
  'usage.dashboard.cards.totalCost': 'Total Cost',
  'usage.dashboard.cards.costDetail': '{average} per request',
  'usage.dashboard.cards.cacheEfficiency': 'Cache Efficiency',
  'usage.dashboard.cards.cacheDetail': '{tokens} cache read',
  'usage.dashboard.chart.input': 'Input',
  'usage.dashboard.chart.output': 'Output',
  'usage.dashboard.chart.cache': 'Cache',
  'usage.dashboard.chart.bucket.day': 'Daily',
  'usage.dashboard.chart.bucket.week': 'Weekly',
  'usage.dashboard.chart.bucket.month': 'Monthly',
  'usage.dashboard.chart.trendSubtitle': '{window}, aggregated by {granularity}, {points} key points',
  'usage.dashboard.chart.others': 'Others',
  'usage.dashboard.chart.distributionAllVisible': '{total} models are visible in this window',
  'usage.dashboard.chart.distributionSubtitle': 'Showing the top {visible} models; the remaining {total} are grouped into Others',
  'usage.dashboard.highlights.density': 'Trend Density',
  'usage.dashboard.highlights.densityDetail': '{points} key points across {window}',
  'usage.dashboard.highlights.topModel': 'Top Model',
  'usage.dashboard.highlights.topProject': 'Top Project',
  'usage.dashboard.highlights.cacheRead': 'Cache Read',
  'usage.dashboard.highlights.cacheReadDetail': 'Cache efficiency {percent}',
  'usage.dashboard.table.requests': 'requests',
  'usage.dashboard.table.noData': 'No data',
  'usage.dashboard.meta.scope': 'Scope',
  'usage.dashboard.meta.window': 'Window',
  'usage.dashboard.meta.models': 'Models',
  'usage.dashboard.meta.projects': 'Projects',
  'usage.dashboard.diagnostics.noRecentRecord': 'No recent record',
  'usage.dashboard.diagnostics.repairNeeded': 'Codex history should be repaired',
  'usage.dashboard.diagnostics.codexRepairHint': 'Detected {unknown} broken records',
  'usage.dashboard.diagnostics.healthy': 'Raw records look healthy',
  'usage.dashboard.diagnostics.rawLogsHint': 'Inspect raw logs here',
  'usage.dashboard.diagnostics.repairCodex': 'Repair Codex history',
  'usage.dashboard.diagnostics.repairingCodex': 'Rebuilding Codex history...',
}

const interpolate = (template: string, values?: Record<string, unknown>) => {
  if (!values) return template

  return template.replace(/\{([a-zA-Z_][a-zA-Z0-9_]*)\}/g, (_match, key) => {
    const value = values[key]
    return value == null ? `{${key}}` : String(value)
  })
}

vi.mock('vue-i18n', async (importOriginal) => {
  const actual = await importOriginal<typeof import('vue-i18n')>()
  const { ref } = await import('vue')
  return {
    ...actual,
    useI18n: () => ({
      t: (key: string, values?: Record<string, unknown>) => {
        const template = translationTemplates[key] ?? key
        return localeHydrated ? interpolate(template, values) : template
      },
      locale: ref('en-US'),
    }),
  }
})

const flushPromises = async () => {
  await Promise.resolve()
  await Promise.resolve()
  await Promise.resolve()
  await nextTick()
  await nextTick()
}

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
  await flushPromises()

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
  localeHydrated = false
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
      await vi.waitFor(() => {
        expect(usageStore.initializeDashboard).toHaveBeenCalledTimes(1)
        expect(usageStore.startAutoRefresh).toHaveBeenCalledTimes(1)
      })
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

  it('accepts qwen as a dashboard filter platform', async () => {
    tauriRuntime = true
    const { state, unmount } = await mountComposable()

    try {
      state.selectedPlatform.value = 'qwen'
      state.onFilterChange()

      expect(usageStore.setFilters).toHaveBeenCalledWith(expect.objectContaining({
        platform: 'qwen',
      }))
      expect(state.selectedPlatformLabel.value).toBe('Qwen')
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

  it('hydrates interpolated dashboard copy without leaking placeholder literals', async () => {
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
    usageStore.trends = [
      {
        date: '2026-03-01',
        request_count: 8,
        input_tokens: 1200,
        output_tokens: 600,
        cache_read_tokens: 300,
        cost_usd: 1.2,
      },
    ]

    const { state, unmount } = await mountComposable()

    try {
      expect(state.dashboardReady.value).toBe(true)
      expect(state.summaryCards.value[0]?.detail).toBe('2 models · 1 projects')
      expect(state.trendSubtitle.value).toBe('30 Days, aggregated by Daily, 1 key points')
      expect(state.overviewHighlights.value[0]?.detail).toBe('1 key points across 30 Days')

      const renderedCopy = [
        state.summaryCards.value[0]?.detail,
        state.summaryCards.value[1]?.detail,
        state.summaryCards.value[2]?.detail,
        state.summaryCards.value[3]?.detail,
        state.trendSubtitle.value,
        state.overviewHighlights.value[0]?.detail,
        state.overviewHighlights.value[3]?.detail,
      ].filter(Boolean) as string[]

      for (const line of renderedCopy) {
        expect(line).not.toMatch(/\{[a-zA-Z_][a-zA-Z0-9_]*\}/)
        expect(line).not.toMatch(/^usage\./)
      }
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
      expect(state.trendOptions.value.chart.parentHeightOffset).toBe(0)
      expect(state.trendOptions.value.markers).toMatchObject({
        size: 0,
        hover: { size: 0, sizeOffset: 0 },
      })

      document.documentElement.setAttribute('data-theme', 'light')
      await nextTick()
      await nextTick()

      expect(state.trendOptions.value.theme.mode).toBe('light')
    } finally {
      unmount()
    }
  })
})
