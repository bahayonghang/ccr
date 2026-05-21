import { createApp, defineComponent, h, KeepAlive, nextTick, reactive, ref } from 'vue'
import type { DailyTrend, UsageSummary } from '@/types/usage'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

let localeHydrated = false
const perfMarks: string[] = []

const usageStore = reactive({
  summary: null as UsageSummary | null,
  trends: [] as DailyTrend[],
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
  archive: null as null | {
    archive_root: string
    live_sources: number
    missing_sources: number
    deleted_sources: number
    archived_sessions: number
    recent_completed_at?: string | null
    history_completed_at?: string | null
  },
  logsModelFilter: undefined as string | undefined,
  lastImportResults: [] as Array<{ platform: string; error?: string }>,
  warning: '',
  currentImportJob: null as null | {
    warnings: string[]
    status: string
    files_total: number
    files_scanned: number
    records_imported: number
  },
  hasNoUsageData: false,
  isBootstrapping: false,
  importing: false,
  lastImportSummary: null as null | { processed_files: number; imported_records: number },
  initializeDashboard: vi.fn(async () => undefined),
  startAutoRefresh: vi.fn(),
  stopAutoRefresh: vi.fn(),
  setFilters: vi.fn(),
  setLogsModelFilter: vi.fn(),
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

vi.mock('@/utils/perfTelemetry', () => ({
  perfMark: vi.fn((name: string) => {
    perfMarks.push(name)
  }),
  perfMeasure: vi.fn(),
}))

const translationTemplates: Record<string, string> = {
  'usage.dashboard.allPlatforms': 'All Platforms',
  'usage.dashboard.days7': '7 Days',
  'usage.dashboard.days30': '30 Days',
  'usage.dashboard.days90': '90 Days',
  'usage.dashboard.days365': '365 Days',
  'usage.dashboard.cards.totalRequests': 'Total Requests',
  'usage.dashboard.cards.requestsDetail': '{models} models · {projects} projects',
  'usage.dashboard.cards.totalTokens': 'Total Tokens',
  'usage.dashboard.cards.tokensDetail': '{input} in · {output} out · {cache} cache read',
  'usage.dashboard.cards.totalCost': 'Total Cost',
  'usage.dashboard.cards.costDetail': '{average} per request',
  'usage.dashboard.cards.cacheEfficiency': 'Cache Reuse Rate',
  'usage.dashboard.cards.cacheDetail': 'cache read / (input + cache read) · {tokens} cache read',
  'usage.dashboard.chart.input': 'Input',
  'usage.dashboard.chart.output': 'Output',
  'usage.dashboard.chart.cache': 'Cache Read',
  'usage.dashboard.chart.bucket.day': 'Daily',
  'usage.dashboard.chart.bucket.week': 'Weekly',
  'usage.dashboard.chart.bucket.month': 'Monthly',
  'usage.dashboard.chart.trendSubtitle':
    '{window}, aggregated by {granularity}, {points} key points',
  'usage.dashboard.chart.others': 'Others',
  'usage.dashboard.chart.distributionAllVisible': '{total} models are visible in this window',
  'usage.dashboard.chart.distributionSubtitle':
    'Showing the top {visible} models; the remaining {total} are grouped into Others',
  'usage.dashboard.highlights.density': 'Trend Density',
  'usage.dashboard.highlights.densityDetail': '{points} key points across {window}',
  'usage.dashboard.highlights.topModel': 'Top Model',
  'usage.dashboard.highlights.topProject': 'Top Project',
  'usage.dashboard.highlights.cacheRead': 'Cache Read',
  'usage.dashboard.highlights.cacheReadDetail': 'Cache reuse {percent}',
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
  'usage.dashboard.diagnostics.rawLogsHint':
    'Use this area to debug imports and archive quality; deleting raw sessions does not remove archived history.',
  'usage.dashboard.diagnostics.repairCodex': 'Rebuild Codex archive index',
  'usage.dashboard.diagnostics.repairingCodex': 'Rebuilding Codex archive...',
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

const withFakeNow = async <T>(isoTimestamp: string, callback: () => Promise<T>) => {
  vi.useFakeTimers()
  vi.setSystemTime(new Date(isoTimestamp))
  try {
    return await callback()
  } finally {
    vi.useRealTimers()
  }
}

const mountComposable = async () => {
  const { useUsageDashboardState } = await import('@/views/usage/useUsageDashboardState')
  let state: ReturnType<typeof useUsageDashboardState> | null = null
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(
    defineComponent({
      setup() {
        state = useUsageDashboardState()
        return () => h('div')
      },
    })
  )

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

const mountKeepAliveComposable = async () => {
  const { useUsageDashboardState } = await import('@/views/usage/useUsageDashboardState')
  let state: ReturnType<typeof useUsageDashboardState> | null = null
  const active = ref(true)
  const el = document.createElement('div')
  document.body.appendChild(el)

  const DashboardHarness = defineComponent({
    name: 'UsageDashboardView',
    setup() {
      state = useUsageDashboardState()
      return () => h('div')
    },
  })

  const app = createApp(
    defineComponent({
      setup() {
        return () => h(KeepAlive, null, () => (active.value ? h(DashboardHarness) : null))
      },
    })
  )

  app.mount(el)
  await flushPromises()

  return {
    active,
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
  usageStore.archive = null
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
  usageStore.setLogsModelFilter.mockClear()
  usageStore.startImportJob.mockClear()
  usageStore.triggerImport.mockClear()
  usageStore.fetchLogs.mockClear()
  perfMarks.length = 0
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
      expect(usageStore.startAutoRefresh).toHaveBeenCalledWith({ immediate: false })
      expect(addEventListenerSpy).not.toHaveBeenCalledWith('visibilitychange', expect.any(Function))
    } finally {
      unmount()
    }

    expect(usageStore.stopAutoRefresh).toHaveBeenCalledTimes(1)
  })

  it('pauses auto refresh while keep-alive deactivates the usage page and resumes on activation', async () => {
    tauriRuntime = true
    const { active, unmount } = await mountKeepAliveComposable()

    try {
      await vi.waitFor(() => {
        expect(usageStore.initializeDashboard).toHaveBeenCalledTimes(1)
        expect(usageStore.startAutoRefresh).toHaveBeenCalledTimes(1)
      })

      active.value = false
      await nextTick()
      expect(usageStore.stopAutoRefresh).toHaveBeenCalledTimes(1)

      active.value = true
      await nextTick()
      expect(usageStore.startAutoRefresh).toHaveBeenCalledTimes(2)
      expect(usageStore.startAutoRefresh).toHaveBeenLastCalledWith()
    } finally {
      unmount()
    }

    expect(usageStore.stopAutoRefresh).toHaveBeenCalledTimes(2)
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

  it('keeps logs refresh scoped to logs activation and keep-alive resume', async () => {
    tauriRuntime = true
    const { active, state, unmount } = await mountKeepAliveComposable()

    try {
      await vi.waitFor(() => {
        expect(usageStore.initializeDashboard).toHaveBeenCalledTimes(1)
      })

      expect(usageStore.fetchLogs).not.toHaveBeenCalled()

      state.activeTab.value = 'logs'
      await nextTick()
      expect(usageStore.fetchLogs).toHaveBeenCalledWith('reset')

      active.value = false
      await nextTick()
      active.value = true
      await nextTick()

      expect(usageStore.fetchLogs).toHaveBeenLastCalledWith('same')
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

  it('accepts codex as a dashboard filter platform', async () => {
    tauriRuntime = true
    const { state, unmount } = await mountComposable()

    try {
      state.selectedPlatform.value = 'codex'
      state.onFilterChange()

      expect(usageStore.setFilters).toHaveBeenCalledWith(
        expect.objectContaining({
          platform: 'codex',
        })
      )
      expect(state.selectedPlatformLabel.value).toBe('Codex')
    } finally {
      unmount()
    }
  })

  it('accepts opencode as a dashboard filter platform', async () => {
    tauriRuntime = true
    const { state, unmount } = await mountComposable()

    try {
      state.selectedPlatform.value = 'opencode'
      state.onFilterChange()

      expect(usageStore.setFilters).toHaveBeenCalledWith(
        expect.objectContaining({
          platform: 'opencode',
        })
      )
      expect(state.selectedPlatformLabel.value).toBe('OpenCode')
    } finally {
      unmount()
    }
  })

  it('exposes codex repair diagnostics when codex records look unhealthy', async () => {
    tauriRuntime = true
    usageStore.summary = {
      total_requests: 42,
      total_tokens: 43800,
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
      total_tokens: 69000,
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
      {
        project_path: 'D:/workspace/heavy-project',
        request_count: 80,
        total_tokens: 36000,
        total_cost: 15.6,
      },
    ]
    usageStore.archive = {
      archive_root: 'C:/Users/test/.ccr/analytics/usage.db',
      live_sources: 3,
      missing_sources: 1,
      deleted_sources: 2,
      archived_sessions: 14,
      recent_completed_at: '2026-04-01T00:00:00Z',
      history_completed_at: '2026-04-02T00:00:00Z',
    }

    const { state, unmount } = await mountComposable()

    try {
      expect(state.summaryCards.value.map((card) => card.id)).toEqual([
        'requests',
        'tokens',
        'cost',
        'cache',
      ])
      expect(state.dashboardMetaItems.value).toHaveLength(7)
      expect(state.dashboardMetaItems.value.map((item) => item.id)).toEqual([
        'scope',
        'window',
        'models',
        'projects',
        'archive',
        'archive-root',
        'archive-time',
      ])
      expect(state.topModelRankings.value[0]?.label).toBe('claude-opus')
      expect(state.topProjectRankings.value[0]?.title).toBe('D:/workspace/heavy-project')
    } finally {
      unmount()
    }
  })

  it('defers chart gates until after dashboard content is ready', async () => {
    vi.useFakeTimers()
    tauriRuntime = true
    usageStore.summary = {
      total_requests: 120,
      total_tokens: 69000,
      total_input_tokens: 42000,
      total_output_tokens: 18000,
      total_cache_read_tokens: 9000,
      total_cost_usd: 24.5,
      cache_efficiency: 0.375,
    }
    usageStore.trends = [
      {
        date: '2026-03-01',
        request_count: 8,
        total_tokens: 2120,
        input_tokens: 1200,
        output_tokens: 600,
        cache_read_tokens: 300,
        cache_creation_tokens: 20,
        cost_usd: 1.2,
      },
    ]
    usageStore.modelStats = [
      { model: 'claude-opus', request_count: 72, total_tokens: 30000, total_cost: 18.4 },
    ]

    const { state, unmount } = await mountComposable()

    try {
      expect(state.shouldRenderTrendChart.value).toBe(false)
      expect(state.shouldRenderDistributionChart.value).toBe(false)
      expect(perfMarks).not.toContain('usage_first_content_ready')

      vi.advanceTimersByTime(180)
      await nextTick()
      vi.advanceTimersByTime(180)
      await nextTick()
      expect(state.shouldRenderTrendChart.value).toBe(true)
      expect(state.shouldRenderDistributionChart.value).toBe(false)
      expect(perfMarks).toContain('usage_first_content_ready')

      vi.advanceTimersByTime(180)
      await nextTick()
      expect(state.shouldRenderDistributionChart.value).toBe(true)
      expect(perfMarks).toContain('usage_trend_chart_gate_ready')
      expect(perfMarks).toContain('usage_distribution_chart_gate_ready')
    } finally {
      unmount()
      vi.useRealTimers()
    }
  })

  it('does not hydrate charts for true empty usage data', async () => {
    vi.useFakeTimers()
    tauriRuntime = true
    usageStore.hasNoUsageData = true

    const { state, unmount } = await mountComposable()

    try {
      vi.advanceTimersByTime(500)
      await nextTick()
      expect(state.shouldRenderTrendChart.value).toBe(false)
      expect(state.shouldRenderDistributionChart.value).toBe(false)
      expect(perfMarks).not.toContain('usage_trend_chart_gate_ready')
    } finally {
      unmount()
      vi.useRealTimers()
    }
  })

  it('keeps overview model distribution cost-based while exposing token-based models data', async () => {
    tauriRuntime = true
    usageStore.modelStats = [
      {
        model: 'expensive-small',
        request_count: 3,
        total_tokens: 1000,
        total_cost: 100,
        cost_with_cache: 100,
      },
      {
        model: 'cheap-large',
        request_count: 2,
        total_tokens: 10000,
        total_cost: 1,
        cost_with_cache: 1,
      },
    ]

    const { state, unmount } = await mountComposable()

    try {
      expect(state.modelDistribution.value[0]?.label).toBe('expensive-small')
      expect(state.modelTokenDistribution.value[0]?.label).toBe('cheap-large')
      expect(state.modelTokenPieSeries.value).toEqual([10000, 1000])
    } finally {
      unmount()
    }
  })

  it('hydrates interpolated dashboard copy without leaking placeholder literals', async () => {
    tauriRuntime = true
    usageStore.summary = {
      total_requests: 120,
      total_tokens: 69000,
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
      {
        project_path: 'D:/workspace/heavy-project',
        request_count: 80,
        total_tokens: 36000,
        total_cost: 15.6,
      },
    ]
    usageStore.trends = [
      {
        date: '2026-03-01',
        request_count: 8,
        total_tokens: 2120,
        input_tokens: 1200,
        output_tokens: 600,
        cache_read_tokens: 300,
        cache_creation_tokens: 20,
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

  it('uses a local inclusive 7 day window instead of UTC ISO slicing', async () => {
    await withFakeNow('2026-05-10T00:30:00+08:00', async () => {
      tauriRuntime = true
      const { state, unmount } = await mountComposable()

      try {
        state.selectedDays.value = 7
        state.onFilterChange()

        expect(usageStore.setFilters).toHaveBeenLastCalledWith(
          expect.objectContaining({
            start: '2026-05-04',
            end: '2026-05-10',
          })
        )
      } finally {
        unmount()
      }
    })
  })

  it('renders total token card from backend total_tokens and keeps cache formula copy visible', async () => {
    tauriRuntime = true
    usageStore.summary = {
      total_requests: 3,
      total_tokens: 195,
      total_input_tokens: 120,
      total_output_tokens: 45,
      total_cache_read_tokens: 30,
      total_cost_usd: 1.5,
      cache_efficiency: 0.2,
    }

    const { state, unmount } = await mountComposable()

    try {
      const tokenCard = state.summaryCards.value.find((card) => card.id === 'tokens')
      const cacheCard = state.summaryCards.value.find((card) => card.id === 'cache')

      expect(tokenCard?.value).toBe('195')
      expect(tokenCard?.detail).toContain('30 cache read')
      expect(cacheCard?.label).toBe('Cache Reuse Rate')
      expect(cacheCard?.detail).toContain('cache read / (input + cache read)')
    } finally {
      unmount()
    }
  })

  it('keeps all three trend series visible with a dedicated output axis for skewed data', async () => {
    tauriRuntime = true
    usageStore.trends = [
      {
        date: '2026-05-08',
        request_count: 4,
        total_tokens: 15110,
        input_tokens: 10000,
        output_tokens: 10,
        cache_read_tokens: 5000,
        cache_creation_tokens: 100,
        cost_usd: 0.1,
      } satisfies DailyTrend,
    ]

    const { state, unmount } = await mountComposable()

    try {
      expect(state.trendSeries.value.map((series) => series.name)).toEqual([
        'Input',
        'Output',
        'Cache Read',
      ])
      expect(state.trendSeries.value.every((series) => series.data.length === 1)).toBe(true)
      expect(state.trendOptions.value.legend).toMatchObject({ show: true })
      expect(state.trendOptions.value.yaxis.length).toBeGreaterThanOrEqual(3)
      expect(state.trendOptions.value.yaxis[1]).toMatchObject({
        opposite: true,
        showAlways: true,
        seriesName: 'Output',
      })
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
      expect(state.trendOptions.value.legend).toMatchObject({ show: true })
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
