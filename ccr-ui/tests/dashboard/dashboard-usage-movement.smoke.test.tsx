import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, fireEvent, render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  DashboardCostMetric,
  deriveStackedUsageBars,
  formatCostMetric,
  formatCostUsd,
  homeDateWindow,
  type UsageStackPlatform,
} from '@/features/usage/dashboard/DashboardCostMetric'
import { DashboardUsageMovement } from '@/features/usage/dashboard/DashboardUsageMovement'
import type { HomeOverviewSeriesItem, HomeUsageOverviewResponse } from '@/types/usage'
import { makeArchiveDiagnostics, makeSnapshotProjection } from '../helpers/usageFixtures'

const { idleTasks, useUsageSummary } = vi.hoisted(() => ({
  idleTasks: [] as Array<() => void>,
  useUsageSummary: vi.fn(),
}))

vi.mock('@/utils/scheduling', () => ({
  scheduleWhenIdle: (task: () => void) => {
    idleTasks.push(task)
    return () => undefined
  },
}))

vi.mock('@/features/usage/queries', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/features/usage/queries')>()
  return { ...actual, useUsageSummary }
})

const zeroStats = { sessions: 0, requests: 0, tokens: 0 }
const stats = (requests: number, extra: { sessions?: number; tokens?: number } = {}) => ({
  sessions: extra.sessions ?? 0,
  requests,
  tokens: extra.tokens ?? 0,
})

const seriesItem = (
  date: string,
  values: Partial<Record<UsageStackPlatform, number>> = {},
): HomeOverviewSeriesItem => ({
  date,
  claude: stats(values.claude ?? 0),
  codex: stats(values.codex ?? 0),
  antigravity: stats(values.antigravity ?? 0),
  opencode: stats(values.opencode ?? 0),
})

const overview = (overrides: Partial<HomeUsageOverviewResponse> = {}): HomeUsageOverviewResponse => ({
  summary: {
    total_sessions: 0,
    total_requests: 12,
    total_tokens: 3400,
    active_days: 2,
    platforms: 2,
  },
  by_platform: {
    claude: stats(8),
    codex: stats(4),
    gemini: zeroStats,
    opencode: zeroStats,
  },
  series: [
    seriesItem('2026-08-19', { claude: 3, codex: 1 }),
    seriesItem('2026-08-20', { claude: 5, codex: 3 }),
  ],
  archive: makeArchiveDiagnostics(),
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
  snapshot: makeSnapshotProjection(),
  empty_reason: null,
  last_updated: '2026-08-25T03:09:00Z',
  ...overrides,
})

const flushIdle = () => {
  const pending = [...idleTasks]
  idleTasks.length = 0
  pending.forEach((task) => task())
}

const renderMovement = (props: Partial<Parameters<typeof DashboardUsageMovement>[0]> = {}) => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const onChangeDays = vi.fn()
  const view = render(
    <QueryClientProvider client={client}>
      <MemoryRouter>
        <DashboardUsageMovement
          overview={overview()}
          loading={false}
          error={null}
          activeDays={7}
          onChangeDays={onChangeDays}
          {...props}
        />
      </MemoryRouter>
    </QueryClientProvider>,
  )
  return { ...view, onChangeDays, client }
}

describe('homeDateWindow', () => {
  it('maps 7/30/90 days onto a local inclusive window ending today', () => {
    const now = new Date(2026, 7, 25, 15, 40, 0)
    expect(homeDateWindow(7, now)).toEqual({ startDate: '2026-08-19', endDate: '2026-08-25' })
    expect(homeDateWindow(30, now)).toEqual({ startDate: '2026-07-27', endDate: '2026-08-25' })
    expect(homeDateWindow(90, now)).toEqual({ startDate: '2026-05-28', endDate: '2026-08-25' })
  })

  it('formats local YYYY-MM-DD near midnight without UTC day shift', () => {
    const now = new Date(2026, 7, 25, 0, 30, 0)
    const spy = vi.spyOn(Date.prototype, 'toISOString')
    const window = homeDateWindow(7, now)
    expect(window).toEqual({ startDate: '2026-08-19', endDate: '2026-08-25' })
    expect(spy).not.toHaveBeenCalled()
    spy.mockRestore()
  })

  it('clamps invalid day counts to a one-day window', () => {
    const now = new Date(2026, 7, 25, 12, 0, 0)
    expect(homeDateWindow(0, now)).toEqual({ startDate: '2026-08-25', endDate: '2026-08-25' })
  })
})

describe('deriveStackedUsageBars', () => {
  it('stacks per-day platform requests and keeps only nonzero legend entries', () => {
    const chart = deriveStackedUsageBars([
      seriesItem('2026-08-19', { claude: 2, codex: 2 }),
      seriesItem('2026-08-20', { claude: 6, opencode: 2 }),
    ])
    expect(chart.empty).toBe(false)
    expect(chart.maxDailyTotal).toBe(8)
    expect(chart.legend).toEqual(['claude', 'codex', 'opencode'])
    expect(chart.bars).toHaveLength(2)
    expect(chart.bars[1]?.heightPercent).toBe(100)
    expect(chart.bars[0]?.segments.map((segment) => segment.platform)).toEqual(['claude', 'codex'])
    expect(chart.bars[1]?.segments.find((segment) => segment.platform === 'opencode')?.heightPercent).toBe(25)
  })

  it('uses the empty branch when maxDailyTotal is 0', () => {
    const chart = deriveStackedUsageBars([
      seriesItem('2026-08-19'),
      seriesItem('2026-08-20'),
    ])
    expect(chart).toEqual({ maxDailyTotal: 0, empty: true, legend: [], bars: [] })
  })
})

describe('formatCostMetric', () => {
  it('keeps unavailable, zero, and positive values distinguishable', () => {
    expect(formatCostMetric({ mounted: false, isLoading: false, isError: false, totalCostUsd: 12 })).toBe('—')
    expect(formatCostMetric({ mounted: true, isLoading: true, isError: false, totalCostUsd: 0 })).toBe('—')
    expect(formatCostMetric({ mounted: true, isLoading: false, isError: true, totalCostUsd: 4 })).toBe('—')
    expect(formatCostMetric({ mounted: true, isLoading: false, isError: false, totalCostUsd: null })).toBe('—')
    expect(formatCostMetric({ mounted: true, isLoading: false, isError: false, totalCostUsd: 0 })).toBe('$0.00')
    expect(formatCostMetric({ mounted: true, isLoading: false, isError: false, totalCostUsd: 128.4 })).toBe('$128.40')
    expect(formatCostUsd(0)).toBe('$0.00')
  })
})

describe('DashboardUsageMovement', () => {
  beforeEach(() => {
    idleTasks.length = 0
    useUsageSummary.mockReturnValue({ isPending: true, isError: false, data: undefined })
  })

  it('does not mount the cost metric until idle', () => {
    renderMovement()
    expect(document.querySelector('[data-dashboard-cost-placeholder]')?.textContent).toBe('—')
    expect(document.querySelector('[data-dashboard-cost-metric]')).toBeNull()
    expect(useUsageSummary).not.toHaveBeenCalled()
    act(() => {
      flushIdle()
    })
    expect(document.querySelector('[data-dashboard-cost-placeholder]')).toBeNull()
    expect(document.querySelector('[data-dashboard-cost-metric]')).toBeTruthy()
    const window = homeDateWindow(7)
    expect(useUsageSummary).toHaveBeenCalledWith(undefined, window.startDate, window.endDate)
  })

  it('renders stacked bars and a legend of platforms with requests', () => {
    renderMovement()
    const stacks = document.querySelectorAll('[data-dashboard-usage-bars] .dashboard-usage-stack')
    expect(stacks).toHaveLength(2)
    expect(document.querySelector('[data-platform="claude"]')).toBeTruthy()
    expect(document.querySelector('[data-platform="codex"]')).toBeTruthy()
    expect(document.querySelector('.dashboard-usage__legend [data-platform="opencode"]')).toBeNull()
    expect(document.querySelector('[data-hero="true"]')?.textContent).toBe('12')
    expect(screen.getByRole('img', { name: /7/ })).toBeTruthy()
  })

  it('keeps loading and error cards visible and switches 7D/30D', () => {
    const loading = renderMovement({ overview: null, loading: true })
    expect(document.querySelector('[data-state="loading"]')).toBeTruthy()
    expect(document.querySelector('[data-dashboard-usage-skeleton]')).toBeTruthy()
    loading.unmount()

    const errored = renderMovement({ overview: null, loading: false, error: 'boom' })
    expect(document.querySelector('[data-state="error"]')).toBeTruthy()
    expect(screen.getByText('boom')).toBeTruthy()
    fireEvent.click(screen.getByRole('button', { name: '重试' }))
    expect(errored.onChangeDays).toHaveBeenCalledWith(7)
    fireEvent.click(screen.getByRole('radio', { name: '30D' }))
    expect(errored.onChangeDays).toHaveBeenCalledWith(30)
    errored.unmount()

    renderMovement({
      overview: overview({
        summary: { total_sessions: 0, total_requests: 0, total_tokens: 0, active_days: 2, platforms: 0 },
        series: [seriesItem('2026-08-19'), seriesItem('2026-08-20')],
      }),
    })
    expect(document.querySelector('[data-state="empty"]')).toBeTruthy()
    expect(document.querySelector('[data-dashboard-usage-bars]')).toBeNull()
    expect(document.querySelector('[data-zero="true"]')?.textContent).toBe('0')
  })
})

describe('DashboardCostMetric', () => {
  beforeEach(() => {
    useUsageSummary.mockReset()
  })

  it('passes the activeDays window into useUsageSummary and shows three cost states', () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    useUsageSummary.mockReturnValue({ isPending: false, isError: false, data: { total_cost_usd: 0 } })
    const view = render(
      <QueryClientProvider client={client}>
        <DashboardCostMetric days={7} />
      </QueryClientProvider>,
    )
    expect(document.querySelector('[data-cost-state="zero"]')?.textContent).toBe('$0.00')
    expect(useUsageSummary.mock.calls[0]?.slice(0, 3)).toEqual([
      undefined,
      homeDateWindow(7).startDate,
      homeDateWindow(7).endDate,
    ])

    useUsageSummary.mockReturnValue({ isPending: false, isError: true, data: undefined })
    view.rerender(
      <QueryClientProvider client={client}>
        <DashboardCostMetric days={30} />
      </QueryClientProvider>,
    )
    expect(document.querySelector('[data-cost-state="unavailable"]')?.textContent).toBe('—')
    expect(useUsageSummary.mock.calls.at(-1)?.slice(0, 3)).toEqual([
      undefined,
      homeDateWindow(30).startDate,
      homeDateWindow(30).endDate,
    ])

    useUsageSummary.mockReturnValue({ isPending: false, isError: false, data: { total_cost_usd: 12.5 } })
    view.rerender(
      <QueryClientProvider client={client}>
        <DashboardCostMetric days={90} />
      </QueryClientProvider>,
    )
    expect(document.querySelector('[data-cost-state="value"]')?.textContent).toBe('$12.50')
  })
})
