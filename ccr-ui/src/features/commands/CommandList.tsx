import { memo, useCallback } from 'react'
import type { SlashCommand } from '@/types/platform'
import { SIcon, Spinner } from '@/ui'
import { useCommandsT } from './locale'

interface CommandListProps {
  commands: SlashCommand[]
  loading: boolean
  onEdit: (command: SlashCommand) => void
  onDelete: (name: string) => void
  onToggle: (name: string) => void
}

const CommandCard = memo(function CommandCard({
  command,
  onEdit,
  onDelete,
  onToggle,
}: {
  command: SlashCommand
  onEdit: (command: SlashCommand) => void
  onDelete: (name: string) => void
  onToggle: (name: string) => void
}) {
  const t = useCommandsT()
  const handleEdit = useCallback(() => {
    onEdit(command)
  }, [command, onEdit])
  const handleDelete = useCallback(() => {
    onDelete(command.name)
  }, [command.name, onDelete])
  const handleToggle = useCallback(() => {
    onToggle(command.name)
  }, [command.name, onToggle])
  return (
    <div className="flex rounded-xl border border-border-default bg-bg-elevated p-4">
      <div className="min-w-0 flex-1">
        <div className="mb-2 flex items-center justify-between">
          <h3 className="text-base font-semibold text-text-primary">{command.name}</h3>
          <span className={`rounded-full px-2 py-1 text-xs font-medium ${command.enabled ? 'bg-accent-success/15 text-accent-success' : 'bg-accent-danger/15 text-accent-danger'}`}>
            {command.enabled ? t('common.enabled') : t('common.disabled')}
          </span>
        </div>
        <code className="mb-2 block rounded bg-bg-base px-2 py-1 font-mono text-sm">{command.command}</code>
        <p className="text-sm text-text-secondary">{command.description}</p>
        <span className="mt-3 inline-block rounded-md bg-accent-primary/10 px-2 py-1 text-xs text-accent-primary">{command.folder}</span>
      </div>
      <div className="ml-4 flex items-center gap-2">
        <button type="button" className="rounded-lg p-2 text-text-muted" aria-label={command.enabled ? t('common.disable') : t('common.enable')} onClick={handleToggle}>
          <SIcon name="Power" size="w-4 h-4" />
        </button>
        <button type="button" className="rounded-lg p-2 text-text-muted" aria-label={t('common.edit')} onClick={handleEdit}>
          <SIcon name="Edit" size="w-4 h-4" />
        </button>
        <button type="button" className="rounded-lg p-2 text-text-muted" aria-label={t('common.delete')} onClick={handleDelete}>
          <SIcon name="Trash2" size="w-4 h-4" />
        </button>
      </div>
    </div>
  )
})

export function CommandList({ commands, loading, onEdit, onDelete, onToggle }: CommandListProps) {
  const t = useCommandsT()
  if (loading) {
    return (
      <div className="flex items-center justify-center gap-2 py-8 text-text-secondary">
        <Spinner size="sm" />
        <span>{t('common.loading')}</span>
      </div>
    )
  }
  return (
    <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
      {commands.map((command) => (
        <CommandCard key={command.name} command={command} onEdit={onEdit} onDelete={onDelete} onToggle={onToggle} />
      ))}
    </div>
  )
}
