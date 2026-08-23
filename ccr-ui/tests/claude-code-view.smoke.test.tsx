import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { describe, expect, it, vi } from 'vitest'

vi.mock('@/api', () => ({
  getCurrentEnvironment: vi.fn().mockResolvedValue({ env_type: 'local' }),
  claudeObserver: {
    getInsight: vi.fn().mockResolvedValue({
      today_value_usd: 0,
      month_value_usd: 0,
      total_value_usd: 0,
      today_tokens: 0,
      month_tokens: 0,
      total_sessions: 0,
      total_projects: 0,
      subscription: { mode: 'auto', plan: 'free_pro', monthly_usd: 0 },
      roi: null,
      pricing_version: '1',
    }),
    dailyTrend: vi.fn().mockResolvedValue([]),
    costBreakdown: vi.fn().mockResolvedValue([]),
    cacheStats: vi.fn().mockResolvedValue({
      hit_rate: 0,
      total_input_tokens: 0,
      total_output_tokens: 0,
      total_cache_read_tokens: 0,
      total_cache_write_tokens: 0,
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

import { ClaudeCodeView } from '@/features/claude/ClaudeCodeView'

const renderView = (node: ReactNode) => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } })
  const router = createMemoryRouter([{ path: '/', Component: () => node as React.ReactElement }], {
    initialEntries: ['/'],
  })
  return render(
    <QueryClientProvider client={client}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  )
}

describe('claude-code-view', () => {
  it('renders the console contract class and module cards', async () => {
    renderView(<ClaudeCodeView />)
    await waitFor(() => {
      expect(document.querySelector('.claude-console')).toBeTruthy()
    })
    expect(screen.getByLabelText('Claude Code usage insight')).toBeTruthy()
    expect(screen.getByLabelText('Claude Code capabilities')).toBeTruthy()
  })
})
