import { describe, expect, it } from 'vitest'
import type { CheckinJobSnapshot, CheckinLogEntry } from '@/types/checkin'
import {
  applyRecoveryFailureToLogs,
  mergeRetryLogsIntoProgress,
} from '@/views/checkin/composables/useCheckinState'

const baseLogs: CheckinLogEntry[] = [
  {
    accountId: 'acc-1',
    accountName: 'anyrouter_stumail',
    providerName: 'AnyRouter',
    status: 'failed',
    message: 'API error: 检测到 WAF 挑战页面',
    errorCode: 'waf_blocked',
    timestamp: new Date('2026-03-23T09:00:00.000Z'),
  },
  {
    accountId: 'acc-2',
    accountName: 'duckcoding_main',
    providerName: 'DuckCoding',
    status: 'success',
    message: '签到成功',
    timestamp: new Date('2026-03-23T09:00:01.000Z'),
  },
]

describe('checkin state helpers smoke', () => {
  it('overwrites retried accounts with final recovered logs while preserving order', () => {
    const retrySnapshot: CheckinJobSnapshot = {
      job_id: 'retry-job',
      status: 'finished',
      total: 1,
      completed: 1,
      current_account_name: '',
      logs: [
        {
          account_id: 'acc-1',
          account_name: 'anyrouter_stumail',
          provider_name: 'AnyRouter',
          status: 'success',
          message: '签到成功，获得 $25 额度',
          reward: '$25 额度',
          timestamp: '2026-03-23T09:00:05.000Z',
        },
      ],
      results: [
        {
          account_id: 'acc-1',
          account_name: 'anyrouter_stumail',
          provider_name: 'AnyRouter',
          status: 'success',
          message: '签到成功，获得 $25 额度',
          reward: '$25 额度',
        },
      ],
      summary: {
        total: 1,
        success: 1,
        already_checked_in: 0,
        failed: 0,
      },
      started_at: '2026-03-23T09:00:02.000Z',
      finished_at: '2026-03-23T09:00:05.000Z',
    }

    const merged = mergeRetryLogsIntoProgress(baseLogs, retrySnapshot, ['acc-1'])

    expect(merged).toHaveLength(2)
    expect(merged[0]).toMatchObject({
      accountId: 'acc-1',
      status: 'success',
      message: '签到成功，获得 $25 额度',
      wafRecoveryAttempted: true,
      wafRecovered: true,
    })
    expect(merged[1]).toMatchObject({
      accountId: 'acc-2',
      status: 'success',
    })
    expect(merged[1].wafRecoveryAttempted).toBeUndefined()
  })

  it('marks recovery failures without changing unaffected accounts', () => {
    const merged = applyRecoveryFailureToLogs(
      baseLogs,
      ['acc-1'],
      '自动获取 WAF Cookie 失败：登录超时'
    )

    expect(merged[0]).toMatchObject({
      accountId: 'acc-1',
      status: 'failed',
      wafRecoveryAttempted: true,
      wafRecovered: false,
      wafRecoveryError: '自动获取 WAF Cookie 失败：登录超时',
    })
    expect(merged[1]).toMatchObject({
      accountId: 'acc-2',
      status: 'success',
    })
    expect(merged[1].wafRecoveryAttempted).toBeUndefined()
  })
})
