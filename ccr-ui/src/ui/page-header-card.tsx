import type { ReactNode } from 'react'
import { cn } from './cn'
import { SIcon } from './s-icon'

export type HeaderTone = 'primary' | 'secondary' | 'success' | 'danger' | 'info'

interface PageHeaderCardProps {
  title: string
  icon: string
  description?: string
  badge?: string
  tone?: HeaderTone
  meta?: ReactNode
  actions?: ReactNode
  children?: ReactNode
  className?: string
}

const TONE = {
  primary: {
    iconBox: 'page-header-card__icon--primary',
    icon: 'text-accent-primary',
    badge: 'page-header-card__badge--primary',
  },
  secondary: {
    iconBox: 'page-header-card__icon--secondary',
    icon: 'text-accent-primary',
    badge: 'page-header-card__badge--secondary',
  },
  success: {
    iconBox: 'page-header-card__icon--success',
    icon: 'text-success',
    badge: 'page-header-card__badge--success',
  },
  danger: {
    iconBox: 'page-header-card__icon--danger',
    icon: 'text-danger',
    badge: 'page-header-card__badge--danger',
  },
  info: {
    iconBox: 'page-header-card__icon--info',
    icon: 'text-info',
    badge: 'page-header-card__badge--info',
  },
} as const

export function PageHeaderCard({
  title,
  icon,
  description,
  badge,
  tone = 'primary',
  meta,
  actions,
  children,
  className,
}: PageHeaderCardProps) {
  const toneClasses = TONE[tone]
  return (
    <section className={cn('page-header-card', className)}>
      <div className="page-header-card__content">
        <div className="page-header-card__top">
          <div className="page-header-card__intro">
            <div className={cn('page-header-card__icon', toneClasses.iconBox)}>
              <SIcon name={icon} size="w-6 h-6" className={toneClasses.icon} />
            </div>
            <div className="min-w-0 flex-1">
              <div className="page-header-card__title-row">
                <h1 className="page-header-card__title">{title}</h1>
                {badge ? (
                  <span className={cn('page-header-card__badge', toneClasses.badge)}>{badge}</span>
                ) : null}
              </div>
              {description ? <p className="page-header-card__description">{description}</p> : null}
              {meta ? <div className="page-header-card__meta">{meta}</div> : null}
            </div>
          </div>
          {actions ? <div className="page-header-card__actions">{actions}</div> : null}
        </div>
        {children ? <div className="page-header-card__body">{children}</div> : null}
      </div>
    </section>
  )
}
