import { memo } from 'react'
import { barPercent } from '@/features/claude/observer/formatters'

interface RankItem {
  key: string
  label: string
  title?: string
  value: string
  amount: number
}

interface RankListProps {
  rows: RankItem[]
  empty: string
  tone: 'primary' | 'secondary' | 'info'
}

const TONE_CLASS: Record<RankListProps['tone'], string> = {
  primary: 'bg-accent-primary/80',
  secondary: 'bg-accent-secondary/80',
  info: 'bg-accent-info/75',
}

const RankRow = memo(function RankRow({
  item,
  max,
  tone,
}: {
  item: RankItem
  max: number
  tone: RankListProps['tone']
}) {
  const width = barPercent(item.amount, max)
  return (
    <li className="grid grid-cols-[minmax(0,7rem)_minmax(0,1fr)_auto] items-center gap-2 text-sm">
      <span className="truncate font-semibold text-text-primary" title={item.title ?? item.label}>
        {item.label}
      </span>
      <span className="h-1.5 overflow-hidden rounded-full bg-border-default/20">
        <span className={`block h-full rounded-full ${TONE_CLASS[tone]}`} style={{ width: `${width}%` }} />
      </span>
      <span className="font-semibold tabular-nums text-text-primary">{item.value}</span>
    </li>
  )
})

/** 横向条形排行。列表项 memo，不传内联函数。 */
export function RankList({ rows, empty, tone }: RankListProps) {
  if (rows.length === 0) {
    return (
      <div className="flex min-h-30 items-center justify-center rounded-2xl border border-dashed border-border-default/25 text-sm text-text-muted">
        {empty}
      </div>
    )
  }
  const max = rows.reduce((current, row) => Math.max(current, row.amount), 0)
  return (
    <ol className="m-0 grid list-none gap-2 p-0">
      {rows.map((item) => (
        <RankRow key={item.key} item={item} max={max} tone={tone} />
      ))}
    </ol>
  )
}
