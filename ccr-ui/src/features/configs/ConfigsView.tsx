import { useCallback, useMemo } from 'react'
import { useForm } from 'react-hook-form'
import { HistoryList, PageHeader, PageShell, SIcon } from '@/ui'
import { t } from './locale'
import { useConfigsPage } from './hooks/useConfigsPage'
import { ConfigFilters } from './components/ConfigFilters'
import { ConfigList } from './components/ConfigList'
import { ConfigsRuntimeBadge } from './components/ConfigsRuntimeBadge'
import { ConfigsSubnav } from './components/ConfigsSubnav'
import { ConfigsTabButton } from './components/ConfigsTabButton'
import { AddConfigModal } from './components/AddConfigModal'
import { EditConfigModal } from './components/EditConfigModal'
import { ProviderStatsModal } from './components/ProviderStatsModal'
import { QuickJumpChip } from './components/QuickJumpChip'
import { SummaryCard } from './components/SummaryCard'
import './styles/configs.css'

interface SearchForm {
  search: string
}

export function ConfigsView() {
  const page = useConfigsPage()
  const form = useForm<SearchForm>({ defaultValues: { search: page.searchQuery } })
  const onSearch = useCallback(
    (event: { target: EventTarget | null }) => {
      page.setSearchQuery((event.target as HTMLInputElement).value)
    },
    [page],
  )

  const tabs = useMemo(
    () => [
      { id: 'configs' as const, label: t('configs.tabs.configList'), icon: 'Settings' },
      { id: 'history' as const, label: t('configs.tabs.history'), icon: 'History' },
    ],
    [],
  )

  return (
    <PageShell
      className="configs-page"
      header={<PageHeader title={t('configs.title')} description={t('configs.description')} status={<ConfigsRuntimeBadge />} />}
      subnav={<ConfigsSubnav module="claude-code" />}
    >
      <div className="configs-workspace space-y-6">
        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
          {page.summary.map((item) => (
            <SummaryCard
              key={item.key}
              filterKey={item.key}
              label={item.label}
              count={item.count}
              icon={item.icon}
              active={item.key === page.currentFilter}
              activeClass={item.activeClass}
              idleClass={item.idleClass}
              onSelect={page.setCurrentFilter}
            />
          ))}
        </div>
        <div className="rounded-2xl border border-border-default/50 bg-bg-base p-4">
          <div className="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
            <div className="space-y-2">
              <p className="text-xs font-medium text-text-muted">{t('configs.description')}</p>
              <div className="flex flex-wrap items-center gap-2 text-sm text-text-secondary">
                <span className="rounded-full border border-accent-primary/20 bg-accent-primary/10 px-3 py-1 font-medium text-accent-primary">
                  {t('configs.currentConfig')}: {page.currentName}
                </span>
                <span className="rounded-full border border-border-default/50 bg-bg-elevated px-3 py-1 font-medium">
                  {page.filtered.length} / {page.configs.length} {t('configs.availableConfigs')}
                </span>
              </div>
            </div>
            <label className="relative block w-full xl:max-w-md">
              <SIcon name="Search" size="w-4 h-4" className="pointer-events-none absolute top-1/2 left-3 -translate-y-1/2 text-text-muted" />
              <input
                className="configs-search"
                type="text"
                placeholder={t('common.search')}
                defaultValue={page.searchQuery}
                {...form.register('search')}
                onInput={onSearch}
              />
            </label>
          </div>
          {page.jumps.length > 0 ? (
            <div className="mt-4 flex flex-wrap gap-2">
              {page.jumps.map((config) => (
                <QuickJumpChip key={config.name} config={config} onJump={page.handleJump} />
              ))}
            </div>
          ) : null}
        </div>
        <div className="flex gap-4 border-b border-border-default/10 pb-4">
          {tabs.map((tab) => (
            <ConfigsTabButton
              key={tab.id}
              id={tab.id}
              label={tab.label}
              icon={tab.icon}
              active={page.activeTab === tab.id}
              onSelect={page.setActiveTab}
            />
          ))}
        </div>
        {page.activeTab === 'configs' ? (
          <div className="space-y-6">
            <div className="flex flex-col justify-between gap-4 md:flex-row md:items-center">
              <ConfigFilters
                currentFilter={page.currentFilter}
                currentSort={page.currentSort}
                onUpdateFilter={page.setCurrentFilter}
                onUpdateSort={page.setCurrentSort}
                onShowProviderStats={page.openProvider}
                onAddConfig={page.openAdd}
              />
              <button type="button" className="rounded-lg border border-border-default px-3 py-2" onClick={page.refresh}>
                <SIcon name="RefreshCw" size="w-4 h-4" className={page.loading ? 'animate-spin' : ''} />
              </button>
            </div>
            <ConfigList
              configs={page.filtered}
              loading={page.loading}
              error={page.error}
              highlightedName={page.highlightedName}
              onSwitch={page.handleSwitch}
              onEdit={page.handleEdit}
            />
          </div>
        ) : (
          <HistoryList entries={page.historyEntries} loading={page.historyLoading} />
        )}
      </div>
      <EditConfigModal isOpen={page.isEditOpen} configName={page.editingName} onClose={page.closeEdit} onSaved={page.refresh} />
      <AddConfigModal isOpen={page.isAddOpen} onClose={page.closeAdd} onSaved={page.refresh} />
      <ProviderStatsModal
        visible={page.showProvider}
        providerUsage={page.providerUsage}
        loading={page.providerLoading}
        error={page.providerError}
        sortMode={page.providerSortMode}
        onClose={page.closeProvider}
        onRefresh={page.refresh}
        onUpdateSortMode={page.setProviderSortMode}
      />
    </PageShell>
  )
}
