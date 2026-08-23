import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { describe, expect, it, vi } from 'vitest'
import { appRoutes } from '@/shell/router'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  InvokeArgs: {},
  InvokeOptions: {},
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}))

describe('React 外壳启动', () => {
  it('根路径渲染占位页面且无白屏', async () => {
    const router = createMemoryRouter(appRoutes, { initialEntries: ['/'] })
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    render(
      <QueryClientProvider client={client}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    )
    const placeholder = await screen.findByTestId('route-placeholder')
    expect(placeholder.getAttribute('data-route-id')).toBe('dashboard')
  })
})
