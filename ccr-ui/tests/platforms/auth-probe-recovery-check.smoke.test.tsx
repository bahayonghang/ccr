import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it, vi } from 'vitest'
import type { AuthSessionConfig } from '@/configs/auth'
import { BaseAuth } from '@/features/platform/auth/BaseAuth'

const t = (key: string) => key

describe('Auth probe recovery independent check', () => {
  it('refreshes stale session data after a failed probe succeeds on explicit retry', async () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 30_000 } } })
    const config: AuthSessionConfig = {
      cacheKey: 'auth-check-probe-recovery', homePath: '/', module: 'grok', i18nPrefix: 'grok.auth',
      titleKey: 'auth-title', subtitleKey: 'auth-subtitle', confirmOffKey: 'confirm-off',
      features: { localOnly: true },
      notify: { success: vi.fn(), error: vi.fn(), warning: vi.fn(), confirm: vi.fn().mockResolvedValue(true) },
      probe: vi.fn().mockResolvedValue('ok'),
      load: vi.fn().mockResolvedValueOnce({ loggedIn: true, canAuthOff: true }).mockResolvedValue({ loggedIn: false, canAuthOff: false }),
      authOff: vi.fn().mockResolvedValue({ changed: true }),
    }
    render(<QueryClientProvider client={client}><BaseAuth config={config} t={t} /></QueryClientProvider>)
    await screen.findByText('grok.auth.signedIn')
    vi.mocked(config.probe!).mockRejectedValueOnce(new Error('probe unavailable'))
    await act(async () => { await client.refetchQueries({ queryKey: ['platform-auth-probe', config.cacheKey] }) })
    await screen.findByText('probe unavailable')
    expect(screen.getByText('auth.stale')).toBeTruthy()
    expect(screen.getByText('grok.auth.signedIn')).toBeTruthy()
    fireEvent.click(screen.getByText('common.retry'))
    await waitFor(() => expect(config.load).toHaveBeenCalledTimes(2))
    expect(await screen.findByText('grok.auth.signedOut')).toBeTruthy()
    expect(screen.queryByText('auth.stale')).toBeNull()
    expect(config.probe).toHaveBeenCalledTimes(3)
  })
})
