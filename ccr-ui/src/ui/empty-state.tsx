import type { ReactNode } from 'react'
import { cn } from './cn'
import { SIcon } from './s-icon'

interface EmptyStateProps {
  icon?: string
  title: string
  description?: string
  actionText?: string
  actionIcon?: string
  onAction?: () => void
  children?: ReactNode
  className?: string
}

export function EmptyState({
  icon = 'FileX',
  title,
  description,
  actionText,
  actionIcon,
  onAction,
  children,
  className,
}: EmptyStateProps) {
  return (
    <div
      className={cn(
        'empty-state flex min-h-[300px] flex-col items-center justify-center p-12 text-center',
        className,
      )}
      role="status"
      aria-live="polite"
    >
      <div
        className="empty-state__icon mb-4 flex h-20 w-20 items-center justify-center rounded-full text-text-muted"
        aria-hidden="true"
      >
        <SIcon name={icon} />
      </div>
      <h3 className="mb-2 text-xl font-semibold text-text-primary">{title}</h3>
      {description ? (
        <p className="mb-4 max-w-[480px] text-base text-text-secondary">{description}</p>
      ) : null}
      {actionText && onAction ? (
        <button
          type="button"
          className="inline-flex min-h-11 items-center justify-center gap-2 rounded-full border border-accent-primary/15 bg-accent-primary px-5 py-2.5 text-base font-medium text-text-inverted transition-[background-color,transform] duration-200 ease-out hover:-translate-y-px hover:bg-accent-primary-hover active:translate-y-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/30"
          onClick={onAction}
        >
          {actionIcon ? <SIcon name={actionIcon} size="w-[18px] h-[18px]" /> : null}
          {actionText}
        </button>
      ) : null}
      {children}
    </div>
  )
}
