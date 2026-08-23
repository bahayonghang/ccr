// @vitest-environment node

import { readFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import type { DailyTrend } from '@/types/usage'
import {
  buildPlatformUsageTrendSeries,
  platformUsageTrendSeriesKey,
} from '@/views/platform-usage/platformUsageTrendChart'
import { getTrendTickAmount, parseUtcDate } from '@/views/usage/usageChartOptions'

const CHART_VUE_PATH = fileURLToPath(
  new URL('../src/features/usage/platform/PlatformUsageTrendChart.tsx', import.meta.url),
)
const CHART_HELPER_PATH = fileURLToPath(
  new URL('../src/views/platform-usage/platformUsageTrendChart.ts', import.meta.url),
)

const trend = (date: string, overrides: Partial<DailyTrend> = {}): DailyTrend => ({
  date,
  request_count: 4,
  total_tokens: 2_000,
  input_tokens: 800,
  output_tokens: 900,
  reasoning_output_tokens: 0,
  cache_read_tokens: 300,
  cache_creation_tokens: 0,
  cost_usd: 0.7,
  ...overrides,
})

describe('platform usage trend chart helpers', () => {
  it('maps YYYY-MM-DD rows onto UTC midnight timestamps', () => {
    const series = buildPlatformUsageTrendSeries(
      [trend('2026-07-22', { cost_usd: 1.25 }), trend('2026-08-01', { cost_usd: 2 })],
      'cost',
    )

    expect(series).toHaveLength(1)
    expect(series[0]?.name).toBe('Cost')
    expect(series[0]?.data).toEqual([
      { x: Date.UTC(2026, 6, 22), y: 1.25 },
      { x: Date.UTC(2026, 7, 1), y: 2 },
    ])
    expect(parseUtcDate('2026-07-22').getTime()).toBe(Date.UTC(2026, 6, 22))
  })

  it('keeps a 30-point window at six axis ticks and memoizes equal series', () => {
    const trends = Array.from({ length: 30 }, (_, index) =>
      trend(`2026-07-${String(index + 1).padStart(2, '0')}`),
    )
    const first = buildPlatformUsageTrendSeries(trends, 'tokens')
    const second = buildPlatformUsageTrendSeries([...trends], 'tokens')

    expect(getTrendTickAmount(trends.length)).toBe(6)
    expect(first).toHaveLength(4)
    expect(platformUsageTrendSeriesKey(first)).toBe(platformUsageTrendSeriesKey(second))
  })

  it('keeps the platform chart on a datetime axis without label trimming', async () => {
    const vueSource = await readFile(CHART_VUE_PATH, 'utf8')
    const helperSource = await readFile(CHART_HELPER_PATH, 'utf8')

    expect(helperSource).not.toContain('trim: true')
    expect(vueSource).not.toContain('trim: true')
    expect(vueSource).toContain("type: 'datetime'")
    expect(vueSource).toContain('trim: false')
    expect(vueSource).toContain('redrawOnParentResize: false')
    expect(vueSource).toContain('redrawOnWindowResize: false')
    expect(vueSource).toContain('buildChartAnimations()')
    expect(vueSource).toContain('formatTrendAxisLabel')
    expect(vueSource).toContain('getTrendTickAmount')
  })
})
