import { describe, expect, it } from 'vitest'
import type {
  DailyTrend,
  ModelStat,
  PaginatedLogs,
  ProviderBreakdown,
  ProjectStat,
  SourceBreakdown,
  UsageArchiveDiagnostics,
  UsageSummary,
} from '@/types/usage'
import {
  buildDashboardCachePayload,
  buildDashboardFetchKey,
  buildUsageLogsQuery,
  isCapabilityUnsupported,
  normalizeDashboardPayload,
  normalizePaginatedLogs,
  parseEnvFlag,
  type UsageDashboardPayload,
} from '@/stores/usageDashboardPayload'
import {
  makeArchiveDiagnostics,
  makeFeatureCapability,
  makeModelStat,
} from './helpers/usageFixtures'

const summary: UsageSummary = {
  total_requests: 1,
  total_tokens: 10,
  total_input_tokens: 4,
  total_output_tokens: 3,
  total_cache_read_tokens: 3,
  total_cost_usd: 0.01,
  cache_efficiency: 0.2,
}

const trend: DailyTrend = {
  date: '2026-05-19',
  request_count: 1,
  total_tokens: 10,
  input_tokens: 4,
  output_tokens: 3,
  reasoning_output_tokens: 1,
  cache_read_tokens: 3,
  cache_creation_tokens: 0,
  cost_usd: 0.01,
}

const model: ModelStat = makeModelStat({
  model: 'gpt-5.5',
  request_count: 1,
  total_tokens: 10,
  total_cost: 0.01,
  cost_with_cache: 0.01,
})

const project: ProjectStat = {
  project_path: 'D:/repo',
  request_count: 1,
  total_tokens: 10,
  total_cost: 0.01,
}

const source: SourceBreakdown = {
  source: 'codex',
  event_count: 1,
  total_tokens: 10,
  total_cost: 0.01,
  active_days: 1,
  share_tokens: 1,
  share_cost: 1,
}

const provider: ProviderBreakdown = {
  provider: 'openai',
  request_count: 1,
  input_tokens: 4,
  cache_read_tokens: 3,
  cache_creation_tokens: 0,
  output_tokens: 3,
  reasoning_output_tokens: 1,
  total_tokens: 10,
  cost_with_cache_usd: 0.01,
  cost_without_cache_usd: 0.02,
}

const archive: UsageArchiveDiagnostics = makeArchiveDiagnostics({
  archive_root: 'D:/archive',
  live_sources: 1,
  missing_sources: 0,
  deleted_sources: 0,
  archived_sessions: 1,
})

describe('usage dashboard payload helpers', () => {
  it('parses env flags and builds stable dashboard fetch keys', () => {
    expect(parseEnvFlag(undefined, true)).toBe(true)
    expect(parseEnvFlag('', false)).toBe(false)
    expect(parseEnvFlag('YES', false)).toBe(true)
    expect(parseEnvFlag('0', true)).toBe(false)
    expect(
      buildDashboardFetchKey({
        platform: 'codex',
        provider: 'openai',
        start: '2026-05-01',
        end: '2026-05-19',
        includeHeatmap: false,
      })
    ).toBe('codex|openai|2026-05-01|2026-05-19|core')
  })

  it('normalizes aggregated dashboard payload aliases and heatmap inclusion', () => {
    // 模拟仅带 by_model/by_project 别名的后端载荷（缺少 model_stats/project_stats），用断言绕过必填字段。
    const payload = {
      summary,
      trends: [trend],
      by_model: [model],
      by_project: [project],
      provider_stats: [provider],
      source_stats: [source],
      archive,
      heatmap: { data: { '2026-05-19': 1 } },
    } as unknown as UsageDashboardPayload

    expect(normalizeDashboardPayload(payload, true)).toMatchObject({
      summary,
      trends: [trend],
      modelStats: [model],
      projectStats: [project],
      providerStats: [provider],
      sourceStats: [source],
      archive,
      heatmap: { data: { '2026-05-19': 1 } },
    })
    expect(normalizeDashboardPayload(payload, false).heatmap).toBeUndefined()
  })

  it('builds cache payloads without leaking omitted heatmap fetches', () => {
    expect(
      buildDashboardCachePayload({
        summary,
        trends: [trend],
        modelStats: [model],
        projectStats: [project],
        providerStats: [provider],
        sourceStats: [source],
        archive,
        heatmap: { data: { '2026-05-19': 1 } },
        includeHeatmap: false,
      })
    ).toMatchObject({
      summary,
      trends: [trend],
      model_stats: [model],
      project_stats: [project],
      provider_stats: [provider],
      source_stats: [source],
      archive,
      heatmap: undefined,
    })
  })

  it('builds cursor log queries and normalizes paginated log responses', () => {
    expect(
      buildUsageLogsQuery({
        platform: 'claude',
        model: 'opus',
        startDate: '2026-05-01',
        endDate: '2026-05-19',
        page: 2,
        pageSize: 25,
        cursor: null,
        includeTotal: false,
      })
    ).toEqual({
      platform: 'claude',
      model: 'opus',
      start_date: '2026-05-01',
      end_date: '2026-05-19',
      page: 2,
      page_size: 25,
      cursor: undefined,
      include_total: false,
      mode: 'cursor',
    })

    const logs: PaginatedLogs = {
      records: [],
      page: 1,
      page_size: 10,
      total: null,
      next_cursor: null,
      mode: 'offset',
    }
    expect(normalizePaginatedLogs(logs, 2, 25, 99)).toMatchObject({
      page: 2,
      page_size: 10,
      total: 99,
      mode: 'cursor',
    })
  })

  it('keeps unsupported capability checks explicit', () => {
    expect(isCapabilityUnsupported(null)).toBe(false)
    expect(isCapabilityUnsupported(makeFeatureCapability({ supported: true }))).toBe(false)
    expect(
      isCapabilityUnsupported(makeFeatureCapability({ supported: false, reason: 'cli_missing' }))
    ).toBe(true)
  })
})
