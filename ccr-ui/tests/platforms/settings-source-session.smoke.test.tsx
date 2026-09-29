import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { EditorView } from '@codemirror/view'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { BaseSettings } from '@/features/platform/settings/BaseSettings'
import type { SettingsConfig } from '@/configs/settings'
import { currentEnvironmentKey } from '@/configs/environmentSession'

const runtime = vi.hoisted(() => ({ get: vi.fn(), switch: vi.fn(), confirm: vi.fn(), error: vi.fn() }))
vi.mock('@/api/runtime/environment', () => ({ getCurrentEnvironment: runtime.get, switchEnvironment: runtime.switch }))
vi.mock('@/configs/surfaceNotify', () => ({ surfaceNotify: { confirm: runtime.confirm, error: runtime.error, success: vi.fn(), warning: vi.fn() } }))
vi.mock('@/i18n', async original => {
  const translate = (key: string) => key
  return { ...await original<object>(), useAppT: () => translate, useResolvedT: () => translate }
})
const environment = (id: string) => ({ id, env_type: 'local' as const, name: id, display_name: id, description: '', is_active: true })
let active = environment('a')

function fixture() {
  const source = {
    language: 'toml' as const, probe: vi.fn().mockResolvedValue('ok'),
    getRaw: vi.fn().mockResolvedValue({ status: 'ok', content: 'model = "before"', token: 'old-token', path: '/fixture/config.toml', exists: true }),
    listLayers: vi.fn().mockResolvedValue({ layers: [] }),
    saveRaw: vi.fn().mockResolvedValue({ status: 'saved', token: 'next-token' }),
  }
  const config: SettingsConfig = {
    cacheKey: 'raw-session', homePath: '/', module: 'test', i18nPrefix: 'test', titleKey: 'title', subtitleKey: 'subtitle',
    fields: [{ id: 'model', tab: 'model', labelKey: 'model-label', kind: 'text' }], tabs: [{ id: 'model', labelKey: 'model-tab' }],
    features: { rawSource: true }, rawSource: source,
    notify: { confirm: runtime.confirm, error: runtime.error, success: vi.fn(), warning: vi.fn() },
    load: async () => ({ values: { model: 'before' }, source: {} }), save: vi.fn(),
  }
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } })
  const router = createMemoryRouter([{ path: '/', element: <BaseSettings config={config} /> }], { initialEntries: ['/'] })
  render(<QueryClientProvider client={client}><RouterProvider router={router} /></QueryClientProvider>)
  return { client, source }
}
async function edit() {
  fireEvent.click(await screen.findByRole('button', { name: 'settingsRaw.sourceTab' }))
  const element = await waitFor(() => {
    expect(document.querySelector('.cm-editor')).toBeTruthy()
    return document.querySelector('.cm-editor') as HTMLElement
  })
  const view = EditorView.findFromDOM(element)!
  act(() => view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: 'model = "draft"' } }))
  return { element, view }
}
beforeEach(() => {
  active = environment('a')
  runtime.get.mockReset().mockImplementation(async () => active)
  runtime.switch.mockReset().mockImplementation(async (id: string) => { active = environment(id) })
  runtime.confirm.mockReset().mockResolvedValue(true)
  runtime.error.mockReset()
  Object.defineProperty(Range.prototype, 'getClientRects', { configurable: true, value: () => [] })
  Object.defineProperty(Range.prototype, 'getBoundingClientRect', { configurable: true, value: () => new DOMRect() })
})

describe('Raw Settings environment session', () => {
  it('retains the same raw editor and token while an environment change freezes all writes', async () => {
    const { client, source } = fixture()
    const { element, view } = await edit()
    active = environment('b')
    await act(async () => { await client.invalidateQueries({ queryKey: currentEnvironmentKey }) })
    await screen.findByText('settingsSession.environmentChanged')
    expect(document.querySelector('.cm-editor')).toBe(element)
    expect(view.state.doc.toString()).toBe('model = "draft"')
    expect((screen.getByText('settingsRaw.save').closest('button') as HTMLButtonElement).disabled).toBe(true)
    fireEvent.click(screen.getByText('settingsRaw.save'))
    expect(source.saveRaw).not.toHaveBeenCalled()
    fireEvent.click(screen.getByText('settingsSession.returnEnvironment'))
    await waitFor(() => expect((screen.getByText('settingsRaw.save').closest('button') as HTMLButtonElement).disabled).toBe(false))
    expect(document.querySelector('.cm-editor')).toBe(element)
    expect(view.state.doc.toString()).toBe('model = "draft"')
    expect(source.getRaw).toHaveBeenCalledOnce()
    expect(runtime.confirm).toHaveBeenCalledOnce()
    fireEvent.click(screen.getByText('settingsRaw.save'))
    await waitFor(() => expect(source.saveRaw).toHaveBeenCalledWith('model = "draft"', 'old-token'))
  })

  it.each(['reject', 'unsupported'] as const)('keeps a dirty raw editor mounted after a probe %s and retries the probe', async outcome => {
    const { client, source } = fixture()
    const { element, view } = await edit()
    if (outcome === 'reject') source.probe.mockRejectedValueOnce(new Error('probe failed'))
    else source.probe.mockResolvedValueOnce('unsupported_environment')
    await act(async () => { await client.refetchQueries({ queryKey: ['settings-source-environment'] }) })
    await screen.findByText(outcome === 'reject' ? 'settingsRaw.loadFailed' : 'settingsRaw.unsupportedEnvironment')
    expect(document.querySelector('.cm-editor')).toBe(element)
    expect(view.state.doc.toString()).toBe('model = "draft"')
    expect((screen.getByText('settingsRaw.save').closest('button') as HTMLButtonElement).disabled).toBe(true)
    fireEvent.click(screen.getByText('common.retry'))
    await waitFor(() => expect((screen.getByText('settingsRaw.save').closest('button') as HTMLButtonElement).disabled).toBe(false))
    expect(document.querySelector('.cm-editor')).toBe(element)
    expect(source.getRaw).toHaveBeenCalledOnce()
    expect(runtime.confirm).toHaveBeenCalledOnce()
  })

  it('does not discard a raw draft until the user confirms reload into the target environment', async () => {
    const { client, source } = fixture()
    const { element, view } = await edit()
    active = environment('b')
    await act(async () => { await client.invalidateQueries({ queryKey: currentEnvironmentKey }) })
    await screen.findByText('settingsSession.environmentChanged')
    runtime.confirm.mockResolvedValueOnce(false)
    fireEvent.click(screen.getByText('settingsSession.discardReload'))
    await waitFor(() => expect(runtime.confirm).toHaveBeenCalledTimes(2))
    expect(document.querySelector('.cm-editor')).toBe(element)
    expect(view.state.doc.toString()).toBe('model = "draft"')
    fireEvent.click(screen.getByText('settingsSession.discardReload'))
    await screen.findByLabelText('model-label')
    expect(document.querySelector('.cm-editor')).toBeNull()
    expect(source.saveRaw).not.toHaveBeenCalled()
  })
})
