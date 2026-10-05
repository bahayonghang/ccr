import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { describe, expect, it, vi } from 'vitest'

vi.mock('@/api', () => ({
  getUsageDashboardV2: vi.fn().mockResolvedValue({
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
  }),
  getCurrentEnvironment: vi.fn().mockResolvedValue({ env_type: 'local' }),
}))

import { GeminiCliView } from '@/features/gemini/GeminiCliView'

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

describe('gemini-cli-view', () => {
  it('renders the terminal card and module grid', async () => {
    renderView(<GeminiCliView />)
    await waitFor(() => {
      expect(document.querySelector('.gemini-terminal-card')).toBeTruthy()
    })
    expect(document.querySelector('[aria-label="Gemini modules"]')).toBeTruthy()
  })
})
