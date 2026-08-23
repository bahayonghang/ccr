import { describe, expect, it } from 'vitest'
import type { ModelStat, ProjectStat, UsageArchiveDiagnostics, UsageSummary } from '@/types/usage'
import {
  buildDashboardMetaItems,
  buildOverviewHighlights,
  buildSelectedPlatformLabel,
  buildSelectedWindowLabel,
  buildTopModelRankings,
  buildTopProjectRankings,
  shortenPath,
} from '@/views/usage/usageOverviewInsights'
import type { UsageTrendBucket } from '@/views/usage/usageDashboardPresentation'
import { makeArchiveDiagnostics } from './helpers/usageFixtures'

const translate = (
  key: string,
  values: Record<string, number | string> | undefined,
  fallback: string
) => (values ? `${fallback} ${JSON.stringify(values)}` : fallback)

const model = (overrides: Partial<ModelStat> & { model: string }): ModelStat => ({
  request_count: 1,
  total_tokens: 100,
  total_cost: 1,
  cost_with_cache: overrides.total_cost ?? 1,
  input_tokens: 0,
  output_tokens: 0,
  cache_read_tokens: 0,
  cache_creation_tokens: 0,
  cost_without_cache: 0,
  cache_savings: 0,
  pricing_status: 'priced',
  pricing_source: null,
  pricing_rate: null,
  ...overrides,
})

const project = (overrides: Partial<ProjectStat> & { project_path: string }): ProjectStat => ({
  request_count: 1,
  total_tokens: 100,
  total_cost: 1,
  ...overrides,
})

const bucket = (overrides: Partial<UsageTrendBucket> = {}): UsageTrendBucket => ({
  id: '2026-05-19',
  startDate: '2026-05-19',
  endDate: '2026-05-19',
  requestCount: 1,
  inputTokens: 10,
  outputTokens: 5,
  reasoningOutputTokens: 0,
  totalTokens: 15,
  cacheReadTokens: 2,
  cacheCreationTokens: 1,
  costUsd: 0.1,
  ...overrides,
})

const summary = (overrides: Partial<UsageSummary> = {}): UsageSummary => ({
  total_requests: 10,
  total_tokens: 1000,
  total_input_tokens: 500,
  total_output_tokens: 250,
  total_cache_read_tokens: 250,
  total_cost_usd: 3,
  cache_efficiency: 0.25,
  ...overrides,
})

describe('usage overview insight helpers', () => {
  it('builds selected labels and archive meta without depending on view-local derived state', () => {
    expect(buildSelectedPlatformLabel('', translate)).toBe('All Platforms')
    expect(buildSelectedPlatformLabel('codex', translate)).toBe('Codex')
    expect(buildSelectedPlatformLabel('custom', translate)).toBe('custom')
    expect(buildSelectedWindowLabel('last_30d', translate)).toBe('Last 30 Days')
    expect(buildSelectedWindowLabel('all_time', translate)).toBe('All Time')
    expect(shortenPath('D:/workspace/a/b/c')).toBe('.../b/c')

    const archive: UsageArchiveDiagnostics = makeArchiveDiagnostics({
      archive_root: 'D:/workspace/archive/live',
      live_sources: 2,
      missing_sources: 1,
      deleted_sources: 0,
      archived_sessions: 4,
      recent_completed_at: '2026-05-19T01:00:00Z',
      history_completed_at: null,
    })

    expect(
      buildDashboardMetaItems({
        archive,
        locale: 'en-US',
        modelCount: 3,
        projectCount: 2,
        selectedPlatformLabel: 'Codex',
        selectedWindowLabel: '30 Days',
        translate,
      }).map((item) => [item.id, item.value])
    ).toEqual([
      ['scope', 'Codex'],
      ['window', '30 Days'],
      ['models', '3'],
      ['projects', '2'],
      ['archive', 'Live 2 · Missing 1 · Deleted 0'],
      ['archive-root', '.../archive/live'],
      ['archive-time', expect.any(String)],
    ])
  })

  it('builds overview highlights with top model/project/cache semantics', () => {
    const highlights = buildOverviewHighlights({
      modelStats: [
        model({ model: 'gpt-5.5', total_tokens: 2000, total_cost: 8, cost_with_cache: 6 }),
      ],
      projectStats: [
        project({ project_path: 'D:/repos/acme/app', request_count: 12, total_cost: 2 }),
      ],
      summary: summary({ total_cache_read_tokens: 1200, cache_efficiency: 0.4 }),
      trendBuckets: [bucket(), bucket({ id: '2026-05-20', startDate: '2026-05-20' })],
      trendGranularityLabel: 'Daily',
      selectedWindowLabel: '30 Days',
      translate,
    })

    expect(highlights.map((item) => item.id)).toEqual([
      'density',
      'top-model',
      'top-project',
      'cache',
    ])
    expect(highlights.find((item) => item.id === 'top-model')).toMatchObject({
      value: 'gpt-5.5',
      detail: '$6.00 · 2.0K',
    })
    expect(highlights.find((item) => item.id === 'top-project')?.value).toBe('.../acme/app')
    expect(highlights.find((item) => item.id === 'cache')?.detail).toContain('40.0%')
  })

  it('sorts top rankings with stable tie breakers and share values', () => {
    const modelRankings = buildTopModelRankings(
      [
        model({
          model: 'cheap-large',
          request_count: 5,
          total_tokens: 1000,
          total_cost: 1,
          cost_with_cache: 1,
        }),
        model({
          model: 'expensive',
          request_count: 2,
          total_tokens: 100,
          total_cost: 5,
          cost_with_cache: 5,
        }),
        model({
          model: 'same-cost-more-tokens',
          request_count: 1,
          total_tokens: 200,
          total_cost: 5,
          cost_with_cache: 5,
        }),
      ],
      translate
    )

    expect(modelRankings.map((item) => item.id)).toEqual([
      'same-cost-more-tokens',
      'expensive',
      'cheap-large',
    ])
    expect(modelRankings[0].share).toBeCloseTo(5 / 11)

    const projectRankings = buildTopProjectRankings(
      [
        project({ project_path: 'D:/a', total_cost: 1, total_tokens: 200, request_count: 2 }),
        project({ project_path: 'D:/b', total_cost: 3, total_tokens: 100, request_count: 1 }),
      ],
      translate
    )

    expect(projectRankings.map((item) => item.id)).toEqual(['D:/b', 'D:/a'])
    expect(projectRankings[0]).toMatchObject({
      label: 'D:/b',
      value: '$3.00',
      share: 0.75,
    })
  })
})
