import { describe, expect, it } from 'vitest'
import type { UsageSummary } from '@/types/usage'
import type { UsageTrendBucket } from '@/views/usage/usageDashboardPresentation'
import {
  buildMetricStats,
  buildSummarySparklinePoints,
  buildUsageSummaryCards,
  formatCost,
} from '@/views/usage/usageSummaryCards'

const buildBucket = (
  startDate: string,
  overrides: Partial<UsageTrendBucket>,
): UsageTrendBucket => ({
  id: startDate,
  startDate,
  endDate: startDate,
  requestCount: 0,
  inputTokens: 0,
  outputTokens: 0,
  totalTokens: 0,
  cacheReadTokens: 0,
  cacheCreationTokens: 0,
  costUsd: 0,
  ...overrides,
})

const buildSummary = (overrides: Partial<UsageSummary> = {}): UsageSummary => ({
  total_requests: 30,
  total_tokens: 5400,
  total_input_tokens: 3000,
  total_output_tokens: 1200,
  total_cache_read_tokens: 1200,
  total_cost_usd: 6,
  cache_efficiency: 0.2857,
  ...overrides,
})

const translate = (
  _key: string,
  _values: Record<string, number | string> | undefined,
  fallback: string,
) => fallback

describe('usage summary card helpers', () => {
  it('builds metric stats with average, peak, positive delta, and tone', () => {
    expect(buildMetricStats([
      { label: '2026-05-01', value: 10 },
      { label: '2026-05-02', value: 20 },
      { label: '2026-05-03', value: 40 },
    ], (value) => value.toFixed(0))).toEqual({
      averageLabel: '23',
      peakLabel: '40',
      deltaLabel: '+300%',
      deltaTone: 'up',
    })
  })

  it('uses stable zero labels and flat tone when a metric has no points', () => {
    expect(buildMetricStats([], formatCost)).toEqual({
      averageLabel: '$0.0000',
      peakLabel: '$0.0000',
      deltaLabel: '0%',
      deltaTone: 'flat',
    })
  })

  it('builds active-day sparklines from token-bearing buckets', () => {
    const sparklines = buildSummarySparklinePoints([
      buildBucket('2026-05-01', {
        requestCount: 4,
        totalTokens: 150,
        inputTokens: 100,
        cacheReadTokens: 50,
        costUsd: 0.25,
      }),
      buildBucket('2026-05-02', {
        requestCount: 2,
        inputTokens: 0,
        cacheReadTokens: 0,
        totalTokens: 0,
      }),
    ])

    expect(sparklines.requests).toEqual([
      { label: '2026-05-01', value: 4 },
      { label: '2026-05-02', value: 2 },
    ])
    expect(sparklines.activeDays).toEqual([
      { label: '2026-05-01', value: 1 },
      { label: '2026-05-02', value: 0 },
    ])
  })

  it('builds summary cards with translated fallback labels and metric-specific formatters', () => {
    const sparklinePoints = buildSummarySparklinePoints([
      buildBucket('2026-05-01', {
        requestCount: 10,
        totalTokens: 1000,
        inputTokens: 800,
        outputTokens: 200,
        cacheReadTokens: 200,
        costUsd: 2,
      }),
      buildBucket('2026-05-02', {
        requestCount: 20,
        totalTokens: 2000,
        inputTokens: 1000,
        outputTokens: 400,
        cacheReadTokens: 500,
        costUsd: 4,
      }),
    ])

    const cards = buildUsageSummaryCards({
      summary: buildSummary(),
      activeDays: 2,
      modelCount: 2,
      projectCount: 1,
      sparklinePoints,
      selectedWindowLabel: '30 Days',
      translate,
    })

    const requests = cards.find((card) => card.id === 'requests')
    const cost = cards.find((card) => card.id === 'cost')
    const activeDays = cards.find((card) => card.id === 'activeDays')

    expect(cards.map((card) => card.id)).toEqual(['tokens', 'cost', 'activeDays', 'requests'])
    expect(requests).toMatchObject({
      label: 'Total Requests',
      value: '30',
      detail: '2 models · 1 projects',
      averageLabel: '15',
      peakLabel: '20',
      deltaLabel: '+100%',
      deltaTone: 'up',
      sparklineLabel: 'Total Requests trend for 30 Days',
    })
    expect(cost).toMatchObject({
      value: '$6.00',
      detail: '$0.2000 per request',
      averageLabel: '$3.00',
      peakLabel: '$4.00',
    })
    expect(activeDays).toMatchObject({
      label: 'Active Days',
      value: '2',
      detail: 'Days with token activity in 30 Days',
      averageLabel: '1',
      peakLabel: '1',
    })
  })
})
