// @vitest-environment node

import { readFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { buildChartTheme, getTrendTickAmount, parseUtcDate } from '@/views/usage/usageChartOptions'
import {
  buildDailyBarChartOptions,
  dailyBarSeriesKey,
  stabilizeDailyBarSeries,
  toDailyBarPoints,
} from '@/views/usage/usageDailyBarChart'
import { formatCost, formatTokens } from '@/views/usage/usageSummaryCards'

const TOKENS_TAB_PATH = fileURLToPath(
  new URL('../../src/features/usage/components/UsageTokensTab.tsx', import.meta.url),
)
const COST_TAB_PATH = fileURLToPath(
  new URL('../../src/features/usage/components/UsageCostTab.tsx', import.meta.url),
)
const FACTORY_PATH = fileURLToPath(
  new URL('../../src/views/usage/usageDailyBarChart.ts', import.meta.url),
)

describe('usage daily bar charts', () => {
  it('maps YYYY-MM-DD rows onto UTC midnight timestamps', () => {
    const points = toDailyBarPoints(
      [{ date: '2026-07-22', cost_usd: 1.25 }, { date: '2026-08-01', cost_usd: 2 }],
      (row) => row.cost_usd,
    )

    expect(points).toEqual([
      { x: Date.UTC(2026, 6, 22), y: 1.25 },
      { x: Date.UTC(2026, 7, 1), y: 2 },
    ])
    expect(parseUtcDate('2026-07-22').getTime()).toBe(Date.UTC(2026, 6, 22))
  })

  it('keeps a 30-point window at six axis ticks and reuses equal series identity', () => {
    const rows = Array.from({ length: 30 }, (_, index) => ({
      date: `2026-07-${String(index + 1).padStart(2, '0')}`,
      value: index + 1,
    }))
    const first = [{ name: 'Cost', data: toDailyBarPoints(rows, (row) => row.value) }]
    const second = [{ name: 'Cost', data: toDailyBarPoints([...rows], (row) => row.value) }]

    expect(getTrendTickAmount(rows.length)).toBe(6)
    expect(dailyBarSeriesKey(first)).toBe(dailyBarSeriesKey(second))
    expect(stabilizeDailyBarSeries(first, second)).toBe(first)
  })

  it('builds datetime options without category ISO labels or parent-resize redraw', () => {
    const options = buildDailyBarChartOptions({
      theme: buildChartTheme(),
      locale: 'en-US',
      granularity: 'day',
      tickAmount: 6,
      stacked: true,
      palette: 'tokens',
      formatY: formatTokens,
    })

    expect(options.chart.redrawOnParentResize).toBe(false)
    expect(options.chart.redrawOnWindowResize).toBe(false)
    expect(options.chart.stacked).toBe(true)
    expect(options.xaxis.type).toBe('datetime')
    expect(options.xaxis.labels.trim).toBe(false)
    expect(options.xaxis.tickAmount).toBe(6)
    expect(options.legend.showForSingleSeries).toBe(true)
    expect(buildDailyBarChartOptions({
      theme: buildChartTheme(),
      locale: 'en-US',
      granularity: 'day',
      tickAmount: 6,
      stacked: false,
      palette: 'cost',
      formatY: formatCost,
    }).chart.stacked).toBe(false)
  })

  it('keeps Tokens and Cost hosts on the datetime factory without category axes', async () => {
    const tokensSource = await readFile(TOKENS_TAB_PATH, 'utf8')
    const costSource = await readFile(COST_TAB_PATH, 'utf8')
    const factorySource = await readFile(FACTORY_PATH, 'utf8')

    expect(factorySource).toContain("type: 'datetime'")
    expect(factorySource).toContain('trim: false')
    expect(factorySource).toContain('redrawOnParentResize: false')
    expect(factorySource).toContain('formatTrendAxisLabel')
    expect(tokensSource).toContain('buildDailyBarChartOptions')
    expect(tokensSource).toContain('getTrendTickAmount')
    expect(tokensSource).toContain('toDailyBarPoints')
    expect(tokensSource).not.toContain('categories:')
    expect(costSource).toContain('buildDailyBarChartOptions')
    expect(costSource).toContain('getTrendTickAmount')
    expect(costSource).not.toContain('categories:')
  })
})
