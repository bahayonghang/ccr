import { CodexAuthAccountsTab } from './CodexAuthAccountsTab'
import { CodexAuthProvidersTab } from './CodexAuthProvidersTab'
import type { useCodexAuthPage } from './useCodexAuthPage'
import type { CodexTf } from '../useCodexLocale'
import { authFilterOptions } from './authFilterOptions'
import type { TranslateFunction } from '@/utils/tf'

interface AuthWorkspaceProps {
  page: ReturnType<typeof useCodexAuthPage>
  t: TranslateFunction
  tf: CodexTf
}

export function AuthWorkspace({ page, t, tf }: AuthWorkspaceProps) {
  const { statusOptions, planOptions, sortOptions } = authFilterOptions(t)
  if (page.activeManagerTab !== 'accounts') {
    return (
      <CodexAuthProvidersTab
        providerForm={page.providersApi.providerForm}
        providerFormApi={page.providersApi.providerFormApi}
        providerError={page.providersApi.providerError}
        providerSaving={page.providersApi.providerSaving}
        providerLoading={page.providersApi.providerLoading}
        providers={page.providersApi.providers}
        selectedProviderTemplate={page.providersApi.selectedProviderTemplate}
        codexTemplateDraft={page.providersApi.codexTemplateDraft}
        formatProviderUpdatedAt={page.providersApi.formatProviderUpdatedAt}
        tf={tf}
        onResetForm={page.providersApi.resetProviderForm}
        onApplyTemplate={page.providersApi.applyCodexProviderTemplate}
        onUseManualTemplate={page.providersApi.useManualProviderTemplate}
        onSaveProvider={page.providersApi.handleSaveProvider}
        onLoadProviders={page.providersApi.loadProviders}
        onUseInApiForm={page.handleUseProviderInApiForm}
        onEditProvider={page.providersApi.editProvider}
        onDeleteProvider={page.providersApi.requestDeleteProvider}
      />
    )
  }
  return (
    <CodexAuthAccountsTab
      loading={page.loading}
      accounts={page.accounts}
      currentInfo={page.currentInfo}
      canManageAuthAccounts={page.canManageAuthAccounts}
      profileGuardMessage={page.profileGuardMessage}
      authActionError={page.authActionError}
      searchQuery={page.searchQuery}
      statusFilter={page.statusFilter}
      planFilter={page.planFilter}
      sortBy={page.sortBy}
      statusOptions={statusOptions}
      planOptions={planOptions}
      sortOptions={sortOptions}
      filteredAccounts={page.filteredAccounts}
      filtersResultsCount={tf('codex.auth.filters.resultsCount', '{shown} / {total} accounts', { shown: page.filteredAccounts.length, total: page.accounts.length })}
      hasActiveFilters={Boolean(page.searchQuery.trim()) || page.statusFilter !== 'all' || page.planFilter !== 'all' || page.sortBy !== 'saved_desc'}
      quotaMap={page.quotaMap}
      quotaLoading={page.quotaLoading}
      busyName={page.busyName}
      busyAction={page.busyAction}
      actionLoading={page.actionLoading}
      formatAuthMethod={page.formatAuthMethod}
      tf={tf}
      onSearchQueryChange={page.setSearchQuery}
      onStatusFilterChange={page.setStatusFilter}
      onPlanFilterChange={page.setPlanFilter}
      onSortByChange={page.setSortBy}
      onClearFilters={page.clearFilters}
      onOpenAddAccount={page.openAddAccount}
      onSwitch={page.handleSwitch}
      onDelete={page.handleDelete}
      onRefresh={page.loadQuotas}
      onTag={page.handleComingSoon}
      onExport={page.handleComingSoon}
      onRename={page.handleRename}
    />
  )
}
