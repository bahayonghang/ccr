import { memo, useCallback } from 'react'
import { ListSearchHeader, PillToggleGroup } from '@/ui'
import type { CommandCollection, CommandUiInfo } from './commands-model'
import { CommandRow } from './CommandRow'
import type { useCommandsPage } from './useCommandsPage'

type Page = ReturnType<typeof useCommandsPage>

const TabButton = memo(function TabButton({ active, label, onClick }: { active: boolean; label: string; onClick: () => void }) {
  return (
    <button type="button" className={`commands-source-tabs__item${active ? ' commands-source-tabs__item--active' : ''}`} onClick={onClick}>
      {label}
    </button>
  )
})

const CategoryTab = memo(function CategoryTab({ category, active, label, onSelect }: { category: string; active: boolean; label: string; onSelect: (value: string) => void }) {
  const handleClick = useCallback(() => {
    onSelect(category)
  }, [category, onSelect])
  return (
    <button type="button" className={`commands-category-tabs__item${active ? ' commands-category-tabs__item--active' : ''}`} onClick={handleClick}>
      {label}
    </button>
  )
})

const PersistedRow = memo(function PersistedRow({
  id,
  title,
  subtitle,
  command,
  args,
  onLoad,
}: {
  id: string
  title: string
  subtitle: string
  command: string
  args: string[]
  onLoad: (command: string, args: string[]) => void
}) {
  const handleClick = useCallback(() => {
    onLoad(command, args)
  }, [args, command, onLoad])
  return (
    <button type="button" className="command-row" data-id={id} onClick={handleClick}>
      <div className="command-row__topline">
        <strong>{title}</strong>
      </div>
      <p>{subtitle}</p>
    </button>
  )
})

export const CommandsPalette = memo(function CommandsPalette({ page }: { page: Page }) {
  const clientOptions = page.CLI_CLIENTS.map((client) => ({
    value: client.id,
    label: client.executable ? client.name : `${client.name} · ${page.t('commands.clientPreview')}`,
  }))
  const categoryTabs = ['all', ...Array.from(new Set(page.commands.map((command) => command.category || 'other')))]
  const badgeLabel = useCallback((badge: 'safe' | 'danger' | 'readonly' | 'args' | 'blocked') => {
    if (badge === 'safe') return page.t('commands.badgeSafe')
    if (badge === 'danger') return page.t('commands.badgeDanger')
    if (badge === 'readonly') return page.t('commands.badgeReadOnly')
    if (badge === 'args') return page.t('commands.badgeArgs')
    return page.t('commands.badgeBlocked')
  }, [page])
  const onCatalog = useCallback(() => page.setActiveCollection('catalog'), [page])
  const onFavorites = useCallback(() => page.setActiveCollection('favorites'), [page])
  const onHistory = useCallback(() => page.setActiveCollection('history'), [page])
  const handleClearHistory = useCallback(() => {
    void page.handleClearHistory()
  }, [page])
  const selectCommand = useCallback((name: string) => {
    page.setActiveCollection('catalog')
    page.setSelectedCommand(name)
  }, [page])
  const loadPersisted = useCallback((command: string, args: string[]) => {
    page.loadPersistedCommand(command, args)
  }, [page])
  const collectionLabel = (collection: CommandCollection) => {
    if (collection === 'catalog') return page.t('commands.catalogTab')
    if (collection === 'favorites') return page.t('commands.favorites')
    return page.t('commands.history')
  }
  const categoryLabel = (category: string) => {
    if (category === 'all') return page.t('commands.categoryAll')
    if (category === 'write') return page.t('commands.categoryWrite')
    if (category === 'danger') return page.t('commands.categoryDanger')
    if (category === 'blocked' || category === 'preview') return page.t('commands.categoryBlocked')
    if (category === 'read' || category === 'diagnostic') return page.t('commands.categoryRead')
    return page.t('commands.categoryOther')
  }

  return (
    <aside className="commands-palette">
      <div className="commands-panel commands-panel--palette">
        <div className="commands-panel__header">
          <div>
            <h2 className="commands-panel__title">{page.t('commands.paletteTitle')}</h2>
            <p className="commands-panel__subtitle">{page.t('commands.paletteSubtitle')}</p>
          </div>
          {page.activeCollection === 'history' ? (
            <button type="button" className="rounded-lg border border-border-default px-3 py-1.5 text-xs" disabled={page.historyItems.length === 0 || page.runtimeUnavailable} onClick={handleClearHistory}>
              {page.t('commands.clearHistory')}
            </button>
          ) : null}
        </div>
        <PillToggleGroup className="commands-client-switcher" options={clientOptions} value={page.selectedClient} onValueChange={page.setSelectedClient} />
        <div className="commands-source-tabs">
          <TabButton active={page.activeCollection === 'catalog'} label={collectionLabel('catalog')} onClick={onCatalog} />
          <TabButton active={page.activeCollection === 'favorites'} label={collectionLabel('favorites')} onClick={onFavorites} />
          <TabButton active={page.activeCollection === 'history'} label={collectionLabel('history')} onClick={onHistory} />
        </div>
        <ListSearchHeader searchValue={page.searchQuery} onSearchValueChange={page.setSearchQuery} placeholder={page.t('commands.searchPlaceholder')} />
        {page.activeCollection === 'catalog' ? (
          <>
            <div className="commands-category-tabs">
              {categoryTabs.map((category) => (
                <CategoryTab key={category} category={category} active={page.activeCategory === category} label={categoryLabel(category)} onSelect={page.setActiveCategory} />
              ))}
            </div>
            <div className="commands-list">
              {page.filteredCommands.map((command: CommandUiInfo) => (
                <CommandRow key={command.name} command={command} active={page.selectedCommand === command.name} badgeLabel={badgeLabel} onSelect={selectCommand} />
              ))}
            </div>
          </>
        ) : null}
        {page.activeCollection === 'favorites' ? (
          <div className="commands-list">
            {page.favorites.map((favorite) => (
              <PersistedRow key={favorite.id} id={favorite.id} title={favorite.display_name || favorite.command} subtitle={`${favorite.command} ${favorite.args.join(' ')}`} command={favorite.command} args={favorite.args} onLoad={loadPersisted} />
            ))}
            {page.favorites.length === 0 ? <div className="commands-list-empty">{page.t('commands.noFavorites')}</div> : null}
          </div>
        ) : null}
        {page.activeCollection === 'history' ? (
          <div className="commands-list">
            {page.historyItems.map((item) => (
              <PersistedRow key={item.id} id={item.id} title={item.full_command || `ccr ${item.command}`} subtitle={`${item.command} ${item.args.join(' ')}`} command={item.command} args={item.args} onLoad={loadPersisted} />
            ))}
            {page.historyItems.length === 0 ? <div className="commands-list-empty">{page.t('commands.noHistory')}</div> : null}
          </div>
        ) : null}
      </div>
    </aside>
  )
})
