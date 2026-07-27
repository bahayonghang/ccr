import { ref } from 'vue'
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

import { createCheckinDataState } from '@/views/checkin/composables/checkinDataState'
import { createCheckinJobRuntime } from '@/views/checkin/composables/checkinJobRuntime'
import {
  createCheckinWafRecovery,
  formatWafCookieRecoveryFailure,
  formatWafCookieValidationFailure,
  mapCheckinJobLogEntry,
  mergeRetryLogsIntoProgress,
  waitForCheckinJobResult,
} from '@/views/checkin/composables/checkinWafRecovery'

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
  status: 'success' | 'already_checked_in' | 'failed' | 'skipped' = 'success'
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
  overrides: Partial<CheckinJobSnapshot> = {}
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

beforeEach(() => {
  vi.clearAllMocks()
  apiMocks.listCheckinProviders.mockResolvedValue({ providers: [provider], total: 1 })
  apiMocks.listCheckinAccounts.mockResolvedValue({ accounts: [account('a')], total: 1 })
  apiMocks.listCheckinRecords.mockResolvedValue({ records: [{ id: 'record-1' }], total: 1 })
  apiMocks.getTodayCheckinStats.mockResolvedValue({ total: 1, success: 1, failed: 0 })
  apiMocks.listBuiltinProviders.mockResolvedValue({ providers: [{ id: 'builtin-1' }], total: 1 })
})

describe('check-in data state coverage', () => {
  const createRefs = () => ({
    loading: ref(false),
    error: ref<string | null>(null),
    recordsLoadError: ref<string | null>(null),
    providers: ref<CheckinProvider[]>([]),
    accounts: ref<AccountInfo[]>([]),
    records: ref([]),
    todayStats: ref(null),
    builtinProviders: ref([]),
  })

  it('loads, refreshes, and applies balance snapshots across every selected surface', async () => {
    const refs = createRefs()
    const state = createCheckinDataState(refs, (error, fallback) =>
      error instanceof Error ? error.message : fallback
    )

    await state.loadAllData()
    expect(refs.loading.value).toBe(false)
    expect(refs.providers.value).toEqual([provider])
    expect(refs.accounts.value[0].id).toBe('a')
    expect(refs.records.value).toEqual([{ id: 'record-1' }])
    expect(refs.todayStats.value).toMatchObject({ total: 1 })
    expect(refs.builtinProviders.value).toEqual([{ id: 'builtin-1' }])

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
    expect(refs.accounts.value[0]).toMatchObject({
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
    expect(apiMocks.listBuiltinProviders).toHaveBeenCalledTimes(2)

    apiMocks.listCheckinRecords.mockRejectedValueOnce(new Error('records unavailable'))
    await state.refreshCheckinData({
      reloadAccounts: false,
      reloadRecords: true,
      reloadStats: false,
    })
    expect(refs.recordsLoadError.value).toBe('records unavailable')
    expect(loggerMocks.error).toHaveBeenCalledWith(
      'Failed to load checkin records',
      expect.any(Error)
    )
    await state.refreshCheckinData({
      reloadAccounts: false,
      reloadRecords: false,
      reloadStats: false,
    })
  })

  it('keeps partial results, reports record failures, and marks an all-rejected load', async () => {
    const refs = createRefs()
    const getError = (error: unknown, fallback: string) =>
      error instanceof Error ? error.message : fallback
    const state = createCheckinDataState(refs, getError)

    apiMocks.listCheckinRecords.mockRejectedValueOnce(new Error('records failed'))
    await state.loadAllData()
    expect(refs.recordsLoadError.value).toBe('records failed')
    expect(refs.error.value).toBeNull()

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
    expect(refs.error.value).toBe('加载签到数据失败')

    apiMocks.listCheckinProviders.mockImplementationOnce(() => {
      throw new Error('synchronous failure')
    })
    await state.loadAllData()
    expect(refs.error.value).toBe('synchronous failure')
    expect(refs.loading.value).toBe(false)
  })
})

describe('check-in job runtime coverage', () => {
  const createRuntimeRefs = () => ({
    accounts: ref([account('a'), account('b', false)]),
    checkinLoading: ref(false),
    checkinResult: ref<CheckinDisplayResponse | null>(null),
    checkinResultRef: ref<HTMLElement | null>({
      scrollIntoView: vi.fn(),
    } as unknown as HTMLElement),
    showProgressModal: ref(false),
    checkinFlowPhase: ref<'running' | 'recovering' | 'finished'>('finished'),
    checkinProgress: ref({ total: 0, completed: 0, currentAccountName: '' }),
    checkinLogs: ref<CheckinLogEntry[]>([]),
    wafRecoveryRunning: ref(false),
    wafRecoveryProviderName: ref<string | null>('stale'),
    wafRecoveryMessage: ref<string | null>('stale'),
    activeCheckinJobId: ref<string | null>(null),
  })

  it('tracks a terminal job, refreshes data, scrolls failures, and runs recovery', async () => {
    const refs = createRuntimeRefs()
    const unlisten = vi.fn(async () => undefined)
    eventMocks.listen.mockResolvedValue(unlisten)
    const terminal = snapshot('finished', {
      results: [result('a', 'failed')],
      summary: { total: 1, success: 0, already_checked_in: 0, failed: 1 },
    })
    apiMocks.startCheckinJob.mockResolvedValue({ job_id: 'job-1', snapshot: snapshot('running') })
    apiMocks.getCheckinJobStatus.mockResolvedValue(terminal)
    const refresh = vi.fn(async () => undefined)
    const recover = vi.fn(async (value: CheckinDisplayResponse) => ({
      ...value,
      summary: { ...value.summary, skipped: 0 },
    }))
    const notify = vi.fn()
    const runtime = createCheckinJobRuntime(refs, refresh, recover, notify)

    await runtime.executeCheckinAll()

    expect(apiMocks.startCheckinJob).toHaveBeenCalledWith(['a'])
    expect(eventMocks.listen).toHaveBeenCalledTimes(3)
    expect(unlisten).toHaveBeenCalledTimes(3)
    expect(refresh).toHaveBeenCalledWith({
      reloadAccounts: true,
      reloadRecords: true,
      reloadStats: true,
    })
    expect(refs.checkinResultRef.value?.scrollIntoView).toHaveBeenCalledWith({
      behavior: 'smooth',
      block: 'start',
    })
    expect(recover).toHaveBeenCalled()
    expect(refs.checkinFlowPhase.value).toBe('finished')
    expect(refs.checkinLoading.value).toBe(false)
    expect(notify).not.toHaveBeenCalled()
  })

  it('merges matching deltas, ignores other jobs, and cleans up listeners', async () => {
    const refs = createRuntimeRefs()
    const handlers = new Map<string, (event: { payload: unknown }) => void>()
    const unlisten = vi.fn(async () => undefined)
    eventMocks.listen.mockImplementation(async (name: string, handler: (event: { payload: unknown }) => void) => {
      handlers.set(name, handler)
      return unlisten
    })
    apiMocks.startCheckinJob.mockResolvedValue({ job_id: 'job-1', snapshot: snapshot('running') })
    apiMocks.getCheckinJobStatus.mockResolvedValue(snapshot('running'))
    const runtime = createCheckinJobRuntime(
      refs,
      vi.fn(async () => undefined),
      vi.fn(async (value) => value),
      vi.fn()
    )

    await runtime.executeCheckinSingle('a')
    const deltaHandler = handlers.get('checkin:job-delta')
    expect(deltaHandler).toBeDefined()

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
    expect(refs.checkinProgress.value.completed).toBe(0)

    deltaHandler?.({ payload: { ...baseDelta, jobId: 'job-1' } })
    expect(refs.checkinProgress.value).toMatchObject({ completed: 1, total: 2 })
    expect(refs.checkinLogs.value[0]).toMatchObject({ accountId: 'a', message: 'updated' })

    deltaHandler?.({
      payload: {
        ...baseDelta,
        jobId: 'job-1',
        changedLogs: [{ ...changedLog, account_id: 'b', account_name: 'Account b' }],
      },
    })
    expect(refs.checkinLogs.value.map((entry) => entry.accountId)).toEqual(['a', 'b'])
    await runtime.cleanupCheckinJobListeners()
    expect(unlisten).toHaveBeenCalledTimes(3)
  })

  it('resets state and notifies when job startup fails or has no enabled accounts', async () => {
    const refs = createRuntimeRefs()
    refs.accounts.value = [account('disabled', false)]
    const notify = vi.fn()
    const runtime = createCheckinJobRuntime(
      refs,
      vi.fn(async () => undefined),
      vi.fn(async (value) => value),
      notify
    )
    await runtime.executeCheckinAll()
    expect(apiMocks.startCheckinJob).not.toHaveBeenCalled()

    eventMocks.listen.mockResolvedValue(vi.fn())
    apiMocks.startCheckinJob.mockRejectedValueOnce(new Error('start failed'))
    await runtime.executeCheckinSingle('a')
    expect(notify).toHaveBeenCalledWith(expect.any(Error))
    expect(refs.showProgressModal.value).toBe(false)
    expect(refs.activeCheckinJobId.value).toBeNull()
    expect(loggerMocks.error).toHaveBeenCalledWith('Checkin job failed', expect.any(Error))
  })
})

describe('check-in WAF recovery coverage', () => {
  const initialResult = (): CheckinDisplayResponse => ({
    results: [result('a', 'failed'), result('ok', 'success')],
    summary: { total: 2, success: 1, already_checked_in: 0, failed: 1 },
  })

  const createRecoveryRefs = (providers: CheckinProvider[] = [provider]) => ({
    providers: ref(providers),
    checkinResult: ref<CheckinDisplayResponse | null>(null),
    checkinLogs: ref<CheckinLogEntry[]>([
      mapCheckinJobLogEntry({
        account_id: 'a',
        account_name: 'Account a',
        provider_name: provider.name,
        status: 'failed',
        message: 'blocked',
        timestamp: '2026-07-26T00:00:00Z',
      }),
    ]),
    checkinFlowPhase: ref<'running' | 'recovering' | 'finished'>('finished'),
    wafRecoveryRunning: ref(false),
    wafRecoveryProviderName: ref<string | null>(null),
    wafRecoveryMessage: ref<string | null>(null),
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
    const refs = createRecoveryRefs()
    const recovery = createCheckinWafRecovery(
      refs,
      vi.fn(async () => undefined),
      () => 'error',
      () => provider.base_url
    )
    const successOnly: CheckinDisplayResponse = {
      results: [result('ok')],
      summary: { total: 1, success: 1, already_checked_in: 0, failed: 0 },
    }
    expect(await recovery.runWafRecovery(successOnly)).toBe(successOnly)
    refs.wafRecoveryRunning.value = true
    const blocked = initialResult()
    expect(await recovery.runWafRecovery(blocked)).toBe(blocked)
  })

  it('marks missing providers and failed cookie acquisition without changing successful rows', async () => {
    const missingRefs = createRecoveryRefs([])
    const missingRecovery = createCheckinWafRecovery(
      missingRefs,
      vi.fn(async () => undefined),
      () => 'error',
      () => provider.base_url
    )
    const missingResult = await missingRecovery.runWafRecovery(initialResult())
    expect(missingResult.results[0]).toMatchObject({
      waf_recovery_attempted: true,
      waf_recovered: false,
    })
    expect(missingResult.results[1].waf_recovery_attempted).toBeUndefined()

    const refs = createRecoveryRefs()
    apiMocks.openWafLogin.mockRejectedValueOnce(new Error('webview failed'))
    const recovery = createCheckinWafRecovery(
      refs,
      vi.fn(async () => undefined),
      (error) => (error instanceof Error ? error.message : 'unknown'),
      () => provider.base_url
    )
    const failedResult = await recovery.runWafRecovery(initialResult())
    expect(failedResult.results[0].waf_recovery_error).toContain('webview failed')
    expect(refs.wafRecoveryRunning.value).toBe(false)
    expect(refs.wafRecoveryProviderName.value).toBeNull()
  })

  it('validates persisted cookies, retries accounts, merges logs, and refreshes data', async () => {
    const refs = createRecoveryRefs()
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
    const recovery = createCheckinWafRecovery(
      refs,
      refresh,
      (error) => String(error),
      (candidate) => `${candidate.base_url}/login`
    )

    const recovered = await recovery.runWafRecovery(initialResult())

    expect(apiMocks.openWafLogin).toHaveBeenCalledWith(
      'https://provider.example.test/login',
      provider.id
    )
    expect(apiMocks.validateWafCookieForAccount).toHaveBeenCalledWith('a')
    expect(apiMocks.startCheckinJob).toHaveBeenCalledWith(['a'])
    expect(recovered.results[0]).toMatchObject({ status: 'success', waf_recovered: true })
    expect(refs.checkinLogs.value[0]).toMatchObject({ message: 'recovered', wafRecovered: true })
    expect(refresh).toHaveBeenCalledWith({
      reloadAccounts: true,
      reloadRecords: true,
      reloadStats: true,
    })
  })

  it('covers validation and retry failure formatting plus missing retry logs', async () => {
    expect(
      formatWafCookieRecoveryFailure({
        ...recoverySuccess,
        persisted: false,
        missing_cookie_names: ['waf-a', 'waf-b'],
      })
    ).toBe('缺少 WAF Cookie: waf-a, waf-b')
    expect(formatWafCookieRecoveryFailure({ ...recoverySuccess, persisted: false })).toBe(
      'WAF Cookie 未获取完整'
    )
    expect(formatWafCookieValidationFailure({ ...validationSuccess, success: false })).toBe(
      'WAF Cookie 验证失败'
    )

    const refs = createRecoveryRefs()
    apiMocks.openWafLogin.mockResolvedValue(recoverySuccess)
    apiMocks.validateWafCookieForAccount.mockResolvedValue({
      ...validationSuccess,
      success: false,
      message: 'still challenged',
    })
    const recovery = createCheckinWafRecovery(
      refs,
      vi.fn(async () => undefined),
      () => 'fallback',
      () => provider.base_url
    )
    const validationFailed = await recovery.runWafRecovery(initialResult())
    expect(validationFailed.results[0].waf_recovery_error).toContain('still challenged')

    const merged = mergeRetryLogsIntoProgress(refs.checkinLogs.value, snapshot('finished', {
      logs: [],
    }), ['a'])
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
    eventMocks.listen.mockImplementation(async (name: string, handler: (event: { payload: CheckinJobSnapshot }) => void) => {
      handlers.set(name, handler)
      return unlisten
    })
    apiMocks.getCheckinJobStatus.mockResolvedValue(snapshot('running'))
    const pending = waitForCheckinJobResult('job-1', snapshot('running'))
    await flushPromises()
    handlers.get('checkin:job-finished')?.({ payload: snapshot('finished') })
    await expect(pending).resolves.toMatchObject({ status: 'finished' })
    expect(unlisten).toHaveBeenCalledTimes(2)
  })
})
