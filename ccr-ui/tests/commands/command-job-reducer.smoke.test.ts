import { describe, expect, it } from 'vitest'
import {
  COMMAND_OUTPUT_BYTES_PER_CHANNEL, boundCommandLines, commandJobMissing, emptyCommandJob,
  mergeCommandDelta, mergeCommandSnapshot,
} from '@/features/commands/commandJobState'
import type { CommandJobSnapshot } from '@/types'

const queued: CommandJobSnapshot = {
  job_id: 'job-reducer', command: 'status', args: [], status: 'queued', started_at: '', finished_at: null,
  duration_ms: null, exit_code: null, stdout_lines: [], stderr_lines: [], system_lines: [],
  dropped_lines: 0, truncated: false, error: null,
}

describe('command snapshot reducer', () => {
  it.each(['success', 'failed', 'cancelled', 'cleanup_failed', 'unavailable'] as const)('keeps %s monotonic against old snapshot and delta delivery', (status) => {
    const terminal = mergeCommandSnapshot(emptyCommandJob(), { ...queued, status, stdout_lines: ['final'] })
    expect(mergeCommandSnapshot(terminal, queued)).toBe(terminal)
    expect(mergeCommandDelta(terminal, { job_id: queued.job_id, seq: 90, channel: 'stdout', lines: ['late'], dropped_count: 0, status: 'running' })).toBe(terminal)
  })

  it('rejects a queued snapshot after running and ignores foreign identities', () => {
    const running = mergeCommandSnapshot(emptyCommandJob(), { ...queued, status: 'running' })
    expect(mergeCommandSnapshot(running, queued)).toBe(running)
    expect(mergeCommandSnapshot(running, { ...queued, job_id: 'other' })).toBe(running)
    expect(mergeCommandDelta(running, { job_id: 'other', seq: 0, channel: 'stdout', lines: ['other'], dropped_count: 0 })).toBe(running)
  })

  it('retains complete duplicate lines while bounding UTF-8 bytes per channel', () => {
    const line = '中'.repeat(90_000)
    const bounded = boundCommandLines(['old', line, line])
    expect(bounded.lines).toEqual([line])
    expect(bounded.dropped).toBe(2)
    expect(new TextEncoder().encode(bounded.lines.join('')).length).toBeLessThanOrEqual(COMMAND_OUTPUT_BYTES_PER_CHANNEL)
    const job = mergeCommandSnapshot(emptyCommandJob(), queued, false)
    const next = mergeCommandDelta(job, { job_id: queued.job_id, seq: 0, channel: 'stdout', lines: ['old', line, line], dropped_count: 3 })
    expect(next.snapshot?.stdout_lines).toEqual([line])
    expect(next.snapshot?.dropped_lines).toBe(5)
    expect(next.snapshot?.truncated).toBe(true)
  })

  it('does not infer backend expiration from unrelated read failures', () => {
    expect(commandJobMissing("Command job 'job-reducer' not found", 'job-reducer')).toBe(true)
    expect(commandJobMissing(new Error("Command job 'job-reducer' not found"), 'job-reducer')).toBe(true)
    expect(commandJobMissing('permission denied', 'job-reducer')).toBe(false)
    expect(commandJobMissing("Command job 'other' not found", 'job-reducer')).toBe(false)
  })
})
