import { useCallback, useEffect, useMemo, useState } from 'react'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { useCommandsViewStore } from '@/features/commands/stores'
import type { PlatformConfig, SlashCommand, SlashCommandRequest } from '@/types/platform'
import { logger } from '@/utils/logger'
import { EmptyState, ListSearchHeader, PageHeader, PageShell, SIcon } from '@/ui'
import { CommandFormModal } from './CommandFormModal'
import { CommandList } from './CommandList'
import { useCommandsT } from './locale'

interface BaseSlashCommandsProps {
  config: PlatformConfig
  hideChrome?: boolean
}

export function BaseSlashCommands({ config, hideChrome = false }: BaseSlashCommandsProps) {
  const t = useCommandsT()
  const sortKey = useCommandsViewStore((s) => s.sortKey)
  const sortDir = useCommandsViewStore((s) => s.sortDir)
  const viewMode = useCommandsViewStore((s) => s.viewMode)
  const showDeprecated = useCommandsViewStore((s) => s.showDeprecated)
  const expandedFolders = useCommandsViewStore((s) => s.expandedFolders)
  const restore = useCommandsViewStore((s) => s.restore)
  const toggleFolder = useCommandsViewStore((s) => s.toggleFolder)
  const [loading, setLoading] = useState(false)
  const [commands, setCommands] = useState<SlashCommand[]>([])
  const [folders, setFolders] = useState<string[]>([])
  const selectedFolder = 'all'
  const [searchQuery, setSearchQuery] = useState('')
  const [showAddModal, setShowAddModal] = useState(false)
  const [editingCommand, setEditingCommand] = useState<SlashCommand | null>(null)

  const loadData = useCallback(async () => {
    setLoading(true)
    try {
      const result = await config.api.list()
      setCommands(result.commands)
      setFolders(result.folders)
    } catch (error) {
      logger.error('Failed to load slash commands:', error)
    } finally {
      setLoading(false)
    }
  }, [config])

  useEffect(() => {
    restore()
    void loadData()
  }, [loadData, restore])

  const availableFolders = useMemo(() => {
    const folderSet = new Set(folders)
    commands.forEach((cmd) => folderSet.add(cmd.folder))
    return Array.from(folderSet).filter(Boolean)
  }, [commands, folders])

  const filteredCommands = useMemo(() => {
    let filtered = commands
    if (!showDeprecated) filtered = filtered.filter((cmd) => !cmd.description?.toLowerCase().includes('deprecated'))
    if (selectedFolder !== 'all') filtered = filtered.filter((cmd) => cmd.folder === selectedFolder)
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase().trim()
      filtered = filtered.filter((cmd) => cmd.name.toLowerCase().includes(query) || cmd.command.toLowerCase().includes(query) || cmd.description.toLowerCase().includes(query))
    }
    return [...filtered].sort((a, b) => {
      const comparison = sortKey === 'usage' ? (a.command?.length || 0) - (b.command?.length || 0) : a.name.localeCompare(b.name)
      return sortDir === 'asc' ? comparison : -comparison
    })
  }, [commands, searchQuery, selectedFolder, showDeprecated, sortDir, sortKey])

  const groupedCommands = useMemo(() => {
    const groups = new Map<string, SlashCommand[]>()
    filteredCommands.forEach((cmd) => {
      const folder = cmd.folder || t(`${config.i18n.prefix}.folders.root`)
      groups.set(folder, [...(groups.get(folder) ?? []), cmd])
    })
    return Array.from(groups.entries()).map(([name, items]) => ({ name, commands: items }))
  }, [config.i18n.prefix, filteredCommands, t])

  const openCreate = useCallback(() => {
    setEditingCommand(null)
    setShowAddModal(true)
  }, [])
  const closeModal = useCallback(() => {
    setShowAddModal(false)
    setEditingCommand(null)
  }, [])
  const handleEdit = useCallback((command: SlashCommand) => {
    setEditingCommand({ ...command })
    setShowAddModal(true)
  }, [])
  const handleDelete = useCallback(async (name: string) => {
    const confirmed = await surfaceNotify.confirm({
      title: t('common.delete'),
      message: t(`${config.i18n.prefix}.confirmDelete`, { name }),
      confirmText: t('common.delete'),
      cancelText: t('common.cancel'),
      type: 'danger',
    })
    if (!confirmed) return
    await config.api.delete(name)
    await loadData()
  }, [config, loadData, t])
  const handleToggle = useCallback(async (name: string) => {
    await config.api.toggle(name)
    await loadData()
  }, [config, loadData])
  const handleSubmit = useCallback(async (data: SlashCommandRequest) => {
    if (editingCommand) await config.api.update(editingCommand.name, data)
    else await config.api.add(data)
    await loadData()
  }, [config, editingCommand, loadData])
  const onRefresh = useCallback(() => {
    void loadData()
  }, [loadData])
  const onDelete = useCallback((name: string) => {
    void handleDelete(name)
  }, [handleDelete])
  const onToggle = useCallback((name: string) => {
    void handleToggle(name)
  }, [handleToggle])
  const onSubmit = useCallback((data: SlashCommandRequest) => {
    void handleSubmit(data)
  }, [handleSubmit])
  const toggleFolderRow = useCallback((name: string) => {
    toggleFolder(name)
  }, [toggleFolder])

  const header = hideChrome ? undefined : (
    <PageHeader
      title={t(`${config.i18n.prefix}.pageTitle`)}
      description={t(`${config.i18n.prefix}.pageSubtitle`, { platform: config.platform.displayName })}
      status={<span>{filteredCommands.length}/{commands.length}</span>}
      actions={
        <>
          <button type="button" className="inline-flex items-center gap-2 rounded-xl border border-border-default px-4 py-2 text-sm" disabled={loading} onClick={onRefresh}>
            <SIcon name="RefreshCw" size="w-4 h-4" className={loading ? 'animate-spin' : ''} />
            {t('common.refresh')}
          </button>
          <button type="button" className="inline-flex items-center gap-2 rounded-lg bg-accent-primary px-5 py-2.5 text-sm text-[color:var(--color-accent-primary-contrast)]" onClick={openCreate}>
            <SIcon name="Plus" size="w-5 h-5" />
            {t('common.add')}
          </button>
        </>
      }
    />
  )

  return (
    <PageShell header={header}>
      <ListSearchHeader searchValue={searchQuery} onSearchValueChange={setSearchQuery} placeholder={t('common.search')} />
      {viewMode === 'tree' ? (
        <div>
          {groupedCommands.map((folder) => (
            <FolderBlock key={folder.name} name={folder.name} count={folder.commands.length} expanded={expandedFolders.includes(folder.name)} commands={folder.commands} loading={loading} onToggleFolder={toggleFolderRow} onEdit={handleEdit} onDelete={onDelete} onToggle={onToggle} />
          ))}
        </div>
      ) : (
        <CommandList commands={filteredCommands} loading={loading} onEdit={handleEdit} onDelete={onDelete} onToggle={onToggle} />
      )}
      {!loading && filteredCommands.length === 0 ? <EmptyState title={t('slashCommands.noCommands')} /> : null}
      <CommandFormModal visible={showAddModal} editingCommand={editingCommand} folders={availableFolders} onClose={closeModal} onSubmit={onSubmit} />
    </PageShell>
  )
}

function FolderBlock({
  name,
  count,
  expanded,
  commands,
  loading,
  onToggleFolder,
  onEdit,
  onDelete,
  onToggle,
}: {
  name: string
  count: number
  expanded: boolean
  commands: SlashCommand[]
  loading: boolean
  onToggleFolder: (name: string) => void
  onEdit: (command: SlashCommand) => void
  onDelete: (name: string) => void
  onToggle: (name: string) => void
}) {
  const handleToggle = useCallback(() => {
    onToggleFolder(name)
  }, [name, onToggleFolder])
  return (
    <div className="mb-4">
      <button type="button" className="flex w-full items-center gap-2 rounded-xl border border-border-default bg-bg-surface px-4 py-2.5 text-left" onClick={handleToggle}>
        <SIcon name="FolderTree" size="w-4 h-4" className="text-accent-primary" />
        <span className="font-medium text-text-primary">{name}</span>
        <span className="text-sm text-text-muted">({count})</span>
        <SIcon name="ChevronDown" size="w-3.5 h-3.5" className={`ml-auto text-text-muted ${expanded ? 'rotate-180' : ''}`} />
      </button>
      {expanded ? <div className="mt-2"><CommandList commands={commands} loading={loading} onEdit={onEdit} onDelete={onDelete} onToggle={onToggle} /></div> : null}
    </div>
  )
}
