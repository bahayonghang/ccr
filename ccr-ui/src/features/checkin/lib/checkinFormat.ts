import type {
  CheckinDisplayResult,
  CheckinDisplayResponse,
  CheckinExecutionResult,
} from '@/types/checkin'
import type { TranslateFunction } from '@/utils/tf'

export const CHECKIN_TABS = [
  { id: 'accounts' as const, nameKey: 'checkin.tabs.accounts', icon: 'Users' },
  { id: 'providers' as const, nameKey: 'checkin.tabs.providers', icon: 'Building2' },
  { id: 'records' as const, nameKey: 'checkin.tabs.records', icon: 'FileText' },
  { id: 'import-export' as const, nameKey: 'checkin.tabs.importExport', icon: 'Package' },
]

export type CheckinTabId = (typeof CHECKIN_TABS)[number]['id']

export const getStatusText = (status: string, t: TranslateFunction): string => {
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

export const getSkipReasonText = (
  skipReason: string | undefined,
  t: TranslateFunction,
): string | null => {
  if (!skipReason) return null
  const reasons: Record<string, string> = {
    account_disabled: t('checkin.skipReasons.account_disabled'),
    provider_disabled: t('checkin.skipReasons.provider_disabled'),
    provider_unsupported: t('checkin.skipReasons.provider_unsupported'),
  }
  return reasons[skipReason] ?? skipReason
}

export const getErrorHint = (code: string | undefined, t: TranslateFunction): string | null => {
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

export const getErrorLabel = (code: string | undefined, t: TranslateFunction): string | null => {
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

export const buildCheckinDetail = (
  item: CheckinExecutionResult,
  fallback: string,
  t: TranslateFunction,
): string => {
  const details: string[] = []
  if (item.reward) details.push(t('checkin.detail.reward', { reward: item.reward }))
  if (item.balance !== undefined && item.balance !== null) {
    details.push(t('checkin.detail.balance', { balance: item.balance }))
  }
  if (item.message) details.push(item.message)
  return details.length > 0 ? details.join(' · ') : fallback
}

export const getSuccessDetail = (item: CheckinDisplayResult, t: TranslateFunction): string =>
  buildCheckinDetail(item, t('checkin.detail.checkinSuccess'), t)

export const getAlreadyCheckedInDetail = (
  item: CheckinDisplayResult,
  t: TranslateFunction,
): string => buildCheckinDetail(item, t('checkin.detail.todayAlreadyCheckedIn'), t)

export const getSkippedDetail = (item: CheckinDisplayResult, t: TranslateFunction): string => {
  const reason = getSkipReasonText(item.skip_reason, t)
  if (reason) return reason
  return item.message || t('checkin.detail.skipped')
}

export const getFailedDetail = (item: CheckinDisplayResult, t: TranslateFunction): string => {
  let detail = item.message || t('checkin.errors.unknownReason')
  const hint = getErrorHint(item.error_code, t)
  if (hint) detail = `${detail}（${hint}）`
  if (item.waf_recovery_attempted && item.waf_recovered === false && item.waf_recovery_error) {
    detail = `${detail} · ${item.waf_recovery_error}`
  }
  return detail
}

export const filterResults = (
  result: CheckinDisplayResponse | null,
  status: CheckinDisplayResult['status'],
): CheckinDisplayResult[] => {
  if (!result) return []
  return result.results.filter((item) => item.status === status)
}

export const sumAccountStats = (
  accounts: Array<{
    latest_balance?: number | null
    total_quota?: number | null
    total_consumed?: number | null
  }>,
) => {
  const result = { currentBalance: 0, totalQuota: 0, totalConsumed: 0 }
  for (const account of accounts) {
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
}

export const getAccountOriginKey = (
  account: { provider_id: string },
  providers: Array<{ id: string; base_url: string }>,
): string => {
  const provider = providers.find((item) => item.id === account.provider_id)
  if (!provider) return account.provider_id
  try {
    return new URL(provider.base_url).origin
  } catch {
    return account.provider_id
  }
}

export const getProviderLoginUrl = (provider: { base_url: string }): string =>
  `${provider.base_url.replace(/\/+$/, '')}/login`
