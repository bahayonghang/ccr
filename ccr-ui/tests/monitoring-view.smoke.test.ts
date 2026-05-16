import { createApp, defineComponent, h, nextTick, ref } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { MonitoringEntry } from '@/composables/useMonitoringFeed'

const apiMocks = vi.hoisted(() => ({
  getUsageCapabilitiesV2: vi.fn(),
  getUsageSummaryV2: vi.fn(),
}))

const monitoringState = vi.hoisted(() => ({
  isConnected: { __v_isRef: true, value: true },
  logs: { __v_isRef: true, value: [] as MonitoringEntry[] },
  clearLogs: vi.fn(),
  refresh: vi.fn(async () => undefined),
}))

const translationTemplates: Record<string, string> = {
  'monitoring.eyebrow': 'Operations Monitor',
  'monitoring.title': 'Operations Monitoring',
  'monitoring.subtitle': 'Runtime state summary',
  'monitoring.connected': 'Event stream connected',
  'monitoring.disconnected': 'Event stream disconnected',
  'monitoring.refresh': 'Refresh',
  'monitoring.clearView': 'Clear current view',
  'monitoring.usageEyebrow': 'Usage Archive',
  'monitoring.usageTitle': 'Real usage summary',
  'monitoring.usageReady': 'Loaded',
  'monitoring.usageIdle': 'Waiting',
  'monitoring.usageLoading': 'Loading',
  'monitoring.usageLoadingDetail': 'Reading the local usage summary',
  'monitoring.usageUnavailable': 'Usage unavailable',
  'monitoring.usageUnavailableDescription': 'The local llmusage database summary cannot be read.',
  'monitoring.usageUnsupportedReason': 'Capability unavailable: {reason}',
  'monitoring.usageMetricUnavailable': 'Real usage data is unavailable, so zero placeholders are hidden.',
  'monitoring.totalRequests': 'Requests',
  'monitoring.totalTokens': 'Total Tokens',
  'monitoring.inputOutputTokens': 'Input / Output',
  'monitoring.estimatedCost': 'Estimated Cost',
  'monitoring.requestsDetail': 'From the local read-only llmusage archive',
  'monitoring.inputOutputDetail': '{input} input · {output} output',
  'monitoring.cacheDetail': '{cache} cache read',
  'monitoring.lastUpdated': 'Updated {time}',
  'monitoring.notUpdated': 'Not updated yet',
  'monitoring.healthEyebrow': 'Event Health',
  'monitoring.healthTitle': 'Event health overview',
  'monitoring.healthCritical': 'Errors present',
  'monitoring.healthAttention': 'Needs attention',
  'monitoring.healthHealthy': 'Healthy',
  'monitoring.healthQuiet': 'No events',
  'monitoring.recentUsageImport': 'Latest usage import event',
  'monitoring.noUsageImportEvent': 'No usage import event has been captured yet.',
  'monitoring.recentIssues': 'Recent errors / warnings',
  'monitoring.noRecentIssues': 'No errors or warnings in recent events.',
  'monitoring.logsEyebrow': 'Live Log',
  'monitoring.realTimeLogs': 'Live Event Stream',
  'monitoring.filteredCount': '{filtered} / {count} events',
  'monitoring.allLevels': 'All Levels',
  'monitoring.columnTime': 'Time',
  'monitoring.columnLevel': 'Level',
  'monitoring.columnChannel': 'Channel',
  'monitoring.columnSource': 'Source',
  'monitoring.columnMessage': 'Message',
  'monitoring.noLogs': 'No logs yet',
  'monitoring.waitingForLogs': 'New local events will appear here as the app records them.',
  'monitoring.noFilteredLogs': 'No events match this filter',
  'monitoring.adjustFilter': 'Switch the level filter to inspect other events.',
  'monitoring.levels.error': 'Error',
  'monitoring.levels.warn': 'Warning',
  'monitoring.levels.info': 'Info',
  'monitoring.levels.debug': 'Debug',
}

vi.mock('@/api', () => apiMocks)

vi.mock('@/composables/useMonitoringFeed', () => ({
  useMonitoringFeed: () => monitoringState,
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    locale: ref('en-US'),
    t: (key: string, values?: Record<string, unknown>) => {
      const template = translationTemplates[key] ?? key
      if (!values) return template
      return template.replace(/\{([a-zA-Z_][a-zA-Z0-9_]*)\}/g, (_match, name: string) => {
        const value = values[name]
        return value == null ? `{${name}}` : String(value)
      })
    },
  }),
}))

vi.mock('@/utils/tauriRuntime', () => ({
  isTauriRuntime: () => true,
}))

const createSummary = () => ({
  total_requests: 128,
  total_tokens: 987654,
  total_input_tokens: 345000,
  total_output_tokens: 642000,
  total_cache_read_tokens: 21000,
  total_cost_usd: 12.3456,
  cache_efficiency: 0.06,
})

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
  },
})

const createUnsupportedCapabilities = () => ({
  ...createSupportedCapabilities(),
  db_exists: false,
  db_readable: false,
  features: {
    overview: {
      supported: false,
      reason: 'db_missing',
      detail: 'llmusage DB does not exist at C:/Users/test/.llmusage/llmusage.db',
    },
  },
})

const createLogs = (): MonitoringEntry[] => [
  {
    id: 'event-error',
    timestamp: '2026-05-16T09:00:00Z',
    level: 'error',
    channel: 'usage',
    eventType: 'usage.import.failed',
    source: 'llmusage',
    message: 'Usage import failed for codex source',
  },
  {
    id: 'event-warn',
    timestamp: '2026-05-16T09:01:00Z',
    level: 'warn',
    channel: 'sync',
    eventType: 'sync.warning',
    source: 'sync-service',
    message: 'Sync completed with warnings',
  },
  {
    id: 'event-info',
    timestamp: '2026-05-16T09:02:00Z',
    level: 'info',
    channel: 'usage',
    eventType: 'usage.import.completed',
    source: 'llmusage',
    message: 'Usage archive refreshed',
  },
  {
    id: 'event-debug',
    timestamp: '2026-05-16T09:03:00Z',
    level: 'debug',
    channel: 'frontend',
    eventType: 'frontend.debug',
    source: 'monitoring-test',
    message: 'Debug payload normalized',
  },
]

const flushPromises = async () => {
  await Promise.resolve()
  await Promise.resolve()
  await Promise.resolve()
  await nextTick()
}

const mountMonitoringView = async () => {
  const MonitoringView = (await import('@/views/MonitoringView.vue')).default
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      return () => h(MonitoringView)
    },
  }))

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

beforeEach(() => {
  vi.clearAllMocks()
  monitoringState.isConnected.value = true
  monitoringState.logs.value = createLogs()
  monitoringState.clearLogs.mockImplementation(() => {
    monitoringState.logs.value = []
  })
  apiMocks.getUsageCapabilitiesV2.mockResolvedValue(createSupportedCapabilities())
  apiMocks.getUsageSummaryV2.mockResolvedValue(createSummary())
})

afterEach(() => {
  document.body.innerHTML = ''
  vi.restoreAllMocks()
})

describe('MonitoringView smoke', () => {
  it('renders real usage summary values instead of token-stats event counters', async () => {
    const { el, unmount } = await mountMonitoringView()

    try {
      expect(apiMocks.getUsageCapabilitiesV2).toHaveBeenCalledTimes(1)
      expect(apiMocks.getUsageSummaryV2).toHaveBeenCalledTimes(1)
      expect(el.textContent).toContain('128')
      expect(el.textContent).toContain('987.7K')
      expect(el.textContent).toContain('$12.3456')
      expect(el.textContent).toContain('345.0K input · 642.0K output')
      expect(el.textContent).toContain('Usage archive refreshed')
      expect(el.textContent).toContain('4 / 4 events')
      expect(el.textContent).not.toContain('0 Token')
      expect(el.textContent).not.toContain('$0.0000')
    } finally {
      unmount()
    }
  })

  it('shows a degraded usage state without misleading zero values when usage is unavailable', async () => {
    apiMocks.getUsageCapabilitiesV2.mockResolvedValue(createUnsupportedCapabilities())

    const { el, unmount } = await mountMonitoringView()

    try {
      expect(apiMocks.getUsageSummaryV2).not.toHaveBeenCalled()
      expect(el.querySelector('[data-testid="monitoring-usage-unavailable"]')).not.toBeNull()
      expect(el.textContent).toContain('Usage unavailable')
      expect(el.textContent).toContain('llmusage DB does not exist')
      expect(el.textContent).toContain('Real usage data is unavailable')
      expect(el.textContent).not.toContain('$0.0000')
      expect(el.textContent).not.toContain('0 Token')
    } finally {
      unmount()
    }
  })

  it('filters events by level and updates the filtered count', async () => {
    const { el, unmount } = await mountMonitoringView()

    try {
      const select = el.querySelector('[data-testid="monitoring-level-filter"]') as HTMLSelectElement
      select.value = 'warn'
      select.dispatchEvent(new Event('change'))
      await nextTick()

      expect(el.querySelector('[data-testid="monitoring-filtered-count"]')?.textContent).toContain('1 / 4 events')
      const rows = Array.from(el.querySelectorAll('[data-testid="monitoring-log-row"]'))
      expect(rows).toHaveLength(1)
      expect(rows[0].textContent).toContain('Sync completed with warnings')
      const tableText = rows.map((row) => row.textContent ?? '').join('\n')
      expect(tableText).not.toContain('Usage import failed for codex source')
      expect(tableText).not.toContain('Debug payload normalized')
    } finally {
      unmount()
    }
  })
})
