import type { CodexAccountQuota, CodexAuthAccountItem, CodexProfileAuthMode, LoginState } from '@/types'

export type AccountStatusFilter = 'all' | 'current' | 'virtual' | 'attention'
export type AccountPlanFilter = 'all' | 'plus' | 'pro' | 'team' | 'unknown'
export type AccountSort = 'saved_desc' | 'used_desc' | 'name_asc'
export type ImportPayloadNamingState = 'empty' | 'single' | 'multiple' | 'bundle' | 'invalid'

export const ACCOUNT_NAME_PATTERN = /^[A-Za-z0-9_-]+$/
export const ACCOUNT_NAME_MAX_LENGTH = 32

export const usesOpenAiAuthMode = (authMode?: CodexProfileAuthMode | null) => {
  return authMode === 'openai_chatgpt' || authMode === 'openai_api_key'
}

export const normalizeAccountNameInput = (value: string) => {
  const trimmed = value.trim()
  return trimmed.length > 0 ? trimmed : null
}

export type AccountNameValidationMessage = 'reserved' | 'length' | 'charset'

export const getAccountNameValidationMessage = (
  value: string | null,
): AccountNameValidationMessage | null => {
  if (!value) return null
  if (value.toLowerCase() === 'default') return 'reserved'
  if (value.length > ACCOUNT_NAME_MAX_LENGTH) return 'length'
  if (!ACCOUNT_NAME_PATTERN.test(value)) return 'charset'
  return null
}

export const canSubmitAccountRename = (oldName: string, rawNewName: string) => {
  const next = rawNewName.trim()
  return Boolean(next) && next !== oldName && getAccountNameValidationMessage(next) == null
}

export const detectImportPayloadNamingState = (rawPayload: string): ImportPayloadNamingState => {
  const raw = rawPayload.trim()
  if (!raw) return 'empty'

  try {
    const parsed = JSON.parse(raw) as unknown
    if (Array.isArray(parsed)) {
      return parsed.length === 1 ? 'single' : 'multiple'
    }
    if (parsed && typeof parsed === 'object') {
      return Object.prototype.hasOwnProperty.call(parsed, 'accounts') ? 'bundle' : 'single'
    }
    return 'invalid'
  } catch {
    return 'invalid'
  }
}

export const canCustomizeAccountName = (
  addMethod: 'oauth' | 'token' | 'api' | 'local',
  namingState: ImportPayloadNamingState,
) => addMethod !== 'token' || (namingState !== 'multiple' && namingState !== 'bundle')

export const resolveAccountPlanType = (
  account: CodexAuthAccountItem,
  quotaMap: ReadonlyMap<string, CodexAccountQuota>,
): AccountPlanFilter => {
  const planType = quotaMap.get(account.name)?.quota?.plan_type?.trim().toLowerCase()
  if (planType === 'plus' || planType === 'pro' || planType === 'team') {
    return planType
  }
  return 'unknown'
}

export const isAttentionAccount = (
  account: CodexAuthAccountItem,
  quotaMap: ReadonlyMap<string, CodexAccountQuota>,
) => Boolean(quotaMap.get(account.name)?.error)

export const compareDateDesc = (left?: string | null, right?: string | null) => {
  const leftTime =
    left && !Number.isNaN(Date.parse(left)) ? Date.parse(left) : Number.NEGATIVE_INFINITY
  const rightTime =
    right && !Number.isNaN(Date.parse(right)) ? Date.parse(right) : Number.NEGATIVE_INFINITY
  return rightTime - leftTime
}

export type FilterCodexAccountsInput = {
  accounts: CodexAuthAccountItem[]
  quotaMap: ReadonlyMap<string, CodexAccountQuota>
  searchQuery: string
  statusFilter: AccountStatusFilter
  planFilter: AccountPlanFilter
  sortBy: AccountSort
}

export const filterAndSortCodexAccounts = ({
  accounts,
  quotaMap,
  searchQuery,
  statusFilter,
  planFilter,
  sortBy,
}: FilterCodexAccountsInput) => {
  const query = searchQuery.trim().toLowerCase()
  const items = accounts.filter((account) => {
    if (query) {
      const haystack = [
        account.name,
        account.email,
        account.description,
        account.api_provider_name,
        account.api_base_url,
      ]
        .filter(Boolean)
        .join(' ')
        .toLowerCase()
      if (!haystack.includes(query)) {
        return false
      }
    }

    if (statusFilter === 'current' && !account.is_current) return false
    if (statusFilter === 'virtual' && !account.is_virtual) return false
    if (statusFilter === 'attention' && !isAttentionAccount(account, quotaMap)) return false

    if (planFilter !== 'all' && resolveAccountPlanType(account, quotaMap) !== planFilter) {
      return false
    }

    return true
  })

  return items.sort((left, right) => {
    switch (sortBy) {
      case 'used_desc':
        return compareDateDesc(left.last_used, right.last_used)
      case 'name_asc':
        return left.name.localeCompare(right.name)
      case 'saved_desc':
      default:
        return compareDateDesc(left.saved_at, right.saved_at)
    }
  })
}

export type CodexLoginStateTone = 'success' | 'warning' | 'primary' | 'danger'

export const getLoginStateTone = (state: LoginState): CodexLoginStateTone => {
  switch (state.type) {
    case 'LoggedInSaved':
      return 'success'
    case 'LoggedInUnsaved':
    case 'Unknown':
      return 'warning'
    case 'ApiKeyActive':
    case 'ProviderKeyActive':
      return 'primary'
    default:
      return 'danger'
  }
}

export const getLoginStateIcon = (state: LoginState) => {
  switch (state.type) {
    case 'LoggedInSaved':
      return 'UserCheck'
    case 'LoggedInUnsaved':
      return 'LogIn'
    case 'ApiKeyActive':
    case 'ProviderKeyActive':
      return 'KeyRound'
    case 'Unknown':
      return 'AlertTriangle'
    default:
      return 'LogOut'
  }
}

export const getLoginStateIconClass = (state: LoginState) => {
  switch (state.type) {
    case 'LoggedInSaved':
      return 'bg-emerald-500/10 text-emerald-500'
    case 'LoggedInUnsaved':
    case 'Unknown':
      return 'bg-yellow-500/10 text-yellow-500'
    case 'ApiKeyActive':
    case 'ProviderKeyActive':
      return 'bg-blue-500/10 text-blue-500'
    default:
      return 'bg-red-500/10 text-red-500'
  }
}
