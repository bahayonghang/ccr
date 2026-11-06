import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { BenchmarkS2Panel } from '@/features/monitoring/BenchmarkS2Panel'
import { setLocale } from '@/i18n'
import { GlobalConfirmDialog } from '@/shell/GlobalConfirmDialog'
import { useUIStore } from '@/shell/stores/ui'
import type { BenchmarkS2SnapshotDto } from '@/types/generated/benchmark/BenchmarkS2SnapshotDto'

const api = vi.hoisted(() => ({
  start: vi.fn<() => Promise<BenchmarkS2SnapshotDto>>(),
  status: vi.fn<() => Promise<BenchmarkS2SnapshotDto>>(),
  cancel: vi.fn<(request: { run_id: string }) => Promise<BenchmarkS2SnapshotDto>>(),
  listen: vi.fn<(name: string, callback: (event: { payload: BenchmarkS2SnapshotDto }) => void) => Promise<() => void>>(),
  native: true,
}))
vi.mock('@/api/domains/benchmark', () => ({
  startBenchmarkS2: api.start, getBenchmarkS2Status: api.status, cancelBenchmarkS2: api.cancel,
}))
vi.mock('@tauri-apps/api/event', () => ({ listen: api.listen }))
vi.mock('@/utils/tauriRuntime', () => ({ isTauriRuntime: () => api.native }))

function snapshot(patch: Partial<BenchmarkS2SnapshotDto> = {}): BenchmarkS2SnapshotDto {
  return {
    phase: 'not_started', run_id: null, fixture_id: 'synthetic-seed', logical_sha256: 'synthetic-hash',
    report_as_of_utc: '2026-11-06T12:00:00Z', failure_code: null, revision: 0,
    requested_rate: 1000, requested_duration_ms: 60000, requested_slots: 60000, queue_capacity: 128,
    scheduled_slots: 0, attempted: 0, scheduler_skipped: 0, queue_accepted: 0, queue_dropped: 0,
    queue_abandoned: 0, queue_pending: 0, ring_accepted: 0, ring_rejected: 0, ring_evicted: 0,
    db_attempted: 0, db_committed: 0, db_failed: 0, db_pending: 0, emit_attempted: 0, emit_failed: 0,
    unacknowledged: 0, elapsed_ms: null, source_elapsed_ms: null, actual_attempted_per_second: null,
    actual_delivered_per_second: null, actual_committed_per_second: null, cleanup_over_budget: false,
    ...patch,
  }
}

function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (reason: Error) => void
  const promise = new Promise<T>((done, fail) => { resolve = done; reject = fail })
  return { promise, resolve, reject }
}

let deliver: (event: { payload: BenchmarkS2SnapshotDto }) => void
const detach = vi.fn()
function emit(value: BenchmarkS2SnapshotDto) {
  act(() => deliver({ payload: value }))
}
function mount() {
  return render(<><BenchmarkS2Panel /><GlobalConfirmDialog /></>)
}

beforeEach(async () => {
  await setLocale('en-US')
  api.native = true
  Reflect.set(window, '__CCR_BENCHMARK_S2__', true)
  api.start.mockReset().mockResolvedValue(snapshot({ phase: 'queued', run_id: 'run-one', revision: 1 }))
  api.status.mockReset().mockResolvedValue(snapshot())
  api.cancel.mockReset().mockResolvedValue(snapshot({ phase: 'cleaning', run_id: 'run-one', revision: 2 }))
  api.listen.mockReset().mockImplementation(async (_name, callback) => {
    deliver = callback
    return detach
  })
  detach.mockClear()
  useUIStore.getState().resolveConfirmDialog(false)
})
afterEach(() => {
  useUIStore.getState().resolveConfirmDialog(false)
  Reflect.deleteProperty(window, '__CCR_BENCHMARK_S2__')
})

describe('S2 explicit benchmark consumer', () => {
  it.each([
    ['web', false, true], ['Local', true, undefined], ['old Fixture', true, false],
    ['S0', true, undefined], ['S1', true, false], ['non-boolean bootstrap', true, 'true'],
  ] as const)('has no controls or requests for %s', (_name, native, flag) => {
    api.native = native
    Reflect.set(window, '__CCR_BENCHMARK_S2__', flag)
    mount()
    expect(screen.queryByRole('heading', { name: 'S2 event-stream trial' })).toBeNull()
    expect(api.listen).not.toHaveBeenCalled()
    expect(api.status).not.toHaveBeenCalled()
    expect(api.start).not.toHaveBeenCalled()
    expect(api.cancel).not.toHaveBeenCalled()
  })

  it('subscribes before recovery and rejects old revisions and another run', async () => {
    const registered = deferred<() => void>()
    const recovery = deferred<BenchmarkS2SnapshotDto>()
    api.listen.mockImplementation((_name, callback) => {
      deliver = callback
      return registered.promise
    })
    api.status.mockReturnValue(recovery.promise)
    mount()
    expect(api.status).not.toHaveBeenCalled()
    await act(async () => registered.resolve(detach))
    expect(api.status).toHaveBeenCalledTimes(1)
    emit(snapshot({ phase: 'cleaning', run_id: 'run-one', revision: 4, elapsed_ms: 62000, cleanup_over_budget: true }))
    await act(async () => recovery.resolve(snapshot()))
    expect(screen.getByRole('status').textContent).toContain('Cleaning up')
    expect(screen.queryByRole('button', { name: 'Start S2' })).toBeNull()
    emit(snapshot({ phase: 'completed', run_id: 'another-run', revision: 99 }))
    expect(screen.getByRole('status').textContent).toContain('Cleaning up')
    emit(snapshot({ phase: 'running', run_id: 'run-one', revision: 3 }))
    expect(screen.getByRole('status').textContent).toContain('Cleaning up')
    // Each active projection owns a new revision; duplicate/older deliveries cannot regress it.
    emit(snapshot({ phase: 'cleaning', run_id: 'run-one', revision: 5, elapsed_ms: 64000, cleanup_over_budget: true }))
    emit(snapshot({ phase: 'cleaning', run_id: 'run-one', revision: 5, elapsed_ms: 63000 }))
    emit(snapshot({ phase: 'cleaning', run_id: 'run-one', revision: 4, elapsed_ms: 63000 }))
    expect(screen.getByText('Trial elapsed: 64,000 ms')).toBeTruthy()
    expect(screen.getByText('Cleanup exceeded its time budget.')).toBeTruthy()
  })

  it('shows measured rates and failure counters separately from the requested profile', async () => {
    api.status.mockResolvedValue(snapshot({
      phase: 'running', run_id: 'run-one', revision: 2, attempted: 234,
      actual_attempted_per_second: 117, actual_delivered_per_second: 93.5,
      actual_committed_per_second: null, queue_dropped: 7, ring_evicted: 31, db_failed: 2, emit_failed: 3,
    }))
    mount()
    await screen.findByText('Running')
    expect(screen.getByText('Actual attempted /s').parentElement?.textContent).toBe('Actual attempted /s117')
    expect(screen.getByText('Actual delivered to ring /s').parentElement?.textContent).toContain('93.5')
    expect(screen.getByText('Actual committed to database /s').parentElement?.textContent).toContain('Not available')
    expect(screen.getByText('Actual attempted /s').getAttribute('title')).toBe('Attempts use active production time.')
    expect(screen.getByText('Actual delivered to ring /s').getAttribute('title')).toContain('including drain, flush and cleanup')
    expect(screen.getByText('Queue dropped').parentElement?.textContent).toContain('7')
    expect(screen.getByText('Ring evicted').parentElement?.textContent).toContain('31')
    expect(screen.getByText('Database failed').parentElement?.textContent).toContain('2')
    expect(screen.getByText('Log emits failed').parentElement?.textContent).toContain('3')
    expect(screen.getByText('Requested: 1,000 events/s for 60 seconds · queue capacity 128')).toBeTruthy()
    expect(screen.queryByRole('textbox')).toBeNull()
    expect(screen.queryByRole('spinbutton')).toBeNull()
    expect(api.start).not.toHaveBeenCalled()
    emit(snapshot({ phase: 'running', run_id: 'run-one', revision: 3, actual_attempted_per_second: 0,
      actual_delivered_per_second: 0, actual_committed_per_second: 0 }))
    expect(screen.getByText('Actual committed to database /s').parentElement?.textContent).toBe('Actual committed to database /s0')
  })

  it('focuses safe cancellation and starts only after explicit confirmation', async () => {
    mount()
    const start = await screen.findByRole('button', { name: 'Start S2' })
    start.focus()
    fireEvent.click(start)
    const keep = await screen.findByRole('button', { name: 'Keep seed' })
    await waitFor(() => expect(document.activeElement).toBe(keep))
    fireEvent.keyDown(keep, { key: 'Enter' })
    expect(api.start).not.toHaveBeenCalled()
    fireEvent.click(keep)
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
    await waitFor(() => expect(document.activeElement).toBe(start))
    expect(api.start).not.toHaveBeenCalled()
    fireEvent.click(start)
    fireEvent.click(await screen.findByRole('button', { name: 'Start trial' }))
    await screen.findByText('Queued')
    expect(api.start).toHaveBeenCalledTimes(1)
    expect(api.start).toHaveBeenCalledWith()
    expect(screen.queryByRole('button', { name: 'Start S2' })).toBeNull()
  })

  it('requests stop without declaring termination, and cannot restart after terminal', async () => {
    const stopped = deferred<BenchmarkS2SnapshotDto>()
    api.status.mockResolvedValue(snapshot({ phase: 'running', run_id: 'run-one', revision: 2 }))
    api.cancel.mockReturnValue(stopped.promise)
    mount()
    fireEvent.click(await screen.findByRole('button', { name: 'Request stop' }))
    expect(api.cancel).toHaveBeenCalledWith({ run_id: 'run-one' })
    expect(screen.getByRole('status').textContent).toBe('Running')
    await act(async () => stopped.resolve(snapshot({ phase: 'cleaning', run_id: 'run-one', revision: 3, cleanup_over_budget: true })))
    expect(screen.getByRole('status').textContent).toContain('Cleaning up')
    expect(screen.queryByRole('button', { name: 'Request stop' })).toBeNull()
    emit(snapshot({ phase: 'cancelled', run_id: 'run-one', revision: 4 }))
    expect(screen.getByRole('status').textContent).toBe('Cancelled after cleanup')
    expect(screen.queryByRole('button', { name: 'Start S2' })).toBeNull()
    expect(screen.queryByRole('button', { name: /reset/i })).toBeNull()
    expect(api.start).not.toHaveBeenCalled()
  })

  it('detaches on leave and recovers on remount without stopping the producer', async () => {
    api.status.mockResolvedValue(snapshot({ phase: 'running', run_id: 'run-one', revision: 2 }))
    const view = mount()
    await screen.findByText('Running')
    view.unmount()
    expect(detach).toHaveBeenCalledTimes(1)
    emit(snapshot({ phase: 'completed', run_id: 'run-one', revision: 3 }))
    expect(api.cancel).not.toHaveBeenCalled()
    mount()
    await screen.findByText('Running')
    expect(api.listen).toHaveBeenCalledTimes(2)
    expect(api.status).toHaveBeenCalledTimes(2)
    expect(screen.queryByRole('button', { name: 'Start S2' })).toBeNull()
  })

  it('retries listener failures before querying and renders no raw refusal details', async () => {
    api.listen.mockRejectedValueOnce(new Error('secret/path/refusal'))
    mount()
    await screen.findByRole('alert')
    expect(api.status).not.toHaveBeenCalled()
    expect(document.body.textContent).not.toContain('secret/path/refusal')
    fireEvent.click(screen.getByRole('button', { name: 'Reconnect status' }))
    await screen.findByRole('button', { name: 'Start S2' })
    expect(api.status).toHaveBeenCalledTimes(1)
    emit(snapshot({ phase: 'failed', run_id: 'run-one', revision: 1, failure_code: 'secret/path/refusal' }))
    expect(document.body.textContent).not.toContain('secret/path/refusal')
    expect(screen.getByText('The trial failed. Review the recorded counters before closing and restoring the seed.')).toBeTruthy()
    expect(screen.queryByRole('button', { name: 'Start S2' })).toBeNull()
  })

  it('cleans up late registration and refuses confirmation completed after leave', async () => {
    const registration = deferred<() => void>()
    api.listen.mockReturnValueOnce(registration.promise)
    const early = mount()
    early.unmount()
    await act(async () => registration.resolve(detach))
    expect(detach).toHaveBeenCalledTimes(1)
    expect(api.status).not.toHaveBeenCalled()
    const view = mount()
    fireEvent.click(await screen.findByRole('button', { name: 'Start S2' }))
    await screen.findByRole('dialog')
    view.unmount()
    await act(async () => useUIStore.getState().resolveConfirmDialog(true))
    expect(api.start).not.toHaveBeenCalled()
  })

  it('requires authoritative recovery after a refused start instead of retrying an old snapshot', async () => {
    api.start.mockRejectedValueOnce(new Error('secret/path/process_used'))
    mount()
    fireEvent.click(await screen.findByRole('button', { name: 'Start S2' }))
    fireEvent.click(await screen.findByRole('button', { name: 'Start trial' }))
    await screen.findByRole('alert')
    expect((screen.getByRole('button', { name: 'Start S2' }) as HTMLButtonElement).disabled).toBe(true)
    expect(document.body.textContent).not.toContain('secret/path/process_used')
    api.status.mockResolvedValueOnce(snapshot({ phase: 'running', run_id: 'run-one', revision: 1 }))
    fireEvent.click(screen.getByRole('button', { name: 'Reconnect status' }))
    await screen.findByText('Running')
    expect(api.listen).toHaveBeenCalledTimes(2)
    expect(api.start).toHaveBeenCalledTimes(1)
    expect(screen.queryByRole('button', { name: 'Start S2' })).toBeNull()
  })

  it('clears a refusal after successful recovery of the same not-started revision', async () => {
    api.start.mockRejectedValueOnce(new Error('synthetic admission refusal'))
    mount()
    fireEvent.click(await screen.findByRole('button', { name: 'Start S2' }))
    fireEvent.click(await screen.findByRole('button', { name: 'Start trial' }))
    await screen.findByRole('alert')
    fireEvent.click(screen.getByRole('button', { name: 'Reconnect status' }))
    await waitFor(() => expect((screen.getByRole('button', { name: 'Start S2' }) as HTMLButtonElement).disabled).toBe(false))
    expect(screen.queryByRole('alert')).toBeNull()
    expect(api.status).toHaveBeenCalledTimes(2)
  })

  it('does not let an old recovery failure overwrite a newer cancellation result', async () => {
    const recovery = deferred<BenchmarkS2SnapshotDto>()
    api.status.mockResolvedValueOnce(snapshot({ phase: 'running', run_id: 'run-one', revision: 2 })).mockReturnValueOnce(recovery.promise)
    api.cancel.mockRejectedValueOnce(new Error('synthetic refusal'))
      .mockResolvedValueOnce(snapshot({ phase: 'cleaning', run_id: 'run-one', revision: 4 }))
    mount()
    fireEvent.click(await screen.findByRole('button', { name: 'Request stop' }))
    await screen.findByRole('alert')
    fireEvent.click(screen.getByRole('button', { name: 'Reconnect status' }))
    await waitFor(() => expect(api.status).toHaveBeenCalledTimes(2))
    emit(snapshot({ phase: 'running', run_id: 'run-one', revision: 3 }))
    fireEvent.click(screen.getByRole('button', { name: 'Request stop' }))
    await screen.findByText('Cleaning up — waiting for completion')
    await act(async () => { recovery.reject(new Error('stale status refusal')) })
    expect(screen.queryByRole('alert')).toBeNull()
    expect(screen.queryByRole('button', { name: 'Reconnect status' })).toBeNull()
  })

  it('describes historical cleanup over-budget without claiming a terminal is still pending', async () => {
    api.status.mockResolvedValue(snapshot({ phase: 'failed', run_id: 'run-one', revision: 5,
      cleanup_over_budget: true, failure_code: 'benchmark_s2_cleanup_over_budget' }))
    mount()
    await screen.findByText('Failed')
    expect(document.body.textContent).not.toContain('Completion is still pending')
    expect(screen.queryByRole('button', { name: 'Start S2' })).toBeNull()
  })

  it('does not let an older successful recovery clear a newer cancellation refusal', async () => {
    const recovery = deferred<BenchmarkS2SnapshotDto>()
    api.status.mockResolvedValueOnce(snapshot({ phase: 'running', run_id: 'run-one', revision: 2 })).mockReturnValueOnce(recovery.promise)
    api.cancel.mockRejectedValue(new Error('synthetic cancellation refusal'))
    mount()
    fireEvent.click(await screen.findByRole('button', { name: 'Request stop' }))
    await screen.findByRole('alert')
    fireEvent.click(screen.getByRole('button', { name: 'Reconnect status' }))
    await waitFor(() => expect(api.status).toHaveBeenCalledTimes(2))
    emit(snapshot({ phase: 'running', run_id: 'run-one', revision: 3 }))
    fireEvent.click(screen.getByRole('button', { name: 'Request stop' }))
    await screen.findByRole('alert')
    await act(async () => recovery.resolve(snapshot({ phase: 'running', run_id: 'run-one', revision: 4 })))
    expect(screen.getByRole('alert')).toBeTruthy()
  })

  it('does not steal focus from a subsequent dialog after the first confirmation is replaced', async () => {
    mount()
    const start = await screen.findByRole('button', { name: 'Start S2' })
    start.focus()
    fireEvent.click(start)
    await screen.findByRole('dialog')
    act(() => {
      void useUIStore.getState().requestConfirm({
        title: 'Another operation', message: 'Synthetic independent confirmation',
        cancelText: 'Keep other operation', type: 'warning',
      })
    })
    const safe = await screen.findByRole('button', { name: 'Keep other operation' })
    await act(async () => { await new Promise(requestAnimationFrame) })
    expect(document.activeElement).toBe(safe)
    expect(api.start).not.toHaveBeenCalled()
  })

  it('preserves a user focus move and a later route after cancellation', async () => {
    const view = render(<><BenchmarkS2Panel /><button>Other action</button><GlobalConfirmDialog /></>)
    const start = await screen.findByRole('button', { name: 'Start S2' })
    const other = screen.getByRole('button', { name: 'Other action' })
    start.focus()
    fireEvent.click(start)
    fireEvent.click(await screen.findByRole('button', { name: 'Keep seed' }))
    other.focus()
    await act(async () => { await new Promise(requestAnimationFrame) })
    expect(document.activeElement).toBe(other)
    fireEvent.click(start)
    fireEvent.click(await screen.findByRole('button', { name: 'Keep seed' }))
    view.unmount()
    render(<button>After leave</button>)
    const after = screen.getByRole('button', { name: 'After leave' })
    after.focus()
    await act(async () => { await new Promise(requestAnimationFrame) })
    expect(document.activeElement).toBe(after)
    expect(api.start).not.toHaveBeenCalled()
  })

  it('updates status and controls when locale switches without a new request', async () => {
    mount()
    await screen.findByRole('button', { name: 'Start S2' })
    await act(async () => { await setLocale('zh-CN') })
    expect(screen.getByRole('heading', { name: 'S2 事件流试验' })).toBeTruthy()
    expect(screen.getByRole('button', { name: '启动 S2' })).toBeTruthy()
    expect(screen.getByRole('status').textContent).toBe('尚未启动')
    expect(api.status).toHaveBeenCalledTimes(1)
  })
})
