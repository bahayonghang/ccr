import { describe, expect, it } from 'vitest'
import {
  buildHeatmap,
  buildHeatTable,
  buildLeaderboard,
  buildStreakCaption,
  buildTotals,
  buildTrend,
  buildTrendTable,
  buildWeekCompare,
  busiestHour,
  defaultLeaderMetric,
  formatChange,
  heatLevel,
  heatThresholds,
  percentChange,
  pickTopSources,
  resolveLeaderMetric,
} from '@/features/usage/dashboard/insights/insightsPresentation'
import {
  INSIGHTS_OTHER_KEY,
  sourceColor,
  sourceLabel,
} from '@/features/usage/dashboard/insights/insightsSources'
import { INSIGHTS_ZEROS, makeInsights } from './insightsFixtures'

const t = (key: string, values?: Record<string, string>) =>
  values ? `${key}(${Object.values(values).join(',')})` : key
const week = (last: number) => [...INSIGHTS_ZEROS(52), last]

describe('insights presentation: top N and other', () => {
  it('picks the top 4 mapped sources by total and breaks ties by SourceKind order', () => {
    const trend = [
      { source: 'pi', weekly: week(10) },
      { source: 'codex', weekly: week(10) },
      { source: 'claude', weekly: week(50) },
      { source: 'zcode', weekly: week(10) },
      { source: 'grok', weekly: week(10) },
      { source: 'mystery', weekly: week(999) },
    ]
    expect(pickTopSources(trend)).toEqual(['claude', 'codex', 'pi', 'grok'])
  })

  it('merges the rest and unmapped sources into other, in fixed stack order', () => {
    const data = makeInsights({
      trend: [
        { source: 'codex', weekly: week(40) },
        { source: 'claude', weekly: week(30) },
        { source: 'pi', weekly: week(20) },
        { source: 'grok', weekly: week(10) },
        { source: 'zcode', weekly: week(5) },
        { source: 'mystery', weekly: week(3) },
      ],
    })
    const model = buildTrend(data)
    expect(model.series.map((series) => series.key)).toEqual([
      'claude',
      'grok',
      'codex',
      'pi',
      INSIGHTS_OTHER_KEY,
    ])
    expect(model.series[4].total).toBe(8)
    expect(model.weeks).toHaveLength(53)
    expect(model.max).toBe(108)
    const table = buildTrendTable(model)
    expect(table[0].weekStart).toBe('2026-09-21')
    expect(table[0].total).toBe(108)
    expect(table[52].weekStart).toBe('2025-09-22')
  })
})

describe('insights presentation: 7-day change', () => {
  it('formats percent, flat, new, and none', () => {
    expect(formatChange(percentChange(73, 60), t)).toBe('+21.7%')
    expect(formatChange(percentChange(47, 57), t)).toBe('−17.5%')
    expect(formatChange(percentChange(5, 5), t)).toBe('dashboard.insights.compare.flat')
    expect(formatChange(percentChange(5, 0), t)).toBe('dashboard.insights.compare.added')
    expect(formatChange(percentChange(0, 0), t)).toBe('—')
  })

  it('uses a day delta for active days and hides values without requests', () => {
    const rows = buildWeekCompare(makeInsights())
    expect(formatChange(rows[2].change, t)).toBe('dashboard.insights.compare.dayDelta(+2)')
    const sessionsOnly = buildWeekCompare(
      makeInsights({
        totals: { requests: 0, tokens: 0, sessions: 4, agents: 0, projects: 0, active_days: 0 },
      })
    )
    expect(sessionsOnly.map((row) => row.last)).toEqual([5, null, null])
  })

  it('shows sessions as null when sessions are not indexed', () => {
    const data = makeInsights({ sessions_indexed: false })
    expect(buildWeekCompare(data)[0]).toMatchObject({
      last: null,
      previous: null,
      change: { kind: 'none' },
    })
    expect(buildTotals(data)[0]).toMatchObject({ key: 'sessions', value: null, unindexed: true })
  })
})

describe('insights presentation: heatmap and streak', () => {
  it('builds 53 x 7 cells, marks days after as_of, and levels by quantile', () => {
    const model = buildHeatmap(makeInsights())
    expect(model.weeks).toHaveLength(53)
    const last = model.weeks[52]
    expect(last.cells.map((cell) => cell.future)).toEqual([
      false,
      false,
      false,
      false,
      true,
      true,
      true,
    ])
    expect(last.cells[3]).toMatchObject({ date: '2026-09-24', requests: 70, level: 4 })
    const rows = buildHeatTable(model)
    expect(rows[0].days).toEqual([0, 40, 0, 70, null, null, null])
    expect(heatLevel(0, heatThresholds([1, 2, 3, 4]))).toBe(0)
    expect(heatLevel(1, heatThresholds([1, 2, 3, 4]))).toBe(1)
  })

  it('builds the streak caption and hides it without requests', () => {
    const parts = buildStreakCaption(makeInsights())
    expect(parts.map((part) => part.key)).toEqual([
      'dashboard.insights.heatmap.currentStreak',
      'dashboard.insights.heatmap.longestStreak',
      'dashboard.insights.heatmap.busiestDay',
    ])
    expect(parts[2].values).toEqual({ date: '2026-09-24', count: 70 })
    expect(
      buildStreakCaption(
        makeInsights({
          totals: { requests: 0, tokens: 0, sessions: 0, agents: 0, projects: 0, active_days: 0 },
        })
      )
    ).toEqual([])
  })

  it('finds the busiest hour and returns null for all-zero data', () => {
    expect(busiestHour(makeInsights())).toBe(14)
    expect(busiestHour(makeInsights({ hourly: INSIGHTS_ZEROS(24) }))).toBeNull()
  })
})

describe('insights presentation: leaderboard', () => {
  const label = (source: string) => sourceLabel(source, (key) => key)

  it('re-sorts by metric', () => {
    const data = makeInsights()
    expect(
      buildLeaderboard(data, { dimension: 'agents', metric: 'requests' }, label).map(
        (row) => row.key
      )
    ).toEqual(['claude', 'codex'])
    expect(
      buildLeaderboard(data, { dimension: 'agents', metric: 'sessions' }, label).map(
        (row) => row.key
      )
    ).toEqual(['codex', 'claude'])
  })

  it('falls back to requests for sessions outside agents or when unindexed', () => {
    expect(resolveLeaderMetric('projects', 'sessions', true)).toBe('requests')
    expect(resolveLeaderMetric('agents', 'sessions', false)).toBe('requests')
    expect(
      defaultLeaderMetric(
        makeInsights({
          totals: { requests: 0, tokens: 0, sessions: 3, agents: 0, projects: 0, active_days: 0 },
        })
      )
    ).toBe('sessions')
  })

  it('places unmapped sources after mapped ones with raw names and a neutral color', () => {
    const data = makeInsights({
      agents: [
        { source: 'mystery', requests: 500, tokens: 0, sessions: 0, unmapped: true },
        { source: 'codex', requests: 40, tokens: 0, sessions: 0, unmapped: false },
      ],
    })
    const rows = buildLeaderboard(data, { dimension: 'agents', metric: 'requests' }, label)
    expect(rows.map((row) => row.label)).toEqual(['Codex', 'mystery'])
    expect(rows[1].color).toBe('var(--color-chart-other)')
    expect(sourceColor('kimi_code')).toBe('var(--color-platform-kimi-code)')
    expect(sourceColor('deepseek_harness')).toBe('var(--color-platform-deepseek-harness)')
    expect(sourceColor('mystery')).toBe('var(--color-chart-other)')
  })
})
