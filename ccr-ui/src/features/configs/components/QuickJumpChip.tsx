import { memo, useCallback } from 'react'
import type { ConfigItem } from '@/types'

interface QuickJumpChipProps {
  config: ConfigItem
  onJump: (name: string) => void
}

export const QuickJumpChip = memo(function QuickJumpChip({ config, onJump }: QuickJumpChipProps) {
  const handleClick = useCallback(() => {
    onJump(config.name)
  }, [config.name, onJump])
  const className = config.is_current
    ? 'border-accent-primary/30 bg-accent-primary/10 text-accent-primary'
    : 'border-border-default/50 bg-bg-elevated text-text-secondary hover:border-accent-primary/20 hover:text-text-primary'
  const badgeClass = config.is_current
    ? 'bg-accent-primary/15 text-accent-primary'
    : 'bg-bg-elevated text-text-muted'
  return (
    <button
      type="button"
      className={`inline-flex items-center gap-2 rounded-full border px-3 py-1.5 text-sm transition-[border-color,background-color,color] duration-200 ${className}`}
      onClick={handleClick}
    >
      <span className="max-w-[11.25rem] truncate">{config.name}</span>
      <span className={`rounded-full px-1.5 py-0.5 text-[0.6875rem] font-semibold ${badgeClass}`}>
        {config.usage_count || 0}
      </span>
    </button>
  )
})
