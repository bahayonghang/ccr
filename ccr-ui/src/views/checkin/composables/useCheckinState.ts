import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { addBuiltinProvider as apiAddBuiltinProvider, queryCheckinBalance } from '@/api'
import { logger } from '@/utils/logger'
import { getErrorMessage } from '@/types/api'
// 过渡期接线（批次 4）：Pinia 已删，Zustand 单例经 getState() 提供同名 API；
// 批次 5 / views-checkin 转换时整体重写。
import { useUIStore } from '@/shell/stores/ui'
import type {
  BalanceSnapshot,
  CheckinProvider,
  AccountInfo,
  CheckinRecordInfo,
  TodayCheckinStats,
  CheckinDisplayResponse,
  CheckinDisplayResult,
  CheckinExecutionResult,
  BuiltinProvider,
  CheckinLogEntry,
  CheckinFlowPhase,
} from '@/types/checkin'
import { createCheckinDataState } from './checkinDataState'
import { createCheckinJobRuntime } from './checkinJobRuntime'
import { filterAvailableBuiltinProviders } from './builtinProviderLookup'
import { runPerKeySequential, shouldSkipBalanceRefresh } from './balanceRefreshQueue'
import {
  applyRecoveryFailureToLogs,
  createCheckinWafRecovery,
  formatWafCookieRecoveryFailure,
  mapCheckinJobLogEntry,
  mergeRetryLogsIntoProgress,
} from './checkinWafRecovery'

export {
  applyRecoveryFailureToLogs,
  formatWafCookieRecoveryFailure,
  mergeRetryLogsIntoProgress,
  mapCheckinJobLogEntry,
}

/**
 * 签到模块共享状态 composable
 * 提供所有 tab 组件共享的状态、计算属性和操作函数
 */
export function useCheckinState() {
  const { t, locale } = useI18n()
  const uiStore = useUIStore.getState()

  // ═══════════════════════════════════════════════════════════
  // 状态
  // ═══════════════════════════════════════════════════════════
  const loading = ref(false)
  const checkinLoading = ref(false)
  const balanceRefreshing = ref(false)
  const error = ref<string | null>(null)
  const recordsLoadError = ref<string | null>(null)
  const checkinResultRef = ref<HTMLElement | null>(null)
  const activeTab = ref<'providers' | 'accounts' | 'records' | 'import-export'>('accounts')
  const showCheckinConfirm = ref(false)
  const showProgressModal = ref(false)
  const showOAuthWizard = ref(false)
  const checkinFlowPhase = ref<CheckinFlowPhase>('finished')
  const checkinProgress = ref({ total: 0, completed: 0, currentAccountName: '' })
  const checkinLogs = ref<CheckinLogEntry[]>([])
  const wafRecoveryRunning = ref(false)
  const wafRecoveryProviderName = ref<string | null>(null)
  const wafRecoveryMessage = ref<string | null>(null)

  // ═══════════════════════════════════════════════════════════
  // 数据
  // ═══════════════════════════════════════════════════════════
  const providers = ref<CheckinProvider[]>([])
  const accounts = ref<AccountInfo[]>([])
  const records = ref<CheckinRecordInfo[]>([])
  const todayStats = ref<TodayCheckinStats | null>(null)
  const checkinResult = ref<CheckinDisplayResponse | null>(null)
  const builtinProviders = ref<BuiltinProvider[]>([])

  const activeCheckinJobId = ref<string | null>(null)

  const getProviderLoginUrl = (provider: CheckinProvider) => {
    return `${provider.base_url.replace(/\/+$/, '')}/login`
  }

  const { loadAllData, refreshCheckinData, applyBalanceSnapshot } = createCheckinDataState(
    {
      loading,
      error,
      recordsLoadError,
      providers,
      accounts,
      records,
      todayStats,
      builtinProviders,
    },
    getErrorMessage
  )

  const { runWafRecovery } = createCheckinWafRecovery(
    {
      providers,
      checkinResult,
      checkinLogs,
      checkinFlowPhase,
      wafRecoveryRunning,
      wafRecoveryProviderName,
      wafRecoveryMessage,
    },
    refreshCheckinData,
    getErrorMessage,
    getProviderLoginUrl
  )

  const { cleanupCheckinJobListeners, executeCheckinAll, executeCheckinSingle } =
    createCheckinJobRuntime(
      {
        accounts,
        checkinLoading,
        checkinResult,
        checkinResultRef,
        showProgressModal,
        checkinFlowPhase,
        checkinProgress,
        checkinLogs,
        wafRecoveryRunning,
        wafRecoveryProviderName,
        wafRecoveryMessage,
        activeCheckinJobId,
      },
      refreshCheckinData,
      runWafRecovery,
      (jobError) => {
        uiStore.showError(
          t('checkin.errors.checkinFailed', {
            error: getErrorMessage(jobError, t('checkin.errors.unknown')),
          })
        )
      }
    )

  // ═══════════════════════════════════════════════════════════
  // 计算属性
  // ═══════════════════════════════════════════════════════════

  // 过滤出尚未添加的内置提供商（builtin_id 优先判定，name 回退兼容旧数据）
  const availableBuiltinProviders = computed(() =>
    filterAvailableBuiltinProviders(builtinProviders.value, providers.value)
  )

  // 汇总统计数据
  const totalStatistics = computed(() => {
    const result = {
      currentBalance: 0,
      totalQuota: 0,
      totalConsumed: 0,
    }
    for (const account of accounts.value) {
      if (account.latest_balance !== undefined && account.latest_balance !== null) {
        result.currentBalance += account.latest_balance
      }
      if (account.total_quota !== undefined && account.total_quota !== null) {
        result.totalQuota += account.total_quota
      }
      if (account.total_consumed !== undefined && account.total_consumed !== null) {
        result.totalConsumed += account.total_consumed
      }
    }
    return result
  })

  // 启用的账号列表
  const enabledAccounts = computed(() => {
    return accounts.value.filter((a) => a.enabled)
  })

  // 签到结果分类
  const failedCheckinResults = computed(() => {
    if (!checkinResult.value) return []
    return checkinResult.value.results.filter((r) => r.status === 'failed')
  })

  const successCheckinResults = computed(() => {
    if (!checkinResult.value) return []
    return checkinResult.value.results.filter((r) => r.status === 'success')
  })

  const alreadyCheckedInResults = computed(() => {
    if (!checkinResult.value) return []
    return checkinResult.value.results.filter((r) => r.status === 'already_checked_in')
  })

  // 跳过结果（4 态契约：不支持签到/禁用等，不计入失败）
  const skippedCheckinResults = computed(() => {
    if (!checkinResult.value) return []
    return checkinResult.value.results.filter((r) => r.status === 'skipped')
  })

  // ═══════════════════════════════════════════════════════════
  // Tab 配置
  // ═══════════════════════════════════════════════════════════
  const tabs = [
    { id: 'accounts' as const, nameKey: 'checkin.tabs.accounts', icon: 'Users' },
    { id: 'providers' as const, nameKey: 'checkin.tabs.providers', icon: 'Building2' },
    { id: 'records' as const, nameKey: 'checkin.tabs.records', icon: 'FileText' },
    { id: 'import-export' as const, nameKey: 'checkin.tabs.importExport', icon: 'Package' },
  ]

  // ═══════════════════════════════════════════════════════════
  // 签到操作
  // ═══════════════════════════════════════════════════════════

  // 确认签到弹窗回调
  const handleCheckinConfirm = () => {
    showCheckinConfirm.value = false
    executeCheckinAll()
  }

  // 关闭签到弹窗
  const closeCheckinModal = () => {
    showProgressModal.value = false
  }

  // OAuth 登录成功后刷新数据
  const handleOAuthSuccess = async () => {
    showOAuthWizard.value = false
    await loadAllData()
  }

  // ═══════════════════════════════════════════════════════════
  // Cookie 过期快捷修复入口
  // ═══════════════════════════════════════════════════════════

  // 待打开编辑弹窗的账号 ID（由 CheckinAccountsTab 消费后清空）
  const pendingEditAccountId = ref<string | null>(null)

  // 失败卡片/记录行点击「更新 Cookie」：切到账号 Tab 并直达编辑弹窗
  const openAccountCookieFix = (accountId: string) => {
    activeTab.value = 'accounts'
    pendingEditAccountId.value = accountId
  }

  const clearPendingEditAccount = () => {
    pendingEditAccountId.value = null
  }

  // 提供商 origin 作为串行 key：同站请求串行、异站并行（解析失败回退 provider_id）
  const getAccountOriginKey = (account: AccountInfo) => {
    const provider = providers.value.find((p) => p.id === account.provider_id)
    if (!provider) return account.provider_id
    try {
      return new URL(provider.base_url).origin
    } catch {
      return account.provider_id
    }
  }

  const refreshAllBalances = async () => {
    if (accounts.value.length === 0) return

    balanceRefreshing.value = true
    const enabledAccs = accounts.value.filter((a) => a.enabled)
    // 30s 内已刷新的账号跳过（单账号手动刷新不受此限制）
    const now = Date.now()
    const skippedAccs = enabledAccs.filter((a) =>
      shouldSkipBalanceRefresh(a.last_balance_check_at, now)
    )
    const accountsToRefresh = enabledAccs.filter(
      (a) => !shouldSkipBalanceRefresh(a.last_balance_check_at, now)
    )

    try {
      if (skippedAccs.length > 0) {
        uiStore.showInfo(t('checkin.info.balanceRefreshSkipped', { count: skippedAccs.length }))
      }
      if (accountsToRefresh.length === 0) return

      const results = await runPerKeySequential(
        accountsToRefresh.map((account) => ({
          key: getAccountOriginKey(account),
          run: () => queryCheckinBalance<BalanceSnapshot>(account.id),
        }))
      )
      const failedNames: string[] = []
      results.forEach((result, index) => {
        if (result.status === 'fulfilled') {
          applyBalanceSnapshot(result.value)
        } else {
          failedNames.push(accountsToRefresh[index].name)
          logger.error(
            `Failed to refresh balance for ${accountsToRefresh[index].name}`,
            result.reason
          )
        }
      })
      if (failedNames.length > 0) {
        uiStore.showError(
          t('checkin.errors.batchRefreshBalanceFailed', {
            count: failedNames.length,
            names: failedNames.join(', '),
          })
        )
      }
      await refreshCheckinData({
        reloadAccounts: false,
        reloadRecords: true,
        reloadStats: true,
      })
    } catch (e: unknown) {
      logger.error('Batch refresh failed', e)
    } finally {
      balanceRefreshing.value = false
    }
  }

  // 刷新单个账号余额（手动 force 路径：不受 30s 节流限制）
  const refreshAccountBalance = async (accountId: string) => {
    try {
      const snapshot = await queryCheckinBalance<BalanceSnapshot>(accountId)
      applyBalanceSnapshot(snapshot)
      await refreshCheckinData({
        reloadAccounts: false,
        reloadRecords: false,
        reloadStats: true,
      })
    } catch (e: unknown) {
      uiStore.showError(
        t('checkin.errors.refreshBalanceFailed', {
          error: getErrorMessage(e, t('checkin.errors.unknown')),
        })
      )
    }
  }

  // ═══════════════════════════════════════════════════════════
  // 内置提供商操作
  // ═══════════════════════════════════════════════════════════

  const addBuiltinProvider = async (builtinId: string) => {
    try {
      await apiAddBuiltinProvider(builtinId)
      await loadAllData()
    } catch (e: unknown) {
      uiStore.showError(
        t('checkin.errors.addProviderFailed', {
          error: getErrorMessage(e, t('checkin.errors.unknown')),
        })
      )
      logger.error('Failed to add builtin provider', e)
    }
  }

  // ═══════════════════════════════════════════════════════════
  // 辅助函数
  // ═══════════════════════════════════════════════════════════

  const getProviderName = (providerId: string) => {
    return providers.value.find((p) => p.id === providerId)?.name || providerId
  }

  const getAccountName = (accountId: string) => {
    return accounts.value.find((a) => a.id === accountId)?.name || accountId
  }

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleString(locale.value)
  }

  const getStatusClass = (status: string) => {
    switch (status) {
      case 'success':
        return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400'
      case 'already_checked_in':
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400'
      case 'failed':
        return 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400'
      case 'skipped':
        return 'bg-slate-100 text-slate-600 dark:bg-slate-800/60 dark:text-slate-300'
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-400'
    }
  }

  const getStatusText = (status: string) => {
    switch (status) {
      case 'success':
        return t('checkin.status.success')
      case 'already_checked_in':
        return t('checkin.status.already_checked_in')
      case 'failed':
        return t('checkin.status.failed')
      case 'skipped':
        return t('checkin.status.skipped')
      default:
        return status
    }
  }

  const buildCheckinDetail = (item: CheckinExecutionResult, fallback: string) => {
    const details: string[] = []
    if (item.reward) {
      details.push(t('checkin.detail.reward', { reward: item.reward }))
    }
    if (item.balance !== undefined && item.balance !== null) {
      details.push(t('checkin.detail.balance', { balance: item.balance }))
    }
    if (item.message) {
      details.push(item.message)
    }
    return details.length > 0 ? details.join(' · ') : fallback
  }

  const getSuccessDetail = (item: CheckinDisplayResult) =>
    buildCheckinDetail(item, t('checkin.detail.checkinSuccess'))

  const getAlreadyCheckedInDetail = (item: CheckinDisplayResult) =>
    buildCheckinDetail(item, t('checkin.detail.todayAlreadyCheckedIn'))

  // skip_reason → i18n 文案（未知原因回退 message 或通用文案）
  const getSkipReasonText = (skipReason?: string): string | null => {
    if (!skipReason) return null
    const reasons: Record<string, string> = {
      account_disabled: t('checkin.skipReasons.account_disabled'),
      provider_disabled: t('checkin.skipReasons.provider_disabled'),
      provider_unsupported: t('checkin.skipReasons.provider_unsupported'),
    }
    return reasons[skipReason] ?? skipReason
  }

  const getSkippedDetail = (item: CheckinDisplayResult) => {
    const reason = getSkipReasonText(item.skip_reason)
    if (reason) return reason
    return item.message || t('checkin.detail.skipped')
  }

  const getErrorHint = (code?: string): string | null => {
    if (!code) return null
    const hints: Record<string, string> = {
      cookie_expired: t('checkin.errors.hints.cookie_expired'),
      waf_blocked: t('checkin.errors.hints.waf_blocked'),
      cf_blocked: t('checkin.errors.hints.cf_blocked'),
      network_error: t('checkin.errors.hints.network_error'),
      timeout: t('checkin.errors.hints.timeout'),
      crypto_error: t('checkin.errors.hints.crypto_error'),
      provider_error: t('checkin.errors.hints.provider_error'),
      account_error: t('checkin.errors.hints.account_error'),
      task_error: t('checkin.errors.hints.task_error'),
      api_error: t('checkin.errors.hints.api_error'),
    }
    return hints[code] ?? null
  }

  const getErrorLabel = (code?: string): string | null => {
    if (!code) return null
    const labels: Record<string, string> = {
      cookie_expired: t('checkin.errors.labels.cookie_expired'),
      waf_blocked: t('checkin.errors.labels.waf_blocked'),
      cf_blocked: t('checkin.errors.labels.cf_blocked'),
      network_error: t('checkin.errors.labels.network_error'),
      timeout: t('checkin.errors.labels.timeout'),
      crypto_error: t('checkin.errors.labels.crypto_error'),
      provider_error: t('checkin.errors.labels.provider_error'),
      account_error: t('checkin.errors.labels.account_error'),
      task_error: t('checkin.errors.labels.task_error'),
      api_error: t('checkin.errors.labels.api_error'),
    }
    return labels[code] ?? code
  }

  const getFailedDetail = (item: CheckinDisplayResult) => {
    let detail = item.message || t('checkin.errors.unknownReason')
    const hint = getErrorHint(item.error_code)
    if (hint) {
      detail = `${detail}（${hint}）`
    }
    if (item.waf_recovery_attempted && item.waf_recovered === false && item.waf_recovery_error) {
      detail = `${detail} · ${item.waf_recovery_error}`
    }
    return detail
  }

  // ═══════════════════════════════════════════════════════════
  // 生命周期
  // ═══════════════════════════════════════════════════════════

  onMounted(() => {
    loadAllData()
  })

  onUnmounted(() => {
    void cleanupCheckinJobListeners()
  })

  return {
    // 状态
    loading,
    checkinLoading,
    balanceRefreshing,
    error,
    checkinResultRef,
    activeTab,
    showCheckinConfirm,
    showProgressModal,
    showOAuthWizard,
    checkinFlowPhase,
    checkinProgress,
    checkinLogs,
    wafRecoveryRunning,
    wafRecoveryProviderName,
    wafRecoveryMessage,

    // 数据
    providers,
    accounts,
    records,
    recordsLoadError,
    todayStats,
    checkinResult,
    builtinProviders,

    // 计算属性
    availableBuiltinProviders,
    totalStatistics,
    enabledAccounts,
    failedCheckinResults,
    successCheckinResults,
    alreadyCheckedInResults,
    skippedCheckinResults,

    // Tab 配置
    tabs,

    // 数据加载
    loadAllData,
    refreshCheckinData,
    applyBalanceSnapshot,

    // 签到操作
    executeCheckinAll,
    executeCheckinSingle,
    handleCheckinConfirm,
    closeCheckinModal,
    handleOAuthSuccess,

    // Cookie 快捷修复
    pendingEditAccountId,
    openAccountCookieFix,
    clearPendingEditAccount,

    // 余额操作
    refreshAllBalances,
    refreshAccountBalance,

    // 内置提供商操作
    addBuiltinProvider,

    // 辅助函数
    getProviderName,
    getAccountName,
    formatDate,
    getStatusClass,
    getStatusText,
    buildCheckinDetail,
    getSuccessDetail,
    getAlreadyCheckedInDetail,
    getFailedDetail,
    getSkipReasonText,
    getSkippedDetail,
    getErrorHint,
    getErrorLabel,
  }
}
