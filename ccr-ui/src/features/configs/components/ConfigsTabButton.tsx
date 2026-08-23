import { memo, useCallback } from 'react'
import { SIcon } from '@/ui'
import type { ConfigsTabId } from '../types'

interface ConfigsTabButtonProps {
  id: ConfigsTabId
  label: string
  icon: string
  active: boolean
  onSelect: (id: ConfigsTabId) => void
}

export const ConfigsTabButton = memo(function ConfigsTabButton({
  id,
  label,
  icon,
  active,
  onSelect,
}: ConfigsTabButtonProps) {
  const handleClick = useCallback(() => {
    onSelect(id)
  }, [id, onSelect])
  const className = active
    ? 'border-accent-primary text-accent-primary'
    : 'border-transparent text-text-muted hover:text-text-primary'
  return (
    <button
      type="button"
      className={`flex items-center gap-2 border-b-2 px-2 pb-2 text-sm font-bold transition-colors duration-300 ${className}`}
      onClick={handleClick}
    >
      <SIcon name={icon} size="w-4 h-4" />
      {label}
    </button>
  )
})
