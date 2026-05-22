import type { DailyTrend } from '@/types/usage'

export type UsageTokenSeriesKey =
  | 'input'
  | 'assistantOutput'
  | 'cacheRead'
  | 'cacheCreation'
  | 'reasoning'

export type UsageTokenBreakdownMode = 'breakdown' | 'total'

export interface UsageTokenBreakdownRow {
  date: string
  inputTokens: number
  assistantOutputTokens: number
  cacheReadTokens: number
  cacheCreationTokens: number
  reasoningOutputTokens: number
  totalTokens: number
}

const safeNumber = (value: number | null | undefined) =>
  Number.isFinite(value) ? Math.max(0, Number(value)) : 0

export const getAssistantOutputTokens = (trend: DailyTrend) =>
  Math.max(0, safeNumber(trend.output_tokens) - safeNumber(trend.reasoning_output_tokens))

export const toUsageTokenBreakdownRows = (trends: DailyTrend[]): UsageTokenBreakdownRow[] =>
  trends.map((trend) => ({
    date: trend.date,
    inputTokens: safeNumber(trend.input_tokens),
    assistantOutputTokens: getAssistantOutputTokens(trend),
    cacheReadTokens: safeNumber(trend.cache_read_tokens),
    cacheCreationTokens: safeNumber(trend.cache_creation_tokens),
    reasoningOutputTokens: safeNumber(trend.reasoning_output_tokens),
    totalTokens: safeNumber(trend.total_tokens),
  }))

export const sumUsageTokenBreakdownRow = (
  row: Pick<
    UsageTokenBreakdownRow,
    | 'inputTokens'
    | 'assistantOutputTokens'
    | 'cacheReadTokens'
    | 'cacheCreationTokens'
    | 'reasoningOutputTokens'
  >,
) =>
  row.inputTokens +
  row.assistantOutputTokens +
  row.cacheReadTokens +
  row.cacheCreationTokens +
  row.reasoningOutputTokens

export const getUsageTokenRowChartTotal = (row: UsageTokenBreakdownRow) =>
  row.totalTokens > 0 ? row.totalTokens : sumUsageTokenBreakdownRow(row)

export const sumUsageTokenBreakdownRows = (rows: UsageTokenBreakdownRow[]) =>
  rows.reduce(
    (totals, row) => ({
      inputTokens: totals.inputTokens + row.inputTokens,
      assistantOutputTokens: totals.assistantOutputTokens + row.assistantOutputTokens,
      cacheReadTokens: totals.cacheReadTokens + row.cacheReadTokens,
      cacheCreationTokens: totals.cacheCreationTokens + row.cacheCreationTokens,
      reasoningOutputTokens: totals.reasoningOutputTokens + row.reasoningOutputTokens,
      totalTokens: totals.totalTokens + getUsageTokenRowChartTotal(row),
    }),
    {
      inputTokens: 0,
      assistantOutputTokens: 0,
      cacheReadTokens: 0,
      cacheCreationTokens: 0,
      reasoningOutputTokens: 0,
      totalTokens: 0,
    },
  )
