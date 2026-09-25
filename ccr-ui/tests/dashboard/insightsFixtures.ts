import type { HomeInsightsResponse } from '@/types/generated/usage/HomeInsightsResponse'

const zeros = (length: number) => Array.from({ length }, () => 0)

/** Home Insights 测试夹具：默认 as_of 2026-09-24（周四），trend_start 为其所在周一减 52 周。 */
export const makeInsights = (
  overrides: Partial<HomeInsightsResponse> = {}
): HomeInsightsResponse => ({
  as_of: '2026-09-24',
  first_date: '2026-09-01',
  totals: { requests: 120, tokens: 50_000, sessions: 12, agents: 2, projects: 1, active_days: 3 },
  sessions_indexed: true,
  last_7_days: { requests: 73, sessions: 5, active_days: 3 },
  previous_7_days: { requests: 60, sessions: 5, active_days: 1 },
  daily: [
    { date: '2026-09-01', requests: 10 },
    { date: '2026-09-22', requests: 40 },
    { date: '2026-09-24', requests: 70 },
  ],
  current_streak: 1,
  longest_streak: 2,
  busiest_day: { date: '2026-09-24', requests: 70 },
  hourly: zeros(24).map((_, index) => (index === 14 ? 50 : index === 9 ? 20 : 0)),
  weekday: [10, 40, 0, 70, 0, 0, 0],
  monthly: zeros(12).map((_, index) => (index === 8 ? 120 : 0)),
  trend_start: '2025-09-22',
  trend: [
    { source: 'claude', weekly: [...zeros(52), 80] },
    { source: 'codex', weekly: [...zeros(51), 10, 30] },
  ],
  agents: [
    { source: 'claude', requests: 80, tokens: 30_000, sessions: 4, unmapped: false },
    { source: 'codex', requests: 40, tokens: 20_000, sessions: 8, unmapped: false },
  ],
  projects: [{ key: 'p1', label: 'ccr', requests: 120, tokens: 50_000 }],
  models: [{ key: 'm1', label: 'model-a', requests: 120, tokens: 50_000 }],
  generated_at: '2026-09-24T08:00:00Z',
  ...overrides,
})

export const INSIGHTS_ZEROS = zeros
