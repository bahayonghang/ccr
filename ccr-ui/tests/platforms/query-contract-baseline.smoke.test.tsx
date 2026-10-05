import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider, useQuery } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import type { AuthSessionConfig } from '@/configs/auth'
import type { SettingsConfig } from '@/configs/settings'
import { BaseAuth } from '@/features/platform/auth/BaseAuth'
import { BaseSettings } from '@/features/platform/settings/BaseSettings'

vi.mock('@/api/runtime/environment', () => ({
  getCurrentEnvironment: async () => ({ id: 'local', env_type: 'local', name: 'Local', is_active: true, available: true }),
}))

const t = (key: string) => key
const notify = () => ({ success: vi.fn(), error: vi.fn(), warning: vi.fn(), confirm: vi.fn().mockResolvedValue(true) })

function mount(node: ReactNode) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 }, mutations: { retry: false } } })
  return { client, ...render(<QueryClientProvider client={client}>{node}</QueryClientProvider>) }
}

function authConfig(): AuthSessionConfig {
  return {
    cacheKey: 't09-auth', homePath: '/', module: 'grok', i18nPrefix: 'grok.auth',
    titleKey: 'auth-title', subtitleKey: 'auth-subtitle', confirmOffKey: 'confirm-off',
    features: { localOnly: true }, notify: notify(), probe: async () => 'ok',
    load: async () => ({ loggedIn: false, canAuthOff: false }), authOff: async () => ({ changed: true }),
  }
}

function ServerSnapshot({ config }: { config: SettingsConfig }) {
  const query = useQuery({ queryKey: ['platform-settings', config.cacheKey, 'local', 'local'], queryFn: () => config.load({ environmentId: 'local' }) })
  return <output data-testid="server-snapshot">{query.data?.values.model}</output>
}

describe('T09 original query failures', () => {
  it('renders a retry for probe failure without claiming signed out', async () => {
    const config = authConfig()
    config.probe = vi.fn().mockRejectedValueOnce(new Error('probe failed')).mockResolvedValue('ok')
    config.load = vi.fn().mockResolvedValue({ loggedIn: true, canAuthOff: true })
    mount(<BaseAuth config={config} t={t} />)
    expect(await screen.findByText('probe failed')).toBeTruthy()
    expect(screen.queryByText('grok.auth.signedOut')).toBeNull()
    expect(config.load).not.toHaveBeenCalled()
    fireEvent.click(screen.getByRole('button', { name: /retry|重试/i }))
    expect(await screen.findByText('grok.auth.signedIn')).toBeTruthy()
    expect(config.probe).toHaveBeenCalledTimes(2)
  })

  it('renders a retry for load failure without claiming signed out', async () => {
    const config = authConfig()
    config.load = vi.fn().mockRejectedValueOnce(new Error('session failed')).mockResolvedValue({ loggedIn: true, canAuthOff: true })
    mount(<BaseAuth config={config} t={t} />)
    expect(await screen.findByText('session failed')).toBeTruthy()
    expect(screen.queryByText('grok.auth.signedOut')).toBeNull()
    fireEvent.click(screen.getByRole('button', { name: /retry|重试/i }))
    expect(await screen.findByText('grok.auth.signedIn')).toBeTruthy()
    expect(config.load).toHaveBeenCalledTimes(2)
  })

  it('preserves a dirty field when the server query refetches', async () => {
    let serverModel = 'server-before'
    const config: SettingsConfig = {
      cacheKey: 't09-settings', homePath: '/', module: 'synthetic', i18nPrefix: 'test',
      titleKey: 'settings-title', subtitleKey: 'settings-subtitle',
      tabs: [{ id: 'model', labelKey: 'model-tab' }],
      fields: [{ id: 'model', tab: 'model', kind: 'text', labelKey: 'model-label' }],
      features: {}, notify: notify(),
      load: async () => ({ values: { model: serverModel }, source: { model: serverModel } }),
      save: vi.fn().mockResolvedValue({ status: 'saved' }),
    }
    const { client } = mount(<><BaseSettings config={config} t={t} /><ServerSnapshot config={config} /></>)
    const field = await screen.findByLabelText('model-label') as HTMLInputElement
    await waitFor(() => expect(field.value).toBe('server-before'))
    fireEvent.change(field, { target: { value: 'user-draft' } })
    expect(field.value).toBe('user-draft')
    serverModel = 'server-refreshed'
    await act(async () => { await client.refetchQueries({ queryKey: ['platform-settings', config.cacheKey] }) })
    await waitFor(() => expect(screen.getByTestId('server-snapshot').textContent).toBe('server-refreshed'))
    expect(field.value).toBe('user-draft')
    expect(config.save).not.toHaveBeenCalled()
  })
})
