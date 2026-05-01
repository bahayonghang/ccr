import { createApp, defineComponent, h, nextTick } from 'vue'
import { createPinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const apiMocks = vi.hoisted(() => ({
  getHomeUsageOverviewV2: vi.fn(),
  ensureSessionIndexV2: vi.fn(),
  getSessionIndexJobStatusV2: vi.fn(),
  getUsageImportJobStatusV2: vi.fn(),
  startUsageImportJobV2: vi.fn(),
}))

const monitoringState = vi.hoisted(() => ({
  logs: [
    {
      id: 'event-1',
      timestamp: '2026-04-29T09:15:00Z',
      level: 'info',
      channel: 'usage',
      eventType: 'usage.import',
      source: 'test',
      message: 'Usage archive refreshed',
    },
  ],
}))

const translationTemplates = vi.hoisted(() => ({
  'home.systemMetricHost': '主机：{host}',
  'home.systemMetricMemory': '已用 {used} / {total} GB',
  'home.usageLastUpdated': '更新于 {time}',
}))

vi.mock('vue-router', () => ({
  useRouter: () => ({
    push: vi.fn(),
  }),
}))

vi.mock('@/api/runtime/system', () => ({
  getSystemInfo: vi.fn(async () => ({
    hostname: 'workstation',
    os: 'windows',
    os_version: '11',
    kernel_version: '10.0',
    cpu_brand: 'Test CPU',
    cpu_cores: 12,
    cpu_usage: 11.4,
    total_memory_gb: 64,
    used_memory_gb: 17.5,
    memory_usage_percent: 27.3,
    total_swap_gb: 0,
    used_swap_gb: 0,
    uptime_seconds: 1200,
  })),
  getCliVersions: vi.fn(async () => ({
    versions: [
      { platform: 'claude', installed: true, version: '1.0.0', status: 'ok' },
      { platform: 'codex', installed: true, version: '2.1.0', status: 'ok' },
      { platform: 'gemini', installed: true, version: '3.2.0', status: 'ok' },
    ],
  })),
}))

vi.mock('@/api', () => apiMocks)

vi.mock('@/composables/useMonitoringFeed', async () => {
  const { ref } = await vi.importActual<typeof import('vue')>('vue')
  return {
    useMonitoringFeed: () => ({
      logs: ref(monitoringState.logs),
    }),
  }
})

vi.mock('@/utils/tauriRuntime', () => ({
  isTauriRuntime: () => true,
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => {
      return translationTemplates[key as keyof typeof translationTemplates] ?? key
    },
  }),
}))

const findPlaceholderLeaks = (text: string) => {
  return text.match(/\{[a-zA-Z_][a-zA-Z0-9_]*\}/g) ?? []
}

vi.mock('@/utils/scheduling', () => ({
  scheduleWhenIdle: (callback: () => void) => {
    callback()
    return () => {}
  },
}))

vi.mock('@/utils/perfTelemetry', () => ({
  perfMark: vi.fn(),
  shouldLogPerfTelemetry: () => false,
}))

vi.mock('@/utils/logger', () => ({
  logger: {
    error: vi.fn(),
    info: vi.fn(),
    getHistory: vi.fn(() => []),
    subscribe: vi.fn(() => () => {}),
  },
}))

const createOverview = (overrides?: {
  empty?: boolean
}) => ({
  summary: {
    total_sessions: overrides?.empty ? 0 : 18,
    total_requests: overrides?.empty ? 0 : 1240,
    total_tokens: overrides?.empty ? 0 : 980000,
    active_days: 30,
    platforms: overrides?.empty ? 0 : 3,
  },
  by_platform: {
    claude: { sessions: overrides?.empty ? 0 : 8, requests: overrides?.empty ? 0 : 520, tokens: overrides?.empty ? 0 : 420000 },
    codex: { sessions: overrides?.empty ? 0 : 7, requests: overrides?.empty ? 0 : 640, tokens: overrides?.empty ? 0 : 510000 },
    gemini: { sessions: overrides?.empty ? 0 : 3, requests: overrides?.empty ? 0 : 80, tokens: overrides?.empty ? 0 : 50000 },
  },
  series: overrides?.empty
    ? []
    : [
        {
          date: '2026-04-27',
          claude: { sessions: 2, requests: 100, tokens: 10000 },
          codex: { sessions: 1, requests: 180, tokens: 20000 },
          gemini: { sessions: 0, requests: 20, tokens: 1000 },
        },
        {
          date: '2026-04-28',
          claude: { sessions: 3, requests: 220, tokens: 12000 },
          codex: { sessions: 2, requests: 240, tokens: 23000 },
          gemini: { sessions: 1, requests: 30, tokens: 1200 },
        },
      ],
  archive: {
    archive_root: 'C:/Users/test/.ccr/analytics/usage.db',
    live_sources: overrides?.empty ? 0 : 3,
    missing_sources: 0,
    deleted_sources: 0,
    archived_sessions: overrides?.empty ? 0 : 18,
    recent_completed_at: null,
    history_completed_at: null,
  },
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
  },
  empty_reason: overrides?.empty ? 'no_usage_and_sessions' as const : undefined,
  last_updated: '2026-04-29T09:00:00Z',
})

const flushPromises = async () => {
  await Promise.resolve()
  await Promise.resolve()
  await nextTick()
}

const mountHomeView = async () => {
  const HomeView = (await import('@/views/HomeView.vue')).default
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      return () => h(HomeView)
    },
  }))

  app.use(createPinia())
  app.component(
    'RouterLink',
    defineComponent({
      props: {
        to: { type: [String, Object], required: true },
      },
      setup(_props, { slots }) {
        return () => h('a', {}, slots.default?.())
      },
    }),
  )

  app.config.globalProperties.$t = (key: string) => key
  app.mount(el)
  await flushPromises()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

const collectStrings = (value: unknown): string[] => {
  if (typeof value === 'string') return [value]
  if (!value || typeof value !== 'object') return []
  return Object.values(value).flatMap(collectStrings)
}

beforeEach(() => {
  vi.clearAllMocks()
  monitoringState.logs = [
    {
      id: 'event-1',
      timestamp: '2026-04-29T09:15:00Z',
      level: 'info',
      channel: 'usage',
      eventType: 'usage.import',
      source: 'test',
      message: 'Usage archive refreshed',
    },
  ]
  apiMocks.getHomeUsageOverviewV2.mockResolvedValue(createOverview())
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('HomeView smoke', () => {
  it('renders the workbench sections with real system, CLI, activity, and usage data', async () => {
    const { el, unmount } = await mountHomeView()

    try {
      expect(el.querySelector('[data-home-hero]')).not.toBeNull()
      expect(el.querySelector('[data-home-actions]')).not.toBeNull()
      expect(el.querySelector('[data-home-activity]')).not.toBeNull()
      expect(el.querySelector('[data-home-platforms]')).not.toBeNull()
      expect(el.querySelector('[data-home-usage-preview]')).not.toBeNull()
      expect(el.querySelector('.home-poster')).toBeNull()
      expect(el.querySelector('.page-header-card')).toBeNull()
      expect(el.textContent).toContain('11.4%')
      expect(el.textContent).toContain('27.3%')
      expect(el.textContent).toContain('3/3')
      expect(el.textContent).toContain('主机：workstation')
      expect(el.textContent).toContain('已用 17.5 / 64.0 GB')
      expect(el.textContent).toContain('Usage archive refreshed')
      expect(el.textContent).toContain('1.2K')
      expect(el.textContent).toContain('更新于')
      expect(el.textContent).not.toContain('Factory Droid')
      expect(findPlaceholderLeaks(el.textContent || '')).toEqual([])
    } finally {
      unmount()
    }
  })

  it('renders empty activity and usage snapshot fallbacks without fake usage', async () => {
    monitoringState.logs = []
    apiMocks.getHomeUsageOverviewV2.mockResolvedValue(createOverview({ empty: true }))

    const { el, unmount } = await mountHomeView()

    try {
      expect(el.textContent).toContain('home.activityEmptyTitle')
      expect(el.textContent).toContain('usageStats.noUsageAndSessions')
      expect(el.textContent).toContain('home.platformUsageUntracked')
    } finally {
      unmount()
    }
  })

  it('uses professional home copy in locales and boot messages', async () => {
    const [{ default: zhCN }, { default: enUS }, { bootLocaleMessages }] = await Promise.all([
      import('@/i18n/locales/zh-CN'),
      import('@/i18n/locales/en-US'),
      import('@/i18n/bootMessages'),
    ])

    expect(zhCN.home.workbenchTitle).toBe('运行态势')
    expect(zhCN.home.activityTitle).toBe('事件流')
    expect(zhCN.home.platformUsageUntracked).toBe('未追踪')
    expect(enUS.home.workbenchTitle).toBe('Operational Workbench')
    expect(enUS.home.activityTitle).toBe('Event Stream')
    expect(enUS.home.platformUsageUntracked).toBe('Untracked')
    expect(bootLocaleMessages['zh-CN'].home.workbenchTitle).toBe('运行态势')
    expect(bootLocaleMessages['en-US'].home.workbenchTitle).toBe('Operational Workbench')

    const homeCopy = [
      ...collectStrings(zhCN.home),
      ...collectStrings(enUS.home),
      ...collectStrings(bootLocaleMessages['zh-CN'].home),
      ...collectStrings(bootLocaleMessages['en-US'].home),
    ].join('\n')

    expect(homeCopy).not.toMatch(/喵|meow|脉冲|pulse/i)
    expect(homeCopy).not.toMatch(/Factory Droid|Droids/i)
  })
})
