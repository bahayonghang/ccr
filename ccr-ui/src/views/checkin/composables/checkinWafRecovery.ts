import {
  openWafLogin,
  startCheckinJob,
  getCheckinJobStatus,
  validateWafCookieForAccount,
} from '@/api'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type {
  CheckinProvider,
  CheckinDisplayResponse,
  CheckinDisplayResult,
  CheckinExecutionResult,
  CheckinFlowPhase,
  CheckinJobLogEntryPayload,
  CheckinJobSnapshot,
  CheckinLogEntry,
  StartCheckinJobResponse,
  WafCookieRecoveryResult,
  WafCookieValidationResult,
} from '@/types/checkin'
import type { Ref } from 'vue'
import type { CheckinRefreshOptions } from './checkinDataState'

interface WafBlockedGroup {
  providerName: string
  provider: CheckinProvider | null
  accountIds: string[]
}

interface WafRecoveryRefs {
  providers: Ref<CheckinProvider[]>
  checkinResult: Ref<CheckinDisplayResponse | null>
  checkinLogs: Ref<CheckinLogEntry[]>
  checkinFlowPhase: Ref<CheckinFlowPhase>
  wafRecoveryRunning: Ref<boolean>
  wafRecoveryProviderName: Ref<string | null>
  wafRecoveryMessage: Ref<string | null>
}

type RefreshCheckinData = (options?: CheckinRefreshOptions) => Promise<void>

const CHECKIN_RECOVERY_MISSING_LOG = '自动重试未返回日志'

export const formatWafCookieRecoveryFailure = (result: WafCookieRecoveryResult): string => {
  if (result.missing_cookie_names.length > 0) {
    return `缺少 WAF Cookie: ${result.missing_cookie_names.join(', ')}`
  }
  return result.message || 'WAF Cookie 未获取完整'
}

export const formatWafCookieValidationFailure = (result: WafCookieValidationResult): string => {
  return result.message || 'WAF Cookie 验证失败'
}

export const mapCheckinJobLogEntry = (entry: CheckinJobLogEntryPayload): CheckinLogEntry => ({
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

const withRecoveryMeta = (
  log: CheckinLogEntry,
  recovered: boolean,
  recoveryError?: string
): CheckinLogEntry => ({
  ...log,
  wafRecoveryAttempted: true,
  wafRecovered: recovered,
  wafRecoveryError: recoveryError,
})

export const applyRecoveryFailureToLogs = (
  logs: CheckinLogEntry[],
  accountIds: string[],
  recoveryError: string
): CheckinLogEntry[] => {
  const accountIdSet = new Set(accountIds)
  return logs.map((log) => {
    if (!accountIdSet.has(log.accountId)) return log
    return withRecoveryMeta(log, false, recoveryError)
  })
}

export const mergeRetryLogsIntoProgress = (
  logs: CheckinLogEntry[],
  retrySnapshot: CheckinJobSnapshot,
  retriedAccountIds: string[]
): CheckinLogEntry[] => {
  const retriedSet = new Set(retriedAccountIds)
  const retryLogMap = new Map(
    retrySnapshot.logs
      .map(mapCheckinJobLogEntry)
      .filter((log) => retriedSet.has(log.accountId))
      .map((log) => [log.accountId, withRecoveryMeta(log, log.status !== 'failed')])
  )

  const seen = new Set<string>()
  const mergedLogs = logs.map((log) => {
    if (!retriedSet.has(log.accountId)) return log
    seen.add(log.accountId)
    const retryLog = retryLogMap.get(log.accountId)
    if (retryLog) return retryLog
    return withRecoveryMeta(log, false, CHECKIN_RECOVERY_MISSING_LOG)
  })

  for (const [accountId, retryLog] of retryLogMap.entries()) {
    if (!seen.has(accountId)) {
      mergedLogs.push(retryLog)
    }
  }

  return mergedLogs
}

const buildCheckinSummary = (results: CheckinDisplayResult[]) => {
  return results.reduce(
    (summary, item) => {
      summary.total += 1
      if (item.status === 'success') {
        summary.success += 1
      } else if (item.status === 'already_checked_in') {
        summary.already_checked_in += 1
      } else if (item.status === 'skipped') {
        summary.skipped += 1
      } else {
        summary.failed += 1
      }
      return summary
    },
    { total: 0, success: 0, already_checked_in: 0, failed: 0, skipped: 0 }
  )
}

const createDisplayResponse = (results: CheckinDisplayResult[]): CheckinDisplayResponse => ({
  results,
  summary: buildCheckinSummary(results),
})

const detectWafBlockedGroups = (
  providers: CheckinProvider[],
  result: CheckinDisplayResponse
): WafBlockedGroup[] => {
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
    provider: providers.find((candidate) => candidate.name === providerName) ?? null,
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

const isTerminalJobSnapshot = (snapshot: CheckinJobSnapshot) =>
  snapshot.status === 'finished' || snapshot.status === 'timed_out'

/**
 * 等待补救重试任务结束：复用 checkin:job-finished / checkin:job-timeout 事件，
 * 监听挂载后再用 getCheckinJobStatus 对账一次，覆盖事件先于监听到达的窗口（无轮询）。
 */
export const waitForCheckinJobResult = async (
  jobId: string,
  initialSnapshot: CheckinJobSnapshot
): Promise<CheckinJobSnapshot> => {
  if (isTerminalJobSnapshot(initialSnapshot)) {
    return initialSnapshot
  }

  return new Promise<CheckinJobSnapshot>((resolve, reject) => {
    const unlisteners: UnlistenFn[] = []
    let settled = false

    const cleanup = () => {
      void Promise.all(unlisteners.splice(0).map((unlisten) => unlisten()))
    }

    const settle = (snapshot: CheckinJobSnapshot) => {
      if (settled || snapshot.job_id !== jobId || !isTerminalJobSnapshot(snapshot)) return
      settled = true
      cleanup()
      resolve(snapshot)
    }

    const setup = async () => {
      unlisteners.push(
        await listen<CheckinJobSnapshot>('checkin:job-finished', (event) => {
          settle(event.payload)
        })
      )
      unlisteners.push(
        await listen<CheckinJobSnapshot>('checkin:job-timeout', (event) => {
          settle(event.payload)
        })
      )
      const latest = await getCheckinJobStatus<CheckinJobSnapshot>(jobId)
      settle(latest)
    }

    setup().catch((error: unknown) => {
      if (settled) return
      settled = true
      cleanup()
      reject(error instanceof Error ? error : new Error(String(error)))
    })
  })
}

const retryAccountsAfterWaf = async (accountIds: string[]): Promise<CheckinJobSnapshot> => {
  const response = await startCheckinJob<StartCheckinJobResponse>(accountIds)
  return waitForCheckinJobResult(response.job_id, response.snapshot)
}

export const createCheckinWafRecovery = (
  refs: WafRecoveryRefs,
  refreshCheckinData: RefreshCheckinData,
  getErrorMessage: (error: unknown, fallback: string) => string,
  getProviderLoginUrl: (provider: CheckinProvider) => string
) => {
  const runWafRecovery = async (
    initialResult: CheckinDisplayResponse
  ): Promise<CheckinDisplayResponse> => {
    const groups = detectWafBlockedGroups(refs.providers.value, initialResult)
    if (groups.length === 0 || refs.wafRecoveryRunning.value) {
      return initialResult
    }

    refs.wafRecoveryRunning.value = true
    refs.checkinFlowPhase.value = 'recovering'
    let mergedResult = initialResult

    try {
      for (const [index, group] of groups.entries()) {
        refs.wafRecoveryProviderName.value = group.providerName

        if (!group.provider) {
          const recoveryError = '未找到对应的提供商配置，无法自动补救'
          mergedResult = markWafRecoveryFailure(mergedResult, group.accountIds, recoveryError)
          refs.checkinResult.value = mergedResult
          refs.checkinLogs.value = applyRecoveryFailureToLogs(
            refs.checkinLogs.value,
            group.accountIds,
            recoveryError
          )
          continue
        }

        refs.wafRecoveryMessage.value = `正在为 ${group.providerName} 获取 WAF Cookie（${index + 1}/${groups.length}）`

        let recoveryResult: WafCookieRecoveryResult
        try {
          recoveryResult = await openWafLogin<WafCookieRecoveryResult>(
            getProviderLoginUrl(group.provider),
            group.provider.id
          )
        } catch (error: unknown) {
          const recoveryError = `自动获取 WAF Cookie 失败：${getErrorMessage(error, '未知错误')}`
          mergedResult = markWafRecoveryFailure(mergedResult, group.accountIds, recoveryError)
          refs.checkinResult.value = mergedResult
          refs.checkinLogs.value = applyRecoveryFailureToLogs(
            refs.checkinLogs.value,
            group.accountIds,
            recoveryError
          )
          continue
        }

        if (!recoveryResult.persisted) {
          const recoveryError = `自动获取 WAF Cookie 失败：${formatWafCookieRecoveryFailure(recoveryResult)}`
          mergedResult = markWafRecoveryFailure(mergedResult, group.accountIds, recoveryError)
          refs.checkinResult.value = mergedResult
          refs.checkinLogs.value = applyRecoveryFailureToLogs(
            refs.checkinLogs.value,
            group.accountIds,
            recoveryError
          )
          continue
        }

        refs.wafRecoveryMessage.value = `已获取 ${group.providerName} 的 WAF Cookie，正在验证`

        try {
          const validation = await validateWafCookieForAccount<WafCookieValidationResult>(
            group.accountIds[0]
          )
          if (!validation.success) {
            const recoveryError = `WAF Cookie 已获取但验证失败：${formatWafCookieValidationFailure(validation)}`
            mergedResult = markWafRecoveryFailure(mergedResult, group.accountIds, recoveryError)
            refs.checkinResult.value = mergedResult
            refs.checkinLogs.value = applyRecoveryFailureToLogs(
              refs.checkinLogs.value,
              group.accountIds,
              recoveryError
            )
            continue
          }
        } catch (error: unknown) {
          const recoveryError = `WAF Cookie 已获取但验证失败：${getErrorMessage(error, '未知错误')}`
          mergedResult = markWafRecoveryFailure(mergedResult, group.accountIds, recoveryError)
          refs.checkinResult.value = mergedResult
          refs.checkinLogs.value = applyRecoveryFailureToLogs(
            refs.checkinLogs.value,
            group.accountIds,
            recoveryError
          )
          continue
        }

        refs.wafRecoveryMessage.value = `已验证 ${group.providerName} 的 WAF Cookie，正在重试 ${group.accountIds.length} 个账号`

        try {
          const retrySnapshot = await retryAccountsAfterWaf(group.accountIds)
          mergedResult = mergeRetryResults(mergedResult, retrySnapshot.results, group.accountIds)
          refs.checkinResult.value = mergedResult
          refs.checkinLogs.value = mergeRetryLogsIntoProgress(
            refs.checkinLogs.value,
            retrySnapshot,
            group.accountIds
          )
          await refreshCheckinData({
            reloadAccounts: true,
            reloadRecords: true,
            reloadStats: true,
          })
        } catch (error: unknown) {
          const recoveryError = `自动重试失败：${getErrorMessage(error, '未知错误')}`
          mergedResult = markWafRecoveryFailure(mergedResult, group.accountIds, recoveryError)
          refs.checkinResult.value = mergedResult
          refs.checkinLogs.value = applyRecoveryFailureToLogs(
            refs.checkinLogs.value,
            group.accountIds,
            recoveryError
          )
        }
      }
    } finally {
      refs.wafRecoveryRunning.value = false
      refs.wafRecoveryProviderName.value = null
      refs.wafRecoveryMessage.value = null
    }

    return mergedResult
  }

  return {
    runWafRecovery,
  }
}
