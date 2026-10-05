import { describe, expect, it } from 'vitest'
import type { DailyTrend } from '@/types/usage'
import {
  getAssistantOutputTokens,
  getUsageTokenRowChartTotal,
  sumUsageTokenBreakdownRow,
  toUsageTokenBreakdownRows,
} from '@/views/usage/usageTokenBreakdown'

const trend = (overrides: Partial<DailyTrend> = {}): DailyTrend => ({
  date: '2026-05-21',
  request_count: 1,
  total_tokens: 150,
  input_tokens: 70,
  output_tokens: 50,
  reasoning_output_tokens: 20,
  cache_read_tokens: 25,
  cache_creation_tokens: 5,
  cost_usd: 0.1,
  ...overrides,
})

describe('usage token breakdown helpers', () => {
  it('derives assistant output from the compatible output field', () => {
    expect(getAssistantOutputTokens(trend())).toBe(30)
    expect(getAssistantOutputTokens(trend({ output_tokens: 8, reasoning_output_tokens: 20 })))
      .toBe(0)
  })

  it('keeps reasoning out of assistant output so breakdown math does not double count', () => {
    const rows = toUsageTokenBreakdownRows([trend()])

    expect(rows[0]).toMatchObject({
      inputTokens: 70,
      assistantOutputTokens: 30,
      cacheReadTokens: 25,
      cacheCreationTokens: 5,
      reasoningOutputTokens: 20,
      totalTokens: 150,
    })
    expect(sumUsageTokenBreakdownRow(rows[0])).toBe(150)
    expect(getUsageTokenRowChartTotal(rows[0])).toBe(150)
  })
})
