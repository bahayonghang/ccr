import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { describe, expect, it, vi } from 'vitest'

vi.mock('@/utils/windowChrome', () => ({
  getClientPlatform: () => 'linux',
}))

import { WslManagementView } from '@/features/sync/WslManagementView'

describe('WSL platform gate', () => {
  it('hides WSL management on non-Windows platforms', () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    render(
      <QueryClientProvider client={client}>
        <MemoryRouter>
          <WslManagementView />
        </MemoryRouter>
      </QueryClientProvider>,
    )
    expect(screen.getByText(/WSL management is unavailable|WSL 管理不可用/i)).toBeTruthy()
  })
})
