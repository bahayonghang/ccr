import { createApp, defineComponent, h, KeepAlive, nextTick, reactive, ref } from 'vue'
import type {
  DailyTrend,
  ModelStat,
  ProviderBreakdown,
  ProjectStat,
  UsageCapabilityReport,
  UsageFeatureCapability,
  UsageSnapshotProjection,
  UsageSummary,
} from '@/types/usage'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { makeModelStat } from './helpers/usageFixtures'

let localeHydrated = false
const perfMarks: string[] = []

const usageStore = reactive({
  summary: null as UsageSummary | null,
  trends: [] as DailyTrend[],
  modelStats: [] as ModelStat[],
  projectStats: [] as ProjectStat[],
  providerStats: [] as ProviderBreakdown[],
  sourceStats: [],
  usageCapabilities: null as UsageCapabilityReport | null,
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
  snapshot: null as UsageSnapshotProjection | null,
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
  'usage.dashboard.range.today': 'Today',
  'usage.dashboard.range.thisWeek': 'This Week',
  'usage.dashboard.range.thisMonth': 'This Month',
  'usage.dashboard.range.last30': 'Last 30 Days',
  'usage.dashboard.range.allTime': 'All Time',
  'usage.dashboard.tabs.overview': 'Overview',
  'usage.dashboard.tabs.tokens': 'Tokens',
  'usage.dashboard.tabs.cost': 'Cost',
  'usage.dashboard.tabs.providers': 'Providers',
  'usage.dashboard.tabs.models': 'Models',
  'usage.dashboard.tabs.projects': 'Projects',
  'usage.dashboard.tabs.logs': 'Diagnostics',
  'usage.dashboard.cards.totalRequests': 'Total Requests',
  'usage.dashboard.cards.requestsDetail': '{models} models · {projects} projects',
  'usage.dashboard.cards.totalTokens': 'Total Tokens',
  'usage.dashboard.cards.tokensDetail': '{input} in · {output} out · {cache} cache read',
  'usage.dashboard.cards.totalCost': 'Total Cost',
  'usage.dashboard.cards.costDetail': '{average} per request',
  'usage.dashboard.cards.activeDays': 'Active Days',
  'usage.dashboard.cards.activeDaysDetail': 'Days with token activity in {window}',
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
  'usage.dashboard.ops.eyebrow': 'Usage operations cockpit',
  'usage.dashboard.ops.states.ready.label': 'Ready',
  'usage.dashboard.ops.states.ready.title': 'Usage data is ready',
  'usage.dashboard.ops.states.ready.detail':
    'The local read model is fresh enough for cost and token decisions.',
  'usage.dashboard.ops.states.stale.label': 'Stale',
  'usage.dashboard.ops.states.stale.title': 'Usage data is stale',
  'usage.dashboard.ops.states.stale.detail':
    'Refresh the local usage archive before making budget or quota decisions.',
  'usage.dashboard.ops.health.readiness': 'Readiness',
  'usage.dashboard.ops.health.nextAction': 'Next action: {action}',
  'usage.dashboard.ops.health.noAction': 'No blocking action',
  'usage.dashboard.ops.health.freshness': 'Freshness',
  'usage.dashboard.ops.health.sourceHealth': 'Source health',
  'usage.dashboard.ops.health.archivedSessions': '{count} archived sessions',
  'usage.dashboard.ops.health.snapshotCache': 'Snapshot cache',
  'usage.dashboard.ops.health.cacheTtl': '{count}s TTL',
  'usage.dashboard.ops.health.generatedAt': 'Generated {time}',
  'usage.dashboard.ops.health.scope': 'Scope',
  'usage.dashboard.ops.health.drilldown': 'Drilldown',
  'usage.dashboard.ops.health.dimensions': '{count} dimensions',
  'usage.dashboard.ops.health.noDrilldown': 'No dimensions yet',
  'usage.dashboard.ops.freshness.fresh': 'fresh',
  'usage.dashboard.ops.freshness.stale': 'stale',
  'usage.dashboard.ops.freshness.missing': 'missing',
  'usage.dashboard.ops.sourceStates.live': 'live',
  'usage.dashboard.ops.sourceStates.degraded': 'degraded',
  'usage.dashboard.ops.sourceStates.missing': 'missing',
  'usage.dashboard.ops.actions.import_usage': 'Import usage',
  'usage.dashboard.ops.actions.refresh_usage': 'Refresh usage',
  'usage.dashboard.ops.actions.diagnostics': 'Inspect diagnostics',
  'usage.dashboard.ops.age.unknown': 'age unknown',
  'usage.dashboard.ops.age.days': '{count}d ago',
  'usage.dashboard.ops.missingTimestamp': 'No completed import yet',
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

const withFakeNow = async <T>(timestamp: string | Date, callback: () => Promise<T>) => {
  vi.useFakeTimers()
  vi.setSystemTime(timestamp instanceof Date ? timestamp : new Date(timestamp))
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
  usageStore.providerStats = []
  usageStore.sourceStats = []
  usageStore.usageCapabilities = null
  usageStore.logs = null
  usageStore.logsLoading = false
  usageStore.archive = null
  usageStore.snapshot = null
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

  it('orders tokens immediately after overview in the dashboard tabs', async () => {
    tauriRuntime = true
    const { state, unmount } = await mountComposable()

    try {
      expect([...state.tabKeys]).toEqual([
        'overview',
        'tokens',
        'cost',
        'providers',
        'models',
        'projects',
        'logs',
      ])
    } finally {
      unmount()
    }
  })

  it('exposes provider stats and provider capability for the provider tab', async () => {
    tauriRuntime = true
    usageStore.providerStats = [
      {
        provider: 'openai',
        request_count: 5,
        input_tokens: 1000,
        cache_read_tokens: 250,
        cache_creation_tokens: 50,
        output_tokens: 400,
        reasoning_output_tokens: 40,
        total_tokens: 1700,
        cost_with_cache_usd: 0.42,
        cost_without_cache_usd: 0.6,
      },
      {
        provider: null,
        request_count: 2,
        input_tokens: 200,
        cache_read_tokens: 0,
        cache_creation_tokens: 0,
        output_tokens: 100,
        reasoning_output_tokens: 0,
        total_tokens: 300,
        cost_with_cache_usd: 0.04,
        cost_without_cache_usd: 0.04,
      },
    ]
    usageStore.usageCapabilities = {
      cli_available: true,
      cli_version: null,
      root_dir: 'C:/Users/test/.llmusage',
      db_path: 'C:/Users/test/.llmusage/llmusage.db',
      db_exists: true,
      db_readable: true,
      schema_version: 14,
      features: {
        // 断言用 toEqual({ supported: true }) 精确比较，补 reason/detail: null 会让断言变红；
        // 这里保持运行时值不变，仅用 as 收窄类型。
        provider_breakdown: { supported: true } as UsageFeatureCapability,
      },
    }

    const { state, unmount } = await mountComposable()

    try {
      expect(state.providerStats.value).toHaveLength(2)
      expect(state.providerStats.value[1]?.provider).toBeNull()
      expect(state.providerCapability.value).toEqual({ supported: true })
    } finally {
      unmount()
    }
  })

  it('starts a background import job with the safe preset import window', async () => {
    tauriRuntime = true
    const { state, unmount } = await mountComposable()

    try {
      state.selectedRange.value = 'all_time'
      await state.doImport()

      expect(usageStore.startImportJob).toHaveBeenCalledWith({
        platform: undefined,
        reason: 'manual',
        recentDays: 30,
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
      makeModelStat({ model: 'unknown', request_count: 42, total_tokens: 43800, total_cost: 0 }),
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

  it('maps this-month repair imports to the bounded current month span', async () => {
    await withFakeNow('2026-05-21T10:00:00+08:00', async () => {
      tauriRuntime = true
      const { state, unmount } = await mountComposable()

      try {
        state.selectedPlatform.value = 'codex'
        state.selectedRange.value = 'this_month'
        await state.repairCodexLogs()

        expect(usageStore.startImportJob).toHaveBeenCalledWith({
          platform: 'codex',
          reason: 'manual',
          recentDays: 21,
          resetSources: true,
        })
      } finally {
        unmount()
      }
    })
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
      makeModelStat({
        model: 'claude-opus',
        request_count: 72,
        total_tokens: 30000,
        total_cost: 18.4,
      }),
      makeModelStat({
        model: 'gemini-flash',
        request_count: 48,
        total_tokens: 30000,
        total_cost: 6.1,
      }),
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
        'tokens',
        'cost',
        'activeDays',
        'requests',
      ])
      expect(state.summaryCards.value[2]?.label).toBe('Active Days')
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

  it('exposes usage ops cockpit from backend snapshot readiness', async () => {
    tauriRuntime = true
    usageStore.summary = {
      total_requests: 1,
      total_tokens: 100,
      total_input_tokens: 50,
      total_output_tokens: 25,
      total_cache_read_tokens: 25,
      total_cost_usd: 0.1,
      cache_efficiency: 0.25,
    }
    usageStore.archive = {
      archive_root: 'C:/Users/test/.ccr/analytics/usage.db',
      live_sources: 1,
      missing_sources: 1,
      deleted_sources: 0,
      archived_sessions: 8,
      recent_completed_at: '2026-05-25T08:00:00Z',
      history_completed_at: '2026-05-25T09:00:00Z',
    }
    usageStore.snapshot = {
      generated_at: '2026-05-25T09:10:00Z',
      platform_scope: 'all',
      start_date: null,
      end_date: null,
      cache_ttl_seconds: 30,
      freshness: {
        state: 'stale',
        latest_completed_at: '2026-05-24T00:00:00Z',
        age_seconds: 90_000,
        stale_after_seconds: 86_400,
      },
      readiness: {
        state: 'stale',
        next_action: 'refresh_usage',
        detail: 'stale',
        has_live_sources: true,
        has_missing_sources: true,
        has_deleted_sources: false,
        active_usage_import: false,
        active_session_index: false,
        recent_completed_at: '2026-05-25T08:00:00Z',
      },
      source_health: [
        {
          source: 'codex',
          state: 'degraded',
          live_sources: 1,
          missing_sources: 1,
          deleted_sources: 0,
          recent_completed_at: null,
          history_completed_at: null,
          freshness: {
            state: 'stale',
            latest_completed_at: '2026-05-24T00:00:00Z',
            age_seconds: 90_000,
            stale_after_seconds: 86_400,
          },
        },
      ],
      drilldown: {
        dimensions: ['source', 'project_path', 'session_id', 'branch'],
        supports_logs: true,
        supports_projects: true,
        supports_sessions: true,
      },
    }

    const { state, unmount } = await mountComposable()

    try {
      expect(state.opsCockpit.value.state).toBe('stale')
      expect(state.opsCockpit.value.primaryAction).toBe('import')
      expect(state.opsCockpit.value.primaryActionLabel).toBe('Refresh usage')
      expect(state.opsCockpit.value.sourceItems[0]?.label).toBe('Codex')
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
        reasoning_output_tokens: 0,
        cache_read_tokens: 300,
        cache_creation_tokens: 20,
        cost_usd: 1.2,
      },
    ]
    usageStore.modelStats = [
      makeModelStat({
        model: 'claude-opus',
        request_count: 72,
        total_tokens: 30000,
        total_cost: 18.4,
      }),
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
      makeModelStat({
        model: 'expensive-small',
        request_count: 3,
        total_tokens: 1000,
        total_cost: 100,
        cost_with_cache: 100,
      }),
      makeModelStat({
        model: 'cheap-large',
        request_count: 2,
        total_tokens: 10000,
        total_cost: 1,
        cost_with_cache: 1,
      }),
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
      makeModelStat({
        model: 'claude-opus',
        request_count: 72,
        total_tokens: 30000,
        total_cost: 18.4,
      }),
      makeModelStat({
        model: 'gemini-flash',
        request_count: 48,
        total_tokens: 30000,
        total_cost: 6.1,
      }),
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
        reasoning_output_tokens: 0,
        cache_read_tokens: 300,
        cache_creation_tokens: 20,
        cost_usd: 1.2,
      },
    ]

    const { state, unmount } = await mountComposable()

    try {
      expect(state.dashboardReady.value).toBe(true)
      expect(state.summaryCards.value.find((card) => card.id === 'requests')?.detail).toBe(
        '2 models · 1 projects'
      )
      expect(state.trendSubtitle.value).toBe('Last 30 Days, aggregated by Daily, 1 key points')
      expect(state.overviewHighlights.value[0]?.detail).toBe('1 key points across Last 30 Days')

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
    await withFakeNow(new Date(2026, 4, 10, 0, 30, 0), async () => {
      tauriRuntime = true
      const { state, unmount } = await mountComposable()

      try {
        state.selectedRange.value = 'this_week'
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

  it('renders total token card from backend total_tokens and leaves cache KPI to the token strip', async () => {
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

      expect(tokenCard?.value).toBe('195')
      expect(tokenCard?.detail).toContain('30 cache read')
      expect(state.summaryCards.value.some((card) => (card.id as string) === 'cache')).toBe(false)
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
        reasoning_output_tokens: 0,
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
