import type { PlatformUsageRankRow } from '@/types/platformUsageInsight'
import '../styles/platform-usage-rank-list.css'

interface PlatformUsageRankListProps {
  title: string
  eyebrow: string
  rows: PlatformUsageRankRow[]
  emptyLabel: string
}

export function PlatformUsageRankList({
  title,
  eyebrow,
  rows,
  emptyLabel,
}: PlatformUsageRankListProps) {
  return (
    <article className="platform-usage-rank">
      <header>
        <p>{eyebrow}</p>
        <h3>{title}</h3>
      </header>
      {rows.length === 0 ? (
        <p>{emptyLabel}</p>
      ) : (
        <ol>
          {rows.map((row) => (
            <li key={row.id}>
              <strong title={row.title}>{row.label}</strong>
              <span>{row.value}</span>
            </li>
          ))}
        </ol>
      )}
    </article>
  )
}
