import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { grokAuthConfig, type AuthSessionConfig } from '@/configs/auth'
import { BaseAuth } from '@/features/platform/auth/BaseAuth'

const api = vi.hoisted(() => ({ current: vi.fn(), off: vi.fn() }))
vi.mock('@/api/domains/grok', () => ({ grokAuthCurrent: api.current, grokAuthOff: api.off }))
const t = (key: string) => key
function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (reason: unknown) => void
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no })
  return { promise, resolve, reject }
}
function config(): AuthSessionConfig {
  return {
    ...grokAuthConfig, cacheKey: 'auth-query-contract', probe: vi.fn().mockResolvedValue('ok'),
    notify: { success: vi.fn(), error: vi.fn(), warning: vi.fn(), confirm: vi.fn().mockResolvedValue(true) },
  }
}
function mount(settings: AuthSessionConfig) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } })
  render(<QueryClientProvider client={client}><BaseAuth config={settings} t={t} /></QueryClientProvider>)
  return client
}
beforeEach(() => {
  api.current.mockReset().mockResolvedValue({ status: 'ok', logged_in: true, can_auth_off: true })
  api.off.mockReset().mockResolvedValue({ status: 'ok', changed: true })
})

describe('Auth query and mutation states', () => {
  it('preserves authoritative backend unsupported after a successful local probe', async () => {
    api.current.mockResolvedValue({ status: 'unsupported_environment', env_type: 'ssh' })
    mount(config())
    expect(await screen.findByText('grok.dashboard.localOnly.description')).toBeTruthy()
    expect(screen.queryByTestId('platform-auth-status')).toBeNull()
    expect(screen.queryByText('grok.auth.signedOut')).toBeNull()
  })

  it('keeps pending distinct from a confirmed signed-out response', async () => {
    const pending = deferred<unknown>()
    api.current.mockReturnValue(pending.promise)
    mount(config())
    await waitFor(() => expect(api.current).toHaveBeenCalledOnce())
    expect(screen.queryByTestId('platform-auth-status')).toBeNull()
    await act(async () => { pending.resolve({ status: 'ok', logged_in: false, can_auth_off: false }) })
    expect(await screen.findByText('grok.auth.signedOut')).toBeTruthy()
  })

  it('shows a failed retry after an unsupported response as an error', async () => {
    api.current.mockResolvedValueOnce({ status: 'unsupported_environment', env_type: 'ssh' }).mockRejectedValueOnce(new Error('retry failed'))
    mount(config())
    await screen.findByText('grok.dashboard.localOnly.description')
    fireEvent.click(screen.getByText('common.retry'))
    expect(await screen.findByText('retry failed')).toBeTruthy()
    expect(screen.queryByTestId('platform-auth-status')).toBeNull()
    expect(screen.getByText('common.retry')).toBeTruthy()
  })

  it('keeps confirmed session data marked stale after a refresh error and retries only the load', async () => {
    const settings = config()
    mount(settings)
    await screen.findByText('grok.auth.signedIn')
    api.current.mockRejectedValueOnce(new Error('refresh failed'))
    fireEvent.click(screen.getByText('common.refresh'))
    await screen.findByText('refresh failed')
    expect(screen.getByText('grok.auth.signedIn')).toBeTruthy()
    expect(screen.getByText('auth.stale')).toBeTruthy()
    expect((screen.getByTestId('platform-auth-off') as HTMLButtonElement).disabled).toBe(true)
    fireEvent.click(screen.getByText('common.retry'))
    await waitFor(() => expect(screen.queryByText('auth.stale')).toBeNull())
    expect(api.current).toHaveBeenCalledTimes(3)
    expect(settings.probe).toHaveBeenCalledOnce()
  })

  it('retries a failed probe before loading a stale session again', async () => {
    const settings = config()
    const client = mount(settings)
    await screen.findByText('grok.auth.signedIn')
    vi.mocked(settings.probe!).mockRejectedValueOnce(new Error('probe refresh failed'))
    await act(async () => { await client.refetchQueries({ queryKey: ['platform-auth-probe'] }) })
    expect(await screen.findByText('probe refresh failed')).toBeTruthy()
    fireEvent.click(screen.getByText('common.retry'))
    await waitFor(() => expect(screen.queryByText('probe refresh failed')).toBeNull())
    expect(settings.probe).toHaveBeenCalledTimes(3)
    await waitFor(() => expect(api.current).toHaveBeenCalledTimes(2))
  })

  it('claims the off action before confirmation, preserves state on failure and allows explicit retry', async () => {
    const settings = config()
    const confirmation = deferred<boolean>()
    const off = deferred<unknown>()
    vi.mocked(settings.notify.confirm).mockReturnValue(confirmation.promise)
    api.off.mockReturnValueOnce(off.promise)
    mount(settings)
    const button = await screen.findByTestId('platform-auth-off') as HTMLButtonElement
    fireEvent.click(button)
    fireEvent.click(button)
    expect(settings.notify.confirm).toHaveBeenCalledOnce()
    expect(button.disabled).toBe(true)
    await act(async () => { confirmation.resolve(true) })
    expect(api.off).toHaveBeenCalledOnce()
    fireEvent.click(button)
    await act(async () => { off.reject(new Error('off failure')) })
    expect(await screen.findByText('auth.offFailed')).toBeTruthy()
    expect(screen.getByText('grok.auth.signedIn')).toBeTruthy()
    expect(settings.notify.success).not.toHaveBeenCalled()
    expect(api.current).toHaveBeenCalledOnce()
    vi.mocked(settings.notify.confirm).mockResolvedValue(true)
    api.current.mockResolvedValue({ status: 'ok', logged_in: false, can_auth_off: false })
    fireEvent.click(button)
    await screen.findByText('grok.auth.signedOut')
    expect(api.off).toHaveBeenCalledTimes(2)
  })

  it('does not submit after cancelled confirmation and renders unsupported off results honestly', async () => {
    const settings = config()
    vi.mocked(settings.notify.confirm).mockResolvedValueOnce(false)
    mount(settings)
    const button = await screen.findByTestId('platform-auth-off')
    fireEvent.click(button)
    await waitFor(() => expect(settings.notify.confirm).toHaveBeenCalledOnce())
    expect(api.off).not.toHaveBeenCalled()
    api.off.mockResolvedValue({ status: 'unsupported_environment', env_type: 'ssh' })
    fireEvent.click(button)
    await screen.findByText('grok.dashboard.localOnly.description')
    expect(settings.notify.success).not.toHaveBeenCalled()
    expect(screen.queryByText('grok.auth.signedOut')).toBeNull()
  })
})
