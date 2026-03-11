import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { Building2, Users, FileText, Package } from 'lucide-vue-next'
import {
  listCheckinProviders,
  listCheckinAccounts,
  listCheckinRecords,
  getTodayCheckinStats,
  listBuiltinProviders,
  addBuiltinProvider as apiAddBuiltinProvider,
  queryCheckinBalance,
  startCheckinJob,
  getCheckinJobStatus,
  openWafLogin,
} from '@/api'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { logger } from '@/utils/logger'
import type {
  AccountsResponse,
  BalanceSnapshot,
  BuiltinProvidersResponse,
  CheckinProvider,
  AccountInfo,
  CheckinRecordInfo,
  CheckinRecordsResponse,
  TodayCheckinStats,
  CheckinResponse,
  CheckinExecutionResult,
  CheckinDisplayResponse,
  CheckinDisplayResult,
  BuiltinProvider,
  CheckinJobLogEntryPayload,
  CheckinJobSnapshot,
  CheckinLogEntry,
  ProvidersResponse,
  StartCheckinJobResponse,
} from '@/types/checkin'

/** 签到数据刷新选项 */
export interface CheckinRefreshOptions {
  reloadProviders?: boolean
  reloadAccounts?: boolean
  reloadRecords?: boolean
  reloadStats?: boolean
  reloadBuiltin?: boolean
  reloadFailedHistory?: boolean
}

interface WafBlockedGroup {
  providerName: string
  provider: CheckinProvider | null
  accountIds: string[]
}

/**
 * 签到模块共享状态 composable
 * 提供所有 tab 组件共享的状态、计算属性和操作函数
 */
export function useCheckinState() {
  const getErrorMessage = (error: unknown, fallback: string) =>
    error instanceof Error ? error.message : fallback

  // ═══════════════════════════════════════════════════════════
  // 状态
  // ═══════════════════════════════════════════════════════════
  const loading = ref(false)
  const checkinLoading = ref(false)
  const balanceRefreshing = ref(false)
  const error = ref<string | null>(null)
  const checkinResultRef = ref<HTMLElement | null>(null)
  const activeTab = ref<'providers' | 'accounts' | 'records' | 'import-export'>('accounts')
  const showCheckinConfirm = ref(false)
  const showProgressModal = ref(false)
  const showOAuthWizard = ref(false)
  const isCheckinFinished = ref(false)
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
  const checkinJobUnlisteners: UnlistenFn[] = []

  const cleanupCheckinJobListeners = async () => {
    const unlisteners = checkinJobUnlisteners.splice(0)
    await Promise.all(unlisteners.map((unlisten) => unlisten()))
  }

  const mapJobLogEntry = (entry: CheckinJobLogEntryPayload): CheckinLogEntry => ({
    accountId: entry.account_id,
    accountName: entry.account_name,
    providerName: entry.provider_name,
    status: entry.status,
    message: entry.message,
    errorCode: entry.error_code,
    reward: entry.reward,
    balance: entry.balance,
    timestamp: new Date(entry.timestamp),
  })

  const getProviderLoginUrl = (provider: CheckinProvider) => {
    return `${provider.base_url.replace(/\/+$/, '')}/login`
  }

  const toDisplayResponse = (response: CheckinResponse): CheckinDisplayResponse => ({
    results: response.results.map((item) => ({ ...item })),
    summary: { ...response.summary },
  })

  const buildCheckinSummary = (results: CheckinDisplayResult[]) => {
    return results.reduce(
      (summary, item) => {
        summary.total += 1
        if (item.status === 'success') {
          summary.success += 1
        } else if (item.status === 'already_checked_in') {
          summary.already_checked_in += 1
        } else {
          summary.failed += 1
        }
        return summary
      },
      { total: 0, success: 0, already_checked_in: 0, failed: 0 }
    )
  }

  const createDisplayResponse = (results: CheckinDisplayResult[]): CheckinDisplayResponse => ({
    results,
    summary: buildCheckinSummary(results),
  })

  const detectWafBlockedGroups = (result: CheckinDisplayResponse): WafBlockedGroup[] => {
    const grouped = new Map<string, string[]>()
    for (const item of result.results) {
      if (item.status !== 'failed' || item.error_code !== 'waf_blocked') continue
      const accountIds = grouped.get(item.provider_name) ?? []
      if (!accountIds.includes(item.account_id)) {
        accountIds.push(item.account_id)
      }
      grouped.set(item.provider_name, accountIds)
    }

    return Array.from(grouped.entries()).map(([providerName, accountIds]) => ({
      providerName,
      provider: providers.value.find((candidate) => candidate.name === providerName) ?? null,
      accountIds,
    }))
  }

  const markWafRecoveryFailure = (
    result: CheckinDisplayResponse,
    accountIds: string[],
    recoveryError: string
  ): CheckinDisplayResponse => {
    return createDisplayResponse(
      result.results.map((item) => {
        if (!accountIds.includes(item.account_id)) return item
        return {
          ...item,
          waf_recovery_attempted: true,
          waf_recovered: false,
          waf_recovery_error: recoveryError,
        }
      })
    )
  }

  const mergeRetryResults = (
    result: CheckinDisplayResponse,
    retryResults: CheckinExecutionResult[],
    retriedAccountIds: string[]
  ): CheckinDisplayResponse => {
    const retryMap = new Map(retryResults.map((item) => [item.account_id, item]))
    return createDisplayResponse(
      result.results.map((item) => {
        const retried = retryMap.get(item.account_id)
        if (!retried) {
          if (!retriedAccountIds.includes(item.account_id)) return item
          return {
            ...item,
            waf_recovery_attempted: true,
            waf_recovered: false,
            waf_recovery_error: '自动重试未返回结果',
          }
        }
        return {
          ...retried,
          waf_recovery_attempted: true,
          waf_recovered: retried.status !== 'failed',
        }
      })
    )
  }

  const waitForCheckinJobResult = async (
    jobId: string,
    initialSnapshot: CheckinJobSnapshot
  ): Promise<CheckinJobSnapshot> => {
    let snapshot = initialSnapshot
    for (let attempt = 0; attempt < 240; attempt += 1) {
      if (snapshot.status === 'finished' || snapshot.status === 'timed_out') {
        return snapshot
      }
      await new Promise((resolve) => setTimeout(resolve, 500))
      snapshot = await getCheckinJobStatus<CheckinJobSnapshot>(jobId)
    }
    throw new Error('自动重试等待超时')
  }

  const retryAccountsAfterWaf = async (accountIds: string[]): Promise<CheckinJobSnapshot> => {
    const response = await startCheckinJob<StartCheckinJobResponse>(accountIds)
    return waitForCheckinJobResult(response.job_id, response.snapshot)
  }

  const runWafRecovery = async (initialResult: CheckinDisplayResponse): Promise<CheckinDisplayResponse> => {
    const groups = detectWafBlockedGroups(initialResult)
    if (groups.length === 0 || wafRecoveryRunning.value) {
      return initialResult
    }

    wafRecoveryRunning.value = true
    let mergedResult = initialResult

    try {
      for (const [index, group] of groups.entries()) {
        wafRecoveryProviderName.value = group.providerName

        if (!group.provider) {
          mergedResult = markWafRecoveryFailure(
            mergedResult,
            group.accountIds,
            '未找到对应的提供商配置，无法自动补救'
          )
          checkinResult.value = mergedResult
          continue
        }

        wafRecoveryMessage.value = `正在为 ${group.providerName} 获取 WAF Cookie（${index + 1}/${groups.length}）`

        try {
          await openWafLogin<string>(getProviderLoginUrl(group.provider), group.provider.id)
        } catch (error: unknown) {
          mergedResult = markWafRecoveryFailure(
            mergedResult,
            group.accountIds,
            `自动获取 WAF Cookie 失败：${getErrorMessage(error, '未知错误')}`
          )
          checkinResult.value = mergedResult
          continue
        }

        wafRecoveryMessage.value = `已获取 ${group.providerName} 的 WAF Cookie，正在重试 ${group.accountIds.length} 个账号`

        try {
          const retrySnapshot = await retryAccountsAfterWaf(group.accountIds)
          mergedResult = mergeRetryResults(mergedResult, retrySnapshot.results, group.accountIds)
          checkinResult.value = mergedResult
          await refreshCheckinData({
            reloadAccounts: true,
            reloadRecords: true,
            reloadStats: true,
          })
        } catch (error: unknown) {
          mergedResult = markWafRecoveryFailure(
            mergedResult,
            group.accountIds,
            `自动重试失败：${getErrorMessage(error, '未知错误')}`
          )
          checkinResult.value = mergedResult
        }
      }
    } finally {
      wafRecoveryRunning.value = false
      wafRecoveryProviderName.value = null
      wafRecoveryMessage.value = null
    }

    return mergedResult
  }

  const applyCheckinJobSnapshot = (snapshot: CheckinJobSnapshot) => {
    checkinProgress.value = {
      total: snapshot.total,
      completed: snapshot.completed,
      currentAccountName: snapshot.current_account_name,
    }
    checkinLogs.value = snapshot.logs.map(mapJobLogEntry)

    const isTerminal = snapshot.status === 'finished' || snapshot.status === 'timed_out'
    isCheckinFinished.value = isTerminal

    if (isTerminal) {
      checkinResult.value = toDisplayResponse({
        results: snapshot.results,
        summary: snapshot.summary,
      })
    }
  }

  const finalizeCheckinJob = async (snapshot: CheckinJobSnapshot) => {
    if (activeCheckinJobId.value !== snapshot.job_id) return

    try {
      applyCheckinJobSnapshot(snapshot)
      activeCheckinJobId.value = null
      await cleanupCheckinJobListeners()

      await refreshCheckinData({
        reloadAccounts: true,
        reloadRecords: true,
        reloadStats: true,
      })

      if (snapshot.summary.failed > 0) {
        await nextTick()
        checkinResultRef.value?.scrollIntoView({ behavior: 'smooth', block: 'start' })
      }

      const currentResult = checkinResult.value ?? toDisplayResponse({
        results: snapshot.results,
        summary: snapshot.summary,
      })
      checkinResult.value = await runWafRecovery(currentResult)
    } finally {
      checkinLoading.value = false
    }
  }

  // ═══════════════════════════════════════════════════════════
  // 计算属性
  // ═══════════════════════════════════════════════════════════

  // 过滤出尚未添加的内置提供商
  const availableBuiltinProviders = computed(() => {
    const addedNames = new Set(providers.value.map((p) => p.name))
    return builtinProviders.value.filter((bp) => !addedNames.has(bp.name))
  })

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

  // ═══════════════════════════════════════════════════════════
  // Tab 配置
  // ═══════════════════════════════════════════════════════════
  const tabs = [
    { id: 'accounts' as const, name: '账号管理', icon: Users },
    { id: 'providers' as const, name: '提供商', icon: Building2 },
    { id: 'records' as const, name: '签到记录', icon: FileText },
    { id: 'import-export' as const, name: '导入导出', icon: Package },
  ]

  // ═══════════════════════════════════════════════════════════
  // 数据加载
  // ═══════════════════════════════════════════════════════════

  // 加载所有数据
  const loadAllData = async () => {
    loading.value = true
    error.value = null

    try {
      const results = await Promise.allSettled([
        listCheckinProviders<ProvidersResponse>(),
        listCheckinAccounts<AccountsResponse>(),
        listCheckinRecords<CheckinRecordsResponse>({ page: 1, page_size: 100 }),
        getTodayCheckinStats<TodayCheckinStats>(),
        listBuiltinProviders<BuiltinProvidersResponse>(),
      ])

      if (results[0].status === 'fulfilled') {
        providers.value = results[0].value.providers ?? []
      }
      if (results[1].status === 'fulfilled') {
        accounts.value = results[1].value.accounts ?? []
      }
      if (results[2].status === 'fulfilled') {
        records.value = results[2].value.records ?? []
      }
      if (results[3].status === 'fulfilled') {
        todayStats.value = results[3].value
      }
      if (results[4].status === 'fulfilled') {
        builtinProviders.value = results[4].value.providers ?? []
      }

      // 如果全部失败，显示错误
      const allFailed = results.every((r) => r.status === 'rejected')
      if (allFailed) {
        error.value = '加载签到数据失败'
      }
    } catch (e: unknown) {
      error.value = getErrorMessage(e, '加载失败')
      logger.error('Failed to load checkin data', e)
    } finally {
      loading.value = false
    }
  }

  // 应用余额快照到本地账号数据
  const applyBalanceSnapshot = (snapshot: {
    account_id: string
    remaining_quota: number
    total_quota: number
    used_quota: number
    currency: string
    recorded_at: string
  }) => {
    const index = accounts.value.findIndex((a) => a.id === snapshot.account_id)
    if (index < 0) return
    const account = accounts.value[index]
    accounts.value[index] = {
      ...account,
      latest_balance: snapshot.remaining_quota,
      total_quota: snapshot.total_quota,
      total_consumed: snapshot.used_quota,
      balance_currency: snapshot.currency,
      last_balance_check_at: snapshot.recorded_at,
    }
  }

  // 按需刷新签到数据
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
        listCheckinProviders<ProvidersResponse>().then((res) => {
          providers.value = res.providers
        })
      )
    }
    if (reloadAccounts) {
      tasks.push(
        listCheckinAccounts<AccountsResponse>().then((res) => {
          accounts.value = res.accounts
        })
      )
    }
    if (reloadRecords) {
      tasks.push(
        listCheckinRecords<CheckinRecordsResponse>({ page: 1, page_size: 100 }).then((res) => {
          records.value = res.records
        })
      )
    }
    if (reloadStats) {
      tasks.push(
        getTodayCheckinStats<TodayCheckinStats>().then((res) => {
          todayStats.value = res
        })
      )
    }
    if (reloadBuiltin) {
      tasks.push(
        listBuiltinProviders<BuiltinProvidersResponse>().then((res) => {
          builtinProviders.value = res.providers
        })
      )
    }

    await Promise.all(tasks)
  }

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
    setTimeout(() => {
      isCheckinFinished.value = false
    }, 300)
  }

  // OAuth 登录成功后刷新数据
  const handleOAuthSuccess = async () => {
    showOAuthWizard.value = false
    await loadAllData()
  }

  // 执行全部签到（逐个签到模式，实现实时进度）
  const startAndTrackCheckinJob = async (accountIds: string[]) => {
    if (accountIds.length === 0) return

    checkinLoading.value = true
    checkinResult.value = null
    wafRecoveryRunning.value = false
    wafRecoveryProviderName.value = null
    wafRecoveryMessage.value = null
    showProgressModal.value = true
    isCheckinFinished.value = false
    checkinProgress.value = {
      total: accountIds.length,
      completed: 0,
      currentAccountName: '',
    }
    checkinLogs.value = []

    await cleanupCheckinJobListeners()

    try {
      const response = await startCheckinJob<StartCheckinJobResponse>(accountIds)
      activeCheckinJobId.value = response.job_id
      applyCheckinJobSnapshot(response.snapshot)

      const handleProgressSnapshot = (snapshot: CheckinJobSnapshot) => {
        if (activeCheckinJobId.value !== snapshot.job_id) return
        applyCheckinJobSnapshot(snapshot)
      }

      const handleTerminalSnapshot = (snapshot: CheckinJobSnapshot) => {
        void finalizeCheckinJob(snapshot)
      }

      checkinJobUnlisteners.push(
        await listen<CheckinJobSnapshot>('checkin:job-progress', (event) => {
          handleProgressSnapshot(event.payload)
        })
      )
      checkinJobUnlisteners.push(
        await listen<CheckinJobSnapshot>('checkin:job-finished', (event) => {
          handleTerminalSnapshot(event.payload)
        })
      )
      checkinJobUnlisteners.push(
        await listen<CheckinJobSnapshot>('checkin:job-timeout', (event) => {
          handleTerminalSnapshot(event.payload)
        })
      )

      const latestSnapshot = await getCheckinJobStatus<CheckinJobSnapshot>(response.job_id)
      if (latestSnapshot.job_id === response.job_id) {
        if (latestSnapshot.status === 'finished' || latestSnapshot.status === 'timed_out') {
          await finalizeCheckinJob(latestSnapshot)
        } else {
          applyCheckinJobSnapshot(latestSnapshot)
        }
      }
    } catch (e: unknown) {
      checkinLoading.value = false
      showProgressModal.value = false
      activeCheckinJobId.value = null
      await cleanupCheckinJobListeners()
      alert('签到失败: ' + getErrorMessage(e, '未知错误'))
      logger.error('Checkin job failed', e)
    }
  }

  const executeCheckinAll = async () => {
    await startAndTrackCheckinJob(enabledAccounts.value.map((account) => account.id))
  }

  const executeCheckinSingle = async (accountId: string) => {
    await startAndTrackCheckinJob([accountId])
  }

  const refreshAllBalances = async () => {
    if (accounts.value.length === 0) return

    balanceRefreshing.value = true
    const enabledAccs = accounts.value.filter((a) => a.enabled)

    try {
      const results = await Promise.allSettled(
        enabledAccs.map((account) => queryCheckinBalance<BalanceSnapshot>(account.id))
      )
      for (const result of results) {
        if (result.status === 'fulfilled') {
          applyBalanceSnapshot(result.value)
        }
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

  // 刷新单个账号余额
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
      alert('刷新余额失败: ' + getErrorMessage(e, '未知错误'))
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
      alert('添加失败: ' + getErrorMessage(e, '未知错误'))
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
    return new Date(dateStr).toLocaleString('zh-CN')
  }

  const getStatusClass = (status: string) => {
    switch (status) {
      case 'success':
        return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400'
      case 'already_checked_in':
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400'
      case 'failed':
        return 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400'
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-400'
    }
  }

  const getStatusText = (status: string) => {
    switch (status) {
      case 'success':
        return '成功'
      case 'already_checked_in':
        return '已签到'
      case 'failed':
        return '失败'
      default:
        return status
    }
  }

  const buildCheckinDetail = (item: CheckinExecutionResult, fallback: string) => {
    const details: string[] = []
    if (item.reward) {
      details.push(`奖励: ${item.reward}`)
    }
    if (item.balance !== undefined && item.balance !== null) {
      details.push(`余额: ${item.balance}`)
    }
    if (item.message) {
      details.push(item.message)
    }
    return details.length > 0 ? details.join(' · ') : fallback
  }

  const getSuccessDetail = (item: CheckinDisplayResult) => buildCheckinDetail(item, '签到成功')

  const getAlreadyCheckedInDetail = (item: CheckinDisplayResult) =>
    buildCheckinDetail(item, '今日已签到')

  const getErrorHint = (code?: string): string | null => {
    if (!code) return null
    const hints: Record<string, string> = {
      cookie_expired: '建议：请更新 Cookie',
      waf_blocked: '建议：先获取 WAF Cookie，再确认网页登录与签到请求的代理/出口一致',
      cf_blocked: '建议：在有 GUI 的环境中获取 cf_clearance 后重试',
      network_error: '建议：检查网络连接',
      timeout: '建议：稍后重试',
      crypto_error: '建议：重新导入 Cookie',
      provider_error: '建议：检查提供商设置',
      account_error: '建议：检查账号设置',
      task_error: '建议：稍后重试',
      api_error: '建议：查看详细信息',
    }
    return hints[code] ?? null
  }

  const getErrorLabel = (code?: string): string | null => {
    if (!code) return null
    const labels: Record<string, string> = {
      cookie_expired: 'Cookie 过期',
      waf_blocked: 'WAF 拦截',
      cf_blocked: 'CF 挑战',
      network_error: '网络错误',
      timeout: '超时',
      crypto_error: '解密失败',
      provider_error: '提供商异常',
      account_error: '账号异常',
      task_error: '任务异常',
      api_error: 'API 错误',
    }
    return labels[code] ?? code
  }

  const getFailedDetail = (item: CheckinDisplayResult) => {
    let detail = item.message || '未知原因'
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
    isCheckinFinished,
    checkinProgress,
    checkinLogs,
    wafRecoveryRunning,
    wafRecoveryProviderName,
    wafRecoveryMessage,

    // 数据
    providers,
    accounts,
    records,
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
    getErrorHint,
    getErrorLabel,
  }
}
