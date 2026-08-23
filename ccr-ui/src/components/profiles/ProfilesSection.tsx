import type { ReactNode } from 'react'
import { SIcon } from '@/ui'
import './profiles-shared.css'

export interface ProfilesSectionProps {
  title: string
  count: number
  children?: ReactNode
}

/** Shared profile group container: heading, count, and content children. */
export function ProfilesSection({ title, count, children }: ProfilesSectionProps) {
  return (
    <section className="cp-section">
      <div className="cp-section__head">
        <SIcon name="Folder" size="w-3.5 h-3.5" className="cp-section__icon" />
        <span className="cp-section__title">{title}</span>
        <span className="cp-section__count">{count}</span>
      </div>
      <div className="cp-section__body">{children}</div>
    </section>
  )
}
