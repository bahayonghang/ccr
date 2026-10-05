import { memo, useCallback, useMemo, useState } from 'react'
import { Link } from 'react-router'
import { SIcon, buttonClass } from '@/ui'
import type {
  PlatformUsageInsightPresentation,
  PlatformUsageInsightSpec,
  PlatformUsageMetric,
} from '@/types/platformUsageInsight'
import { PlatformUsageRankList } from './PlatformUsageRankList'
import { PlatformUsageTrendChart } from './PlatformUsageTrendChart'
import '../styles/platform-usage-insight-panel.css'

interface PlatformUsageInsightPanelProps {
  spec: PlatformUsageInsightSpec
  state: PlatformUsageInsightPresentation
  loading?: boolean
  error?: string | null
  onRefresh: () => void
}

type UsagePanelTab = PlatformUsageMetric | 'breakdown'

export function PlatformUsageInsightPanel({
  spec,
  state,
  loading = false,
  error = null,
  onRefresh,
}: PlatformUsageInsightPanelProps) {
  const [activeTab, setActiveTab] = useState<UsagePanelTab>('cost')
  const tabs = useMemo(() => [
    { id: 'cost' as const, label: spec.tabs.cost },
    { id: 'tokens' as const, label: spec.tabs.tokens },
    { id: 'requests' as const, label: spec.tabs.requests },
    { id: 'breakdown' as const, label: spec.tabs.breakdown },
  ], [spec.tabs])
  const activeMetric: PlatformUsageMetric = activeTab === 'breakdown' ? 'cost' : activeTab
  const isInitialLoading = loading && state.cards.length === 0

  return (
    <section
      className={`platform-usage-panel platform-usage-panel--${spec.tone}`}
      aria-label={spec.title}
      data-testid="platform-usage-insight"
    >
      <div className="platform-usage-panel__header">
        <div className="platform-usage-panel__copy">
          <p className="platform-usage-panel__eyebrow">{spec.eyebrow}</p>
          <h2>{spec.title}</h2>
          <p>{spec.description}</p>
        </div>
        <div className="platform-usage-panel__actions">
          <button type="button" className="platform-usage-panel__button" disabled={loading} onClick={onRefresh}>
            <SIcon name="RefreshCw" size="w-4 h-4" className={loading ? 'animate-spin' : undefined} />
            {spec.retryLabel}
          </button>
          <Link className={buttonClass({ variant: 'primary', className: 'platform-usage-panel__button' })} to={spec.primaryActionTo}>
            {spec.primaryActionLabel}
            <SIcon name="ArrowUpRight" size="w-4 h-4" />
          </Link>
        </div>
      </div>
      {isInitialLoading ? (
        <div className="platform-usage-panel__skeleton-grid" aria-hidden="true" />
      ) : error && state.empty ? (
        <div className="platform-usage-panel__notice platform-usage-panel__notice--error" role="status">
          <strong>{spec.errorTitle}</strong>
          <span>{error}</span>
        </div>
      ) : state.empty ? (
        <div className="platform-usage-panel__notice" role="status">
          <strong>{spec.emptyTitle}</strong>
          <span>{spec.emptyDescription}</span>
        </div>
      ) : (
        <>
          <div className="platform-usage-panel__kpis">
            {state.cards.map((card) => (
              <article key={card.id} className={`platform-usage-panel__kpi platform-usage-panel__kpi--${card.id}`}>
                <div>
                  <span>{card.label}</span>
                  <strong>{card.value}</strong>
                  <p>{card.detail}</p>
                </div>
              </article>
            ))}
          </div>
          <div className="platform-usage-panel__tabs" role="tablist">
            {tabs.map((tab) => (
              <TabButton
                key={tab.id}
                id={tab.id}
                label={tab.label}
                active={activeTab === tab.id}
                onSelect={setActiveTab}
              />
            ))}
          </div>
          {activeTab !== 'breakdown' ? (
            <PlatformUsageTrendChart
              metric={activeMetric}
              trends={state.trends}
              title={tabs.find((tab) => tab.id === activeMetric)?.label ?? spec.title}
              eyebrow={spec.label}
              windowLabel={spec.windowLabel}
              emptyLabel={spec.emptyDescription}
            />
          ) : null}
          <div className="platform-usage-panel__rank-grid">
            <PlatformUsageRankList
              title={spec.modelRankTitle}
              eyebrow={state.topModelLabel}
              rows={state.modelRows}
              emptyLabel={spec.emptyDescription}
            />
            <PlatformUsageRankList
              title={spec.projectRankTitle}
              eyebrow={state.topProjectLabel}
              rows={state.projectRows}
              emptyLabel={spec.emptyDescription}
            />
          </div>
        </>
      )}
    </section>
  )
}

const TabButton = memo(function TabButton({
  id,
  label,
  active,
  onSelect,
}: {
  id: UsagePanelTab
  label: string
  active: boolean
  onSelect: (id: UsagePanelTab) => void
}) {
  const handleClick = useCallback(() => onSelect(id), [id, onSelect])
  return (
    <button
      type="button"
      role="tab"
      className={['platform-usage-panel__tab', active ? 'platform-usage-panel__tab--active' : ''].filter(Boolean).join(' ')}
      aria-selected={active}
      onClick={handleClick}
    >
      {label}
    </button>
  )
})
