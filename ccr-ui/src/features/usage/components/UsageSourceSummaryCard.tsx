import { memo, useCallback } from 'react'
import type { SourceBreakdown } from '@/types/usage'
import { formatPercent } from '@/views/usage/usageSummaryCards'
import { usageSourceFallbackLabel } from '@/views/usage/usageSources'
import { useUsageT } from '../translate'
import '../styles/usage-source-summary-card.css'

interface UsageSourceSummaryCardProps {
  sourceStats: SourceBreakdown[]
  selectedPlatform: string
  formatCost: (value: number) => string
  formatTokens: (value: number) => string
  onSelectSource: (source: string) => void
}

const SourceItem = memo(function SourceItem({
  item,
  active,
  formatCost,
  formatTokens,
  requestsLabel,
  activeDaysLabel,
  onSelect,
}: {
  item: SourceBreakdown
  active: boolean
  formatCost: (value: number) => string
  formatTokens: (value: number) => string
  requestsLabel: string
  activeDaysLabel: string
  onSelect: (source: string) => void
}) {
  const handleClick = useCallback(() => onSelect(item.source), [item.source, onSelect])
  const width = Math.min(100, Math.max(item.share_tokens * 100, item.share_tokens > 0 ? 4 : 0))

  return (
    <button
      type="button"
      className={['source-card__item', active ? 'source-card__item--active' : ''].filter(Boolean).join(' ')}
      onClick={handleClick}
    >
      <span className="source-card__source-row">
        <span className="source-card__source-name">{usageSourceFallbackLabel(item.source)}</span>
        <span className="source-card__share">{formatPercent(item.share_tokens)}</span>
      </span>
      <span className="source-card__metrics">
        <strong>{formatTokens(item.total_tokens)}</strong>
        <span>{formatCost(item.total_cost)}</span>
      </span>
      <span className="source-card__meta">
        {item.event_count.toLocaleString()} {requestsLabel} · {item.active_days.toLocaleString()} {activeDaysLabel}
      </span>
      <span className="source-card__bar">
        <span style={{ width: `${width}%` }} />
      </span>
    </button>
  )
})

export function UsageSourceSummaryCard({
  sourceStats,
  selectedPlatform,
  formatCost,
  formatTokens,
  onSelectSource,
}: UsageSourceSummaryCardProps) {
  const t = useUsageT()
  const visibleSources = sourceStats.filter(
    (item) => item.total_cost > 0 || item.total_tokens > 0 || item.event_count > 0,
  )

  return (
    <section className="source-card glass-panel">
      <div className="source-card__head">
        <div>
          <p className="source-card__eyebrow">{t('usage.dashboard.sources.eyebrow')}</p>
          <h3>{t('usage.dashboard.sources.title')}</h3>
          <p>{t('usage.dashboard.sources.subtitle')}</p>
        </div>
        <span className="source-card__count">
          {visibleSources.length} {t('usage.dashboard.sources.sources')}
        </span>
      </div>
      {visibleSources.length > 0 ? (
        <div className="source-card__grid">
          {visibleSources.map((item) => (
            <SourceItem
              key={item.source}
              item={item}
              active={selectedPlatform === item.source}
              formatCost={formatCost}
              formatTokens={formatTokens}
              requestsLabel={t('usage.dashboard.sources.requests')}
              activeDaysLabel={t('usage.dashboard.sources.activeDays')}
              onSelect={onSelectSource}
            />
          ))}
        </div>
      ) : null}
    </section>
  )
}
