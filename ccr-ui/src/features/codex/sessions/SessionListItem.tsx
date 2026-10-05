import { memo, useCallback } from 'react'
import { cn } from '@/ui'
import type { CodexSessionSummary } from '@/types'
import { formatSessionRelative, formatTokenCount } from './session-format'

interface SessionListItemProps {
  session: CodexSessionSummary
  active: boolean
  unknownModel: string
  unknownTime: string
  messageLabel: string
  onOpen: (filePath: string) => void
}

export const SessionListItem = memo(function SessionListItem({
  session,
  active,
  unknownModel,
  unknownTime,
  messageLabel,
  onOpen,
}: SessionListItemProps) {
  const handleClick = useCallback(() => {
    onOpen(session.file_path)
  }, [onOpen, session.file_path])

  return (
    <button
      type="button"
      className={cn(
        'w-full rounded-2xl border border-border-default/15 bg-bg-elevated p-4 text-left transition-all duration-200 hover:border-platform-codex/25 hover:bg-bg-elevated/80',
        active && 'border-platform-codex/35 bg-platform-codex/10',
      )}
      onClick={handleClick}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <p className="truncate font-mono text-sm font-semibold text-text-primary">{session.session_id}</p>
          <p className="mt-1 text-xs text-text-ghost">
            {session.model || unknownModel} · {formatSessionRelative(session.updated_at, unknownTime)}
          </p>
        </div>
        <span className="shrink-0 rounded-full border border-border-default/15 bg-bg-elevated px-2.5 py-1 text-[0.6875rem] text-text-secondary">
          {messageLabel}
        </span>
      </div>
      {session.preview ? (
        <p className="mt-3 line-clamp-3 text-sm leading-6 text-text-secondary">{session.preview}</p>
      ) : null}
      <div className="mt-3 flex items-center justify-between gap-3 text-xs text-text-ghost">
        <span className="truncate">{session.cwd || session.relative_path}</span>
        <span>{formatTokenCount(session.total_input_tokens + session.total_output_tokens)}</span>
      </div>
    </button>
  )
})
