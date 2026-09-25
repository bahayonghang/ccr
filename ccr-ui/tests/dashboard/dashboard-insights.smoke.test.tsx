import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, render, screen, within } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { HomeInsightsResponse } from '@/types/generated/usage/HomeInsightsResponse'
import { DashboardInsights } from '@/features/usage/dashboard/insights/DashboardInsights'
import { makeInsights } from './insightsFixtures'

const { useHomeInsights } = vi.hoisted(() => ({ useHomeInsights: vi.fn() }))

vi.mock('@/utils/scheduling', () => ({
  scheduleWhenIdle: (task: () => void) => {
    task()
    return () => undefined
  },
}))

vi.mock('@/features/usage/queries', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/features/usage/queries')>()
  return { ...actual, useHomeInsights }
})

const emptyTotals = { requests: 0, tokens: 0, sessions: 0, agents: 0, projects: 0, active_days: 0 }
const onImport = vi.fn()

const renderInsights = (data: HomeInsightsResponse | null, isNativeRuntime = true) => {
  useHomeInsights.mockReturnValue({ data: data ?? undefined, error: null })
  return render(
    <QueryClientProvider client={new QueryClient()}>
      <DashboardInsights isNativeRuntime={isNativeRuntime} onImport={onImport} importing={false} />
    </QueryClientProvider>
  )
}

const panel = (title: RegExp | string) =>
  screen.getByRole('heading', { name: title }).closest('section') as HTMLElement

beforeEach(() => {
  useHomeInsights.mockReset()
  onImport.mockReset()
})

describe('dashboard insights: empty states', () => {
  it('web preview makes no call and shows the preview notice', () => {
    renderInsights(null, false)
    expect(useHomeInsights).toHaveBeenCalledWith({ enabled: false })
    expect(document.querySelector('[data-state="web-preview"]')).not.toBeNull()
  })

  it('empty store shows dashed panel, dash totals, and import action', () => {
    renderInsights(
      makeInsights({
        totals: emptyTotals,
        daily: [],
        trend: [],
        agents: [],
        projects: [],
        models: [],
      })
    )
    expect(document.querySelector('[data-state="empty"]')).not.toBeNull()
    const values = document.querySelectorAll('.insights-totals__value')
    expect(values).toHaveLength(6)
    values.forEach((value) => expect(value.textContent).toBe('—'))
    fireEvent.click(
      within(document.querySelector('[data-state="empty"]') as HTMLElement).getByRole('button')
    )
    expect(onImport).toHaveBeenCalledTimes(1)
  })

  it('sessions only hides trend and distribution and defaults the leaderboard to sessions', () => {
    renderInsights(
      makeInsights({
        totals: { ...emptyTotals, sessions: 12 },
        daily: [],
        trend: [],
        busiest_day: null,
      })
    )
    expect(document.querySelector('.insights-trend')).toBeNull()
    expect(document.querySelector('.insights-distribution')).toBeNull()
    // 仅有榜单时该行必须是单列，不能留半行空白。
    const singleRow = document.querySelector('.insights-row--single') as HTMLElement
    expect(singleRow).not.toBeNull()
    expect(singleRow.querySelectorAll('.insights-panel')).toHaveLength(1)
    const metric = screen.getByRole('radiogroup', {
      name: 'dashboard.insights.leaderboard.metricLabel',
    })
    expect(within(metric).getByRole('radio', { checked: true }).dataset.value).toBe('sessions')
  })

  it('usage only shows — with a title for sessions and disables the session metric', () => {
    renderInsights(makeInsights({ sessions_indexed: false }))
    const sessions = document.querySelector('.insights-totals__value') as HTMLElement
    expect(sessions.textContent).toBe('—')
    expect(sessions.title).toBe('dashboard.insights.unindexedTitle')
    expect(screen.getByText('dashboard.insights.unindexedNotice')).toBeTruthy()
    const compareRow = document.querySelector('[data-row="sessions"]') as HTMLElement
    expect(compareRow.querySelectorAll('td')[0].textContent).toBe('—')
    const metric = screen.getByRole('radiogroup', {
      name: 'dashboard.insights.leaderboard.metricLabel',
    })
    const sessionsOption = within(metric).getByRole('radio', {
      name: 'dashboard.insights.leaderboard.sessions',
    }) as HTMLButtonElement
    expect(sessionsOption.disabled).toBe(true)
    expect(sessionsOption.title).toBe('dashboard.insights.unindexedTitle')
  })
})

describe('dashboard insights: keyboard switches and tables', () => {
  it('moves metric, dimension, and distribution selection with arrow keys', () => {
    renderInsights(makeInsights())
    const metric = screen.getByRole('radiogroup', {
      name: 'dashboard.insights.leaderboard.metricLabel',
    })
    fireEvent.keyDown(within(metric).getByRole('radio', { checked: true }), { key: 'ArrowRight' })
    expect(within(metric).getByRole('radio', { checked: true }).dataset.value).toBe('tokens')
    const dimension = screen.getByRole('radiogroup', {
      name: 'dashboard.insights.leaderboard.dimensionLabel',
    })
    fireEvent.keyDown(within(dimension).getByRole('radio', { checked: true }), {
      key: 'ArrowRight',
    })
    expect(within(dimension).getByRole('radio', { checked: true }).dataset.value).toBe('projects')
    const distribution = screen.getByRole('radiogroup', {
      name: 'dashboard.insights.distribution.viewLabel',
    })
    fireEvent.keyDown(within(distribution).getByRole('radio', { checked: true }), {
      key: 'ArrowLeft',
    })
    expect(within(distribution).getByRole('radio', { checked: true }).dataset.value).toBe('month')
    // 切到月份视图后，小时口径的「最忙时段」说明行必须消失。
    expect(
      document.querySelector('[data-testid="insights-distribution-caption"]')
    ).toBeNull()
    const hourView = within(distribution).getAllByRole('radio')[0]
    fireEvent.click(hourView)
    expect(
      document.querySelector('[data-testid="insights-distribution-caption"]')?.textContent
    ).toContain('busiestHour')
  })

  it('heatmap table lists the newest week first with blank future days', () => {
    renderInsights(makeInsights())
    const heatmap = panel('dashboard.insights.heatmap.title')
    const views = within(heatmap).getByRole('radiogroup')
    fireEvent.keyDown(within(views).getByRole('radio', { checked: true }), { key: 'ArrowRight' })
    const rows = within(heatmap).getAllByRole('row')
    expect(rows).toHaveLength(54)
    const cells = within(rows[1])
      .getAllByRole('cell')
      .map((cell) => cell.textContent)
    expect(within(rows[1]).getByRole('rowheader').textContent).toBe('2026-09-21')
    expect(cells.slice(0, 7)).toEqual(['0', '40', '0', '70', '', '', ''])
  })

  it('trend table lists the newest week first with one column per series', () => {
    renderInsights(makeInsights())
    const trend = panel('dashboard.insights.trend.title')
    fireEvent.keyDown(
      within(within(trend).getByRole('radiogroup')).getByRole('radio', { checked: true }),
      { key: 'ArrowRight' }
    )
    const rows = within(trend).getAllByRole('row')
    expect(rows).toHaveLength(54)
    expect(
      within(rows[0])
        .getAllByRole('columnheader')
        .map((cell) => cell.textContent)
    ).toEqual([
      'dashboard.insights.weekStart',
      'Claude',
      'Codex',
      'dashboard.insights.weekTotal',
    ])
    expect(
      within(rows[1])
        .getAllByRole('cell')
        .map((cell) => cell.textContent)
    ).toEqual(['80', '30', '110'])
  })
})

describe('dashboard insights: error state', () => {
  const renderWithError = (data: HomeInsightsResponse | undefined) => {
    useHomeInsights.mockReturnValue({ data, error: new Error('boom') })
    return render(
      <QueryClientProvider client={new QueryClient()}>
        <DashboardInsights isNativeRuntime onImport={onImport} importing={false} />
      </QueryClientProvider>
    )
  }

  it('shows the error state when there is no data', () => {
    renderWithError(undefined)
    expect(document.querySelector('[data-state="error"]')).not.toBeNull()
  })

  it('keeps the last data on a background refetch error', () => {
    renderWithError(makeInsights())
    expect(document.querySelector('[data-state="error"]')).toBeNull()
    expect(document.querySelectorAll('.insights-totals__value')).toHaveLength(6)
  })
})
