import { createApp, nextTick } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { SystemInfo } from '@/types'
import type { HomeUsageOverviewResponse } from '@/types/usage'

vi.mock('@iconify/vue', () => ({
  Icon: {
    props: ['icon'],
    template: '<span data-icon="true" />',
  },
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      if (!params) return key
      const placeholders = Object.entries(params).map(([k, v]) => `${k}=${v}`).join(',')
      return placeholders ? `${key}{${placeholders}}` : key
    },
  }),
}))

const buildSystemInfo = (overrides: Partial<SystemInfo> = {}): SystemInfo => ({
  hostname: 'workbench-local',
  cpu_usage: 25,
  memory_usage_percent: 40,
  total_memory_gb: 16,
  used_memory_gb: 6.4,
  cpu_cores: 8,
  cpu_brand: 'Mock CPU',
  os_name: 'Windows',
  os_version: '11',
  uptime_seconds: 123,
  ...overrides,
} as SystemInfo)

const buildOverview = (overrides: Partial<HomeUsageOverviewResponse> = {}): HomeUsageOverviewResponse => ({
  summary: {
    total_sessions: 12,
    total_requests: 240,
    total_tokens: 13580,
    platforms: 3,
  },
  series: [],
  by_platform: {
    claude: { sessions: 4, requests: 80, tokens: 4500 },
    codex: { sessions: 4, requests: 80, tokens: 4500 },
    gemini: { sessions: 4, requests: 80, tokens: 4580 },
  },
  last_updated: '2026-05-18T08:00:00.000Z',
  empty_reason: null,
  ...overrides,
} as HomeUsageOverviewResponse)

const mountStatusBar = async (props: Record<string, unknown>) => {
  const { default: HomeStatusBar } = await import('@/components/home/HomeStatusBar.vue')
  const el = document.createElement('div')
  document.body.appendChild(el)
  const app = createApp(HomeStatusBar, props)
  app.mount(el)
  await nextTick()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

describe('HomeStatusBar smoke', () => {
  beforeEach(() => {
    vi.resetModules()
    document.body.innerHTML = ''
  })

  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('renders four chips and stays neutral when metrics are healthy', async () => {
    const { el, unmount } = await mountStatusBar({
      systemInfo: buildSystemInfo(),
      installedCliCount: 3,
      runtimeCliCount: 3,
      overview: buildOverview(),
      usageLoading: false,
    })

    try {
      const chips = el.querySelectorAll('.home-status-chip')
      expect(chips).toHaveLength(4)

      const cli = el.querySelector('.home-status-chip[data-tone="success"]')
      expect(cli).toBeTruthy()
      expect(cli?.textContent).toContain('3/3')

      const usage = el.querySelector('.home-status-chip[data-tone="accent"]')
      expect(usage).toBeTruthy()
    } finally {
      unmount()
    }
  })

  it('flags warning tone when CPU/memory exceeds thresholds', async () => {
    const { el, unmount } = await mountStatusBar({
      systemInfo: buildSystemInfo({ cpu_usage: 82, memory_usage_percent: 81 }),
      installedCliCount: 2,
      runtimeCliCount: 3,
      overview: buildOverview(),
      usageLoading: false,
    })

    try {
      const warning = el.querySelectorAll('.home-status-chip[data-tone="warning"]')
      expect(warning.length).toBeGreaterThanOrEqual(2)
    } finally {
      unmount()
    }
  })

  it('switches to danger when CPU/memory cross the high band', async () => {
    const { el, unmount } = await mountStatusBar({
      systemInfo: buildSystemInfo({ cpu_usage: 95, memory_usage_percent: 95 }),
      installedCliCount: 3,
      runtimeCliCount: 3,
      overview: buildOverview(),
      usageLoading: false,
    })

    try {
      const danger = el.querySelectorAll('.home-status-chip[data-tone="danger"]')
      expect(danger.length).toBeGreaterThanOrEqual(2)
    } finally {
      unmount()
    }
  })

  it('marks usage chip as warning when overview is missing or empty', async () => {
    const { el, unmount } = await mountStatusBar({
      systemInfo: buildSystemInfo(),
      installedCliCount: 3,
      runtimeCliCount: 3,
      overview: null,
      usageLoading: false,
    })

    try {
      const usageChip = Array.from(el.querySelectorAll<HTMLElement>('.home-status-chip'))
        .find((chip) => chip.textContent?.includes('home.usageMetricLabel'))
      expect(usageChip?.dataset.tone).toBe('warning')
    } finally {
      unmount()
    }
  })
})
