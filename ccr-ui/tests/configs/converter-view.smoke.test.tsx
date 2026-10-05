import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { describe, expect, it, vi } from 'vitest'
import { ConverterView } from '@/features/configs/ConverterView'

vi.mock('@/api', () => ({
  convertConfig: vi.fn(),
  getCurrentEnvironment: vi.fn().mockResolvedValue({ env_type: 'local' }),
}))

const renderView = () => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const router = createMemoryRouter([{ path: '/', element: <ConverterView /> }], { initialEntries: ['/'] })
  const tree = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  )
  return render(<RouterProvider router={router} />, { wrapper: tree })
}

describe('ConverterView', () => {
  it('renders source and target format pickers', () => {
    renderView()
    expect(screen.getAllByText('Claude Code').length).toBeGreaterThan(0)
    expect(screen.getAllByText('Codex').length).toBeGreaterThan(0)
  })
})
