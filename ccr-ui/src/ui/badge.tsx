import type { HTMLAttributes } from 'react'
import { cn } from './cn'

export type BadgeMode = 'static' | 'interactive'
export type BadgeTone = 'neutral' | 'accent' | 'warning' | 'success'

interface BadgeProps extends HTMLAttributes<HTMLElement> {
  mode?: BadgeMode
  tone?: BadgeTone
  as?: 'span' | 'button'
}

export function Badge({
  mode = 'static',
  tone = 'neutral',
  as,
  className,
  ...props
}: BadgeProps) {
  const resolvedTag = mode === 'static' ? 'span' : (as ?? 'button')
  const classes = cn('ui-badge', `ui-badge--${tone}`, `ui-badge--${mode}`, className)

  if (resolvedTag === 'button') {
    return (
      <button type="button" className={classes} {...(props as HTMLAttributes<HTMLButtonElement>)} />
    )
  }

  return <span className={classes} {...props} />
}
