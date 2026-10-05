import { memo, useCallback } from 'react'
import { t } from '@/features/claude/locale'
import type { OutputStyle } from '@/types'
import { SIcon } from '@/ui'

interface StyleCardProps {
  style: OutputStyle
  onView: (style: OutputStyle) => void
  onEdit: (style: OutputStyle) => void
  onDelete: (name: string) => void
}

function previewContent(content: string): string {
  return content.length > 300 ? `${content.slice(0, 300)}...` : content
}

export const StyleCard = memo(function StyleCard({ style, onView, onEdit, onDelete }: StyleCardProps) {
  const handleView = useCallback(() => {
    onView(style)
  }, [onView, style])
  const handleEdit = useCallback(() => {
    onEdit(style)
  }, [onEdit, style])
  const handleDelete = useCallback(() => {
    onDelete(style.name)
  }, [onDelete, style.name])
  return (
    <article className="relative z-10 h-full rounded-2xl border border-border-subtle bg-bg-surface p-4" role="listitem">
      <div className="mb-3 flex items-start justify-between">
        <div className="flex items-center gap-2">
          <div className="flex h-10 w-10 items-center justify-center rounded-xl border border-border-default/25 bg-accent-secondary/10 text-accent-secondary">
            <SIcon name="Palette" size="w-5 h-5" />
          </div>
          <h3 className="text-lg font-bold text-text-primary">{style.name}</h3>
        </div>
        <div className="flex gap-1">
          <button
            type="button"
            className="flex min-h-11 min-w-11 items-center justify-center rounded-md text-text-secondary hover:bg-accent-secondary/10 hover:text-accent-secondary"
            aria-label={`${t('common.view')}: ${style.name}`}
            onClick={handleView}
          >
            <SIcon name="Eye" size="w-4 h-4" />
          </button>
          <button
            type="button"
            className="flex min-h-11 min-w-11 items-center justify-center rounded-md text-text-secondary hover:bg-accent-secondary/10 hover:text-accent-secondary"
            aria-label={`${t('common.edit')}: ${style.name}`}
            onClick={handleEdit}
          >
            <SIcon name="Edit2" size="w-4 h-4" />
          </button>
          <button
            type="button"
            className="flex min-h-11 min-w-11 items-center justify-center rounded-md text-text-secondary hover:bg-accent-danger/10 hover:text-accent-danger"
            aria-label={`${t('common.delete')}: ${style.name}`}
            onClick={handleDelete}
          >
            <SIcon name="Trash2" size="w-4 h-4" />
          </button>
        </div>
      </div>
      <button
        type="button"
        className="block w-full rounded-2xl text-left focus-visible:ring-2 focus-visible:ring-accent-secondary/30 focus-visible:outline-none"
        aria-label={`${t('common.view')}: ${style.name}`}
        onClick={handleView}
      >
        <div className="rounded-lg border border-border-default/30 bg-bg-elevated p-3">
          <p className="mb-1 text-xs font-semibold text-text-muted">{t('outputStyles.preview')}:</p>
          <pre className="line-clamp-4 whitespace-pre-wrap break-words font-mono text-xs text-text-secondary">
            {previewContent(style.content)}
          </pre>
        </div>
        <div className="mt-3 flex items-center justify-between text-xs text-text-muted">
          <span>
            {style.content.length} {t('outputStyles.characters')}
          </span>
          <span>
            {style.content.split('\n').length} {t('outputStyles.lines')}
          </span>
        </div>
      </button>
    </article>
  )
})
