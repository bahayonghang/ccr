import type {
  CheckinDisplayResponse,
  CheckinDisplayResult,
  CheckinExecutionResult,
  CheckinJobLogEntryPayload,
  CheckinJobSnapshot,
  CheckinLogEntry,
  WafCookieRecoveryResult,
  WafCookieValidationResult,
} from '@/types/checkin'

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
  recoveryError?: string,
): CheckinLogEntry => ({
  ...log,
  wafRecoveryAttempted: true,
  wafRecovered: recovered,
  wafRecoveryError: recoveryError,
})

export const applyRecoveryFailureToLogs = (
  logs: CheckinLogEntry[],
  accountIds: string[],
  recoveryError: string,
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
  retriedAccountIds: string[],
): CheckinLogEntry[] => {
  const retriedSet = new Set(retriedAccountIds)
  const retryLogMap = new Map(
    retrySnapshot.logs
      .map(mapCheckinJobLogEntry)
      .filter((log) => retriedSet.has(log.accountId))
      .map((log) => [log.accountId, withRecoveryMeta(log, log.status !== 'failed')]),
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
    if (!seen.has(accountId)) mergedLogs.push(retryLog)
  }

  return mergedLogs
}

export const buildCheckinSummary = (results: CheckinDisplayResult[]) =>
  results.reduce(
    (summary, item) => {
      summary.total += 1
      if (item.status === 'success') summary.success += 1
      else if (item.status === 'already_checked_in') summary.already_checked_in += 1
      else if (item.status === 'skipped') summary.skipped += 1
      else summary.failed += 1
      return summary
    },
    { total: 0, success: 0, already_checked_in: 0, failed: 0, skipped: 0 },
  )

export const createDisplayResponse = (results: CheckinDisplayResult[]): CheckinDisplayResponse => ({
  results,
  summary: buildCheckinSummary(results),
})

export const markWafRecoveryFailure = (
  result: CheckinDisplayResponse,
  accountIds: string[],
  recoveryError: string,
): CheckinDisplayResponse =>
  createDisplayResponse(
    result.results.map((item) => {
      if (!accountIds.includes(item.account_id)) return item
      return {
        ...item,
        waf_recovery_attempted: true,
        waf_recovered: false,
        waf_recovery_error: recoveryError,
      }
    }),
  )

export const mergeRetryResults = (
  result: CheckinDisplayResponse,
  retryResults: CheckinExecutionResult[],
  retriedAccountIds: string[],
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
    }),
  )
}

export const isTerminalJobSnapshot = (snapshot: CheckinJobSnapshot): boolean =>
  snapshot.status === 'finished' || snapshot.status === 'timed_out'
