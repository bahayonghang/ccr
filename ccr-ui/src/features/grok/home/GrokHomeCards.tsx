import { memo, useCallback } from 'react'
import { Link } from 'react-router'
import { SIcon, cn } from '@/ui'
import { toneIconClass } from '../ui-classes'
import type { GrokActionItem, GrokManagementItem, GrokReadinessItem } from './grokHomeModel'

export const ReadinessCard = memo(function ReadinessCard({ item }: { item: GrokReadinessItem }) {
  return (
    <article
      className={cn(
        'min-h-48 rounded-2xl border border-[color:var(--stage-border-medium)] bg-[var(--stage-surface-soft)] p-4',
        item.tone === 'success' && 'border-t-2 border-t-accent-success',
        item.tone === 'warning' && 'border-t-2 border-t-accent-warning',
        item.tone === 'danger' && 'border-t-2 border-t-accent-danger',
      )}
    >
      <div className="flex items-center justify-between gap-3">
        <div className={cn('flex h-9 w-9 items-center justify-center rounded-xl border', toneIconClass[item.tone])}>
          <SIcon name={item.icon} size="w-4 h-4" />
        </div>
        <span className="text-xs font-semibold text-[color:var(--stage-text-muted)]">{item.statusLabel}</span>
      </div>
      <p className="mt-5 text-xs font-semibold text-[color:var(--stage-text-muted)]">{item.title}</p>
      <strong className="mt-1 block break-words text-lg font-semibold text-[color:var(--stage-text-primary)]">
        {item.value}
      </strong>
      <p className="mt-3 break-words text-sm leading-6 text-[color:var(--stage-text-secondary)]">{item.detail}</p>
    </article>
  )
})

export const ActionRow = memo(function ActionRow({
  action,
  index,
}: {
  action: GrokActionItem
  index: number
}) {
  const body = (
    <>
      <span className="w-5 shrink-0 text-center text-xs font-semibold text-[color:var(--stage-text-quiet)]">
        {index + 1}
      </span>
      <div className={cn('flex h-9 w-9 shrink-0 items-center justify-center rounded-xl border', toneIconClass[action.tone])}>
        <SIcon name={action.icon} size="w-4 h-4" />
      </div>
      <div className="min-w-0 flex-1">
        <strong className="block text-sm font-semibold text-[color:var(--stage-text-primary)]">{action.title}</strong>
        <p className="mt-1 text-sm leading-6 text-[color:var(--stage-text-secondary)]">{action.description}</p>
      </div>
      <SIcon name={action.external ? 'ExternalLink' : 'ArrowRight'} size="w-4 h-4" className="text-[color:var(--stage-text-quiet)]" />
    </>
  )
  const className =
    'flex items-center gap-3 border-b border-[color:var(--stage-border-soft)] px-1 py-4 hover:bg-[color:color-mix(in_srgb,var(--color-platform-grok)_6%,transparent)]'
  if (action.external) {
    return (
      <a href={action.to} target="_blank" rel="noreferrer" className={className}>
        {body}
      </a>
    )
  }
  return (
    <Link to={action.to} className={className}>
      {body}
    </Link>
  )
})

export const ManageRow = memo(function ManageRow({ item }: { item: GrokManagementItem }) {
  return (
    <Link
      to={item.to}
      className="flex items-center gap-3 border-b border-[color:var(--stage-border-soft)] px-1 py-4 hover:bg-[color:color-mix(in_srgb,var(--color-platform-grok)_6%,transparent)]"
    >
      <div className={cn('flex h-9 w-9 shrink-0 items-center justify-center rounded-xl border', toneIconClass[item.tone])}>
        <SIcon name={item.icon} size="w-4 h-4" />
      </div>
      <div className="min-w-0 flex-1">
        <strong className="block truncate text-sm font-semibold text-[color:var(--stage-text-primary)]">{item.title}</strong>
        <p className="mt-1 truncate text-sm text-[color:var(--stage-text-secondary)]">{item.description}</p>
      </div>
      <span className="max-w-32 truncate text-xs font-semibold text-[color:var(--stage-text-muted)]">{item.badge}</span>
      <SIcon name="ArrowRight" size="w-4 h-4" className="text-[color:var(--stage-text-quiet)]" />
    </Link>
  )
})

export const CommandRow = memo(function CommandRow({
  command,
  copied,
  copyLabel,
  onCopy,
}: {
  command: string
  copied: boolean
  copyLabel: string
  onCopy: (command: string) => void
}) {
  const handleCopy = useCallback(() => {
    onCopy(command)
  }, [command, onCopy])
  return (
    <div className="flex min-h-12 items-center gap-3 rounded-lg border border-[color:var(--stage-border-soft)] bg-[var(--stage-surface-soft)] px-3 py-2 text-[color:var(--stage-text-muted)]">
      <SIcon name="Terminal" size="w-4 h-4" />
      <code className="min-w-0 flex-1 truncate font-mono text-sm text-[color:var(--stage-text-primary)]">{command}</code>
      <button
        type="button"
        className="flex h-8 w-8 items-center justify-center rounded-md text-[color:var(--stage-text-muted)]"
        title={copyLabel}
        aria-label={copyLabel}
        onClick={handleCopy}
      >
        <SIcon name={copied ? 'Check' : 'Copy'} size="w-4 h-4" />
      </button>
    </div>
  )
})
