import type { ReactNode } from 'react'
import { cn } from './cn'

interface PageHeaderProps {
  title: string
  eyebrow?: string
  description?: string
  eyebrowLang?: string
  leading?: ReactNode
  status?: ReactNode
  actions?: ReactNode
  className?: string
}

const isLatinEyebrow = (value: string): boolean => /^[\x20-\x7E\s]+$/.test(value)

/** 页头：eyebrow / title / description + leading / status / actions。 */
export function PageHeader({
  title,
  eyebrow,
  description,
  eyebrowLang,
  leading,
  status,
  actions,
  className,
}: PageHeaderProps) {
  const resolvedLang = eyebrowLang ?? (eyebrow && isLatinEyebrow(eyebrow) ? 'en' : undefined)

  return (
    <header className={cn('page-header', className)}>
      {leading ? <div className="page-header__leading">{leading}</div> : null}
      <div className="page-header__main">
        {eyebrow ? (
          <p className="page-header__eyebrow" lang={resolvedLang}>
            {eyebrow}
          </p>
        ) : null}
        <h1 className="page-header__title">{title}</h1>
        {description ? <p className="page-header__description">{description}</p> : null}
      </div>
      {status || actions ? (
        <div className="page-header__aside">
          {status ? <div className="page-header__status">{status}</div> : null}
          {actions ? <div className="page-header__actions">{actions}</div> : null}
        </div>
      ) : null}
    </header>
  )
}
