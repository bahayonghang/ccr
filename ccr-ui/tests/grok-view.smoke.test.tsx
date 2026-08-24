import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { describe, expect, it, vi } from 'vitest'

const clipboardMocks = vi.hoisted(() => ({
  copyText: vi.fn().mockResolvedValue(true),
}))

vi.mock('@/utils/clipboard', () => ({
  copyText: clipboardMocks.copyText,
}))

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
      expect(screen.getByRole('heading', { name: /Grok|grok\.overview\.title/ })).toBeTruthy()
    })
  })

  it('clears the copy-command timer on unmount before 1600ms', async () => {
    clipboardMocks.copyText.mockResolvedValue(true)
    const setTimeoutSpy = vi.spyOn(window, 'setTimeout')
    const clearTimeoutSpy = vi.spyOn(window, 'clearTimeout')

    const view = renderView(<GrokView />)
    const command = await screen.findByText('ccr grok profile list')
    const copyButton = command.parentElement?.querySelector('button')
    expect(copyButton).toBeTruthy()
    fireEvent.click(copyButton as HTMLButtonElement)
    await waitFor(() => {
      expect(clipboardMocks.copyText).toHaveBeenCalled()
      expect(setTimeoutSpy.mock.calls.some((call) => call[1] === 1600)).toBe(true)
    })
    const copyIndex = setTimeoutSpy.mock.calls.findIndex((call) => call[1] === 1600)
    const copyTimerId = setTimeoutSpy.mock.results[copyIndex]?.value as ReturnType<typeof window.setTimeout>
    view.unmount()
    expect(clearTimeoutSpy).toHaveBeenCalledWith(copyTimerId)
    setTimeoutSpy.mockRestore()
    clearTimeoutSpy.mockRestore()
  })
})
