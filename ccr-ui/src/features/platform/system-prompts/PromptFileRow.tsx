import { memo, useCallback } from 'react'
import type { SystemPromptFile } from '@/api/domains/systemPrompts'
import { SIcon } from '@/ui'
import type { TranslateFunction } from '@/utils/tf'

interface PromptFileRowProps {
  file: SystemPromptFile
  active: boolean
  busy: boolean
  creating: boolean
  t: TranslateFunction
  formatTime: (timestamp: number) => string
  onSelect: (file: SystemPromptFile) => void
  onCreate: (file: SystemPromptFile) => void
}

export const PromptFileRow = memo(function PromptFileRow({
  file,
  active,
  busy,
  creating,
  t,
  formatTime,
  onSelect,
  onCreate,
}: PromptFileRowProps) {
  const handleSelect = useCallback(() => {
    onSelect(file)
  }, [file, onSelect])
  const handleCreate = useCallback(() => {
    onCreate(file)
  }, [file, onCreate])

  return (
    <article
      className={
        active
          ? 'border-b border-border-subtle bg-accent-primary/5 py-3'
          : 'border-b border-border-subtle py-3'
      }
    >
      <button
        type="button"
        className="flex w-full items-center gap-3 bg-transparent p-0 text-left text-text-primary"
        disabled={busy}
        onClick={handleSelect}
      >
        <SIcon name={file.exists ? 'FileCheck2' : 'FileQuestion'} size="w-5 h-5" />
        <span className="min-w-0">
          <strong className="block truncate">{t(file.labelKey)}</strong>
          <code className="mt-0.5 block truncate font-mono text-xs text-text-muted">{file.path}</code>
        </span>
      </button>
      <div className="ml-8 mt-2 flex flex-wrap gap-3 text-xs text-text-muted">
        <span>{file.exists ? t('systemPrompts.exists') : t('systemPrompts.missing')}</span>
        {file.size !== null ? <span>{t('systemPrompts.bytes', { count: file.size })}</span> : null}
        {file.mtime ? <span>{t('systemPrompts.modified', { time: formatTime(file.mtime) })}</span> : null}
      </div>
      {file.exists ? null : (
        <button
          type="button"
          className="ml-8 mt-2 inline-flex min-h-8 items-center gap-1 rounded-md border border-border-default bg-bg-elevated px-3 text-sm text-text-secondary disabled:opacity-50"
          disabled={busy}
          onClick={handleCreate}
        >
          <SIcon name="FilePlus2" size="w-4 h-4" />
          {creating ? t('systemPrompts.creating') : t('systemPrompts.create')}
        </button>
      )}
    </article>
  )
})
