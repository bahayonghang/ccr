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
import type { Ref } from 'vue'

export interface CheckinRefreshOptions {
  reloadProviders?: boolean
  reloadAccounts?: boolean
  reloadRecords?: boolean
  reloadStats?: boolean
  reloadBuiltin?: boolean
  reloadFailedHistory?: boolean
}

interface BalanceSnapshotLike {
  account_id: string
  remaining_quota: number
  total_quota: number
  used_quota: number
  currency: string
  recorded_at: string
}

interface CheckinDataRefs {
  loading: Ref<boolean>
  error: Ref<string | null>
  recordsLoadError: Ref<string | null>
  providers: Ref<CheckinProvider[]>
  accounts: Ref<AccountInfo[]>
  records: Ref<CheckinRecordInfo[]>
  todayStats: Ref<TodayCheckinStats | null>
  builtinProviders: Ref<BuiltinProvider[]>
}

export const createCheckinDataState = (
  refs: CheckinDataRefs,
  getErrorMessage: (error: unknown, fallback: string) => string
) => {
  const loadAllData = async () => {
    refs.loading.value = true
    refs.error.value = null
    refs.recordsLoadError.value = null

    try {
      const results = await Promise.allSettled([
        listCheckinProviders<ProvidersResponse>(),
        listCheckinAccounts<AccountsResponse>(),
        listCheckinRecords<CheckinRecordsResponse>({ page: 1, page_size: 100 }),
        getTodayCheckinStats<TodayCheckinStats>(),
        listBuiltinProviders<BuiltinProvidersResponse>(),
      ])

      if (results[0].status === 'fulfilled') {
        refs.providers.value = results[0].value.providers ?? []
      }
      if (results[1].status === 'fulfilled') {
        refs.accounts.value = results[1].value.accounts ?? []
      }
      if (results[2].status === 'fulfilled') {
        refs.records.value = results[2].value.records ?? []
        refs.recordsLoadError.value = null
      } else {
        refs.recordsLoadError.value = getErrorMessage(
          results[2].reason,
          '加载签到记录失败'
        )
      }
      if (results[3].status === 'fulfilled') {
        refs.todayStats.value = results[3].value
      }
      if (results[4].status === 'fulfilled') {
        refs.builtinProviders.value = results[4].value.providers ?? []
      }

      if (results.every((result) => result.status === 'rejected')) {
        refs.error.value = '加载签到数据失败'
      }
    } catch (error: unknown) {
      refs.error.value = getErrorMessage(error, '加载失败')
      logger.error('Failed to load checkin data', error)
    } finally {
      refs.loading.value = false
    }
  }

  const applyBalanceSnapshot = (snapshot: BalanceSnapshotLike) => {
    const index = refs.accounts.value.findIndex((account) => account.id === snapshot.account_id)
    if (index < 0) return
    const account = refs.accounts.value[index]
    refs.accounts.value[index] = {
      ...account,
      latest_balance: snapshot.remaining_quota,
      total_quota: snapshot.total_quota,
      total_consumed: snapshot.used_quota,
      balance_currency: snapshot.currency,
      last_balance_check_at: snapshot.recorded_at,
    }
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
          refs.providers.value = response.providers
        })
      )
    }
    if (reloadAccounts) {
      tasks.push(
        listCheckinAccounts<AccountsResponse>().then((response) => {
          refs.accounts.value = response.accounts
        })
      )
    }
    if (reloadRecords) {
      tasks.push(
        listCheckinRecords<CheckinRecordsResponse>({ page: 1, page_size: 100 })
          .then((response) => {
            refs.records.value = response.records
            refs.recordsLoadError.value = null
          })
          .catch((error: unknown) => {
            refs.recordsLoadError.value = getErrorMessage(error, '加载签到记录失败')
            logger.error('Failed to load checkin records', error)
          })
      )
    }
    if (reloadStats) {
      tasks.push(
        getTodayCheckinStats<TodayCheckinStats>().then((response) => {
          refs.todayStats.value = response
        })
      )
    }
    if (reloadBuiltin) {
      tasks.push(
        listBuiltinProviders<BuiltinProvidersResponse>().then((response) => {
          refs.builtinProviders.value = response.providers
        })
      )
    }

    await Promise.all(tasks)
  }

  return {
    loadAllData,
    refreshCheckinData,
    applyBalanceSnapshot,
  }
}
