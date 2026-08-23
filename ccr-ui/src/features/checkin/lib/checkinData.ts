import {
  listCheckinProviders,
  listCheckinAccounts,
  listCheckinRecords,
  getTodayCheckinStats,
  listBuiltinProviders,
} from '@/api'
import { logger } from '@/utils/logger'
import type {
  AccountsResponse,
  BuiltinProvidersResponse,
  CheckinProvider,
  AccountInfo,
  CheckinRecordInfo,
  CheckinRecordsResponse,
  TodayCheckinStats,
  BuiltinProvider,
  ProvidersResponse,
} from '@/types/checkin'

export interface CheckinRefreshOptions {
  reloadProviders?: boolean
  reloadAccounts?: boolean
  reloadRecords?: boolean
  reloadStats?: boolean
  reloadBuiltin?: boolean
  reloadFailedHistory?: boolean
}

export interface BalanceSnapshotLike {
  account_id: string
  remaining_quota: number
  total_quota: number
  used_quota: number
  currency: string
  recorded_at: string
}

export interface CheckinDataBox {
  loading: boolean
  error: string | null
  recordsLoadError: string | null
  providers: CheckinProvider[]
  accounts: AccountInfo[]
  records: CheckinRecordInfo[]
  todayStats: TodayCheckinStats | null
  builtinProviders: BuiltinProvider[]
}

export type ErrorMessageFn = (error: unknown, fallback: string) => string

export const createEmptyCheckinDataBox = (): CheckinDataBox => ({
  loading: false,
  error: null,
  recordsLoadError: null,
  providers: [],
  accounts: [],
  records: [],
  todayStats: null,
  builtinProviders: [],
})

export const createCheckinDataState = (
  box: CheckinDataBox,
  getErrorMessage: ErrorMessageFn,
  notify: () => void,
) => {
  const loadAllData = async () => {
    box.loading = true
    box.error = null
    box.recordsLoadError = null
    notify()

    try {
      const results = await Promise.allSettled([
        listCheckinProviders<ProvidersResponse>(),
        listCheckinAccounts<AccountsResponse>(),
        listCheckinRecords<CheckinRecordsResponse>({ page: 1, page_size: 100 }),
        getTodayCheckinStats<TodayCheckinStats>(),
        listBuiltinProviders<BuiltinProvidersResponse>(),
      ])

      if (results[0].status === 'fulfilled') {
        box.providers = results[0].value.providers ?? []
      }
      if (results[1].status === 'fulfilled') {
        box.accounts = results[1].value.accounts ?? []
      }
      if (results[2].status === 'fulfilled') {
        box.records = results[2].value.records ?? []
        box.recordsLoadError = null
      } else {
        box.recordsLoadError = getErrorMessage(results[2].reason, '加载签到记录失败')
      }
      if (results[3].status === 'fulfilled') {
        box.todayStats = results[3].value
      }
      if (results[4].status === 'fulfilled') {
        box.builtinProviders = results[4].value.providers ?? []
      }

      if (results.every((result) => result.status === 'rejected')) {
        box.error = '加载签到数据失败'
      }
    } catch (error: unknown) {
      box.error = getErrorMessage(error, '加载失败')
      logger.error('Failed to load checkin data', error)
    } finally {
      box.loading = false
      notify()
    }
  }

  const applyBalanceSnapshot = (snapshot: BalanceSnapshotLike) => {
    const index = box.accounts.findIndex((account) => account.id === snapshot.account_id)
    if (index < 0) return
    const account = box.accounts[index]
    const next = box.accounts.slice()
    next[index] = {
      ...account,
      latest_balance: snapshot.remaining_quota,
      total_quota: snapshot.total_quota,
      total_consumed: snapshot.used_quota,
      balance_currency: snapshot.currency,
      last_balance_check_at: snapshot.recorded_at,
    }
    box.accounts = next
    notify()
  }

  const refreshCheckinData = async (options: CheckinRefreshOptions = {}) => {
    const {
      reloadProviders = false,
      reloadAccounts = true,
      reloadRecords = true,
      reloadStats = true,
      reloadBuiltin = false,
    } = options

    const tasks: Promise<unknown>[] = []

    if (reloadProviders) {
      tasks.push(
        listCheckinProviders<ProvidersResponse>().then((response) => {
          box.providers = response.providers
        }),
      )
    }
    if (reloadAccounts) {
      tasks.push(
        listCheckinAccounts<AccountsResponse>().then((response) => {
          box.accounts = response.accounts
        }),
      )
    }
    if (reloadRecords) {
      tasks.push(
        listCheckinRecords<CheckinRecordsResponse>({ page: 1, page_size: 100 })
          .then((response) => {
            box.records = response.records
            box.recordsLoadError = null
          })
          .catch((error: unknown) => {
            box.recordsLoadError = getErrorMessage(error, '加载签到记录失败')
            logger.error('Failed to load checkin records', error)
          }),
      )
    }
    if (reloadStats) {
      tasks.push(
        getTodayCheckinStats<TodayCheckinStats>().then((response) => {
          box.todayStats = response
        }),
      )
    }
    if (reloadBuiltin) {
      tasks.push(
        listBuiltinProviders<BuiltinProvidersResponse>().then((response) => {
          box.builtinProviders = response.providers
        }),
      )
    }

    await Promise.all(tasks)
    notify()
  }

  return {
    loadAllData,
    refreshCheckinData,
    applyBalanceSnapshot,
  }
}
