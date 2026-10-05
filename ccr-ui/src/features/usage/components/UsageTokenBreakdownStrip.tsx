import { useMemo } from 'react'
import type { UsageSummary } from '@/types/usage'
import { formatPercent, formatTokens } from '@/views/usage/usageSummaryCards'
import { usageTokenCategoryColors } from '@/views/usage/usageChartOptions'
import { useUsageT } from '../translate'
import '../styles/usage-token-breakdown-strip.css'

interface UsageTokenBreakdownStripProps {
  summary: UsageSummary
  cacheCreationTokens: number
}

export function UsageTokenBreakdownStrip({
  summary,
  cacheCreationTokens,
}: UsageTokenBreakdownStripProps) {
  const t = useUsageT()
  const total = Math.max(
    0,
    summary.total_input_tokens + summary.total_output_tokens + summary.total_cache_read_tokens + cacheCreationTokens,
  )

  const items = useMemo(() => {
    const raw = [
      {
        id: 'input' as const,
        label: t('usage.dashboard.tokenStrip.input'),
        value: summary.total_input_tokens,
        color: usageTokenCategoryColors.input,
      },
      {
        id: 'output' as const,
        label: t('usage.dashboard.tokenStrip.output'),
        value: summary.total_output_tokens,
        color: usageTokenCategoryColors.output,
      },
      {
        id: 'cacheRead' as const,
        label: t('usage.dashboard.tokenStrip.cacheRead'),
        value: summary.total_cache_read_tokens,
        color: usageTokenCategoryColors.cacheRead,
      },
      {
        id: 'cacheCreation' as const,
        label: t('usage.dashboard.tokenStrip.cacheCreation'),
        value: cacheCreationTokens,
        color: usageTokenCategoryColors.cacheCreation,
      },
    ]
    return raw.map((item) => ({
      ...item,
      valueLabel: formatTokens(item.value),
      share: total > 0 ? item.value / total : 0,
    }))
  }, [cacheCreationTokens, summary, t, total])

  const visibleItems = items.filter((item) => item.share > 0)
  const cacheEfficiencyLabel = formatPercent(
    total > 0 ? (summary.total_cache_read_tokens + cacheCreationTokens) / total : 0,
  )

  return (
    <section className="usage-token-strip" aria-label={t('usage.dashboard.tokenStrip.title')}>
      <div className="usage-token-strip__header">
        <div>
          <p className="usage-token-strip__eyebrow">{t('usage.dashboard.tokenStrip.eyebrow')}</p>
          <h2>{t('usage.dashboard.tokenStrip.title')}</h2>
        </div>
        <div className="usage-token-strip__efficiency">
          <span>{t('usage.dashboard.tokenStrip.cacheEfficiency')}</span>
          <strong>{cacheEfficiencyLabel}</strong>
        </div>
      </div>
      <div className="usage-token-strip__track">
        {visibleItems.map((item) => (
          <div
            key={item.id}
            className="usage-token-strip__segment"
            style={{ flexGrow: item.share, ['--usage-token-rgb' as string]: `var(${item.color.rgbVar})` }}
            title={`${item.label}: ${item.valueLabel}`}
          />
        ))}
      </div>
      <div className="usage-token-strip__legend">
        {items.map((item) => (
          <article
            key={item.id}
            className={`usage-token-strip__item usage-token-strip__item--${item.id}`}
            style={{ ['--usage-token-rgb' as string]: `var(${item.color.rgbVar})` }}
          >
            <span className="usage-token-strip__dot" />
            <span className="usage-token-strip__label">{item.label}</span>
            <strong className="usage-token-strip__value">{item.valueLabel}</strong>
          </article>
        ))}
      </div>
      <p className="usage-token-strip__note">{t('usage.dashboard.tokenStrip.outputReasoningNote')}</p>
    </section>
  )
}
