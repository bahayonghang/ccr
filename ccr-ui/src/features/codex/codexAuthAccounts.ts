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
      if (!haystack.includes(query)) return false
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
    if (sortBy === 'used_desc') return compareDateDesc(left.last_used, right.last_used)
    if (sortBy === 'name_asc') return left.name.localeCompare(right.name)
    return compareDateDesc(left.saved_at, right.saved_at)
  })
}

export type CodexLoginStateTone = 'success' | 'warning' | 'primary' | 'danger'

export const getLoginStateTone = (state: LoginState): CodexLoginStateTone => {
  if (state.type === 'LoggedInSaved') return 'success'
  if (state.type === 'LoggedInUnsaved' || state.type === 'Unknown') return 'warning'
  if (state.type === 'ApiKeyActive' || state.type === 'ProviderKeyActive') return 'primary'
  return 'danger'
}

export const getLoginStateIcon = (state: LoginState) => {
  if (state.type === 'LoggedInSaved') return 'UserCheck'
  if (state.type === 'LoggedInUnsaved') return 'LogIn'
  if (state.type === 'ApiKeyActive' || state.type === 'ProviderKeyActive') return 'KeyRound'
  if (state.type === 'Unknown') return 'AlertTriangle'
  return 'LogOut'
}

export const getLoginStateIconClass = (state: LoginState) => {
  if (state.type === 'LoggedInSaved') return 'bg-accent-success/10 text-accent-success'
  if (state.type === 'LoggedInUnsaved' || state.type === 'Unknown') {
    return 'bg-accent-warning/10 text-accent-warning'
  }
  if (state.type === 'ApiKeyActive' || state.type === 'ProviderKeyActive') {
    return 'bg-accent-primary/10 text-accent-primary'
  }
  return 'bg-accent-danger/10 text-accent-danger'
}
