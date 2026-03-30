import { createApp, defineComponent, h, nextTick, ref } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const getCodexDashboardOverviewMock = vi.fn()
const getCodexDashboardUsageSummaryMock = vi.fn()
const getCliVersionMock = vi.fn()

vi.mock('@/api', () => ({
  getCodexDashboardOverview: (...args: unknown[]) => getCodexDashboardOverviewMock(...args),
  getCodexDashboardUsageSummary: (...args: unknown[]) => getCodexDashboardUsageSummaryMock(...args),
}))

vi.mock('@/api/runtime/system', () => ({
  getCliVersion: (...args: unknown[]) => getCliVersionMock(...args),
}))

vi.mock('@/utils/perfTelemetry', () => ({
  perfMark: vi.fn(),
  perfMeasure: vi.fn(),
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => key,
    locale: ref('en-US'),
  }),
}))

const overviewResponse = {
  auth: {
    logged_in: true,
    saved_accounts_total: 1,
    expired_accounts_total: 0,
    current: {
      name: 'primary',
      freshness_description: '已登录，可继续使用',
      is_expired: false,
    },
  },
  profiles: {
    current_profile: 'work',
    total: 1,
    enabled_total: 1,
    disabled_total: 0,
    current: null,
  },
  config: {
    model: 'gpt-5.4',
    approval_policy: 'never',
    sandbox_mode: 'danger-full-access',
  },
  inventory: {
    mcp_servers_total: 0,
    agents_total: 2,
    sessions_total: 8,
    config_profiles_total: 1,
  },
}

const usageSummaryResponse = {
  last_activity_at: '2026-03-30T12:00:00.000Z',
  freshness: 'fresh' as const,
  freshness_description: '最近 6 小时内有使用记录',
  five_hour: {
    total_requests: 4,
    total_input_tokens: 1_000,
    total_output_tokens: 2_000,
  },
  seven_day: {
    total_requests: 12,
    total_input_tokens: 5_000,
    total_output_tokens: 8_000,
  },
  all_time: {
    total_requests: 42,
    total_input_tokens: 15_000,
    total_output_tokens: 21_000,
  },
  top_model: {
    model: 'gpt-5.4',
    total_requests: 42,
    total_input_tokens: 15_000,
    total_output_tokens: 21_000,
    window_end: '2026-03-30T12:00:00.000Z',
  },
}

const cliVersionResponse = {
  platform: 'codex',
  installed: true,
  version: '0.117.0',
  status: 'ok',
  elapsed_ms: 120,
}

const flushPromises = async () => {
  await Promise.resolve()
  await nextTick()
  await nextTick()
}

const mountComposable = async () => {
  const { useCodexDashboard } = await import('@/composables/useCodexDashboard')

  let state: ReturnType<typeof useCodexDashboard> | null = null
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      state = useCodexDashboard()
      return () => h('div')
    },
  }))

  app.mount(el)
  await nextTick()

  return {
    state: state!,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

beforeEach(() => {
  vi.resetModules()
  document.body.innerHTML = ''

  getCodexDashboardOverviewMock.mockReset()
  getCodexDashboardUsageSummaryMock.mockReset()
  getCliVersionMock.mockReset()

  getCodexDashboardOverviewMock.mockResolvedValue(overviewResponse)
  getCodexDashboardUsageSummaryMock.mockResolvedValue(usageSummaryResponse)
  getCliVersionMock.mockResolvedValue(cliVersionResponse)
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('codex dashboard smoke', () => {
  it('reuses shared dashboard cache across mounts within the TTL window', async () => {
    const first = await mountComposable()

    try {
      await first.state.refresh(false)
      expect(getCodexDashboardOverviewMock).toHaveBeenCalledTimes(1)
      expect(getCodexDashboardUsageSummaryMock).toHaveBeenCalledTimes(1)
      expect(getCliVersionMock).toHaveBeenCalledTimes(1)
    } finally {
      first.unmount()
    }

    const second = await mountComposable()

    try {
      await second.state.refresh(false)
      expect(getCodexDashboardOverviewMock).toHaveBeenCalledTimes(1)
      expect(getCodexDashboardUsageSummaryMock).toHaveBeenCalledTimes(1)
      expect(getCliVersionMock).toHaveBeenCalledTimes(1)
      expect(second.state.currentAccountLabel.value).toBe('primary')
      expect(second.state.usageTotalRequests.value).toBe(42)
    } finally {
      second.unmount()
    }
  })

  it('derives overview-driven content without waiting for usage summary', async () => {
    const mounted = await mountComposable()

    try {
      mounted.state.overview.value = overviewResponse
      mounted.state.usageSummary.value = null
      mounted.state.usageLoading.value = true
      await nextTick()

      expect(mounted.state.nextActions.value).toHaveLength(1)
      expect(mounted.state.nextActions.value[0]?.to).toBe('/codex/mcp')
      expect(mounted.state.usageLoading.value).toBe(true)
      expect(mounted.state.usageSummary.value).toBeNull()
      expect(mounted.state.healthItems.value.find((item) => item.key === 'usage')?.value).toBe('分析中')
    } finally {
      mounted.unmount()
    }
  })
})
