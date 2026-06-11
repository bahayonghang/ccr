import { afterEach, describe, expect, it, vi } from 'vitest'
import type { CheckinJobSnapshot } from '@/types/checkin'

// 事件总线 mock：模拟 Tauri listen / emit，验证 WAF 补救等待走事件路径而非轮询
const eventHandlers = new Map<string, Array<(event: { payload: unknown }) => void>>()

const emitMockEvent = (event: string, payload: unknown) => {
  for (const handler of eventHandlers.get(event) ?? []) {
    handler({ payload })
  }
}

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async (event: string, handler: (e: { payload: unknown }) => void) => {
    const handlers = eventHandlers.get(event) ?? []
    handlers.push(handler)
    eventHandlers.set(event, handlers)
    return () => {
      eventHandlers.set(
        event,
        (eventHandlers.get(event) ?? []).filter((h) => h !== handler)
      )
    }
  }),
}))

vi.mock('@/api', () => ({
  openWafLogin: vi.fn(),
  startCheckinJob: vi.fn(),
  getCheckinJobStatus: vi.fn(),
  validateWafCookieForAccount: vi.fn(),
}))

import { getCheckinJobStatus } from '@/api'
import { waitForCheckinJobResult } from '@/views/checkin/composables/checkinWafRecovery'

const mockedGetCheckinJobStatus = getCheckinJobStatus as ReturnType<typeof vi.fn>

const buildSnapshot = (
  jobId: string,
  status: CheckinJobSnapshot['status']
): CheckinJobSnapshot => ({
  job_id: jobId,
  status,
  total: 1,
  completed: status === 'finished' ? 1 : 0,
  current_account_name: '',
  logs: [],
  results: [],
  summary: { total: 1, success: status === 'finished' ? 1 : 0, already_checked_in: 0, failed: 0 },
  started_at: '2026-06-11T08:00:00.000Z',
  finished_at: status === 'finished' ? '2026-06-11T08:00:05.000Z' : undefined,
})

afterEach(() => {
  eventHandlers.clear()
  vi.clearAllMocks()
})

describe('checkin WAF recovery event-based wait smoke', () => {
  it('returns immediately for terminal initial snapshot without registering listeners', async () => {
    const terminal = buildSnapshot('job-1', 'finished')

    const result = await waitForCheckinJobResult('job-1', terminal)

    expect(result).toBe(terminal)
    expect(eventHandlers.size).toBe(0)
    expect(mockedGetCheckinJobStatus).not.toHaveBeenCalled()
  })

  it('resolves through checkin:job-finished event and ignores other job ids', async () => {
    // 对账返回仍在运行，结束信号只能来自事件（无轮询定时器）
    mockedGetCheckinJobStatus.mockResolvedValue(buildSnapshot('job-2', 'running'))

    vi.useFakeTimers()
    try {
      const pending = waitForCheckinJobResult('job-2', buildSnapshot('job-2', 'running'))

      // 等待监听挂载与对账完成（仅微任务，无定时器参与）
      await vi.waitFor(() => {
        expect(mockedGetCheckinJobStatus).toHaveBeenCalledWith('job-2')
      })
      expect(vi.getTimerCount()).toBe(0)

      // 其他 job 的事件不应误触发
      emitMockEvent('checkin:job-finished', buildSnapshot('job-other', 'finished'))
      // 非终态事件不应误触发
      emitMockEvent('checkin:job-finished', buildSnapshot('job-2', 'running'))

      const finishedSnapshot = buildSnapshot('job-2', 'finished')
      emitMockEvent('checkin:job-finished', finishedSnapshot)

      await expect(pending).resolves.toMatchObject({ job_id: 'job-2', status: 'finished' })
    } finally {
      vi.useRealTimers()
    }
  })

  it('resolves through timeout event payloads as terminal results', async () => {
    mockedGetCheckinJobStatus.mockResolvedValue(buildSnapshot('job-3', 'running'))

    const pending = waitForCheckinJobResult('job-3', buildSnapshot('job-3', 'pending'))
    await vi.waitFor(() => {
      expect(mockedGetCheckinJobStatus).toHaveBeenCalled()
    })

    emitMockEvent('checkin:job-timeout', buildSnapshot('job-3', 'timed_out'))

    await expect(pending).resolves.toMatchObject({ job_id: 'job-3', status: 'timed_out' })
  })

  it('reconciles once via getCheckinJobStatus to cover events fired before listeners attach', async () => {
    // 任务在监听挂载前已经结束：事件已错过，对账快照兜底
    mockedGetCheckinJobStatus.mockResolvedValue(buildSnapshot('job-4', 'finished'))

    const result = await waitForCheckinJobResult('job-4', buildSnapshot('job-4', 'running'))

    expect(result).toMatchObject({ job_id: 'job-4', status: 'finished' })
    expect(mockedGetCheckinJobStatus).toHaveBeenCalledTimes(1)
  })
})
