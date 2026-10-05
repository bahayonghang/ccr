import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it, vi } from 'vitest'
import type { SettingsConfig } from '@/configs/settings'
import { BaseSettings } from '@/features/platform/settings/BaseSettings'
import { currentEnvironmentKey } from '@/configs/environmentSession'

const runtime = vi.hoisted(() => ({ get: vi.fn(), switch: vi.fn() }))
vi.mock('@/api/runtime/environment', () => ({ getCurrentEnvironment: runtime.get, switchEnvironment: runtime.switch }))
const t = (key: string) => key
const environment = (id: string) => ({ id, env_type: 'local' as const, name: id, display_name: id, description: '', is_active: true })

describe('Settings origin recovery after cache disposal independent check', () => {
  it('retains the draft and permits save when the restored origin has unchanged data', async () => {
    let active = environment('a')
    runtime.get.mockImplementation(async () => active)
    runtime.switch.mockImplementation(async (id: string) => { active = environment(id) })
    const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 30_000 } } })
    const config: SettingsConfig = {
      cacheKey: 'check-return-cache', homePath: '/', module: 'synthetic', i18nPrefix: 'settings',
      titleKey: 'title', subtitleKey: 'subtitle', tabs: [{ id: 'model', labelKey: 'model-tab' }],
      fields: [{ id: 'model', tab: 'model', kind: 'text', labelKey: 'model-label' }], features: {},
      notify: { success: vi.fn(), error: vi.fn(), warning: vi.fn(), confirm: vi.fn().mockResolvedValue(true) },
      load: vi.fn().mockImplementation(async () => ({ values: { model: 'server-' + active.id }, source: { model: 'server-' + active.id } })),
      save: vi.fn().mockResolvedValue({ status: 'saved' }),
    }
    render(<QueryClientProvider client={client}><BaseSettings config={config} t={t} /></QueryClientProvider>)
    const input = await screen.findByLabelText('model-label') as HTMLInputElement
    await waitFor(() => expect(input.value).toBe('server-a'))
    fireEvent.change(input, { target: { value: 'draft-a' } })
    active = environment('b')
    await act(async () => { await client.invalidateQueries({ queryKey: currentEnvironmentKey }) })
    await screen.findByText('settingsSession.environmentChanged')
    client.removeQueries({ queryKey: ['platform-settings', config.cacheKey, 'a', 'local'], exact: true })
    expect(client.getQueryData(['platform-settings', config.cacheKey, 'a', 'local'])).toBeUndefined()
    fireEvent.click(screen.getByText('settingsSession.returnEnvironment'))
    await waitFor(() => expect(input.disabled).toBe(false))
    await waitFor(() => expect(client.getQueryState(['platform-settings', config.cacheKey, 'a', 'local'])?.status).toBe('success'))
    await act(async () => { await client.refetchQueries({ queryKey: ['platform-settings', config.cacheKey, 'a', 'local'], exact: true }) })
    const save = screen.getByRole('button', { name: 'settings.save' }) as HTMLButtonElement
    await waitFor(() => expect(save.disabled).toBe(false))
    expect(input.value).toBe('draft-a')
    expect(screen.queryByText('settingsSession.serverChanged')).toBeNull()
    fireEvent.click(save)
    await waitFor(() => expect(config.save).toHaveBeenCalledOnce())
    expect(vi.mocked(config.save).mock.calls[0][0]).toMatchObject({
      values: { model: 'draft-a' }, snapshot: { values: { model: 'server-a' } }, environmentId: 'a',
    })
  })

  it('blocks old draft saves while the returned origin snapshot is being checked', async () => {
    let active = environment('a')
    let aReads = 0
    let resolveOrigin!: (value: { values: { model: string }; source: { model: string } }) => void
    const pendingOrigin = new Promise<{ values: { model: string }; source: { model: string } }>((resolve) => { resolveOrigin = resolve })
    runtime.get.mockImplementation(async () => active)
    runtime.switch.mockImplementation(async (id: string) => { active = environment(id) })
    const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 30_000 } } })
    const config: SettingsConfig = {
      cacheKey: 'check-return-pending', homePath: '/', module: 'synthetic', i18nPrefix: 'settings',
      titleKey: 'title', subtitleKey: 'subtitle', tabs: [{ id: 'model', labelKey: 'model-tab' }],
      fields: [{ id: 'model', tab: 'model', kind: 'text', labelKey: 'model-label' }], features: {},
      notify: { success: vi.fn(), error: vi.fn(), warning: vi.fn(), confirm: vi.fn().mockResolvedValue(true) },
      load: vi.fn().mockImplementation(async () => {
        if (active.id === 'a' && ++aReads > 1) return pendingOrigin
        return { values: { model: 'server-' + active.id }, source: { model: 'server-' + active.id } }
      }),
      save: vi.fn().mockResolvedValue({ status: 'saved' }),
    }
    render(<QueryClientProvider client={client}><BaseSettings config={config} t={t} /></QueryClientProvider>)
    const input = await screen.findByLabelText('model-label') as HTMLInputElement
    await waitFor(() => expect(input.value).toBe('server-a'))
    fireEvent.change(input, { target: { value: 'draft-a' } })
    active = environment('b')
    await act(async () => { await client.invalidateQueries({ queryKey: currentEnvironmentKey }) })
    await screen.findByText('settingsSession.environmentChanged')
    client.removeQueries({ queryKey: ['platform-settings', config.cacheKey, 'a', 'local'], exact: true })
    fireEvent.click(screen.getByText('settingsSession.returnEnvironment'))
    await waitFor(() => expect(aReads).toBe(2))
    expect.soft((screen.getByRole('button', { name: 'settings.save' }) as HTMLButtonElement).disabled).toBe(true)
    await act(async () => { fireEvent.submit(document.getElementById('platform-settings-form')!) })
    expect(config.save).not.toHaveBeenCalled()
    await act(async () => { resolveOrigin({ values: { model: 'external-a' }, source: { model: 'external-a' } }) })
    await screen.findByText('settingsSession.serverChanged')
    expect(input.value).toBe('draft-a')
    expect(config.save).not.toHaveBeenCalled()
  })
})
