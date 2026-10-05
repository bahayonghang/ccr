import type { ReactNode } from 'react'
import { cn } from './cn'

export type StatTileTone = 'neutral' | 'success' | 'warning' | 'danger' | 'accent'

interface StatTileProps {
  label?: string
  value?: string | number
  hint?: string
  tone?: StatTileTone
  labelSlot?: ReactNode
  valueSlot?: ReactNode
  hintSlot?: ReactNode
  className?: string
}

export function StatTile({
  label,
  value,
  hint,
  tone,
  labelSlot,
  valueSlot,
  hintSlot,
  className,
}: StatTileProps) {
  const displayValue = value === undefined ? '—' : value
  const showHint = hint || hintSlot

  return (
    <div className={cn('stat-tile', className)}>
      <p className="stat-tile__label">{labelSlot ?? label}</p>
      <p
        className={cn('stat-tile__value', tone && 'stat-tile__value--badge')}
        data-tone={tone}
      >
        {tone ? <span className="stat-tile__tone-dot" aria-hidden="true" /> : null}
        {tone ? (
          <span className="stat-tile__value-text">{valueSlot ?? displayValue}</span>
        ) : (
          (valueSlot ?? displayValue)
        )}
      </p>
      {showHint ? <p className="stat-tile__hint">{hintSlot ?? hint}</p> : null}
    </div>
  )
}
