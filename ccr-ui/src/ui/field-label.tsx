import type { HTMLAttributes } from 'react'
import { cn } from './cn'

interface FieldLabelProps extends HTMLAttributes<HTMLElement> {
  as?: 'span' | 'dt'
}

export function FieldLabel({ as = 'span', className, ...props }: FieldLabelProps) {
  const classes = cn('ui-field-label', className)

  if (as === 'dt') {
    return <dt className={classes} {...props} />
  }

  return <span className={classes} {...props} />
}
