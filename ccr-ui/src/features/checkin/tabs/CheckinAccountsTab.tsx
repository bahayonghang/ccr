import { useCallback, useEffect, useMemo, useRef } from 'react'
import { useForm } from 'react-hook-form'
import { deleteCheckinAccount as apiDeleteAccount } from '@/api'
import { getErrorMessage } from '@/types/api'
import type { AccountInfo, BuiltinProvider, CheckinProvider } from '@/types/checkin'
import { SIcon } from '@/ui'
import { AccountFormModal, type AccountFormModalHandle } from '../components/AccountFormModal'
import { AccountsTable } from '../components/AccountsTable'
import { checkinNotify } from '../lib/checkinNotify'
import { useCheckinT } from '../hooks/useCheckinT'
import '../styles/accounts.css'

interface FilterValues {
  search: string
  provider: string
}

interface CheckinAccountsTabProps {
  providers: CheckinProvider[]
  accounts: AccountInfo[]
  builtinProviders: BuiltinProvider[]
  checkinLoading: boolean
  pendingEditAccountId?: string | null
  onRefresh?: () => void
  onCheckin?: (accountId: string) => void
  onRefreshBalance?: (accountId: string) => void
  onNavigate?: (accountId: string) => void
  onShowOauthWizard?: () => void
  onPendingEditConsumed?: () => void
}

export function CheckinAccountsTab({
  providers,
  accounts,
  builtinProviders,
  checkinLoading,
  pendingEditAccountId,
  onRefresh,
  onCheckin,
  onRefreshBalance,
  onNavigate,
  onShowOauthWizard,
  onPendingEditConsumed,
}: CheckinAccountsTabProps) {
  const t = useCheckinT()
  const formRef = useRef<AccountFormModalHandle>(null)
  const { register, watch } = useForm<FilterValues>({
    defaultValues: { search: '', provider: 'all' },
  })
  const searchQuery = watch('search')
  const providerFilter = watch('provider')
  const oauthCount = builtinProviders.filter((provider) => provider.oauth_config).length

  const filteredAccounts = useMemo(() => {
    let result = accounts
    if (searchQuery) {
      const query = searchQuery.toLowerCase()
      result = result.filter(
        (account) =>
          account.name.toLowerCase().includes(query) ||
          (account.provider_name && account.provider_name.toLowerCase().includes(query)),
      )
    }
    if (providerFilter !== 'all') {
      result = result.filter((account) => account.provider_id === providerFilter)
    }
    return result
  }, [accounts, providerFilter, searchQuery])

  const openCreate = useCallback(() => {
    void formRef.current?.open()
  }, [])

  const openAccountEditor = useCallback((account: AccountInfo) => {
    void formRef.current?.open(account)
  }, [])

  const deleteAccount = useCallback(
    async (id: string) => {
      const confirmed = await checkinNotify.confirm({
        title: t('checkin.accounts.deleteAccount'),
        message: t('checkin.accounts.deleteConfirm'),
        confirmText: t('checkin.accounts.delete'),
        cancelText: t('common.cancel'),
        type: 'danger',
        surface: 'solid',
      })
      if (!confirmed) return
      try {
        await apiDeleteAccount(id)
        onRefresh?.()
      } catch (error: unknown) {
        checkinNotify.error(
          t('checkin.accounts.errors.deleteFailed', {
            error: getErrorMessage(error, t('checkin.errors.unknown')),
          }),
        )
      }
    },
    [onRefresh, t],
  )

  const handleCheckin = useCallback(
    (accountId: string) => {
      onCheckin?.(accountId)
    },
    [onCheckin],
  )
  const handleRefreshBalance = useCallback(
    (accountId: string) => {
      onRefreshBalance?.(accountId)
    },
    [onRefreshBalance],
  )
  const handleNavigate = useCallback(
    (accountId: string) => {
      onNavigate?.(accountId)
    },
    [onNavigate],
  )

  useEffect(() => {
    if (!pendingEditAccountId) return
    const account = accounts.find((item) => item.id === pendingEditAccountId)
    onPendingEditConsumed?.()
    if (!account) return
    void formRef.current?.open(account, { focusSession: true })
  }, [accounts, onPendingEditConsumed, pendingEditAccountId])

  const oauthDisabled = oauthCount === 0
  const oauthTitle = oauthDisabled
    ? t('checkin.actions.oauthLoginUnavailable')
    : t('checkin.actions.oauthLoginTitle')

  return (
    <div className="checkin-accounts-tab">
      <div className="checkin-accounts-tab__panel checkin-surface-card">
        <div className="checkin-accounts-tab__toolbar">
          <h2 className="checkin-accounts-tab__title">{t('checkin.accounts.title')}</h2>
          <div className="checkin-accounts-tab__filters">
            <div className="checkin-accounts-tab__search">
              <input
                type="text"
                placeholder={t('checkin.accounts.searchPlaceholder')}
                className="checkin-accounts-tab__input checkin-accounts-tab__input--search"
                {...register('search')}
              />
              <svg
                className="checkin-accounts-tab__search-icon"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                />
              </svg>
            </div>
            <select
              className="checkin-accounts-tab__input checkin-accounts-tab__select"
              {...register('provider')}
            >
              <option value="all">{t('checkin.accounts.allProviders')}</option>
              {providers.map((provider) => (
                <option key={provider.id} value={provider.id}>
                  {provider.name}
                </option>
              ))}
            </select>
          </div>
          <div className="checkin-accounts-tab__actions">
            <button
              type="button"
              disabled={providers.length === 0}
              className="checkin-accounts-tab__action-button checkin-accounts-tab__action-button--primary"
              onClick={openCreate}
            >
              <span>{t('checkin.accounts.addAccount')}</span>
            </button>
            <button
              type="button"
              disabled={oauthDisabled}
              className="checkin-accounts-tab__action-button checkin-accounts-tab__action-button--secondary"
              title={oauthTitle}
              onClick={onShowOauthWizard}
            >
              <SIcon name="Shield" size="w-5 h-5" />
              <span>{t('checkin.actions.oauthLogin')}</span>
            </button>
          </div>
        </div>
      </div>
      {accounts.length === 0 ? (
        <div className="checkin-accounts-tab__empty checkin-surface-card">
          {providers.length === 0
            ? t('checkin.accounts.emptyNoProviders')
            : t('checkin.accounts.emptyNoAccounts')}
        </div>
      ) : (
        <AccountsTable
          accounts={filteredAccounts}
          providers={providers}
          checkinLoading={checkinLoading}
          onNavigate={handleNavigate}
          onCheckin={handleCheckin}
          onRefreshBalance={handleRefreshBalance}
          onEdit={openAccountEditor}
          onDelete={deleteAccount}
        />
      )}
      <AccountFormModal
        ref={formRef}
        providers={providers}
        builtinProviders={builtinProviders}
        onRefresh={onRefresh}
      />
    </div>
  )
}
