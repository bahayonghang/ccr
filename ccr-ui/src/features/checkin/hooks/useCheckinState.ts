import { useCallback, useEffect, useMemo, useReducer, useRef } from 'react'
import { addBuiltinProvider as apiAddBuiltinProvider, queryCheckinBalance } from '@/api'
import { getErrorMessage } from '@/types/api'
import { logger } from '@/utils/logger'
import type { BalanceSnapshot } from '@/types/checkin'
import { filterAvailableBuiltinProviders } from '../lib/builtinProviderLookup'
import { runPerKeySequential, shouldSkipBalanceRefresh } from '../lib/balanceRefreshQueue'
import { createCheckinDataState } from '../lib/checkinData'
import {
  CHECKIN_TABS,
  filterResults,
  getAccountOriginKey,
  getAlreadyCheckedInDetail,
  getErrorLabel,
  getFailedDetail,
  getProviderLoginUrl,
  getSkippedDetail,
  getSuccessDetail,
  sumAccountStats,
  type CheckinTabId,
} from '../lib/checkinFormat'
import { createCheckinJobRuntime } from '../lib/checkinJob'
import { checkinNotify } from '../lib/checkinNotify'
import { createCheckinRuntimeBox } from '../lib/checkinRuntimeBox'
import { createCheckinWafRecovery } from '../lib/checkinWafRecovery'
import { useCheckinLocale, useCheckinT } from './useCheckinT'

export {
  applyRecoveryFailureToLogs,
  formatWafCookieRecoveryFailure,
  mergeRetryLogsIntoProgress,
  mapCheckinJobLogEntry,
} from '../lib/checkinWafRecovery'

export function useCheckinState() {
  const t = useCheckinT()
  const locale = useCheckinLocale()
  const [, bump] = useReducer((count: number) => count + 1, 0)
  const notify = useCallback(() => bump(), [])
  const boxRef = useRef(createCheckinRuntimeBox())
  const box = boxRef.current

  const data = useMemo(
    () => createCheckinDataState(box, getErrorMessage, notify),
    [box, notify],
  )
  const waf = useMemo(
    () =>
      createCheckinWafRecovery({
        box,
        refreshCheckinData: data.refreshCheckinData,
        getErrorMessage,
        getProviderLoginUrl,
        notify,
      }),
    [box, data.refreshCheckinData, notify],
  )
  const job = useMemo(
    () =>
      createCheckinJobRuntime({
        box,
        refreshCheckinData: data.refreshCheckinData,
        runWafRecovery: waf.runWafRecovery,
        notifyJobStartFailed: (jobError) => {
          checkinNotify.error(
            t('checkin.errors.checkinFailed', {
              error: getErrorMessage(jobError, t('checkin.errors.unknown')),
            }),
          )
        },
        notify,
      }),
    [box, data.refreshCheckinData, notify, t, waf.runWafRecovery],
  )

  useEffect(() => {
    void data.loadAllData()
    return () => {
      void job.cleanupCheckinJobListeners()
    }
  }, [data, job])

  const availableBuiltinProviders = filterAvailableBuiltinProviders(
    box.builtinProviders,
    box.providers,
  )
  const totalStatistics = sumAccountStats(box.accounts)
  const enabledAccounts = box.accounts.filter((account) => account.enabled)
  const failedCheckinResults = filterResults(box.checkinResult, 'failed')
  const successCheckinResults = filterResults(box.checkinResult, 'success')
  const alreadyCheckedInResults = filterResults(box.checkinResult, 'already_checked_in')
  const skippedCheckinResults = filterResults(box.checkinResult, 'skipped')

  const setActiveTab = useCallback(
    (tab: CheckinTabId) => {
      box.activeTab = tab
      notify()
    },
    [box, notify],
  )

  const setShowCheckinConfirm = useCallback(
    (open: boolean) => {
      box.showCheckinConfirm = open
      notify()
    },
    [box, notify],
  )

  const setShowOAuthWizard = useCallback(
    (open: boolean) => {
      box.showOAuthWizard = open
      notify()
    },
    [box, notify],
  )

  const setCheckinResultNull = useCallback(() => {
    box.checkinResult = null
    notify()
  }, [box, notify])

  const setCheckinResultRef = useCallback(
    (node: HTMLElement | null) => {
      box.checkinResultRef = node
    },
    [box],
  )

  const handleCheckinConfirm = useCallback(() => {
    box.showCheckinConfirm = false
    notify()
    void job.executeCheckinAll()
  }, [box, job, notify])

  const closeCheckinModal = useCallback(() => {
    box.showProgressModal = false
    notify()
  }, [box, notify])

  const handleOAuthSuccess = useCallback(async () => {
    box.showOAuthWizard = false
    notify()
    await data.loadAllData()
  }, [box, data, notify])

  const openAccountCookieFix = useCallback(
    (accountId: string) => {
      box.activeTab = 'accounts'
      box.pendingEditAccountId = accountId
      notify()
    },
    [box, notify],
  )

  const clearPendingEditAccount = useCallback(() => {
    box.pendingEditAccountId = null
    notify()
  }, [box, notify])

  const refreshAllBalances = useCallback(async () => {
    if (box.accounts.length === 0) return
    const enabledAccs = box.accounts.filter((account) => account.enabled)
    const now = Date.now()
    const skippedAccs = enabledAccs.filter((account) =>
      shouldSkipBalanceRefresh(account.last_balance_check_at, now),
    )
    const accountsToRefresh = enabledAccs.filter(
      (account) => !shouldSkipBalanceRefresh(account.last_balance_check_at, now),
    )
    box.balanceRefreshing = true
    notify()
    try {
      if (skippedAccs.length > 0) {
        checkinNotify.info(t('checkin.info.balanceRefreshSkipped', { count: skippedAccs.length }))
      }
      if (accountsToRefresh.length === 0) return
      const results = await runPerKeySequential(
        accountsToRefresh.map((account) => ({
          key: getAccountOriginKey(account, box.providers),
          run: () => queryCheckinBalance<BalanceSnapshot>(account.id),
        })),
      )
      const failedNames: string[] = []
      results.forEach((result, index) => {
        if (result.status === 'fulfilled') data.applyBalanceSnapshot(result.value)
        else {
          failedNames.push(accountsToRefresh[index].name)
          logger.error(`Failed to refresh balance for ${accountsToRefresh[index].name}`, result.reason)
        }
      })
      if (failedNames.length > 0) {
        checkinNotify.error(
          t('checkin.errors.batchRefreshBalanceFailed', {
            count: failedNames.length,
            names: failedNames.join(', '),
          }),
        )
      }
      await data.refreshCheckinData({
        reloadAccounts: false,
        reloadRecords: true,
        reloadStats: true,
      })
    } catch (error: unknown) {
      logger.error('Batch refresh failed', error)
    } finally {
      box.balanceRefreshing = false
      notify()
    }
  }, [box, data, notify, t])

  const refreshAccountBalance = useCallback(
    async (accountId: string) => {
      try {
        const snapshot = await queryCheckinBalance<BalanceSnapshot>(accountId)
        data.applyBalanceSnapshot(snapshot)
        await data.refreshCheckinData({
          reloadAccounts: false,
          reloadRecords: false,
          reloadStats: true,
        })
      } catch (error: unknown) {
        checkinNotify.error(
          t('checkin.errors.refreshBalanceFailed', {
            error: getErrorMessage(error, t('checkin.errors.unknown')),
          }),
        )
      }
    },
    [data, t],
  )

  const addBuiltinProvider = useCallback(
    async (builtinId: string) => {
      try {
        await apiAddBuiltinProvider(builtinId)
        await data.loadAllData()
      } catch (error: unknown) {
        checkinNotify.error(
          t('checkin.errors.addProviderFailed', {
            error: getErrorMessage(error, t('checkin.errors.unknown')),
          }),
        )
        logger.error('Failed to add builtin provider', error)
      }
    },
    [data, t],
  )

  const formatDate = useCallback(
    (dateStr: string) => new Date(dateStr).toLocaleString(locale),
    [locale],
  )

  return {
    loading: box.loading,
    checkinLoading: box.checkinLoading,
    balanceRefreshing: box.balanceRefreshing,
    error: box.error,
    recordsLoadError: box.recordsLoadError,
    setCheckinResultRef,
    activeTab: box.activeTab,
    setActiveTab,
    showCheckinConfirm: box.showCheckinConfirm,
    setShowCheckinConfirm,
    showProgressModal: box.showProgressModal,
    showOAuthWizard: box.showOAuthWizard,
    setShowOAuthWizard,
    checkinFlowPhase: box.checkinFlowPhase,
    checkinProgress: box.checkinProgress,
    checkinLogs: box.checkinLogs,
    wafRecoveryRunning: box.wafRecoveryRunning,
    wafRecoveryProviderName: box.wafRecoveryProviderName,
    wafRecoveryMessage: box.wafRecoveryMessage,
    providers: box.providers,
    accounts: box.accounts,
    records: box.records,
    checkinResult: box.checkinResult,
    setCheckinResultNull,
    builtinProviders: box.builtinProviders,
    todayStats: box.todayStats,
    availableBuiltinProviders,
    totalStatistics,
    enabledAccounts,
    failedCheckinResults,
    successCheckinResults,
    alreadyCheckedInResults,
    skippedCheckinResults,
    tabs: CHECKIN_TABS,
    loadAllData: data.loadAllData,
    executeCheckinSingle: job.executeCheckinSingle,
    handleCheckinConfirm,
    closeCheckinModal,
    handleOAuthSuccess,
    pendingEditAccountId: box.pendingEditAccountId,
    openAccountCookieFix,
    clearPendingEditAccount,
    refreshAllBalances,
    refreshAccountBalance,
    addBuiltinProvider,
    formatDate,
    getSuccessDetail: (item: Parameters<typeof getSuccessDetail>[0]) => getSuccessDetail(item, t),
    getAlreadyCheckedInDetail: (item: Parameters<typeof getAlreadyCheckedInDetail>[0]) =>
      getAlreadyCheckedInDetail(item, t),
    getFailedDetail: (item: Parameters<typeof getFailedDetail>[0]) => getFailedDetail(item, t),
    getSkippedDetail: (item: Parameters<typeof getSkippedDetail>[0]) => getSkippedDetail(item, t),
    getErrorLabel: (code?: string) => getErrorLabel(code, t),
    t,
  }
}
