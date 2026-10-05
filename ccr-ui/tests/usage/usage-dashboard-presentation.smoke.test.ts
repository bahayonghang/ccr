import { describe, expect, it } from 'vitest'
import type { DailyTrend, ModelStat } from '@/types/usage'
import {
  aggregateDailyTrends,
  buildUsageDashboardPresentation,
  groupModelDistribution,
  groupModelTokenDistribution,
  selectTrendGranularity,
} from '@/views/usage/usageDashboardPresentation'
import { makeModelStat } from '../helpers/usageFixtures'

const buildTrend = (date: string, multiplier: number): DailyTrend => ({
  date,
  request_count: multiplier,
  total_tokens: multiplier * 20,
  input_tokens: multiplier * 10,
  output_tokens: multiplier * 5,
  reasoning_output_tokens: multiplier,
  cache_read_tokens: multiplier * 2,
  cache_creation_tokens: multiplier * 3,
  cost_usd: multiplier,
})

describe('usage dashboard presentation helpers', () => {
  it('uses weekly buckets for 90 day windows and preserves totals', () => {
    const raw = [
      buildTrend('2026-01-05', 1),
      buildTrend('2026-01-06', 2),
      buildTrend('2026-01-12', 3),
      buildTrend('2026-01-20', 4),
    ]

    expect(selectTrendGranularity(90)).toBe('week')

    const buckets = aggregateDailyTrends(raw, 'week')

    expect(buckets).toHaveLength(3)
    expect(buckets[0]).toMatchObject({
      startDate: '2026-01-05',
      endDate: '2026-01-06',
      totalTokens: 60,
      inputTokens: 30,
      outputTokens: 15,
      reasoningOutputTokens: 3,
      cacheReadTokens: 6,
      cacheCreationTokens: 9,
    })

    expect(buckets.reduce((sum, bucket) => sum + bucket.costUsd, 0)).toBe(10)
    expect(buckets.reduce((sum, bucket) => sum + bucket.requestCount, 0)).toBe(10)
  })

  it('uses monthly buckets for 365 day windows', () => {
    expect(selectTrendGranularity(365)).toBe('month')

    const buckets = aggregateDailyTrends(
      [
        buildTrend('2026-01-03', 1),
        buildTrend('2026-01-18', 2),
        buildTrend('2026-02-02', 3),
        buildTrend('2026-02-27', 4),
      ],
      'month'
    )

    expect(buckets).toHaveLength(2)
    expect(buckets[0]).toMatchObject({
      startDate: '2026-01-01',
      endDate: '2026-01-18',
      totalTokens: 60,
      inputTokens: 30,
      reasoningOutputTokens: 3,
      cacheCreationTokens: 9,
    })
    expect(buckets[1]).toMatchObject({
      startDate: '2026-02-01',
      endDate: '2026-02-27',
      totalTokens: 140,
      inputTokens: 70,
      reasoningOutputTokens: 7,
      cacheCreationTokens: 21,
    })
  })

  it('groups model cost distribution into top six plus others', () => {
    const models = Array.from({ length: 8 }, (_, index): ModelStat =>
      makeModelStat({
        model: `model-${index + 1}`,
        request_count: 10 - index,
        total_tokens: (index + 1) * 100,
        total_cost: 8 - index,
        cost_with_cache: 8 - index,
      })
    )

    const slices = groupModelDistribution(models, 6)

    expect(slices).toHaveLength(7)
    expect(slices[slices.length - 1]).toMatchObject({
      id: 'others',
      isOther: true,
      childCount: 2,
      totalCost: 3,
      totalTokens: 1500,
      requestCount: 7,
    })
    expect(slices.reduce((sum, slice) => sum + slice.totalCost, 0)).toBe(36)
  })

  it('can group model distribution by token usage for the models tab', () => {
    const models: ModelStat[] = [
      makeModelStat({
        model: 'expensive-small',
        request_count: 3,
        total_tokens: 1000,
        total_cost: 100,
        cost_with_cache: 100,
      }),
      makeModelStat({
        model: 'cheap-large',
        request_count: 2,
        total_tokens: 10000,
        total_cost: 1,
        cost_with_cache: 1,
      }),
    ]

    const costSlices = groupModelDistribution(models)
    const tokenSlices = groupModelTokenDistribution(models)

    expect(costSlices[0]).toMatchObject({
      id: 'expensive-small',
      share: 100 / 101,
    })
    expect(tokenSlices[0]).toMatchObject({
      id: 'cheap-large',
      share: 10000 / 11000,
    })
  })

  it('builds centralized dashboard presentation without repeating chart derivations', () => {
    const presentation = buildUsageDashboardPresentation({
      modelStats: [
        makeModelStat({
          model: 'expensive-small',
          request_count: 3,
          total_tokens: 1000,
          total_cost: 100,
          cost_with_cache: 100,
        }),
        makeModelStat({
          model: 'cheap-large',
          request_count: 2,
          total_tokens: 10000,
          total_cost: 1,
          cost_with_cache: 1,
        }),
      ],
      projectStats: [
        {
          project_path: 'D:/repo/usage-dashboard',
          request_count: 5,
          total_tokens: 11000,
          total_cost: 101,
        },
      ],
      selectedWindowLabel: '30 Days',
      summary: {
        total_requests: 5,
        total_tokens: 11000,
        total_input_tokens: 7000,
        total_output_tokens: 3000,
        total_cache_read_tokens: 1000,
        total_cost_usd: 101,
        cache_efficiency: 0.125,
      },
      translate: (_key, _values, fallback) => fallback,
      trendGranularity: 'day',
      trendGranularityLabel: 'Daily',
      trends: [buildTrend('2026-01-05', 1), buildTrend('2026-01-06', 2)],
    })

    expect(presentation.trendBuckets).toHaveLength(2)
    expect(presentation.summaryCards.map((card) => card.id)).toEqual([
      'tokens',
      'cost',
      'activeDays',
      'requests',
    ])
    expect(presentation.summaryCards.find((card) => card.id === 'activeDays')?.value).toBe('2')
    expect(presentation.trendSeries.map((series) => series.name)).toEqual([
      'Input',
      'Output',
      'Cache Read',
    ])
    expect(presentation.modelDistribution[0]?.label).toBe('expensive-small')
    expect(presentation.modelTokenDistribution[0]?.label).toBe('cheap-large')
    expect(presentation.pieSeries).toEqual([100, 1])
    expect(presentation.modelTokenPieSeries).toEqual([10000, 1000])
    expect(presentation.overviewHighlights.map((item) => item.id)).toEqual([
      'density',
      'top-model',
      'top-project',
      'cache',
    ])
    expect(presentation.topModelRankings[0]?.label).toBe('expensive-small')
    expect(presentation.topProjectRankings[0]?.title).toBe('D:/repo/usage-dashboard')
  })
})
