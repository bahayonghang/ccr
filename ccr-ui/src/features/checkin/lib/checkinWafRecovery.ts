import {
  openWafLogin,
  startCheckinJob,
  validateWafCookieForAccount,
} from '@/api'
import type {
  CheckinProvider,
  CheckinDisplayResponse,
  CheckinJobSnapshot,
  StartCheckinJobResponse,
  WafCookieRecoveryResult,
  WafCookieValidationResult,
} from '@/types/checkin'
import type { CheckinRefreshOptions } from './checkinData'
import { waitForCheckinJobResult } from './waitForCheckinJob'
import {
  applyRecoveryFailureToLogs,
  formatWafCookieRecoveryFailure,
  formatWafCookieValidationFailure,
  markWafRecoveryFailure,
  mergeRetryLogsIntoProgress,
  mergeRetryResults,
} from './wafFormat'

export {
  applyRecoveryFailureToLogs,
  formatWafCookieRecoveryFailure,
  formatWafCookieValidationFailure,
  mapCheckinJobLogEntry,
  mergeRetryLogsIntoProgress,
} from './wafFormat'
export { waitForCheckinJobResult } from './waitForCheckinJob'

export interface WafRecoveryBox {
  providers: CheckinProvider[]
  checkinResult: CheckinDisplayResponse | null
  checkinLogs: import('@/types/checkin').CheckinLogEntry[]
  checkinFlowPhase: import('@/types/checkin').CheckinFlowPhase
  wafRecoveryRunning: boolean
  wafRecoveryProviderName: string | null
  wafRecoveryMessage: string | null
}

type RefreshCheckinData = (options?: CheckinRefreshOptions) => Promise<void>
type ErrorMessageFn = (error: unknown, fallback: string) => string
type LoginUrlFn = (provider: CheckinProvider) => string

interface WafBlockedGroup {
  providerName: string
  provider: CheckinProvider | null
  accountIds: string[]
}

interface WafRecoveryOptions {
  box: WafRecoveryBox
  refreshCheckinData: RefreshCheckinData
  getErrorMessage: ErrorMessageFn
  getProviderLoginUrl: LoginUrlFn
  notify: () => void
}

const RECOVERY_COOLDOWN_MS = 60_000

const detectWafBlockedGroups = (
  providers: CheckinProvider[],
  result: CheckinDisplayResponse,
): WafBlockedGroup[] => {
  const grouped = new Map<string, string[]>()
  for (const item of result.results) {
    if (item.status !== 'failed' || item.error_code !== 'waf_blocked') continue
    const accountIds = grouped.get(item.provider_name) ?? []
    if (!accountIds.includes(item.account_id)) accountIds.push(item.account_id)
    grouped.set(item.provider_name, accountIds)
  }

  return Array.from(grouped.entries()).map(([providerName, accountIds]) => ({
    providerName,
    provider: providers.find((candidate) => candidate.name === providerName) ?? null,
    accountIds,
  }))
}

const retryAccountsAfterWaf = async (accountIds: string[]): Promise<CheckinJobSnapshot> => {
  const response = await startCheckinJob<StartCheckinJobResponse>(accountIds)
  return waitForCheckinJobResult(response.job_id, response.snapshot)
}

export const createCheckinWafRecovery = (options: WafRecoveryOptions) => {
  const { box, refreshCheckinData, getErrorMessage, getProviderLoginUrl, notify } = options
  const recoveryAttemptAt = new Map<string, number>()

  const failGroup = (input: {
    result: CheckinDisplayResponse
    accountIds: string[]
    recoveryError: string
  }): CheckinDisplayResponse => {
    const merged = markWafRecoveryFailure(input.result, input.accountIds, input.recoveryError)
    box.checkinResult = merged
    box.checkinLogs = applyRecoveryFailureToLogs(box.checkinLogs, input.accountIds, input.recoveryError)
    notify()
    return merged
  }

  const recoverGroup = async (input: {
    group: WafBlockedGroup
    index: number
    total: number
    merged: CheckinDisplayResponse
  }): Promise<CheckinDisplayResponse> => {
    const { group } = input
    box.wafRecoveryProviderName = group.providerName
    notify()

    if (!group.provider) {
      return failGroup({
        result: input.merged,
        accountIds: group.accountIds,
        recoveryError: '未找到对应的提供商配置，无法自动补救',
      })
    }

    const providerId = group.provider.id
    const lastAttemptAt = recoveryAttemptAt.get(providerId)
    if (lastAttemptAt && Date.now() - lastAttemptAt < RECOVERY_COOLDOWN_MS) {
      return failGroup({
        result: input.merged,
        accountIds: group.accountIds,
        recoveryError: '刚刚已尝试自动获取 WAF Cookie，请稍后重试或前往“提供商”页手动获取',
      })
    }
    recoveryAttemptAt.set(providerId, Date.now())
    box.wafRecoveryMessage = `正在为 ${group.providerName} 获取 WAF Cookie（${input.index + 1}/${input.total}）`
    notify()

    let recoveryResult: WafCookieRecoveryResult
    try {
      recoveryResult = await openWafLogin<WafCookieRecoveryResult>(
        getProviderLoginUrl(group.provider),
        group.provider.id,
      )
    } catch (error: unknown) {
      return failGroup({
        result: input.merged,
        accountIds: group.accountIds,
        recoveryError: `自动获取 WAF Cookie 失败：${getErrorMessage(error, '未知错误')}`,
      })
    }

    if (!recoveryResult.persisted) {
      return failGroup({
        result: input.merged,
        accountIds: group.accountIds,
        recoveryError: `自动获取 WAF Cookie 失败：${formatWafCookieRecoveryFailure(recoveryResult)}`,
      })
    }

    box.wafRecoveryMessage = `已获取 ${group.providerName} 的 WAF Cookie，正在验证`
    notify()

    try {
      const validation = await validateWafCookieForAccount<WafCookieValidationResult>(
        group.accountIds[0],
      )
      if (!validation.success) {
        return failGroup({
          result: input.merged,
          accountIds: group.accountIds,
          recoveryError: `WAF Cookie 已获取但验证失败：${formatWafCookieValidationFailure(validation)}`,
        })
      }
    } catch (error: unknown) {
      return failGroup({
        result: input.merged,
        accountIds: group.accountIds,
        recoveryError: `WAF Cookie 已获取但验证失败：${getErrorMessage(error, '未知错误')}`,
      })
    }

    box.wafRecoveryMessage = `已验证 ${group.providerName} 的 WAF Cookie，正在重试 ${group.accountIds.length} 个账号`
    notify()

    try {
      const retrySnapshot = await retryAccountsAfterWaf(group.accountIds)
      const merged = mergeRetryResults(input.merged, retrySnapshot.results, group.accountIds)
      box.checkinResult = merged
      box.checkinLogs = mergeRetryLogsIntoProgress(box.checkinLogs, retrySnapshot, group.accountIds)
      notify()
      await refreshCheckinData({
        reloadAccounts: true,
        reloadRecords: true,
        reloadStats: true,
      })
      recoveryAttemptAt.delete(providerId)
      return merged
    } catch (error: unknown) {
      return failGroup({
        result: input.merged,
        accountIds: group.accountIds,
        recoveryError: `自动重试失败：${getErrorMessage(error, '未知错误')}`,
      })
    }
  }

  const runWafRecovery = async (
    initialResult: CheckinDisplayResponse,
  ): Promise<CheckinDisplayResponse> => {
    const groups = detectWafBlockedGroups(box.providers, initialResult)
    if (groups.length === 0 || box.wafRecoveryRunning) return initialResult

    box.wafRecoveryRunning = true
    box.checkinFlowPhase = 'recovering'
    notify()
    let mergedResult = initialResult

    try {
      for (const [index, group] of groups.entries()) {
        mergedResult = await recoverGroup({
          group,
          index,
          total: groups.length,
          merged: mergedResult,
        })
      }
    } finally {
      box.wafRecoveryRunning = false
      box.wafRecoveryProviderName = null
      box.wafRecoveryMessage = null
      notify()
    }

    return mergedResult
  }

  return { runWafRecovery }
}
