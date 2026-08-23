import { beforeEach, describe, expect, it, vi } from 'vitest'
import type {
  AccountInfo,
  CheckinDisplayResponse,
  CheckinJobDelta,
  CheckinJobSnapshot,
  CheckinLogEntry,
  CheckinProvider,
  WafCookieRecoveryResult,
  WafCookieValidationResult,
} from '@/types/checkin'

const apiMocks = vi.hoisted(() => ({
  listCheckinProviders: vi.fn(),
  listCheckinAccounts: vi.fn(),
  listCheckinRecords: vi.fn(),
  getTodayCheckinStats: vi.fn(),
  listBuiltinProviders: vi.fn(),
  getCheckinJobStatus: vi.fn(),
  startCheckinJob: vi.fn(),
  openWafLogin: vi.fn(),
  validateWafCookieForAccount: vi.fn(),
}))

const eventMocks = vi.hoisted(() => ({
  listen: vi.fn(),
}))

const loggerMocks = vi.hoisted(() => ({
  error: vi.fn(),
}))

vi.mock('@/api', () => apiMocks)
vi.mock('@tauri-apps/api/event', () => eventMocks)
vi.mock('@/utils/logger', () => ({ logger: loggerMocks }))

import {
  createCheckinDataState,
  createCheckinJobRuntime,
  createCheckinWafRecovery,
  createEmptyCheckinDataBox,
  formatWafCookieRecoveryFailure,
  formatWafCookieValidationFailure,
  mapCheckinJobLogEntry,
  mergeRetryLogsIntoProgress,
  waitForCheckinJobResult,
} from '@/features/checkin'
import type { CheckinJobBox } from '@/features/checkin/lib/checkinJob'
import type { WafRecoveryBox } from '@/features/checkin/lib/checkinWafRecovery'

const provider: CheckinProvider = {
  id: 'provider-1',
  name: 'Provider One',
  base_url: 'https://provider.example.test',
  checkin_path: '/checkin',
  balance_path: '/balance',
  user_info_path: '/user',
  auth_header: 'Authorization',
  auth_prefix: 'Bearer ',
  enabled: true,
  created_at: '2026-07-26T00:00:00Z',
}

const account = (id: string, enabled = true): AccountInfo => ({
  id,
  provider_id: provider.id,
  provider_name: provider.name,
  name: `Account ${id}`,
  cookies_masked: '***',
  api_user: id,
  enabled,
  created_at: '2026-07-26T00:00:00Z',
})

const result = (
  accountId: string,
  status: 'success' | 'already_checked_in' | 'failed' | 'skipped' = 'success',
) => ({
  account_id: accountId,
  account_name: `Account ${accountId}`,
  provider_name: provider.name,
  status,
  message: `${status} message`,
  error_code: status === 'failed' ? 'waf_blocked' : undefined,
})

const snapshot = (
  status: CheckinJobSnapshot['status'],
  overrides: Partial<CheckinJobSnapshot> = {},
): CheckinJobSnapshot => ({
  job_id: 'job-1',
  status,
  total: 1,
  completed: status === 'running' ? 0 : 1,
  current_account_name: status === 'running' ? 'Account a' : '',
  logs: [
    {
      account_id: 'a',
      account_name: 'Account a',
      provider_name: provider.name,
      status: status === 'running' ? 'processing' : 'success',
      message: 'progress',
      timestamp: '2026-07-26T00:00:00Z',
    },
  ],
  results: status === 'running' ? [] : [result('a')],
  summary: {
    total: status === 'running' ? 0 : 1,
    success: status === 'running' ? 0 : 1,
    already_checked_in: 0,
    failed: 0,
  },
  started_at: '2026-07-26T00:00:00Z',
  ...overrides,
})

const flushPromises = async () => {
  await Promise.resolve()
  await Promise.resolve()
  await new Promise((resolve) => setTimeout(resolve, 0))
}

const noop = () => undefined

beforeEach(() => {
  vi.clearAllMocks()
  apiMocks.listCheckinProviders.mockResolvedValue({ providers: [provider], total: 1 })
  apiMocks.listCheckinAccounts.mockResolvedValue({ accounts: [account('a')], total: 1 })
  apiMocks.listCheckinRecords.mockResolvedValue({ records: [{ id: 'record-1' }], total: 1 })
  apiMocks.getTodayCheckinStats.mockResolvedValue({ total: 1, success: 1, failed: 0 })
  apiMocks.listBuiltinProviders.mockResolvedValue({ providers: [{ id: 'builtin-1' }], total: 1 })
})

describe('check-in data state coverage', () => {
  it('loads, refreshes, and applies balance snapshots across every selected surface', async () => {
    const box = createEmptyCheckinDataBox()
    const state = createCheckinDataState(box, (error, fallback) =>
      error instanceof Error ? error.message : fallback,
      noop,
    )

    await state.loadAllData()
    expect(box.loading).toBe(false)
    expect(box.providers).toEqual([provider])
    expect(box.accounts[0].id).toBe('a')
    expect(box.records).toEqual([{ id: 'record-1' }])
    expect(box.todayStats).toMatchObject({ total: 1 })
    expect(box.builtinProviders).toEqual([{ id: 'builtin-1' }])

    state.applyBalanceSnapshot({
      account_id: 'missing',
      remaining_quota: 1,
      total_quota: 2,
      used_quota: 1,
      currency: 'USD',
      recorded_at: '2026-07-26T01:00:00Z',
    })
    state.applyBalanceSnapshot({
      account_id: 'a',
      remaining_quota: 75,
      total_quota: 100,
      used_quota: 25,
      currency: 'USD',
      recorded_at: '2026-07-26T01:00:00Z',
    })
    expect(box.accounts[0]).toMatchObject({
      latest_balance: 75,
      total_quota: 100,
      total_consumed: 25,
      balance_currency: 'USD',
    })

    await state.refreshCheckinData({
      reloadProviders: true,
      reloadAccounts: true,
      reloadRecords: true,
      reloadStats: true,
      reloadBuiltin: true,
    })
    expect(apiMocks.listCheckinProviders).toHaveBeenCalledTimes(2)

    apiMocks.listCheckinRecords.mockRejectedValueOnce(new Error('records unavailable'))
    await state.refreshCheckinData({
      reloadAccounts: false,
      reloadRecords: true,
      reloadStats: false,
    })
    expect(box.recordsLoadError).toBe('records unavailable')
  })

  it('keeps partial results, reports record failures, and marks an all-rejected load', async () => {
    const box = createEmptyCheckinDataBox()
    const state = createCheckinDataState(
      box,
      (error, fallback) => (error instanceof Error ? error.message : fallback),
      noop,
    )

    apiMocks.listCheckinRecords.mockRejectedValueOnce(new Error('records failed'))
    await state.loadAllData()
    expect(box.recordsLoadError).toBe('records failed')
    expect(box.error).toBeNull()

    for (const mock of [
      apiMocks.listCheckinProviders,
      apiMocks.listCheckinAccounts,
      apiMocks.listCheckinRecords,
      apiMocks.getTodayCheckinStats,
      apiMocks.listBuiltinProviders,
    ]) {
      mock.mockRejectedValueOnce(new Error('offline'))
    }
    await state.loadAllData()
    expect(box.error).toBe('加载签到数据失败')
  })
})

describe('check-in job runtime coverage', () => {
  const createRuntimeBox = (): CheckinJobBox => ({
    accounts: [account('a'), account('b', false)],
    checkinLoading: false,
    checkinResult: null,
    checkinResultRef: { scrollIntoView: vi.fn() } as unknown as HTMLElement,
    showProgressModal: false,
    checkinFlowPhase: 'finished',
    checkinProgress: { total: 0, completed: 0, currentAccountName: '' },
    checkinLogs: [],
    wafRecoveryRunning: false,
    wafRecoveryProviderName: 'stale',
    wafRecoveryMessage: 'stale',
    activeCheckinJobId: null,
  })

  it('tracks a terminal job, refreshes data, scrolls failures, and runs recovery', async () => {
    const box = createRuntimeBox()
    const unlisten = vi.fn()
    eventMocks.listen.mockResolvedValue(unlisten)
    const terminal = snapshot('finished', {
      results: [result('a', 'failed')],
      summary: { total: 1, success: 0, already_checked_in: 0, failed: 1 },
    })
    apiMocks.startCheckinJob.mockResolvedValue({ job_id: 'job-1', snapshot: snapshot('running') })
    apiMocks.getCheckinJobStatus.mockResolvedValue(terminal)
    const refresh = vi.fn(async () => undefined)
    const recover = vi.fn(async (value: CheckinDisplayResponse) => value)
    const runtime = createCheckinJobRuntime({
      box,
      refreshCheckinData: refresh,
      runWafRecovery: recover,
      notifyJobStartFailed: vi.fn(),
      notify: noop,
    })

    await runtime.executeCheckinAll()
    await flushPromises()

    expect(apiMocks.startCheckinJob).toHaveBeenCalledWith(['a'])
    expect(eventMocks.listen).toHaveBeenCalledTimes(3)
    expect(refresh).toHaveBeenCalledWith({
      reloadAccounts: true,
      reloadRecords: true,
      reloadStats: true,
    })
    expect(recover).toHaveBeenCalled()
    expect(box.checkinFlowPhase).toBe('finished')
    expect(box.checkinLoading).toBe(false)
  })

  it('merges matching deltas, ignores other jobs, and cleans up listeners', async () => {
    const box = createRuntimeBox()
    const handlers = new Map<string, (event: { payload: unknown }) => void>()
    const unlisten = vi.fn()
    eventMocks.listen.mockImplementation(async (name: string, handler: (event: { payload: unknown }) => void) => {
      handlers.set(name, handler)
      return unlisten
    })
    apiMocks.startCheckinJob.mockResolvedValue({ job_id: 'job-1', snapshot: snapshot('running') })
    apiMocks.getCheckinJobStatus.mockResolvedValue(snapshot('running'))
    const runtime = createCheckinJobRuntime({
      box,
      refreshCheckinData: vi.fn(async () => undefined),
      runWafRecovery: vi.fn(
        async (value: CheckinDisplayResponse): Promise<CheckinDisplayResponse> => value,
      ),
      notifyJobStartFailed: vi.fn(),
      notify: noop,
    })

    await runtime.executeCheckinSingle('a')
    await flushPromises()
    const deltaHandler = handlers.get('checkin:job-delta')
    const changedLog = {
      account_id: 'a',
      account_name: 'Account a',
      provider_name: provider.name,
      status: 'success' as const,
      message: 'updated',
      timestamp: '2026-07-26T01:00:00Z',
    }
    const baseDelta: CheckinJobDelta = {
      jobId: 'other-job',
      status: 'running',
      completed: 1,
      total: 2,
      currentAccountName: 'Account b',
      summary: { total: 1, success: 1, already_checked_in: 0, failed: 0 },
      changedLogs: [changedLog],
      newResults: [],
    }
    deltaHandler?.({ payload: baseDelta })
    expect(box.checkinProgress.completed).toBe(0)
    deltaHandler?.({ payload: { ...baseDelta, jobId: 'job-1' } })
    expect(box.checkinProgress).toMatchObject({ completed: 1, total: 2 })
    await runtime.cleanupCheckinJobListeners()
    expect(unlisten).toHaveBeenCalled()
  })

  it('resets state and notifies when job startup fails or has no enabled accounts', async () => {
    const box = createRuntimeBox()
    box.accounts = [account('disabled', false)]
    const notify = vi.fn()
    const runtime = createCheckinJobRuntime({
      box,
      refreshCheckinData: vi.fn(async () => undefined),
      runWafRecovery: vi.fn(
        async (value: CheckinDisplayResponse): Promise<CheckinDisplayResponse> => value,
      ),
      notifyJobStartFailed: notify,
      notify: noop,
    })
    await runtime.executeCheckinAll()
    expect(apiMocks.startCheckinJob).not.toHaveBeenCalled()

    eventMocks.listen.mockResolvedValue(vi.fn())
    apiMocks.startCheckinJob.mockRejectedValueOnce(new Error('start failed'))
    await runtime.executeCheckinSingle('a')
    expect(notify).toHaveBeenCalledWith(expect.any(Error))
    expect(box.showProgressModal).toBe(false)
  })
})

describe('check-in WAF recovery coverage', () => {
  const initialResult = (): CheckinDisplayResponse => ({
    results: [result('a', 'failed'), result('ok', 'success')],
    summary: { total: 2, success: 1, already_checked_in: 0, failed: 1 },
  })

  const createRecoveryBox = (providers: CheckinProvider[] = [provider]): WafRecoveryBox => ({
    providers,
    checkinResult: null,
    checkinLogs: [
      mapCheckinJobLogEntry({
        account_id: 'a',
        account_name: 'Account a',
        provider_name: provider.name,
        status: 'failed',
        message: 'blocked',
        timestamp: '2026-07-26T00:00:00Z',
      }),
    ],
    checkinFlowPhase: 'finished',
    wafRecoveryRunning: false,
    wafRecoveryProviderName: null,
    wafRecoveryMessage: null,
  })

  const recoverySuccess: WafCookieRecoveryResult = {
    provider_id: provider.id,
    provider_name: provider.name,
    found_cookie_names: ['waf'],
    missing_cookie_names: [],
    required_cookie_names: ['waf'],
    persisted: true,
    source: 'webview_store',
    message: '',
  }

  const validationSuccess: WafCookieValidationResult = {
    account_id: 'a',
    provider_id: provider.id,
    provider_name: provider.name,
    success: true,
    challenge: 'none',
    message: '',
  }

  it('returns immediately for no blocked groups and in-progress recovery', async () => {
    const box = createRecoveryBox()
    const recovery = createCheckinWafRecovery({
      box,
      refreshCheckinData: vi.fn(async () => undefined),
      getErrorMessage: () => 'error',
      getProviderLoginUrl: () => provider.base_url,
      notify: noop,
    })
    const successOnly: CheckinDisplayResponse = {
      results: [result('ok')],
      summary: { total: 1, success: 1, already_checked_in: 0, failed: 0 },
    }
    expect(await recovery.runWafRecovery(successOnly)).toBe(successOnly)
    box.wafRecoveryRunning = true
    const blocked = initialResult()
    expect(await recovery.runWafRecovery(blocked)).toBe(blocked)
  })

  it('marks missing providers and failed cookie acquisition without changing successful rows', async () => {
    const missingBox = createRecoveryBox([])
    const missingRecovery = createCheckinWafRecovery({
      box: missingBox,
      refreshCheckinData: vi.fn(async () => undefined),
      getErrorMessage: () => 'error',
      getProviderLoginUrl: () => provider.base_url,
      notify: noop,
    })
    const missingResult = await missingRecovery.runWafRecovery(initialResult())
    expect(missingResult.results[0]).toMatchObject({
      waf_recovery_attempted: true,
      waf_recovered: false,
    })

    const box = createRecoveryBox()
    apiMocks.openWafLogin.mockRejectedValueOnce(new Error('webview failed'))
    const recovery = createCheckinWafRecovery({
      box,
      refreshCheckinData: vi.fn(async () => undefined),
      getErrorMessage: (error) => (error instanceof Error ? error.message : 'unknown'),
      getProviderLoginUrl: () => provider.base_url,
      notify: noop,
    })
    const failedResult = await recovery.runWafRecovery(initialResult())
    expect(failedResult.results[0].waf_recovery_error).toContain('webview failed')
  })

  it('validates persisted cookies, retries accounts, merges logs, and refreshes data', async () => {
    const box = createRecoveryBox()
    apiMocks.openWafLogin.mockResolvedValue(recoverySuccess)
    apiMocks.validateWafCookieForAccount.mockResolvedValue(validationSuccess)
    const retry = snapshot('finished', {
      logs: [
        {
          account_id: 'a',
          account_name: 'Account a',
          provider_name: provider.name,
          status: 'success',
          message: 'recovered',
          timestamp: '2026-07-26T02:00:00Z',
        },
      ],
      results: [result('a', 'success')],
    })
    apiMocks.startCheckinJob.mockResolvedValue({ job_id: 'job-1', snapshot: retry })
    const refresh = vi.fn(async () => undefined)
    const recovery = createCheckinWafRecovery({
      box,
      refreshCheckinData: refresh,
      getErrorMessage: (error) => String(error),
      getProviderLoginUrl: (candidate) => `${candidate.base_url}/login`,
      notify: noop,
    })
    const recovered = await recovery.runWafRecovery(initialResult())
    expect(apiMocks.openWafLogin).toHaveBeenCalledWith(
      'https://provider.example.test/login',
      provider.id,
    )
    expect(recovered.results[0]).toMatchObject({ status: 'success', waf_recovered: true })
    expect(refresh).toHaveBeenCalled()
  })

  it('covers validation and retry failure formatting plus missing retry logs', async () => {
    expect(
      formatWafCookieRecoveryFailure({
        ...recoverySuccess,
        persisted: false,
        missing_cookie_names: ['waf-a', 'waf-b'],
      }),
    ).toBe('缺少 WAF Cookie: waf-a, waf-b')
    expect(formatWafCookieValidationFailure({ ...validationSuccess, success: false })).toBe(
      'WAF Cookie 验证失败',
    )
    const logs: CheckinLogEntry[] = [
      mapCheckinJobLogEntry({
        account_id: 'a',
        account_name: 'Account a',
        provider_name: provider.name,
        status: 'failed',
        message: 'blocked',
        timestamp: '2026-07-26T00:00:00Z',
      }),
    ]
    const merged = mergeRetryLogsIntoProgress(logs, snapshot('finished', { logs: [] }), ['a'])
    expect(merged[0]).toMatchObject({
      wafRecoveryAttempted: true,
      wafRecovered: false,
      wafRecoveryError: '自动重试未返回日志',
    })
  })

  it('resolves terminal snapshots immediately and event-driven snapshots after listener setup', async () => {
    const terminal = snapshot('timed_out')
    await expect(waitForCheckinJobResult('job-1', terminal)).resolves.toBe(terminal)

    const handlers = new Map<string, (event: { payload: CheckinJobSnapshot }) => void>()
    const unlisten = vi.fn()
    eventMocks.listen.mockImplementation(
      async (name: string, handler: (event: { payload: CheckinJobSnapshot }) => void) => {
        handlers.set(name, handler)
        return unlisten
      },
    )
    apiMocks.getCheckinJobStatus.mockResolvedValue(snapshot('running'))
    const pending = waitForCheckinJobResult('job-1', snapshot('running'))
    await flushPromises()
    handlers.get('checkin:job-finished')?.({ payload: snapshot('finished') })
    await expect(pending).resolves.toMatchObject({ status: 'finished' })
    expect(unlisten).toHaveBeenCalledTimes(2)
  })
})
