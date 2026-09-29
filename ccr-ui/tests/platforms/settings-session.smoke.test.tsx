import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { SettingsConfig, SettingsSnapshot } from '@/configs/settings'
import { BaseSettings } from '@/features/platform/settings/BaseSettings'
import { currentEnvironmentKey } from '@/configs/environmentSession'
import { useTauriEventBridge } from '@/shell/eventBridge'

const runtime = vi.hoisted(() => ({ get: vi.fn(), switch: vi.fn(), listeners: new Map<string, (event: { payload: unknown }) => void>() }))
vi.mock('@/api/runtime/environment', () => ({ getCurrentEnvironment: runtime.get, switchEnvironment: runtime.switch }))
vi.mock('@/utils/tauriRuntime', () => ({ isTauriRuntime: () => true }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async (name: string, callback: (event: { payload: unknown }) => void) => {
  runtime.listeners.set(name, callback)
  return () => runtime.listeners.delete(name)
} }))
const t = (key: string) => key
const environment = (id: string) => ({ id, env_type: 'local' as const, name: id, display_name: id, description: '', is_active: true })
const snapshot = (model: string): SettingsSnapshot => ({ values: { model }, source: { model, extension: 'preserved' } })
function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (reason: unknown) => void
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no })
  return { promise, resolve, reject }
}
function config(): SettingsConfig {
  return {
    cacheKey: 'session-test', homePath: '/', module: 'synthetic', i18nPrefix: 'settings',
    titleKey: 'title', subtitleKey: 'subtitle', tabs: [{ id: 'model', labelKey: 'model-tab' }],
    fields: [{ id: 'model', tab: 'model', kind: 'text', labelKey: 'model-label' }], features: {},
    notify: { success: vi.fn(), error: vi.fn(), warning: vi.fn(), confirm: vi.fn().mockResolvedValue(true) },
    load: vi.fn().mockResolvedValue(snapshot('original')), save: vi.fn().mockResolvedValue({ status: 'saved' }),
  }
}
function Bridge() { useTauriEventBridge(); return null }
function mount(settings: SettingsConfig, bridge = false) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 }, mutations: { retry: false } } })
  render(<QueryClientProvider client={client}>{bridge ? <Bridge /> : null}<BaseSettings config={settings} t={t} /></QueryClientProvider>)
  return client
}
async function input(value = 'original') {
  const field = await screen.findByLabelText('model-label') as HTMLInputElement
  await waitFor(() => expect(field.value).toBe(value))
  return field
}
const saveButton = () => screen.getByRole('button', { name: 'settings.save' }) as HTMLButtonElement
let active = environment('a')
beforeEach(() => {
  runtime.listeners.clear()
  active = environment('a')
  runtime.get.mockReset().mockImplementation(async () => active)
  runtime.switch.mockReset().mockImplementation(async (id: string) => { active = environment(id) })
})

describe('Settings acknowledged draft session', () => {
  it('freezes synchronously from the real shell environment event before any target read', async () => {
    const settings = config()
    mount(settings, true)
    const field = await input()
    fireEvent.change(field, { target: { value: 'event-draft' } })
    const detection = deferred<ReturnType<typeof environment>>()
    runtime.get.mockReturnValueOnce(detection.promise)
    active = environment('b')
    act(() => { runtime.listeners.get('env:changed')!({ payload: { env_id: 'b', env_type: 'local', status: 'changed' } }) })
    await waitFor(() => expect(field.disabled).toBe(true))
    expect(settings.load).toHaveBeenCalledOnce()
    fireEvent.submit(document.getElementById('platform-settings-form')!)
    expect(settings.save).not.toHaveBeenCalled()
    await act(async () => { detection.resolve(active) })
    await waitFor(() => expect(settings.load).toHaveBeenCalledWith({ environmentId: 'b' }))
    expect(field.value).toBe('event-draft')
    expect(field.disabled).toBe(true)
  })

  it('does not accept an old A read after A to B to A establishes a new session', async () => {
    const settings = config()
    const firstRead = deferred<SettingsSnapshot>()
    vi.mocked(settings.load).mockReturnValueOnce(firstRead.promise).mockImplementation(async () => snapshot('fresh-' + active.id))
    const client = mount(settings)
    await waitFor(() => expect(settings.load).toHaveBeenCalledOnce())
    active = environment('b')
    await act(async () => { await client.invalidateQueries({ queryKey: currentEnvironmentKey }) })
    const field = await input('fresh-b')
    active = environment('a')
    await act(async () => { await client.invalidateQueries({ queryKey: currentEnvironmentKey }) })
    await screen.findByText('settingsSession.environmentChanged')
    fireEvent.click(screen.getByText('settingsSession.discardReload'))
    await waitFor(() => expect(field.value).toBe('fresh-a'))
    fireEvent.change(field, { target: { value: 'new-draft-a' } })
    await act(async () => { firstRead.resolve(snapshot('obsolete-a')) })
    expect(field.value).toBe('new-draft-a')
    expect(client.getQueryData<SettingsSnapshot>(['platform-settings', settings.cacheKey, 'a', 'local'])?.values.model).toBe('fresh-a')
  })

  it('keeps the original baseline after refetch and requires confirmed reload before save', async () => {
    const settings = config()
    const client = mount(settings)
    const field = await input()
    fireEvent.change(field, { target: { value: 'draft' } })
    vi.mocked(settings.load).mockResolvedValue(snapshot('external'))
    await act(async () => { await client.refetchQueries({ queryKey: ['platform-settings'] }) })
    expect(field.value).toBe('draft')
    expect(await screen.findByText('settingsSession.serverChanged')).toBeTruthy()
    expect(saveButton().disabled).toBe(true)
    fireEvent.submit(document.getElementById('platform-settings-form')!)
    expect(settings.save).not.toHaveBeenCalled()
    vi.mocked(settings.notify.confirm).mockResolvedValueOnce(false)
    fireEvent.click(screen.getByText('settingsSession.discardReload'))
    await waitFor(() => expect(settings.notify.confirm).toHaveBeenCalledOnce())
    expect(field.value).toBe('draft')
    fireEvent.click(screen.getByText('settingsSession.discardReload'))
    await waitFor(() => expect(field.value).toBe('external'))
    fireEvent.change(field, { target: { value: 'next-draft' } })
    fireEvent.click(saveButton())
    await waitFor(() => expect(settings.save).toHaveBeenCalledWith({ values: { model: 'next-draft' }, dirtyKeys: ['model'], snapshot: snapshot('external'), environmentId: 'a' }))
  })

  it('retains a failed-save draft, then rebuilds the baseline only after a successful save', async () => {
    const settings = config()
    vi.mocked(settings.save).mockRejectedValueOnce(new Error('save failed')).mockResolvedValue({ status: 'saved' })
    mount(settings)
    const field = await input()
    fireEvent.change(field, { target: { value: 'draft' } })
    fireEvent.click(saveButton())
    await screen.findByText('save failed')
    expect(field.value).toBe('draft')
    expect(settings.notify.success).not.toHaveBeenCalled()
    vi.mocked(settings.load).mockResolvedValue(snapshot('normalized'))
    fireEvent.click(saveButton())
    await waitFor(() => expect(field.value).toBe('normalized'))
    expect(saveButton().disabled).toBe(true)
    expect(settings.save).toHaveBeenCalledTimes(2)
  })

  it('does not replay a successful save when its follow-up read fails', async () => {
    const settings = config()
    mount(settings)
    const field = await input()
    vi.mocked(settings.load).mockRejectedValue(new Error('refresh failed'))
    fireEvent.change(field, { target: { value: 'draft' } })
    fireEvent.click(saveButton())
    await screen.findByText('settingsSession.reloadRequired')
    expect(field.value).toBe('draft')
    expect(saveButton().disabled).toBe(true)
    fireEvent.submit(document.getElementById('platform-settings-form')!)
    expect(settings.save).toHaveBeenCalledOnce()
  })

  it('keeps the old draft frozen on environment change and restores access on return', async () => {
    const settings = config()
    const client = mount(settings)
    const field = await input()
    fireEvent.change(field, { target: { value: 'draft-a' } })
    active = environment('b')
    vi.mocked(settings.load).mockImplementation(async () => snapshot(active.id === 'a' ? 'original' : 'server-b'))
    await act(async () => { await client.invalidateQueries({ queryKey: currentEnvironmentKey }) })
    await screen.findByText('settingsSession.environmentChanged')
    expect(field.value).toBe('draft-a')
    expect(field.disabled).toBe(true)
    fireEvent.submit(document.getElementById('platform-settings-form')!)
    expect(settings.save).not.toHaveBeenCalled()
    fireEvent.click(screen.getByText('settingsSession.returnEnvironment'))
    await waitFor(() => expect(field.disabled).toBe(false))
    expect(runtime.switch).toHaveBeenCalledWith('a')
    expect(field.value).toBe('draft-a')
    await waitFor(() => expect(saveButton().disabled).toBe(false))
  })

  it('preserves the old draft on cancelled environment discard, then binds a fresh target snapshot', async () => {
    const settings = config()
    const client = mount(settings)
    const field = await input()
    fireEvent.change(field, { target: { value: 'draft-a' } })
    active = environment('b')
    vi.mocked(settings.load).mockResolvedValue(snapshot('server-b'))
    await act(async () => { await client.invalidateQueries({ queryKey: currentEnvironmentKey }) })
    await screen.findByText('settingsSession.environmentChanged')
    vi.mocked(settings.notify.confirm).mockResolvedValueOnce(false)
    fireEvent.click(screen.getByText('settingsSession.discardReload'))
    await waitFor(() => expect(settings.notify.confirm).toHaveBeenCalledOnce())
    expect(field.value).toBe('draft-a')
    fireEvent.click(screen.getByText('settingsSession.discardReload'))
    await waitFor(() => expect(field.value).toBe('server-b'))
    expect(field.disabled).toBe(false)
    fireEvent.change(field, { target: { value: 'draft-b' } })
    fireEvent.click(saveButton())
    await waitFor(() => expect(settings.save).toHaveBeenCalledWith({ values: { model: 'draft-b' }, dirtyKeys: ['model'], snapshot: snapshot('server-b'), environmentId: 'b' }))
  })

  it('rejects a late initial response from the old environment', async () => {
    const settings = config()
    const oldRead = deferred<SettingsSnapshot>()
    vi.mocked(settings.load).mockReturnValueOnce(oldRead.promise).mockResolvedValue(snapshot('server-b'))
    const client = mount(settings)
    await waitFor(() => expect(settings.load).toHaveBeenCalledOnce())
    active = environment('b')
    await act(async () => { await client.invalidateQueries({ queryKey: currentEnvironmentKey }) })
    const field = await input('server-b')
    fireEvent.change(field, { target: { value: 'draft-b' } })
    await act(async () => { oldRead.resolve(snapshot('late-a')) })
    expect(field.value).toBe('draft-b')
    expect(screen.queryByDisplayValue('late-a')).toBeNull()
  })

  it('freezes before environment detection completes and preserves the draft on detection failure', async () => {
    const settings = config()
    const client = mount(settings)
    const field = await input()
    fireEvent.change(field, { target: { value: 'draft' } })
    const pending = deferred<ReturnType<typeof environment>>()
    runtime.get.mockReturnValue(pending.promise)
    act(() => { void client.invalidateQueries({ queryKey: currentEnvironmentKey }) })
    await waitFor(() => expect(field.disabled).toBe(true))
    fireEvent.submit(document.getElementById('platform-settings-form')!)
    expect(settings.save).not.toHaveBeenCalled()
    await act(async () => { pending.reject(new Error('environment failed')) })
    expect(await screen.findByText('environment failed')).toBeTruthy()
    expect(field.value).toBe('draft')
    expect(field.disabled).toBe(true)
  })

  it('does not apply an old save completion after an environment transition', async () => {
    const settings = config()
    const saving = deferred<{ status: 'saved' }>()
    vi.mocked(settings.save).mockReturnValue(saving.promise)
    const client = mount(settings)
    const field = await input()
    fireEvent.change(field, { target: { value: 'draft-a' } })
    fireEvent.click(saveButton())
    await waitFor(() => expect(settings.save).toHaveBeenCalledOnce())
    active = environment('b')
    vi.mocked(settings.load).mockResolvedValue(snapshot('server-b'))
    await act(async () => { await client.invalidateQueries({ queryKey: currentEnvironmentKey }) })
    await act(async () => { saving.resolve({ status: 'saved' }) })
    expect(field.value).toBe('draft-a')
    expect(field.disabled).toBe(true)
    expect(settings.notify.success).not.toHaveBeenCalled()
  })
})
