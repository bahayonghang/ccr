import type { ReactNode } from 'react'
import { cn } from './cn'

interface PageShellProps {
  header?: ReactNode
  subnav?: ReactNode
  children?: ReactNode
  className?: string
}

/** 页壳：header / subnav / content 三槽。 */
export function PageShell({ header, subnav, children, className }: PageShellProps) {
  return (
    <div className={cn('page-shell', className)}>
      <div className="page-shell__inner">
        {header ? <div className="page-shell__header">{header}</div> : null}
        {subnav ? <div className="page-shell__subnav">{subnav}</div> : null}
        <div className="page-shell__content">{children}</div>
      </div>
    </div>
  )
}
