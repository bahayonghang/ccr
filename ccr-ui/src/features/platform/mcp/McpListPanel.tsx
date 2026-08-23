import { memo, useCallback } from 'react'
import type { McpGroup } from '@/types/mcpManager'
import type { TranslateFunction } from '@/utils/tf'
import {
  AgentIcons,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  EmptyState,
  ListSearchHeader,
  MultiSelectFloatingBar,
  SIcon,
  cn,
} from '@/ui'
import { mcpListActionBtnClass } from './mcp-classes'
import { formatScopeList, shortenTransportLabel } from './mcp-format'

export interface McpListPanelProps {
  groups: McpGroup[]
  searchQuery: string
  selectedKeys: Set<string>
  isMultiSelectMode: boolean
  loading: boolean
  t: TranslateFunction
  onSearchQueryChange: (value: string) => void
  onSelect: (name: string) => void
  onCreate: () => void
  onImport: () => void
  onRefresh: () => void
  onToggleMultiSelect: () => void
  onBulkDelete: () => void
}

interface McpListItemProps {
  group: McpGroup
  selected: boolean
  onSelect: (name: string) => void
  t: TranslateFunction
}

const McpListItem = memo(function McpListItem({ group, selected, onSelect, t }: McpListItemProps) {
  const handleClick = useCallback(() => {
    onSelect(group.name)
  }, [group.name, onSelect])

  return (
    <button
      type="button"
      className={cn(
        'flex w-full items-center gap-2.5 rounded-xl border border-transparent px-3 py-2.5 text-left transition-colors',
        'hover:bg-bg-elevated/72',
        selected && 'border-accent-primary/15 bg-bg-elevated/85',
      )}
      onClick={handleClick}
    >
      <div className="flex h-6 w-6 shrink-0 items-center justify-center text-text-muted">
        <SIcon name={group.transportType === 'stdio' ? 'Terminal' : 'Globe'} size="w-4 h-4" />
      </div>
      <div className="flex min-w-0 flex-1 flex-col gap-0.5">
        <span className="flex items-center gap-1.5 overflow-hidden text-ellipsis whitespace-nowrap text-sm font-medium text-text-primary">
          {group.name}
          {group.hiddenCount ? (
            <span className="shrink-0 rounded-full bg-warning/10 px-1.5 py-0.5 text-[0.58rem] font-bold uppercase tracking-wide text-warning/92">
              {t('mcp.manager.list.hiddenCount', { count: group.hiddenCount })}
            </span>
          ) : null}
        </span>
        <span className="overflow-hidden text-ellipsis whitespace-nowrap text-xs text-text-muted">
          {shortenTransportLabel(group.transportLabel)}
        </span>
        <span className="overflow-hidden text-ellipsis whitespace-nowrap text-[0.62rem] font-semibold uppercase tracking-wide text-accent-primary/82">
          {formatScopeList(group.scopes ?? [], t)}
        </span>
      </div>
      <AgentIcons agents={group.platforms} compact />
    </button>
  )
})

export function McpListPanel({
  groups,
  searchQuery,
  selectedKeys,
  isMultiSelectMode,
  loading,
  t,
  onSearchQueryChange,
  onSelect,
  onCreate,
  onImport,
  onRefresh,
  onToggleMultiSelect,
  onBulkDelete,
}: McpListPanelProps) {
  const multiSelectLabel = isMultiSelectMode
    ? t('mcp.manager.list.doneSelecting')
    : t('mcp.manager.list.multiSelect')
  const emptyAction = searchQuery ? undefined : onCreate

  return (
    <div className="relative flex h-full flex-col overflow-hidden">
      <ListSearchHeader
        searchValue={searchQuery}
        onSearchValueChange={onSearchQueryChange}
        placeholder={t('mcp.searchServers')}
        label={t('mcp.searchServers')}
      >
        <button
          type="button"
          className={cn(mcpListActionBtnClass, isMultiSelectMode && 'bg-accent-primary/10 text-accent-primary')}
          aria-label={multiSelectLabel}
          title={multiSelectLabel}
          onClick={onToggleMultiSelect}
        >
          <SIcon name={isMultiSelectMode ? 'CheckCircle2' : 'LayoutGrid'} size="w-4 h-4" />
        </button>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <button
              type="button"
              className={mcpListActionBtnClass}
              aria-label={t('mcp.manager.actions.addServer')}
            >
              <SIcon name="Plus" size="w-4 h-4" />
            </button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onSelect={onCreate}>{t('mcp.manager.list.manualCreation')}</DropdownMenuItem>
            <DropdownMenuItem onSelect={onImport}>{t('mcp.manager.import.title')}</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>

        <button
          type="button"
          className={mcpListActionBtnClass}
          aria-label={t('common.refresh')}
          onClick={onRefresh}
        >
          <SIcon name="RefreshCw" size="w-4 h-4" className={cn(loading && 'animate-spin')} />
        </button>
      </ListSearchHeader>

      <div className="flex flex-1 flex-col gap-0.5 overflow-y-auto p-2">
        {groups.length === 0 && !loading ? (
          <EmptyState
            icon="Server"
            title={searchQuery ? t('mcp.manager.list.noSearchResults') : t('mcp.manager.list.empty')}
            actionText={searchQuery ? undefined : t('mcp.manager.actions.addServer')}
            actionIcon="Plus"
            onAction={emptyAction}
          />
        ) : null}

        {groups.map((group) => (
          <McpListItem
            key={group.name}
            group={group}
            selected={selectedKeys.has(group.name)}
            onSelect={onSelect}
            t={t}
          />
        ))}
      </div>

      <MultiSelectFloatingBar
        selectedCount={selectedKeys.size}
        totalCount={groups.length}
        countLabel={t('mcp.manager.list.multiSelectCount', {
          selected: selectedKeys.size,
          total: groups.length,
        })}
        deleteLabel={t('common.delete')}
        deleteAriaLabel={t('mcp.manager.list.deleteSelectedAria', { count: selectedKeys.size })}
        onDelete={onBulkDelete}
      />
    </div>
  )
}
