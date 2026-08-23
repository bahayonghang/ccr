import { memo, useCallback, type ChangeEvent } from 'react'
import { useForm } from 'react-hook-form'
import type { CodexAccountQuota, CodexAuthAccountItem, CodexAuthCurrentInfo } from '@/types'
import { EmptyState, SIcon } from '@/ui'
import type { AccountPlanFilter, AccountSort, AccountStatusFilter } from '../codexAuthAccounts'
import { ghostBtnClass, panelCardClass } from '../ui-classes'
import { CodexAccountCard } from './CodexAccountCard'
import { useAppT } from '@/i18n'
import type { CodexTf } from '../useCodexLocale'

interface FilterOption<T extends string> {
  value: T
  label: string
}

interface CodexAuthAccountsTabProps {
  loading: boolean
  accounts: CodexAuthAccountItem[]
  currentInfo: CodexAuthCurrentInfo | null
  canManageAuthAccounts: boolean
  profileGuardMessage: string
  authActionError: string | null
  searchQuery: string
  statusFilter: AccountStatusFilter
  planFilter: AccountPlanFilter
  sortBy: AccountSort
  statusOptions: FilterOption<AccountStatusFilter>[]
  planOptions: FilterOption<AccountPlanFilter>[]
  sortOptions: FilterOption<AccountSort>[]
  filteredAccounts: CodexAuthAccountItem[]
  filtersResultsCount: string
  hasActiveFilters: boolean
  quotaMap: Map<string, CodexAccountQuota>
  quotaLoading: boolean
  busyName: string | null
  busyAction: 'switch' | 'delete' | null
  actionLoading: boolean
  formatAuthMethod: (method: string) => string
  tf: CodexTf
  onSearchQueryChange: (value: string) => void
  onStatusFilterChange: (value: AccountStatusFilter) => void
  onPlanFilterChange: (value: AccountPlanFilter) => void
  onSortByChange: (value: AccountSort) => void
  onClearFilters: () => void
  onOpenAddAccount: () => void
  onSwitch: (name: string) => void
  onDelete: (name: string) => void
  onRefresh: (name: string) => void
  onTag: (name: string) => void
  onExport: (name: string) => void
  onRename: (name: string) => void
}

const StatusPill = memo(function StatusPill({
  option,
  active,
  onSelect,
}: {
  option: FilterOption<AccountStatusFilter>
  active: boolean
  onSelect: (value: AccountStatusFilter) => void
}) {
  const handleClick = useCallback(() => onSelect(option.value), [onSelect, option.value])
  return (
    <button
      type="button"
      className={active ? 'codex-auth-view__filter-pill codex-auth-view__filter-pill--active' : 'codex-auth-view__filter-pill'}
      onClick={handleClick}
    >
      {option.label}
    </button>
  )
})

export function CodexAuthAccountsTab({
  loading,
  accounts,
  currentInfo,
  canManageAuthAccounts,
  profileGuardMessage,
  authActionError,
  searchQuery,
  statusFilter,
  planFilter,
  sortBy,
  statusOptions,
  planOptions,
  sortOptions,
  filteredAccounts,
  filtersResultsCount,
  hasActiveFilters,
  quotaMap,
  quotaLoading,
  busyName,
  busyAction,
  actionLoading,
  formatAuthMethod,
  tf,
  onSearchQueryChange,
  onStatusFilterChange,
  onPlanFilterChange,
  onSortByChange,
  onClearFilters,
  onOpenAddAccount,
  onSwitch,
  onDelete,
  onRefresh,
  onTag,
  onExport,
  onRename,
}: CodexAuthAccountsTabProps) {
  const t = useAppT()
  const searchForm = useForm({ defaultValues: { q: searchQuery } })
  const handleSearch = searchForm.register('q', {
    onChange: (event: ChangeEvent<HTMLInputElement>) => onSearchQueryChange(event.target.value),
  })
  const handlePlan = useCallback(
    (event: ChangeEvent<HTMLSelectElement>) => onPlanFilterChange(event.target.value as AccountPlanFilter),
    [onPlanFilterChange],
  )
  const handleSort = useCallback(
    (event: ChangeEvent<HTMLSelectElement>) => onSortByChange(event.target.value as AccountSort),
    [onSortByChange],
  )

  return (
    <div className="codex-auth-accounts-tab space-y-4">
      {currentInfo ? (
        <section className={panelCardClass}>
          <div className="codex-auth-view__section-header">
            <SIcon name="Info" size="w-5 h-5" className="codex-auth-view__section-icon" />
            <h3 className="codex-auth-view__section-title">{t('codex.auth.currentSession')}</h3>
          </div>
          <div className="codex-auth-view__session-grid">
            <div className="codex-auth-view__session-field">
              <span className="codex-auth-view__field-label">{t('codex.auth.fields.accountId')}</span>
              <code className="codex-auth-view__field-code">{currentInfo.account_id}</code>
            </div>
            <div className="codex-auth-view__session-field">
              <span className="codex-auth-view__field-label">{t('codex.auth.fields.email')}</span>
              <span className="codex-auth-view__field-value">{currentInfo.email || t('codex.auth.status.notAvailable')}</span>
            </div>
            <div className="codex-auth-view__session-field">
              <span className="codex-auth-view__field-label">{tf('codex.auth.fields.authMethod', 'Auth method')}</span>
              <span className="codex-auth-view__field-value">{formatAuthMethod(currentInfo.auth_method || '')}</span>
            </div>
          </div>
        </section>
      ) : null}

      <section className={panelCardClass}>
        <div className="codex-auth-view__guard">
          <div className={canManageAuthAccounts ? 'codex-auth-view__guard-icon-shell bg-accent-success/10 text-accent-success' : 'codex-auth-view__guard-icon-shell bg-accent-warning/10 text-accent-warning'}>
            <SIcon name="AlertTriangle" size="w-5 h-5" />
          </div>
          <div className="codex-auth-view__guard-body">
            <p className="codex-auth-view__guard-title">{t('codex.auth.profileGuard.title')}</p>
            <p className="codex-auth-view__guard-message">{profileGuardMessage}</p>
            {authActionError ? <p className="codex-auth-view__guard-error">{authActionError}</p> : null}
          </div>
        </div>
      </section>

      {!loading && accounts.length > 0 ? (
        <section className={`${panelCardClass} codex-auth-view__filters-card`}>
          <div className="codex-auth-view__filters-grid">
            <label className="codex-auth-view__search-box">
              <SIcon name="Search" size="w-4 h-4" />
              <input type="text" placeholder={t('codex.auth.filters.searchPlaceholder')} {...handleSearch} />
            </label>
            <div className="codex-auth-view__filter-group">
              <p className="codex-auth-view__filter-label">{t('codex.auth.filters.statusLabel')}</p>
              <div className="codex-auth-view__filter-row">
                {statusOptions.map((option) => (
                  <StatusPill key={option.value} option={option} active={statusFilter === option.value} onSelect={onStatusFilterChange} />
                ))}
              </div>
            </div>
            <label className="codex-auth-view__filter-group">
              <span className="codex-auth-view__filter-label">{t('codex.auth.filters.planLabel')}</span>
              <select className="codex-auth-view__filter-select" value={planFilter} onChange={handlePlan}>
                {planOptions.map((option) => (
                  <option key={option.value} value={option.value}>{option.label}</option>
                ))}
              </select>
            </label>
            <label className="codex-auth-view__filter-group">
              <span className="codex-auth-view__filter-label">{t('codex.auth.filters.sortLabel')}</span>
              <select className="codex-auth-view__filter-select" value={sortBy} onChange={handleSort}>
                {sortOptions.map((option) => (
                  <option key={option.value} value={option.value}>{option.label}</option>
                ))}
              </select>
            </label>
          </div>
          <div className="codex-auth-view__filters-footer">
            <p className="codex-auth-view__filters-summary">{filtersResultsCount}</p>
            {hasActiveFilters ? (
              <button type="button" className={ghostBtnClass} onClick={onClearFilters}>{t('common.clearFilters')}</button>
            ) : null}
          </div>
        </section>
      ) : null}

      {loading ? (
        <div className="flex justify-center py-20">
          <div className="h-12 w-12 animate-spin rounded-full border-4 border-transparent border-t-accent-primary" />
        </div>
      ) : accounts.length === 0 ? (
        <EmptyState
          icon="KeyRound"
          title={t('codex.auth.emptyState')}
          description={tf('codex.auth.emptyStateHintV2', 'Add a new account through OAuth, API key, token JSON, or import the local runtime snapshot.')}
          actionText={tf('codex.auth.actions.addAccount', 'Add account')}
          actionIcon="Plus"
          onAction={onOpenAddAccount}
        />
      ) : filteredAccounts.length === 0 ? (
        <EmptyState
          icon="Search"
          title={t('codex.auth.filters.noResultsTitle')}
          description={t('codex.auth.filters.noResultsHint')}
          actionText={t('common.clearFilters')}
          onAction={onClearFilters}
        />
      ) : (
        <div className="grid grid-cols-1 gap-4 xl:grid-cols-2">
          {filteredAccounts.map((account) => (
            <CodexAccountCard
              key={account.name}
              account={account}
              quota={quotaMap.get(account.name) ?? null}
              quotaLoading={quotaLoading}
              isCurrent={account.is_current}
              busyAction={busyName === account.name ? busyAction : null}
              disabled={actionLoading}
              onSwitch={onSwitch}
              onDelete={onDelete}
              onRefresh={onRefresh}
              onTag={onTag}
              onExport={onExport}
              onRename={onRename}
            />
          ))}
        </div>
      )}
    </div>
  )
}
