import { act, renderHook } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useUsageImport } from '@/features/usage/useUsageImport'
import { isUsageImportJobTerminal, shouldApplyUsageImportJob } from '@/utils/usageImportNormalization'
import type { UsageImportJobSnapshot, UsageImportJobStatus } from '@/types/usage'

const mocks = vi.hoisted(() => ({
  start: vi.fn(),
  status: vi.fn(),
  callbacks: new Map<string, (event: { payload: UsageImportJobSnapshot }) => void>(),
}))

vi.mock('@/api', () => ({
  startUsageImportJobV2: mocks.start,
  getUsageImportJobStatusV2: mocks.status,
}))
vi.mock('@/utils/tauriRuntime', () => ({ isTauriRuntime: () => true }))
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async (channel: string, callback: (event: { payload: UsageImportJobSnapshot }) => void) => {
    mocks.callbacks.set(channel, callback)
    return () => mocks.callbacks.delete(channel)
  }),
}))

function snapshot(status: UsageImportJobStatus = 'pending'): UsageImportJobSnapshot {
  return {
    job_id: 'job-1', status, stage: 'queued', platform_scope: 'all', recent_window_days: 30,
    files_total: 0, files_scanned: 0, files_imported: 0, records_imported: 0, records_skipped: 0,
    history_cursor_hit: false, live_sources: 0, missing_sources: 0, deleted_sources: 0,
    started_at: '2026-09-28T00:00:00Z', updated_at: '2026-09-28T00:00:01Z', warnings: [], results: [],
  }
}

beforeEach(() => {
  mocks.callbacks.clear()
  mocks.start.mockReset().mockResolvedValue({ job_id: 'job-1', snapshot: snapshot() })
  mocks.status.mockReset().mockResolvedValue(snapshot('running'))
})

describe('usage import lifecycle', () => {
  it.each(['pending', 'running', 'recent_ready', 'cancel_requested'] as const)('keeps %s active', status => {
    expect(isUsageImportJobTerminal(snapshot(status))).toBe(false)
  })

  it.each(['finished', 'failed', 'cancelled', 'timed_out', 'cleanup_failed'] as const)('preserves terminal %s', status => {
    const current = snapshot(status)
    expect(isUsageImportJobTerminal(current)).toBe(true)
    expect(shouldApplyUsageImportJob(current, snapshot('running'))).toBe(false)
    expect(shouldApplyUsageImportJob(current, snapshot('failed'))).toBe(false)
  })

  it('keeps cancellation pending until the cleanup result arrives', async () => {
    const hook = renderHook(() => useUsageImport(vi.fn(async () => {})))
    await act(async () => { await hook.result.current.startImportJob({}) })
    act(() => { mocks.callbacks.get('usage:job-progress')?.({ payload: snapshot('cancel_requested') }) })
    expect(hook.result.current.importing).toBe(true)
    const lateProgress = mocks.callbacks.get('usage:job-progress')
    act(() => { mocks.callbacks.get('usage:job-failed')?.({ payload: snapshot('cancelled') }) })
    expect(hook.result.current.importing).toBe(false)
    act(() => { lateProgress?.({ payload: snapshot('running') }) })
    expect(hook.result.current.currentImportJob?.status).toBe('cancelled')
    expect(hook.result.current.importing).toBe(false)
    hook.unmount()
  })

  it.each(['timed_out', 'cleanup_failed'] as const)('shows %s and stops the loading state', async status => {
    const hook = renderHook(() => useUsageImport(vi.fn(async () => {})))
    await act(async () => { await hook.result.current.startImportJob({}) })
    act(() => {
      mocks.callbacks.get('usage:job-failed')?.({ payload: { ...snapshot(status), error: status } })
    })
    expect(hook.result.current.error).toBe(status)
    expect(hook.result.current.importing).toBe(false)
    hook.unmount()
  })

  it('recovers a terminal result emitted before subscription', async () => {
    mocks.status.mockResolvedValue(snapshot('cancelled'))
    const hook = renderHook(() => useUsageImport(vi.fn(async () => {})))
    await act(async () => { await hook.result.current.startImportJob({}) })
    expect(hook.result.current.currentImportJob?.status).toBe('cancelled')
    expect(hook.result.current.importing).toBe(false)
    expect(mocks.callbacks.size).toBe(0)
    hook.unmount()
  })
})
