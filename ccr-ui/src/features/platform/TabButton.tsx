import { memo, useCallback } from 'react'

interface TabButtonProps {
  id: string
  label: string
  active: boolean
  onSelect: (id: string) => void
}

export const TabButton = memo(function TabButton({ id, label, active, onSelect }: TabButtonProps) {
  const handleClick = useCallback(() => {
    onSelect(id)
  }, [id, onSelect])

  const className = active
    ? 'rounded-xl border border-accent-primary/40 bg-accent-primary/10 px-3 py-2 text-sm text-accent-primary'
    : 'rounded-xl border border-border-default px-3 py-2 text-sm text-text-secondary'

  return (
    <button type="button" className={className} onClick={handleClick}>
      {label}
    </button>
  )
})
