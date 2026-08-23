import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { describe, expect, it, vi } from 'vitest'

vi.mock('@/api', () => ({
  grokApi: {
    getGrokDashboardOverview: vi.fn().mockResolvedValue({
      status: 'ok',
      activation: 'inactive',
      activation_name: null,
      current_profile: null,
      auth_mode: null,
      profiles_total: 0,
      profiles_enabled: 0,
      config_exists: false,
      config_path_display: null,
    }),
  },
  getCurrentEnvironment: vi.fn().mockResolvedValue({ id: 'local', env_type: 'local' }),
  getCliVersion: vi.fn().mockResolvedValue({ status: 'ok', installed: true, version: '1.0.0' }),
}))

vi.mock('@/api/runtime/environment', () => ({
  getCurrentEnvironment: vi.fn().mockResolvedValue({ id: 'local', env_type: 'local' }),
}))

vi.mock('@/api/runtime/system', () => ({
  getCliVersion: vi.fn().mockResolvedValue({ status: 'ok', installed: true, version: '1.0.0' }),
}))

import { GrokView } from '@/features/grok/GrokView'

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

describe('grok-view', () => {
  it('renders the Grok overview header', async () => {
    renderView(<GrokView />)
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'grok.overview.title' })).toBeTruthy()
    })
  })
})
