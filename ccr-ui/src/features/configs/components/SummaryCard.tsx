import { memo, useCallback } from 'react'
import { SIcon } from '@/ui'
import type { ConfigFilter } from '../types'

interface SummaryCardProps {
  filterKey: ConfigFilter
  label: string
  count: number
  icon: string
  active: boolean
  activeClass: string
  idleClass: string
  onSelect: (key: ConfigFilter) => void
}

export const SummaryCard = memo(function SummaryCard({
  filterKey,
  label,
  count,
  icon,
  active,
  activeClass,
  idleClass,
  onSelect,
}: SummaryCardProps) {
  const handleClick = useCallback(() => {
    onSelect(filterKey)
  }, [filterKey, onSelect])
  return (
    <button
      type="button"
      className={`rounded-2xl border px-4 py-4 text-left transition-[border-color,background-color,transform] duration-200 hover:-translate-y-0.5 ${active ? activeClass : idleClass}`}
      onClick={handleClick}
    >
      <p className="text-xs font-medium text-text-muted">{label}</p>
      <div className="mt-3 flex items-end justify-between gap-3">
        <span className="text-3xl leading-none font-bold">{count}</span>
        <SIcon name={icon} size="w-5 h-5" className="shrink-0 opacity-80" />
      </div>
    </button>
  )
})
