import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { Link, MemoryRouter, Route, Routes } from 'react-router'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { CommandsView } from '@/features/commands/CommandsView'
import { useCommandsStreamStore } from '@/features/commands/stores'
import { useTauriEventBridge } from '@/shell/eventBridge'
import type { CommandJobSnapshot } from '@/types'

const api = vi.hoisted(() => ({
  start: vi.fn(), status: vi.fn(), cancel: vi.fn(), addHistory: vi.fn(),
  catalog: vi.fn(),
  history: vi.fn().mockResolvedValue([]),
  listeners: new Map<string, Set<(event: { payload: unknown }) => void>>(),
  listen: vi.fn(),
  listenFailure: null as { event: string; promise: Promise<() => void> } | null,
}))

vi.mock('@/api', () => ({
  listCommands: api.catalog,
  listConfigs: vi.fn().mockResolvedValue([]),
  startCcrCommandJob: api.start, getCcrCommandJobStatus: api.status, cancelCcrCommandJob: api.cancel,
}))
vi.mock('@/api/domains/uiState', () => ({
  getFavorites: vi.fn().mockResolvedValue([]), getRecentItems: api.history,
  addRecentItem: api.addHistory, clearRecentItems: vi.fn().mockResolvedValue(undefined),
  addFavorite: vi.fn(), removeFavorite: vi.fn(),
}))
vi.mock('@tauri-apps/api/event', () => ({ listen: api.listen }))
vi.mock('@/features/commands/locale', () => {
  const t = (key: string) => key
  return { useCommandsT: () => t }
})

function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (error: unknown) => void
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no })
  return { promise, resolve, reject }
}

const snapshot = (overrides: Partial<CommandJobSnapshot> = {}): CommandJobSnapshot => ({
  job_id: 'job-1', command: 'status', args: [], status: 'running',
  started_at: '2026-09-28T10:00:00Z', finished_at: null, duration_ms: null, exit_code: null,
  stdout_lines: ['before route exit'], stderr_lines: [], system_lines: [], truncated: false,
  dropped_lines: 0, error: null, ...overrides,
})

function Shell() {
  useTauriEventBridge()
  return <>
    <Link to='/other'>Leave commands</Link><Link to='/commands/ccr'>Return to commands</Link>
    <Routes>
      <Route path='/commands/:client' element={<CommandsView />} />
      <Route path='/other' element={<p>Other route</p>} />
    </Routes>
  </>
}

const mount = () => render(
  <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
    <MemoryRouter initialEntries={['/commands/ccr']}><Shell /></MemoryRouter>
  </QueryClientProvider>,
)
const emit = (name: string, payload: unknown) => act(() => {
  api.listeners.get(name)?.forEach((listener) => listener({ payload }))
})
const runButton = () => screen.getByRole('button', { name: /commands.(run|executing)$/ }) as HTMLButtonElement

beforeEach(() => {
  useCommandsStreamStore.setState(useCommandsStreamStore.getInitialState())
  api.listeners.clear()
  api.listenFailure = null
  api.start.mockReset().mockResolvedValue({ job_id: 'job-1', snapshot: snapshot() })
  api.status.mockReset().mockResolvedValue(snapshot())
  api.cancel.mockReset().mockResolvedValue(snapshot({ system_lines: ['Cancel requested'] }))
  api.addHistory.mockReset().mockResolvedValue(undefined)
  api.history.mockReset().mockResolvedValue([])
  api.catalog.mockReset().mockResolvedValue([{ name: 'status', executable: true, category: 'read', description: 'Status', usage: 'ccr status' }])
  api.listen.mockImplementation((name: string, listener: (event: { payload: unknown }) => void) => {
    if (api.listenFailure?.event === name) return api.listenFailure.promise
    const listeners = api.listeners.get(name) ?? new Set()
    listeners.add(listener)
    api.listeners.set(name, listeners)
    return Promise.resolve(() => { listeners.delete(listener) })
  })
  Object.assign(window, { __TAURI_INTERNALS__: {} })
})

afterEach(() => {
  useCommandsStreamStore.getState().suspend()
  vi.useRealTimers()
  delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__
})

const progress = (seq: number, lines: string[] = [], jobId = 'job-1') => ({
  job_id: jobId, seq, channel: 'stdout', lines, dropped_count: 0, status: 'running',
})
const advance = (ms: number) => act(async () => { await vi.advanceTimersByTimeAsync(ms) })
const startPage = async (initial = snapshot()) => {
  api.start.mockResolvedValue({ job_id: initial.job_id, snapshot: initial })
  mount()
  await waitFor(() => expect(runButton().disabled).toBe(false))
  fireEvent.click(runButton())
  await waitFor(() => expect(screen.queryByRole('button', { name: 'commands.cancelJob' })).not.toBeNull())
}

describe('command workbench lifecycle', () => {
  it('restores the running job and output after a real route unmount, then cancels the same job', async () => {
    mount()
    await waitFor(() => expect(runButton().disabled).toBe(false))
    fireEvent.click(runButton())
    await screen.findByText('before route exit')
    fireEvent.click(screen.getByText('Leave commands'))
    expect(screen.queryByText('before route exit')).toBeNull()
    api.status.mockResolvedValue(snapshot({ stdout_lines: ['before route exit', 'while away'] }))
    fireEvent.click(screen.getByText('Return to commands'))
    await screen.findByText('while away')
    expect(api.status).toHaveBeenCalledWith('job-1')
    fireEvent.click(screen.getByRole('button', { name: 'commands.cancelJob' }))
    await waitFor(() => expect(api.cancel).toHaveBeenCalledWith('job-1'))
    expect(runButton().disabled).toBe(true)
    expect(api.listeners.get('commands:job-finished')?.size).toBe(1)
  })

  it('records terminal history once while the command route is absent', async () => {
    mount()
    await waitFor(() => expect(runButton().disabled).toBe(false))
    fireEvent.click(runButton())
    await screen.findByText('before route exit')
    fireEvent.click(screen.getByText('Leave commands'))
    const terminal = snapshot({ status: 'success', finished_at: '2026-09-28T10:00:01Z', duration_ms: 1000, exit_code: 0 })
    emit('commands:job-finished', terminal)
    emit('commands:job-finished', terminal)
    await waitFor(() => expect(api.addHistory).toHaveBeenCalledTimes(1))
    fireEvent.click(screen.getByText('Return to commands'))
    await waitFor(() => expect(runButton().disabled).toBe(false))
    expect(screen.getAllByText('commands.status.success').length).toBeGreaterThan(0)
    expect(api.addHistory).toHaveBeenCalledTimes(1)
  })

  it('blocks synchronous duplicate submission and keeps a terminal event over the older start response', async () => {
    const pending = deferred<{ job_id: string; snapshot: CommandJobSnapshot }>()
    api.start.mockReturnValue(pending.promise)
    mount()
    await waitFor(() => expect(runButton().disabled).toBe(false))
    const button = runButton()
    act(() => { button.click(); button.click() })
    expect(api.start).toHaveBeenCalledTimes(1)
    emit('commands:job-finished', snapshot({ status: 'success', duration_ms: 20 }))
    await act(async () => { pending.resolve({ job_id: 'job-1', snapshot: snapshot({ status: 'queued', stdout_lines: [] }) }) })
    await waitFor(() => expect(runButton().disabled).toBe(false))
    expect(screen.getAllByText('commands.status.success').length).toBeGreaterThan(0)
    expect(screen.queryByRole('button', { name: 'commands.cancelJob' })).toBeNull()
    expect(api.addHistory).toHaveBeenCalledTimes(1)
  })

  it('merges continuous sequence numbers once and ignores old events after the final snapshot', async () => {
    await startPage(snapshot({ status: 'queued', stdout_lines: [] }))
    vi.useFakeTimers()
    emit('commands:job-progress', progress(0, ['same']))
    emit('commands:job-progress', progress(1, ['same']))
    emit('commands:job-progress', progress(1, ['duplicate delivery']))
    emit('commands:job-progress', progress(0, ['out of order']))
    await advance(250)
    expect(screen.getAllByText('same')).toHaveLength(2)
    expect(screen.queryByText('duplicate delivery')).toBeNull()
    expect(screen.queryByText('out of order')).toBeNull()
    expect(api.status).not.toHaveBeenCalled()
    expect((screen.getByRole('button', { name: 'commands.clear' }) as HTMLButtonElement).disabled).toBe(true)
    emit('commands:job-progress', progress(2, ['final line']))
    const terminal = snapshot({ status: 'success', stdout_lines: ['same', 'same', 'final line'] })
    emit('commands:job-finished', terminal)
    emit('commands:job-cancelled', terminal)
    emit('commands:job-progress', progress(3, ['late progress']))
    await advance(500)
    expect(screen.getAllByText('same')).toHaveLength(2)
    expect(screen.getAllByText('final line')).toHaveLength(1)
    expect(screen.queryByText('late progress')).toBeNull()
    expect(runButton().disabled).toBe(false)
    expect(api.addHistory).toHaveBeenCalledTimes(1)
  })

  it('reconciles sequence gaps and never appends late output already covered by a snapshot', async () => {
    await startPage(snapshot({ status: 'queued', stdout_lines: [] }))
    api.status.mockResolvedValue(snapshot({ stdout_lines: ['zero', 'one', 'two'] }))
    vi.useFakeTimers()
    emit('commands:job-progress', progress(0, ['zero']))
    emit('commands:job-progress', progress(2, ['two']))
    await advance(500)
    expect(api.status).toHaveBeenCalledTimes(1)
    expect(screen.getAllByText('two')).toHaveLength(1)
    emit('commands:job-progress', progress(1, ['one']))
    api.status.mockResolvedValue(snapshot({ stdout_lines: ['zero', 'one', 'two', 'three', 'three'] }))
    emit('commands:job-progress', progress(3, ['three']))
    emit('commands:job-progress', progress(4, ['three']))
    await advance(500)
    expect(screen.getAllByText('one')).toHaveLength(1)
    expect(screen.getAllByText('three')).toHaveLength(2)
    expect(api.status).toHaveBeenCalledTimes(2)
    expect(api.addHistory).not.toHaveBeenCalled()
    expect(runButton().disabled).toBe(true)
  })

  it('keeps a fixed reconciliation deadline during continuous output and coalesces in-flight reads', async () => {
    await startPage()
    const first = deferred<CommandJobSnapshot>()
    api.status.mockReturnValueOnce(first.promise).mockResolvedValue(snapshot({ stdout_lines: ['new authoritative output'] }))
    vi.useFakeTimers()
    for (let seq = 0; seq < 10; seq++) {
      emit('commands:job-progress', progress(seq, ['stream']))
      await advance(100)
      if (seq === 4) expect(api.status).toHaveBeenCalledTimes(1)
    }
    expect(api.status).toHaveBeenCalledTimes(1)
    await act(async () => { first.resolve(snapshot({ stdout_lines: ['older snapshot output'] })) })
    await advance(250)
    expect(api.status).toHaveBeenCalledTimes(2)
    expect(screen.getAllByText('new authoritative output')).toHaveLength(1)
    expect(screen.queryByText('stream')).toBeNull()
    expect(screen.queryByText('older snapshot output')).toBeNull()
  })

  it('recovers a lost terminal event by status lookup on route return', async () => {
    await startPage()
    fireEvent.click(screen.getByText('Leave commands'))
    api.status.mockResolvedValue(snapshot({ status: 'cleanup_failed', stdout_lines: ['cleanup error output'], error: 'Cleanup failed' }))
    fireEvent.click(screen.getByText('Return to commands'))
    await screen.findByText('cleanup error output')
    expect(screen.getAllByText('commands.status.cleanup_failed').length).toBeGreaterThan(0)
    await waitFor(() => expect(api.addHistory).toHaveBeenCalledTimes(1))
    expect(api.addHistory).toHaveBeenCalledWith('status', [], false, 0)
    expect(runButton().disabled).toBe(false)
  })

  it('reports an expired retained snapshot without inventing completion or continuing cancellation', async () => {
    await startPage()
    fireEvent.click(screen.getByText('Leave commands'))
    api.status.mockRejectedValue("Command job 'job-1' not found")
    fireEvent.click(screen.getByText('Return to commands'))
    await screen.findByText('commands.jobExpired')
    expect(screen.getAllByText('commands.status.unavailable').length).toBeGreaterThan(0)
    expect(screen.getByText('before route exit')).toBeTruthy()
    expect(screen.queryByRole('button', { name: 'commands.cancelJob' })).toBeNull()
    expect(runButton().disabled).toBe(false)
    expect(api.addHistory).not.toHaveBeenCalled()
  })

  it('preserves a running job after a transient status error and supports explicit retry', async () => {
    await startPage()
    fireEvent.click(screen.getByText('Leave commands'))
    api.status.mockRejectedValueOnce(new Error('transport unavailable'))
    fireEvent.click(screen.getByText('Return to commands'))
    await screen.findByText('transport unavailable')
    expect(screen.getByText('before route exit')).toBeTruthy()
    expect(screen.getByRole('button', { name: 'commands.cancelJob' })).toBeTruthy()
    expect(runButton().disabled).toBe(true)
    api.status.mockResolvedValue(snapshot({ status: 'success', stdout_lines: ['recovered final output'] }))
    fireEvent.click(screen.getByRole('button', { name: 'common.retry' }))
    await screen.findByText('recovered final output')
    expect(screen.queryByText('transport unavailable')).toBeNull()
    expect(runButton().disabled).toBe(false)
    expect(api.addHistory).toHaveBeenCalledTimes(1)
  })

  it('rejects stale status and cancel responses plus old job events after a new job starts', async () => {
    await startPage()
    const status = deferred<CommandJobSnapshot>()
    const cancel = deferred<CommandJobSnapshot>()
    api.status.mockReturnValueOnce(status.promise)
    api.cancel.mockReturnValueOnce(cancel.promise)
    act(() => { void useCommandsStreamStore.getState().reconcile() })
    fireEvent.click(screen.getByRole('button', { name: 'commands.cancelJob' }))
    emit('commands:job-finished', snapshot({ status: 'cancelled' }))
    api.start.mockResolvedValue({ job_id: 'job-2', snapshot: snapshot({ job_id: 'job-2', stdout_lines: ['second job output'] }) })
    fireEvent.click(runButton())
    await screen.findByText('second job output')
    await act(async () => {
      status.resolve(snapshot({ stdout_lines: ['old status response'] }))
      cancel.resolve(snapshot({ status: 'cancelled', stdout_lines: ['old cancel response'] }))
    })
    emit('commands:job-finished', snapshot({ status: 'success', stdout_lines: ['old terminal'] }))
    emit('commands:job-progress', progress(8, ['old progress']))
    expect(screen.getByText('second job output')).toBeTruthy()
    expect(screen.queryByText('old status response')).toBeNull()
    expect(screen.queryByText('old cancel response')).toBeNull()
    expect(screen.queryByText('old terminal')).toBeNull()
    expect(api.addHistory).toHaveBeenCalledTimes(1)
    expect(runButton().disabled).toBe(true)
    fireEvent.click(screen.getByRole('button', { name: 'commands.cancelJob' }))
    await waitFor(() => expect(api.cancel).toHaveBeenLastCalledWith('job-2'))
  })

  it('keeps cancel requested nonterminal and prevents clearing the running job', async () => {
    await startPage()
    const pending = deferred<CommandJobSnapshot>()
    api.cancel.mockReturnValue(pending.promise)
    const cancelButton = screen.getByRole('button', { name: 'commands.cancelJob' }) as HTMLButtonElement
    act(() => { cancelButton.click(); cancelButton.click() })
    expect(api.cancel).toHaveBeenCalledTimes(1)
    expect(cancelButton.disabled).toBe(true)
    await act(async () => { pending.resolve(snapshot({ system_lines: ['Cancel requested'] })) })
    expect(screen.getByText('Cancel requested')).toBeTruthy()
    expect(runButton().disabled).toBe(true)
    act(() => { useCommandsStreamStore.getState().clearOutput() })
    expect(screen.getByText('Cancel requested')).toBeTruthy()
    expect(api.addHistory).not.toHaveBeenCalled()
    emit('commands:job-cancelled', snapshot({ status: 'cancelled', system_lines: ['Cleanup finished'] }))
    expect(runButton().disabled).toBe(false)
    expect(api.addHistory).toHaveBeenCalledTimes(1)
  })

  it('renders failed starts without adopting unrelated early events or persisting fake history', async () => {
    const pending = deferred<never>()
    api.start.mockReturnValue(pending.promise)
    mount()
    await waitFor(() => expect(runButton().disabled).toBe(false))
    fireEvent.click(runButton())
    emit('commands:job-finished', snapshot({ job_id: 'unrelated', status: 'success' }))
    await act(async () => { pending.reject(new Error('backend admission failed')) })
    expect(screen.getByText('backend admission failed')).toBeTruthy()
    expect(runButton().disabled).toBe(false)
    expect(screen.queryByText('before route exit')).toBeNull()
    expect(api.addHistory).not.toHaveBeenCalled()
  })

  it('keeps the existing destructive-command acknowledgement before the shared start action', async () => {
    api.catalog.mockResolvedValue([{ name: 'delete', executable: true, requiresConfirmation: true, description: 'Delete saved profile' }])
    mount()
    const acknowledgement = await screen.findByRole('checkbox')
    expect(runButton().disabled).toBe(true)
    fireEvent.click(acknowledgement)
    await waitFor(() => expect(runButton().disabled).toBe(false))
    fireEvent.click(runButton())
    await waitFor(() => expect(api.start).toHaveBeenCalledTimes(1))
    expect(api.start).toHaveBeenCalledWith({ command: 'delete', args: [], confirmationToken: 'desktop-confirm:delete' })
  })

  it('does not let an older history read erase the newly persisted terminal summary', async () => {
    const older = deferred<[]>()
    api.history.mockReturnValueOnce(older.promise)
    await startPage()
    const item = { id: 'history-1', full_command: 'ccr status', command: 'status', args: [], success: true, executed_at: '2026-09-28T10:00:01Z', duration_ms: 10 }
    api.history.mockResolvedValue([item])
    emit('commands:job-finished', snapshot({ status: 'success', duration_ms: 10 }))
    await waitFor(() => expect(useCommandsStreamStore.getState().historyItems).toEqual([item]))
    await act(async () => { older.resolve([]) })
    fireEvent.click(screen.getByRole('button', { name: 'commands.history' }))
    expect(screen.getByRole('button', { name: /ccr status/ })).toBeTruthy()
    expect(useCommandsStreamStore.getState().historyItems).toEqual([item])
    expect(api.addHistory).toHaveBeenCalledTimes(1)
  })

  it('shows a failed terminal subscription and recovers completion through status checks without leaving the route', async () => {
    const listener = deferred<() => void>()
    api.listenFailure = { event: 'commands:job-finished', promise: listener.promise }
    await startPage(snapshot({ status: 'queued', stdout_lines: [] }))
    vi.useFakeTimers()
    api.status.mockResolvedValueOnce(snapshot()).mockResolvedValue(snapshot({ status: 'success', stdout_lines: ['status recovered terminal'] }))
    await act(async () => { listener.reject(new Error('finished subscription failed')) })
    expect(screen.getByText('commands.liveUpdatesUnavailable')).toBeTruthy()
    expect(screen.getByRole('button', { name: 'common.retry' })).toBeTruthy()
    emit('commands:job-progress', progress(0, ['normal contiguous progress']))
    await advance(250)
    expect(api.status).toHaveBeenCalledTimes(1)
    expect(runButton().disabled).toBe(true)
    await advance(1000)
    expect(screen.getByText('status recovered terminal')).toBeTruthy()
    expect(runButton().disabled).toBe(false)
    expect(screen.queryByRole('button', { name: 'commands.cancelJob' })).toBeNull()
    expect(api.addHistory).toHaveBeenCalledTimes(1)
    await advance(2000)
    expect(api.status).toHaveBeenCalledTimes(2)
    expect(api.listeners.has('commands:job-finished')).toBe(false)
  })

  it('keeps a failed history summary visible and never retries the non-idempotent write', async () => {
    await startPage()
    const persistence = deferred<void>()
    api.addHistory.mockReturnValue(persistence.promise)
    const terminal = snapshot({ status: 'success', stdout_lines: ['retained final output'], duration_ms: 20 })
    emit('commands:job-finished', terminal)
    expect(screen.getByText('commands.historyPending')).toBeTruthy()
    expect(useCommandsStreamStore.getState().historyWrites['job-1'].status).toBe('pending')
    emit('commands:job-finished', terminal)
    await act(async () => { persistence.reject(new Error('history database write rejected')) })
    expect(screen.getByText('commands.historySaveFailed')).toBeTruthy()
    expect(screen.queryByText('commands.historySaved')).toBeNull()
    expect(screen.getByText('retained final output')).toBeTruthy()
    expect(useCommandsStreamStore.getState().historyWrites['job-1']).toEqual({
      status: 'failed', error: 'history database write rejected',
      summary: { job_id: 'job-1', command: 'status', args: [], status: 'success', duration_ms: 20 },
    })
    emit('commands:job-cancelled', terminal)
    fireEvent.click(screen.getByRole('button', { name: 'common.refresh' }))
    await waitFor(() => expect(api.history).toHaveBeenCalledTimes(2))
    fireEvent.click(screen.getByText('Leave commands'))
    fireEvent.click(screen.getByText('Return to commands'))
    expect(screen.getByText('commands.historySaveFailed')).toBeTruthy()
    expect(api.addHistory).toHaveBeenCalledTimes(1)
    expect(runButton().disabled).toBe(false)
  })

  it('keeps recovery suspended when a pending start resolves after the shell unmounts', async () => {
    const listener = deferred<() => void>()
    api.listenFailure = { event: 'commands:job-finished', promise: listener.promise }
    const pending = deferred<{ job_id: string; snapshot: CommandJobSnapshot }>()
    api.start.mockReturnValue(pending.promise)
    const view = mount()
    await waitFor(() => expect(runButton().disabled).toBe(false))
    await act(async () => { listener.reject(new Error('finished subscription failed')) })
    fireEvent.click(runButton())
    expect(api.start).toHaveBeenCalledTimes(1)
    vi.useFakeTimers()
    view.unmount()
    await act(async () => { pending.resolve({ job_id: 'job-1', snapshot: snapshot({ status: 'queued' }) }) })
    await advance(2000)
    expect(api.status).not.toHaveBeenCalled()
    expect(useCommandsStreamStore.getState().job.snapshot?.job_id).toBe('job-1')
    expect(api.addHistory).not.toHaveBeenCalled()
  })

  it('stops an already scheduled recovery when an explicit status retry fails', async () => {
    const listener = deferred<() => void>()
    api.listenFailure = { event: 'commands:job-finished', promise: listener.promise }
    mount()
    await waitFor(() => expect(runButton().disabled).toBe(false))
    await act(async () => { listener.reject(new Error('finished subscription failed')) })
    vi.useFakeTimers()
    fireEvent.click(runButton())
    await act(async () => {})
    api.status.mockRejectedValueOnce(new Error('status transport unavailable'))
    fireEvent.click(screen.getByRole('button', { name: 'common.retry' }))
    await act(async () => {})
    expect(screen.getByText('status transport unavailable')).toBeTruthy()
    await advance(2000)
    expect(api.status).toHaveBeenCalledTimes(1)
    expect(screen.getByText('status transport unavailable')).toBeTruthy()
    api.status.mockResolvedValue(snapshot({ status: 'success', stdout_lines: ['explicit retry recovered'] }))
    fireEvent.click(screen.getByRole('button', { name: 'common.retry' }))
    await act(async () => {})
    expect(screen.getByText('explicit retry recovered')).toBeTruthy()
    expect(runButton().disabled).toBe(false)
    expect(api.addHistory).toHaveBeenCalledTimes(1)
  })

  it('keeps recovery suspended after a late cancel response and resumes on shell remount', async () => {
    const view = mount()
    await waitFor(() => expect(runButton().disabled).toBe(false))
    fireEvent.click(runButton())
    await screen.findByText('before route exit')
    const pending = deferred<CommandJobSnapshot>()
    api.cancel.mockReturnValue(pending.promise)
    fireEvent.click(screen.getByRole('button', { name: 'commands.cancelJob' }))
    expect(api.cancel).toHaveBeenCalledTimes(1)
    vi.useFakeTimers()
    view.unmount()
    await act(async () => { pending.resolve(snapshot({ system_lines: ['Cancel requested'] })) })
    await advance(2000)
    expect(api.status).not.toHaveBeenCalled()
    expect(api.addHistory).not.toHaveBeenCalled()
    vi.useRealTimers()
    api.status.mockResolvedValue(snapshot({ status: 'cancelled', stdout_lines: ['remounted final output'] }))
    mount()
    await screen.findByText('remounted final output')
    expect(runButton().disabled).toBe(false)
    expect(api.addHistory).toHaveBeenCalledTimes(1)
  })
})
