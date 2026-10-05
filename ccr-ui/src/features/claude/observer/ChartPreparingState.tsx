import { t } from '@/features/claude/locale'

const BAR_HEIGHTS = [34, 58, 46, 76, 52, 68, 40, 60]

interface ChartPreparingStateProps {
  label?: string
}

/** 图表准备态：条形占位，不用 px / rgba 字面量。 */
export function ChartPreparingState({ label }: ChartPreparingStateProps) {
  const displayLabel = label ?? t('claudeCode.observer.chart.preparing')
  return (
    <div
      className="grid min-h-full w-full place-items-center gap-3 rounded-2xl border border-dashed border-border-default/25 bg-bg-surface text-sm text-text-muted"
      role="status"
    >
      <div className="flex h-13 items-end gap-1" aria-hidden="true">
        {BAR_HEIGHTS.map((height) => (
          <span
            key={height}
            className="min-h-3 w-1.5 rounded-full bg-accent-primary/30 motion-safe:animate-pulse"
            style={{ height: `${height}%` }}
          />
        ))}
      </div>
      <span className="font-semibold text-text-muted">{displayLabel}</span>
    </div>
  )
}
