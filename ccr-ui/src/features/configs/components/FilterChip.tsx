import { memo, useCallback } from 'react'
import { SIcon } from '@/ui'
import type { ConfigFilter } from '../types'

interface FilterChipProps {
  type: ConfigFilter
  label: string
  icon: string
  iconColor: string
  active: boolean
  onSelect: (type: ConfigFilter) => void
}

export const FilterChip = memo(function FilterChip({
  type,
  label,
  icon,
  iconColor,
  active,
  onSelect,
}: FilterChipProps) {
  const handleClick = useCallback(() => {
    onSelect(type)
  }, [onSelect, type])
  return (
    <button
      type="button"
      className={`filter-btn flex flex-1 items-center justify-center gap-2 rounded-xl px-4 py-2.5 text-sm font-semibold transition-colors duration-300 ${active ? 'filter-btn-active' : ''}`}
      onClick={handleClick}
    >
      <SIcon
        name={icon}
        size="w-4 h-4"
        className={active ? 'text-[color:var(--color-accent-primary-contrast)]' : iconColor}
      />
      <span>{label}</span>
    </button>
  )
})
