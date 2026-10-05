import type { ReactNode } from 'react'
import { Button } from './button'
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
        'empty-state flex min-h-[18.75rem] flex-col items-center justify-center p-12 text-center',
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
        <p className="mb-4 max-w-[30rem] text-base text-text-secondary">{description}</p>
      ) : null}
      {actionText && onAction ? (
        <Button variant="primary" onClick={onAction}>
          {actionIcon ? <SIcon name={actionIcon} size="w-[1.125rem] h-[1.125rem]" /> : null}
          {actionText}
        </Button>
      ) : null}
      {children}
    </div>
  )
}
