import { describe, expect, it } from 'vitest'
import { buildPlatformUsageSpec } from '@/views/platform-usage/platformUsageSpecs'
import {
  buildPlatformUsageInsight,
  hasPlatformUsageData,
} from '@/views/platform-usage/platformUsagePresentation'
import type { PlatformUsageDashboardData } from '@/types/platformUsageInsight'
import { makeModelStat } from './helpers/usageFixtures'

const baseData: PlatformUsageDashboardData = {
  generated_at: '2026-05-20T12:00:00.000Z',
  summary: {
    total_requests: 12,
    total_tokens: 9_000,
    total_input_tokens: 3_000,
    total_output_tokens: 4_000,
    total_cache_read_tokens: 2_000,
    total_cost_usd: 2.4,
    cache_efficiency: 0.4,
  },
  trends: [
    {
      date: '2026-05-19',
      request_count: 4,
      total_tokens: 2_000,
      input_tokens: 800,
      output_tokens: 900,
      reasoning_output_tokens: 0,
      cache_read_tokens: 300,
      cache_creation_tokens: 0,
      cost_usd: 0.7,
    },
    {
      date: '2026-05-20',
      request_count: 8,
      total_tokens: 7_000,
      input_tokens: 2_200,
      output_tokens: 3_100,
      reasoning_output_tokens: 0,
      cache_read_tokens: 1_700,
      cache_creation_tokens: 0,
      cost_usd: 1.7,
    },
  ],
  model_stats: [
    makeModelStat({
      model: 'gpt-5.5',
      request_count: 8,
      total_tokens: 7_000,
      total_cost: 1.7,
    }),
    makeModelStat({
      model: 'gpt-5.4',
      request_count: 4,
      total_tokens: 2_000,
      total_cost: 0.7,
    }),
  ],
  project_stats: [
    {
      project_path: 'D:/repo/a',
      request_count: 8,
      total_tokens: 7_000,
      total_cost: 1.7,
    },
    {
      project_path: '',
      request_count: 4,
      total_tokens: 2_000,
      total_cost: 0.7,
    },
  ],
}

describe('platform usage presentation smoke', () => {
  it('builds three KPI cards plus model and project rank rows from V2 usage data', () => {
    const result = buildPlatformUsageInsight({
      data: baseData,
      tone: 'codex',
      labels: {
        noProject: 'Unknown project',
      },
    })

    expect(result.empty).toBe(false)
    expect(result.cards).toHaveLength(3)
    expect(result.cards.map((card) => card.id)).toEqual(['cost', 'tokens', 'requests'])
    expect(result.modelRows[0]?.label).toBe('gpt-5.5')
    expect(result.projectRows[1]?.label).toBe('Unknown project')
    expect(result.topModelLabel).toBe('gpt-5.5')
    expect(result.pricingState).toBe('available')
  })

  it('marks token-only usage when tokens exist but pricing is unavailable', () => {
    const result = buildPlatformUsageInsight({
      data: {
        ...baseData,
        summary: {
          ...baseData.summary,
          total_cost_usd: 0,
        },
        model_stats: baseData.model_stats.map((item) => ({ ...item, total_cost: 0 })),
        project_stats: baseData.project_stats.map((item) => ({ ...item, total_cost: 0 })),
      },
    })

    expect(result.pricingState).toBe('token_only')
    expect(result.cards[0]?.value).toBe('Token only')
    expect(result.cards[0]?.detail).toBe('Pricing unavailable')
  })

  it('does not turn an empty dashboard into fake zero KPI data', () => {
    const result = buildPlatformUsageInsight({
      data: {
        generated_at: '2026-05-20T12:00:00.000Z',
        summary: {
          total_requests: 0,
          total_tokens: 0,
          total_input_tokens: 0,
          total_output_tokens: 0,
          total_cache_read_tokens: 0,
          total_cost_usd: 0,
          cache_efficiency: 0,
        },
        trends: [],
        model_stats: [],
        project_stats: [],
      },
    })

    expect(
      hasPlatformUsageData(result.summary, result.trends, result.modelStats, result.projectStats)
    ).toBe(false)
    expect(result.empty).toBe(true)
    expect(result.cards).toEqual([])
  })

  it('keeps the Antigravity label while using the legacy gemini platform key', () => {
    const messages: Record<string, string> = {
      'platformUsage.platforms.antigravity.label': 'Antigravity CLI',
      'platformUsage.platforms.antigravity.title': 'Antigravity Usage Insight',
    }
    const spec = buildPlatformUsageSpec((key) => messages[key] ?? key, 'gemini')

    expect(spec.label).toBe('Antigravity CLI')
    expect(spec.title).toBe('Antigravity Usage Insight')
    expect(spec.primaryActionTo).toBe('/usage?platform=gemini')
    expect(spec.platform).toBe('gemini')
  })
})
