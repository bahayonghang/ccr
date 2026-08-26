import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { describe, expect, it, vi } from 'vitest'
import { readFileSync } from 'node:fs'
import path from 'node:path'

vi.mock('@/api', () => ({
  claudeObserver: {
    getInsight: vi.fn().mockResolvedValue({
      today_value_usd: 1.2,
      month_value_usd: 8,
      total_value_usd: 20,
      today_tokens: 1000,
      month_tokens: 8000,
      total_sessions: 3,
      total_projects: 2,
      subscription: { mode: 'auto', plan: 'free_pro', monthly_usd: 0 },
      roi: null,
      pricing_version: '1',
    }),
    dailyTrend: vi.fn().mockResolvedValue([]),
    costBreakdown: vi.fn().mockResolvedValue([]),
    cacheStats: vi.fn().mockResolvedValue({
      hit_rate: 0.4,
      total_input_tokens: 10,
      total_output_tokens: 4,
      total_cache_read_tokens: 2,
      total_cache_write_tokens: 1,
    }),
    topSessions: vi.fn().mockResolvedValue([]),
    toolHeatmap: vi.fn().mockResolvedValue([]),
    topTools: vi.fn().mockResolvedValue([]),
    subscriptionGet: vi.fn().mockResolvedValue({ mode: 'auto', plan: 'free_pro', monthly_usd: 0 }),
    subscriptionSet: vi.fn(),
  },
}))

vi.mock('react-apexcharts', () => ({
  default: () => <div data-testid="mock-apex-chart" />,
}))

vi.mock('@/utils/apexChartsCore', () => ({
  default: () => <div data-testid="mock-apex-chart" />,
}))

import { UsageInsightPanel } from '@/features/claude/observer/UsageInsightPanel'

const renderPanel = () => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } })
  const router = createMemoryRouter(
    [{ path: '/', Component: UsageInsightPanel }],
    { initialEntries: ['/'] },
  )
  return render(
    <QueryClientProvider client={client}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  )
}

describe('claude-observer-tabs', () => {
  it('renders observer tabs without component-level listen()', async () => {
    renderPanel()
    await waitFor(() => {
      expect(screen.getByTestId('claude-observer-tabs')).toBeTruthy()
    })
    expect(screen.getByTestId('claude-observer-cost')).toBeTruthy()
  })

  it('keeps event subscription in the shell bridge, not the panel', () => {
    const panelPath = path.join(process.cwd(), 'src/features/claude/observer/UsageInsightPanel.tsx')
    const bridgePath = path.join(process.cwd(), 'src/shell/eventBridge.ts')
    const panel = readFileSync(panelPath, 'utf8')
    const bridge = readFileSync(bridgePath, 'utf8')
    expect(panel).not.toMatch(/\blisten\s*\(/)
    expect(bridge).toContain('claude_observer:updated')
  })
})
