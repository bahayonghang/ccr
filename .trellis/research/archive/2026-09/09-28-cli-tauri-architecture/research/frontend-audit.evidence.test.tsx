import React from 'react'
import { act, render, renderHook, screen, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { buildCodexSettingsPayload, flattenCodexSettings } from '@/configs/settings-codex-map'

const h = vi.hoisted(() => ({
  handlers: new Map<string, (event: { payload: unknown }) => void>(),
  pending: [] as Array<{ resolve: (unlisten: () => void) => void; unlisten: ReturnType<typeof vi.fn> }>,
  delayed: false,
  start: vi.fn(), cancel: vi.fn(), recent: vi.fn().mockResolvedValue(undefined),
  t: (key: string) => key,
  query: { data: [{ name: 'status', description: 'Status', executable: true, category: 'read', risk: 'safe' }] },
  navigate: vi.fn(),
}))
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn((name, fn) => {
  h.handlers.set(name, fn)
  const unlisten = vi.fn(() => h.handlers.delete(name))
  return h.delayed ? new Promise(resolve => h.pending.push({ resolve, unlisten })) : Promise.resolve(unlisten)
}) }))
vi.mock('react-router', () => ({ useParams: () => ({ client: 'ccr' }), useNavigate: () => h.navigate }))
vi.mock('@/api', () => ({ startCcrCommandJob: (...args: unknown[]) => h.start(...args), cancelCcrCommandJob: (...args: unknown[]) => h.cancel(...args), listConfigs: vi.fn().mockResolvedValue({ configs: [] }) }))
vi.mock('@/features/commands/queries', () => ({ useCommands: () => h.query }))
vi.mock('@/features/commands/locale', () => ({ useCommandsT: () => h.t }))
vi.mock('@/api/domains/uiState', () => ({ addFavorite: vi.fn(), removeFavorite: vi.fn(), clearRecentItems: vi.fn(), getFavorites: vi.fn().mockResolvedValue([]), getRecentItems: vi.fn().mockResolvedValue([]), addRecentItem: (...args: unknown[]) => h.recent(...args) }))
vi.mock('@/utils/tauriRuntime', () => ({ isTauriRuntime: () => true }))
vi.mock('@/utils/ansiRenderer', () => ({ createAnsiRenderer: () => ({ renderLine: (line: string) => line, clear: vi.fn() }) }))
vi.mock('@/utils/clipboard', () => ({ copyText: vi.fn() }))
vi.mock('@/utils/runtimeState', () => ({ getRuntimeUnavailableCopy: () => ({ title: 'unavailable' }) }))
vi.mock('@/utils/logger', () => ({ logger: { error: vi.fn(), warn: vi.fn() } }))
vi.mock('@/i18n', () => ({ useResolvedT: (t?: (key: string) => string) => t ?? h.t }))
vi.mock('@/features/platform/SurfacePage', () => ({ SurfacePage: ({ state, children, actions }: { state?: string; children?: React.ReactNode; actions?: React.ReactNode }) => <div data-testid="surface-state" data-state={state ?? 'content'}>{actions}{children}</div> }))

import { useCommandsPage } from '@/features/commands/useCommandsPage'
import { BaseAuth } from '@/features/platform/auth/BaseAuth'

const snapshot = (status: string) => ({ job_id: 'job-a', command: 'status', args: [], status, started_at: '2026-09-28T00:00:00Z', finished_at: null, duration_ms: 1, exit_code: null, stdout_lines: [], stderr_lines: [], system_lines: [], truncated: false, dropped_lines: 0, error: null })
const auth = (load: () => Promise<unknown>, probe?: () => Promise<unknown>) => ({ cacheKey: 'audit-auth', homePath: '/', module: 'audit', i18nPrefix: 'audit', titleKey: 'title', subtitleKey: 'subtitle', confirmOffKey: 'confirm', features: { localOnly: true }, notify: { success: vi.fn(), error: vi.fn(), warning: vi.fn(), confirm: vi.fn().mockResolvedValue(true) }, load, probe, authOff: vi.fn() })
const renderAuth = (config: ReturnType<typeof auth>) => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } })
  render(<QueryClientProvider client={client}><BaseAuth config={config as never} /></QueryClientProvider>)
  return client
}
beforeEach(() => {
  h.handlers.clear(); h.pending.length = 0; h.delayed = false
  h.start.mockReset().mockResolvedValue({ job_id: 'job-a', snapshot: snapshot('queued') })
})

// These assertions characterize existing defects. Passing means the defect was reproduced.
// All IPC and file-facing functions are mocked. No real user config is read or written.
describe('Architecture audit defect evidence', () => {
  it('route remount loses the active command snapshot', async () => {
    const first = renderHook(() => useCommandsPage())
    await waitFor(() => expect(first.result.current.canExecuteSelected).toBe(true))
    await act(async () => { await first.result.current.handleExecute() })
    expect(first.result.current.isRunning).toBe(true)
    first.unmount()
    const next = renderHook(() => useCommandsPage())
    await waitFor(() => expect(next.result.current.canExecuteSelected).toBe(true))
    expect(next.result.current.currentSnapshot).toBeNull()
    expect(next.result.current.isRunning).toBe(false)
  })
  it('late command listeners are not cleaned after unmount', async () => {
    h.delayed = true
    const view = renderHook(() => useCommandsPage())
    await waitFor(() => expect(h.pending).toHaveLength(3))
    view.unmount()
    const pending = [...h.pending]
    await act(async () => { pending.forEach(entry => entry.resolve(entry.unlisten)) })
    expect(pending.every(entry => entry.unlisten.mock.calls.length === 0)).toBe(true)
  })
  it('a late start response overwrites the terminal event with queued', async () => {
    let resolveStart!: (value: unknown) => void
    h.start.mockImplementation(() => new Promise(resolve => { resolveStart = resolve }))
    const view = renderHook(() => useCommandsPage())
    await waitFor(() => expect(view.result.current.canExecuteSelected).toBe(true))
    let started!: Promise<void>
    act(() => { started = view.result.current.handleExecute() })
    await act(async () => { h.handlers.get('commands:job-finished')?.({ payload: snapshot('success') }) })
    expect(view.result.current.currentSnapshot?.status).toBe('success')
    await act(async () => { resolveStart({ job_id: 'job-a', snapshot: snapshot('queued') }); await started })
    expect(view.result.current.currentSnapshot?.status).toBe('queued')
  })
  it('auth read rejection renders signed out', async () => {
    const config = auth(vi.fn().mockRejectedValue(new Error('read failed')))
    const client = renderAuth(config)
    await waitFor(() => expect(client.getQueryState(['platform-auth', 'audit-auth'])?.status).toBe('error'))
    await waitFor(() => expect(screen.getByTestId('platform-auth-status').textContent).toBe('audit.signedOut'))
    expect(config.notify.error).not.toHaveBeenCalled()
  })
  it('auth environment rejection remains loading', async () => {
    const load = vi.fn().mockResolvedValue({ loggedIn: true, canAuthOff: true })
    const client = renderAuth(auth(load, vi.fn().mockRejectedValue(new Error('probe failed'))))
    await waitFor(() => expect(client.getQueryState(['platform-auth-probe', 'audit-auth'])?.status).toBe('error'))
    expect(screen.getByTestId('surface-state').getAttribute('data-state')).toBe('loading')
    expect(load).not.toHaveBeenCalled()
  })
  it('an unrelated Codex model save overwrites notification event arrays with false', () => {
    const values = flattenCodexSettings({ model: 'old', tui: { notifications: ['agent-turn-complete'] } })
    values.model = 'new'
    expect(buildCodexSettingsPayload(values).tui?.notifications).toBe(false)
  })
})
