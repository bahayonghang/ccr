import { describe, expect, it } from 'vitest'
import type { ModelStat, UsageArchiveDiagnostics, UsageRecordV2, UsageSummary } from '@/types/usage'
import {
  buildChartTheme,
  buildTrendTooltipHtml,
  escapeTooltipText,
  formatTrendAxisLabel,
  formatTrendTooltipLabel,
  getTrendTickAmount,
  parseUtcDate,
} from '@/views/usage/usageChartOptions'
import {
  buildUsageDiagnosticsSummary,
  shouldRecommendCodexRepair,
} from '@/views/usage/usageDiagnostics'
import { formatCost, formatTokens } from '@/views/usage/usageSummaryCards'
import { makeArchiveDiagnostics, makeModelStat } from './helpers/usageFixtures'

const summary = (overrides: Partial<UsageSummary> = {}): UsageSummary => ({
  total_requests: 10,
  total_tokens: 100,
  total_input_tokens: 50,
  total_output_tokens: 30,
  total_cache_read_tokens: 20,
  total_cost_usd: 1,
  cache_efficiency: 0.2,
  ...overrides,
})

const logRecord = (recordedAt: string): UsageRecordV2 => ({
  id: 'log-1',
  platform: 'codex',
  project_path: 'D:/repo',
  record_json: '{}',
  recorded_at: recordedAt,
  source_id: 'source-1',
  model: 'unknown',
  input_tokens: 1,
  output_tokens: 2,
  cache_read_tokens: 3,
  cache_creation_tokens: 4,
  cost_with_cache_usd: 0,
  cost_without_cache_usd: 0,
  pricing_status: 'missing',
  pricing_source: null,
})

describe('usage chart and diagnostics helpers', () => {
  it('builds chart theme tokens without enabling resize or animation defaults', () => {
    document.documentElement.setAttribute('data-theme', 'dark')
    expect(buildChartTheme()).toMatchObject({ mode: 'dark' })
    document.documentElement.setAttribute('data-theme', 'light')
    expect(buildChartTheme()).toMatchObject({ mode: 'light' })
  })

  it('keeps trend axis and tooltip labels deterministic', () => {
    expect(getTrendTickAmount(0)).toBeUndefined()
    expect(getTrendTickAmount(8)).toBe(8)
    expect(getTrendTickAmount(20)).toBe(6)
    expect(parseUtcDate('2026-07-22').getTime()).toBe(Date.UTC(2026, 6, 22))
    expect(formatTrendAxisLabel(Date.UTC(2026, 0, 5), 'day', 'en-US')).toBe('Jan 5')
    expect(formatTrendAxisLabel(Date.UTC(2026, 6, 22), 'day', 'en-US')).toBe('Jul 22')
    expect(formatTrendAxisLabel(Date.UTC(2026, 6, 22), 'day', 'zh-CN')).toBe('7月22日')
    expect(formatTrendTooltipLabel('2026-01-05', '2026-01-11', 'week', 'en-US')).toBe(
      'Jan 5 - Jan 11'
    )
    expect(formatTrendTooltipLabel('2026-01-01', '2026-01-31', 'month', 'en-US')).toBe('Jan 2026')
    expect(escapeTooltipText('<Cost & "tokens">')).toBe('&lt;Cost &amp; &quot;tokens&quot;&gt;')
  })

  it('builds escaped shared tooltip html with cost row', () => {
    const html = buildTrendTooltipHtml({
      context: {
        dataPointIndex: 0,
        series: [[1000], [2000]],
        w: { globals: { colors: ['#111', '#222'], seriesNames: ['Input <unsafe>', 'Output'] } },
      },
      buckets: [
        {
          id: '2026-01-05',
          startDate: '2026-01-05',
          endDate: '2026-01-05',
          displayEndDate: '2026-01-05',
          requestCount: 1,
          inputTokens: 1000,
          outputTokens: 2000,
          reasoningOutputTokens: 0,
          totalTokens: 3000,
          cacheReadTokens: 0,
          cacheCreationTokens: 0,
          costUsd: 0.25,
        },
      ],
      granularity: 'day',
      locale: 'en-US',
      seriesNames: ['Input', 'Output'],
      fallbackColor: '#000',
      costLabel: 'Cost <USD>',
      formatTokens,
      formatCost,
    })

    expect(html).toContain('Input &lt;unsafe&gt;')
    expect(html).toContain('Cost &lt;USD&gt;')
    expect(html).toContain('$0.2500')
  })

  it('summarizes diagnostics and recommends Codex repair only for risky Codex data', () => {
    const unknownModel: ModelStat = makeModelStat({
      model: 'unknown',
      request_count: 2,
      total_tokens: 10,
      total_cost: 0,
    })
    const archive: UsageArchiveDiagnostics = makeArchiveDiagnostics({
      archive_root: 'C:/usage.db',
      live_sources: 1,
      missing_sources: 2,
      deleted_sources: 3,
      archived_sessions: 4,
    })
    const messages = {
      noRecentRecord: 'No recent record',
      rawLogsHint: 'Raw logs unavailable',
      repairNeeded: 'Repair needed',
      codexRepairHint: (unknown: string) => `${unknown} unknown model requests`,
      healthy: 'Healthy',
    }

    expect(
      shouldRecommendCodexRepair({
        selectedPlatform: 'codex',
        summary: summary(),
        unknownModelStat: unknownModel,
      })
    ).toBe(true)

    expect(
      buildUsageDiagnosticsSummary({
        selectedPlatform: 'codex',
        summary: summary(),
        logsRecords: [logRecord('2026-05-19T01:00:00Z')],
        logsTotalCount: 12,
        unknownModelStat: unknownModel,
        archive,
        locale: 'en-US',
        messages,
      })
    ).toMatchObject({
      totalRecords: '12',
      healthLabel: 'Repair needed',
      healthDetail: '2 unknown model requests · Archive 4 · L 1 / M 2 / D 3',
      repairRecommended: true,
      canRepairCodex: true,
    })

    expect(
      buildUsageDiagnosticsSummary({
        selectedPlatform: 'claude',
        summary: summary(),
        logsRecords: [],
        logsTotalCount: 0,
        unknownModelStat: unknownModel,
        archive: null,
        locale: 'en-US',
        messages,
      })
    ).toMatchObject({
      latestRecordAt: 'No recent record',
      healthLabel: 'Healthy',
      healthDetail: 'Raw logs unavailable',
      repairRecommended: false,
      canRepairCodex: false,
    })
  })
})
