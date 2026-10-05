import { CostAttributionTab } from '@/features/claude/observer/CostAttributionTab'
import { BehaviorAnalysisTab } from '@/features/claude/observer/BehaviorAnalysisTab'
import {
  ObserverHeader,
  ObserverHeroGrid,
  ObserverSubscriptionBar,
  ObserverTabButton,
  type TabId,
} from '@/features/claude/observer/ObserverChrome'
import { SubscriptionDialog } from '@/features/claude/observer/SubscriptionDialog'
import { TokenDetailTab } from '@/features/claude/observer/TokenDetailTab'
import { useObserverPanel } from '@/features/claude/observer/useObserverPanel'
import { t } from '@/features/claude/locale'
import { AsyncStatePanel } from '@/ui'

function ObserverStatus(props: {
  state: 'loading' | 'error' | 'empty' | 'ready'
  loadError: string | null
  emptyDescription: string
  onRetry: () => void
  onUsage: () => void
}) {
  if (props.state === 'loading') {
    return <AsyncStatePanel state="loading" title={t('claudeCode.observer.loading')} compact />
  }
  if (props.state === 'error') {
    return (
      <AsyncStatePanel
        state="error"
        title={t('claudeCode.observer.errorTitle')}
        description={props.loadError ?? ''}
        actionLabel={t('common.retry')}
        actionIcon="RefreshCw"
        onAction={props.onRetry}
      />
    )
  }
  if (props.state === 'empty') {
    return (
      <AsyncStatePanel
        state="empty"
        title={t('claudeCode.observer.empty.noUsage')}
        description={props.emptyDescription}
        icon="Database"
        actionLabel={t('claudeCode.observer.empty.openFullDashboard')}
        actionIcon="ArrowUpRight"
        onAction={props.onUsage}
      />
    )
  }
  return null
}

function ObserverTabBody(props: ReturnType<typeof useObserverPanel>) {
  if (props.activeTab === 'token') {
    return (
      <TokenDetailTab
        stats={props.stats ?? null}
        daily={props.daily ?? []}
        animationsEnabled={props.animationsEnabled}
        shouldRenderChart={props.renderedTabs.has('token')}
      />
    )
  }
  if (props.activeTab === 'behavior') {
    return (
      <BehaviorAnalysisTab
        heatmap={props.heatmap ?? []}
        topTools={props.topTools ?? []}
        sessions={props.sessions ?? []}
        animationsEnabled={props.animationsEnabled}
        shouldRenderChart={props.renderedTabs.has('behavior')}
      />
    )
  }
  return (
    <CostAttributionTab
      daily={props.daily ?? []}
      byProject={props.byProject ?? []}
      byModel={props.byModel ?? []}
      animationsEnabled={props.animationsEnabled}
      shouldRenderChart={props.renderedTabs.has('cost')}
    />
  )
}

function ObserverReady(props: ReturnType<typeof useObserverPanel>) {
  return (
    <>
      <ObserverHeroGrid insight={props.insight} subscription={props.subscription} hasRoi={props.hasRoi} />
      <div className="flex flex-wrap gap-1.5" data-testid="claude-observer-tabs">
        {props.tabs.map((tab: { id: TabId; label: string }) => (
          <ObserverTabButton
            key={tab.id}
            id={tab.id}
            label={tab.label}
            active={props.activeTab === tab.id}
            onSelect={props.selectTab}
          />
        ))}
      </div>
      <div className="min-w-0">
        <ObserverTabBody {...props} />
      </div>
    </>
  )
}

/** Claude 用量洞察。数据走 Query hook，事件失效由 shell eventBridge 负责。 */
export function UsageInsightPanel() {
  const model = useObserverPanel()
  return (
    <div className="grid gap-3" data-testid="claude-observer-panel">
      <ObserverHeader pricingNote={model.pricingNote} />
      <ObserverSubscriptionBar
        showBanner={model.subscription?.mode === 'subscription'}
        subscription={model.subscription}
        roi={model.insight?.roi ?? null}
        onOpen={model.openDialog}
      />
      <ObserverStatus
        state={model.state}
        loadError={model.loadError}
        emptyDescription={model.emptyDescription}
        onRetry={model.refresh}
        onUsage={model.goToUsage}
      />
      {model.state === 'ready' ? <ObserverReady {...model} /> : null}
      <SubscriptionDialog modelValue={model.dialogOpen} current={model.subscription} onClose={model.closeDialog} />
    </div>
  )
}
