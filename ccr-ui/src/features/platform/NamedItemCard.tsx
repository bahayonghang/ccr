import { memo, useCallback } from 'react'

export interface NamedItem {
  id: string
  name: string
  description?: string
  enabled?: boolean
  badge?: string
}

interface NamedItemCardProps {
  item: NamedItem
  onEdit?: (id: string) => void
  onDelete?: (id: string) => void
  onToggle?: (id: string) => void
  onRun?: (id: string) => void
  editLabel: string
  deleteLabel: string
  toggleLabel?: string
  runLabel?: string
}

export const NamedItemCard = memo(function NamedItemCard({
  item,
  onEdit,
  onDelete,
  onToggle,
  onRun,
  editLabel,
  deleteLabel,
  toggleLabel,
  runLabel,
}: NamedItemCardProps) {
  const handleEdit = useCallback(() => {
    onEdit?.(item.id)
  }, [item.id, onEdit])
  const handleDelete = useCallback(() => {
    onDelete?.(item.id)
  }, [item.id, onDelete])
  const handleToggle = useCallback(() => {
    onToggle?.(item.id)
  }, [item.id, onToggle])
  const handleRun = useCallback(() => {
    onRun?.(item.id)
  }, [item.id, onRun])

  return (
    <article className="rounded-xl border border-border-default/55 bg-bg-surface p-4">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h3 className="truncate font-mono text-sm font-semibold text-text-primary">{item.name}</h3>
          {item.description ? (
            <p className="mt-1 text-sm text-text-secondary">{item.description}</p>
          ) : null}
          {item.badge ? (
            <span className="mt-2 inline-block rounded-md bg-bg-elevated px-2 py-0.5 text-xs text-text-muted">
              {item.badge}
            </span>
          ) : null}
        </div>
        <div className="flex shrink-0 flex-wrap gap-2">
          {onRun ? (
            <button type="button" className="rounded-lg border border-border-default px-3 py-1.5 text-sm" onClick={handleRun}>
              {runLabel}
            </button>
          ) : null}
          {onToggle ? (
            <button type="button" className="rounded-lg border border-border-default px-3 py-1.5 text-sm" onClick={handleToggle}>
              {toggleLabel}
            </button>
          ) : null}
          {onEdit ? (
            <button type="button" className="rounded-lg border border-border-default px-3 py-1.5 text-sm" onClick={handleEdit}>
              {editLabel}
            </button>
          ) : null}
          {onDelete ? (
            <button type="button" className="rounded-lg border border-border-default px-3 py-1.5 text-sm" onClick={handleDelete}>
              {deleteLabel}
            </button>
          ) : null}
        </div>
      </div>
    </article>
  )
})
