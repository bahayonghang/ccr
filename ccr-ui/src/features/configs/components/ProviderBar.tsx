import { memo } from 'react'
import { t } from '../locale'

interface ProviderBarProps {
  provider: string
  count: number
  maxCount: number
  color: string
  shareLabel: string
}

export const ProviderBar = memo(function ProviderBar({
  provider,
  count,
  maxCount,
  color,
  shareLabel,
}: ProviderBarProps) {
  const height = Math.max((count / (maxCount || 1)) * 100, 4)
  return (
    <div className="group flex w-16 flex-col items-center gap-2">
      <div className="pointer-events-none absolute -top-8 z-20 rounded px-2 py-1 text-xs whitespace-nowrap opacity-0 shadow-lg transition-opacity group-hover:opacity-100 bg-bg-elevated text-text-primary">
        {shareLabel}
      </div>
      <div className="relative flex h-[18.75rem] w-full items-end justify-center">
        <div
          className="relative w-full overflow-hidden rounded-t-lg transition-all duration-300 group-hover:brightness-110"
          style={{ height: `${height}%`, background: color }}
        />
      </div>
      <div className="w-full text-center">
        <div className="w-full cursor-help truncate text-xs font-medium text-text-secondary" title={provider || t('configs.provider.unknown')}>
          {provider || t('configs.provider.unknown')}
        </div>
        <div className="mt-0.5 font-mono text-[0.625rem] text-text-muted">{count}</div>
      </div>
    </div>
  )
})
