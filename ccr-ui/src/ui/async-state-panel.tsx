import { cn } from './cn'
import { SIcon } from './s-icon'
import { Spinner } from './spinner'

export type AsyncPanelState = 'loading' | 'error' | 'empty' | 'runtime-unavailable'

interface AsyncStatePanelProps {
  state: AsyncPanelState
  title: string
  description?: string
  icon?: string
  actionLabel?: string
  actionIcon?: string
  compact?: boolean
  onAction?: () => void
  className?: string
}

const iconNameOf = (state: AsyncPanelState, icon?: string): string => {
  if (icon) return icon
  if (state === 'error') return 'AlertCircle'
  if (state === 'runtime-unavailable') return 'MonitorOff'
  if (state === 'empty') return 'FileX'
  return 'Loader2'
}

export function AsyncStatePanel({
  state,
  title,
  description,
  icon,
  actionLabel,
  actionIcon,
  compact = false,
  onAction,
  className,
}: AsyncStatePanelProps) {
  const iconName = iconNameOf(state, icon)
  const iconClass =
    state === 'error'
      ? 'text-accent-danger'
      : state === 'runtime-unavailable'
        ? 'text-accent-secondary'
        : 'text-text-muted'
  const iconWrapClass =
    state === 'error'
      ? 'bg-danger/10 border-danger/18'
      : state === 'runtime-unavailable'
        ? 'bg-accent-primary/10 border-accent-primary/18'
        : 'bg-bg-surface border-border-default/55'

  return (
    <div
      className={cn('async-state-panel', compact && 'async-state-panel--compact', className)}
      role="status"
      aria-live="polite"
    >
      {state === 'loading' ? (
        <>
          <Spinner size="xl" className="text-accent-primary" />
          <p className="mt-3 text-sm text-text-secondary">{title}</p>
          {description ? (
            <p className="mt-1 text-center text-xs text-text-muted">{description}</p>
          ) : null}
        </>
      ) : (
        <>
          <div className={cn('async-state-panel__icon', iconWrapClass)}>
            <SIcon name={iconName} size="w-8 h-8" className={iconClass} />
          </div>
          <h3 className="mt-4 text-lg font-semibold text-text-primary">{title}</h3>
          {description ? (
            <p className="mt-2 max-w-[32.5rem] text-center text-sm text-text-secondary">
              {description}
            </p>
          ) : null}
          {actionLabel ? (
            <button
              type="button"
              className="mt-5 inline-flex min-h-10 items-center gap-2 rounded-xl bg-accent-primary px-4 py-2 text-sm font-medium text-text-inverted"
              onClick={onAction}
            >
              {actionIcon ? <SIcon name={actionIcon} size="w-4 h-4" /> : null}
              {actionLabel}
            </button>
          ) : null}
        </>
      )}
    </div>
  )
}
