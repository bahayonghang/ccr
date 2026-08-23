import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { describe, expect, it, vi } from 'vitest'
import { ConfigsView } from '@/features/configs/ConfigsView'

vi.mock('@/api', () => ({
  listConfigs: vi.fn().mockResolvedValue({ configs: [], current_config: '', default_config: '' }),
  getHistory: vi.fn().mockResolvedValue({ entries: [] }),
  getUsageByProviderV2: vi.fn().mockResolvedValue([]),
  getCurrentEnvironment: vi.fn().mockResolvedValue({ env_type: 'local' }),
  switchConfig: vi.fn(),
  deleteConfig: vi.fn(),
  enableConfig: vi.fn(),
  disableConfig: vi.fn(),
}))

vi.mock('@/api/runtime/environment', async () => {
  const actual = await vi.importActual<typeof import('@/api/runtime/environment')>('@/api/runtime/environment')
  return {
    ...actual,
    isTauriEnvironment: () => false,
    TauriRuntimeApi: { getTauriVersion: vi.fn() },
  }
})

const renderView = () => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const router = createMemoryRouter([{ path: '/configs', element: <ConfigsView /> }], {
    initialEntries: ['/configs'],
  })
  const tree = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  )
  return render(<RouterProvider router={router} />, { wrapper: tree })
}

describe('ConfigsView', () => {
  it('renders the configs workspace', async () => {
    renderView()
    expect(await screen.findByPlaceholderText(/.+/)).toBeTruthy()
  })
})
