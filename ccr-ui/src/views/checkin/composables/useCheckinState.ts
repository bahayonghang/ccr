import { ref, computed, onMounted, nextTick } from 'vue'
import {
  Building2,
  Users,
  FileText,
  Package,
} from 'lucide-vue-next'
import {
  listCheckinProviders,
  listCheckinAccounts,
  listCheckinRecords,
  getTodayCheckinStats,
  listBuiltinProviders,
  addBuiltinProvider as apiAddBuiltinProvider,
  queryCheckinBalance,
  checkinAccount,
} from '@/api'
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
  BuiltinProvider,
  CheckinLogEntry,
  ProvidersResponse,
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

  // ═══════════════════════════════════════════════════════════
  // 数据
  // ═══════════════════════════════════════════════════════════
  const providers = ref<CheckinProvider[]>([])
  const accounts = ref<AccountInfo[]>([])
  const records = ref<CheckinRecordInfo[]>([])
  const todayStats = ref<TodayCheckinStats | null>(null)
  const checkinResult = ref<CheckinResponse | null>(null)
  const builtinProviders = ref<BuiltinProvider[]>([])

  // ═══════════════════════════════════════════════════════════
  // 计算属性
  // ═══════════════════════════════════════════════════════════

  // 过滤出尚未添加的内置提供商
  const availableBuiltinProviders = computed(() => {
    const addedNames = new Set(providers.value.map(p => p.name))
    return builtinProviders.value.filter(bp => !addedNames.has(bp.name))
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
    return accounts.value.filter(a => a.enabled)
  })

  // 签到结果分类
  const failedCheckinResults = computed(() => {
    if (!checkinResult.value) return []
    return checkinResult.value.results.filter(r => r.status === 'failed')
  })

  const successCheckinResults = computed(() => {
    if (!checkinResult.value) return []
    return checkinResult.value.results.filter(r => r.status === 'success')
  })

  const alreadyCheckedInResults = computed(() => {
    if (!checkinResult.value) return []
    return checkinResult.value.results.filter(r => r.status === 'already_checked_in')
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
      const allFailed = results.every(r => r.status === 'rejected')
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
    const index = accounts.value.findIndex(a => a.id === snapshot.account_id)
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
      tasks.push(listCheckinProviders<ProvidersResponse>().then((res) => {
        providers.value = res.providers
      }))
    }
    if (reloadAccounts) {
      tasks.push(listCheckinAccounts<AccountsResponse>().then((res) => {
        accounts.value = res.accounts
      }))
    }
    if (reloadRecords) {
      tasks.push(listCheckinRecords<CheckinRecordsResponse>({ page: 1, page_size: 100 }).then((res) => {
        records.value = res.records
      }))
    }
    if (reloadStats) {
      tasks.push(getTodayCheckinStats<TodayCheckinStats>().then((res) => {
        todayStats.value = res
      }))
    }
    if (reloadBuiltin) {
      tasks.push(listBuiltinProviders<BuiltinProvidersResponse>().then((res) => {
        builtinProviders.value = res.providers
      }))
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
  const executeCheckinAll = async () => {
    const accountsToCheckin = enabledAccounts.value
    if (accountsToCheckin.length === 0) return

    checkinLoading.value = true
    checkinResult.value = null
    showProgressModal.value = true
    isCheckinFinished.value = false

    // 初始化进度
    checkinProgress.value = {
      total: accountsToCheckin.length,
      completed: 0,
      currentAccountName: ''
    }

    // 初始化日志
    checkinLogs.value = accountsToCheckin.map(acc => ({
      accountId: acc.id,
      accountName: acc.name,
      providerName: acc.provider_name || '未知',
      status: 'pending' as const,
      timestamp: new Date()
    }))

    // 收集结果用于最终汇总
    const results: CheckinExecutionResult[] = []
    let successCount = 0
    let alreadyCheckedInCount = 0
    let failedCount = 0

    try {
      for (let i = 0; i < accountsToCheckin.length; i++) {
        const account = accountsToCheckin[i]

        // 更新当前进度
        checkinProgress.value.currentAccountName = account.name

        // 更新日志状态为处理中
        const logIndex = checkinLogs.value.findIndex(l => l.accountId === account.id)
        if (logIndex >= 0) {
          checkinLogs.value[logIndex].status = 'processing'
          checkinLogs.value[logIndex].timestamp = new Date()
        }

        try {
          const result = await checkinAccount<CheckinExecutionResult>(account.id)
          results.push(result)

          // 更新日志
          // 注意：后端使用 #[serde(rename_all = "snake_case")] 序列化枚举
          // 因此 status 值为 'success' / 'already_checked_in' / 'failed'（小写 snake_case）
          if (logIndex >= 0) {
            if (result.status === 'success') {
              checkinLogs.value[logIndex].status = 'success'
              checkinLogs.value[logIndex].message = result.reward ? `签到成功，获得 ${result.reward}` : '签到成功'
              successCount++
            } else if (result.status === 'already_checked_in') {
              checkinLogs.value[logIndex].status = 'already_checked_in'
              checkinLogs.value[logIndex].message = '今日已签到'
              alreadyCheckedInCount++
            } else {
              checkinLogs.value[logIndex].status = 'failed'
              checkinLogs.value[logIndex].message = result.message || '签到失败'
              failedCount++
            }
            checkinLogs.value[logIndex].balance = result.balance
            checkinLogs.value[logIndex].reward = result.reward
          }
        } catch (e: unknown) {
          const message = getErrorMessage(e, '请求失败')
          // 单个账号签到失败
          if (logIndex >= 0) {
            checkinLogs.value[logIndex].status = 'failed'
            checkinLogs.value[logIndex].message = message
          }
          failedCount++
          results.push({
            account_id: account.id,
            account_name: account.name,
            provider_name: account.provider_name || '未知',
            status: 'failed',
            message,
          })
        }

        // 更新完成进度
        checkinProgress.value.completed = i + 1
      }

      // 构建签到结果
      checkinResult.value = {
        results,
        summary: {
          total: accountsToCheckin.length,
          success: successCount,
          already_checked_in: alreadyCheckedInCount,
          failed: failedCount
        }
      }

      // 标记签到完成
      isCheckinFinished.value = true

      await refreshCheckinData({
        reloadAccounts: true,
        reloadRecords: true,
        reloadStats: true,
      })

      // 如果有失败的签到，自动滚动到结果区域确保用户能看到详情
      if (failedCount > 0) {
        await nextTick()
        checkinResultRef.value?.scrollIntoView({ behavior: 'smooth', block: 'start' })
      }
    } catch (e: unknown) {
      showProgressModal.value = false
      alert('签到失败: ' + getErrorMessage(e, '未知错误'))
      logger.error('Checkin failed', e)
    } finally {
      checkinLoading.value = false
    }
  }

  // 单账号签到
  const executeCheckinSingle = async (accountId: string) => {
    try {
      await checkinAccount(accountId)
      await refreshCheckinData({
        reloadAccounts: true,
        reloadRecords: true,
        reloadStats: true,
      })
    } catch (e: unknown) {
      alert('签到失败: ' + getErrorMessage(e, '未知错误'))
      logger.error('Single checkin failed', e)
    }
  }

  // ═══════════════════════════════════════════════════════════
  // 余额操作
  // ═══════════════════════════════════════════════════════════

  // 批量刷新所有账号余额
  const refreshAllBalances = async () => {
    if (accounts.value.length === 0) return

    balanceRefreshing.value = true
    const enabledAccs = accounts.value.filter(a => a.enabled)

    try {
      const results = await Promise.allSettled(
        enabledAccs.map(account => queryCheckinBalance<BalanceSnapshot>(account.id))
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
    return providers.value.find(p => p.id === providerId)?.name || providerId
  }

  const getAccountName = (accountId: string) => {
    return accounts.value.find(a => a.id === accountId)?.name || accountId
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

  const getSuccessDetail = (item: CheckinExecutionResult) => buildCheckinDetail(item, '签到成功')

  const getAlreadyCheckedInDetail = (item: CheckinExecutionResult) =>
    buildCheckinDetail(item, '今日已签到')

  const getFailedDetail = (item: CheckinExecutionResult) => item.message || '未知原因'

  // ═══════════════════════════════════════════════════════════
  // 生命周期
  // ═══════════════════════════════════════════════════════════

  onMounted(() => {
    loadAllData()
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
  }
}
