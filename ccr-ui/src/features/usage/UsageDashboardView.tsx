import { useCallback, useState } from 'react'
import { useNavigate } from 'react-router'
import { PageHeader, PageShell, PillToggleGroup, StatTile } from '@/ui'
import { getRuntimeUnavailableCopy } from '@/utils/runtimeState'
import { LlmusageInstallDialog } from './components/LlmusageInstallDialog'
import { UsageCostConclusionCard } from './components/UsageCostConclusionCard'
import { UsageCostTab } from './components/UsageCostTab'
import { UsageDashboardToolbar } from './components/UsageDashboardToolbar'
import { UsageDiagnosticsDrawer } from './components/UsageDiagnosticsDrawer'
import { UsageLogsTab } from './components/UsageLogsTab'
import { UsageMetricCard } from './components/UsageMetricCard'
import { UsageModelsTab, UsageProjectsTab, UsageProvidersTab } from './components/UsageTableTabs'
import { UsageOverviewTab } from './components/UsageOverviewTab'
import { UsageDashboardStates } from './components/UsageDashboardStates'
import { UsageStaleBanner } from './components/UsageStaleBanner'
import { UsageTokenBreakdownStrip } from './components/UsageTokenBreakdownStrip'
import { UsageTokensTab } from './components/UsageTokensTab'
import { UsageDashboardProvider } from './UsageDashboardContext'
import { useUsageDashboard } from './useUsageDashboard'
import './styles/usage-dashboard-view.css'

const TAB_COMPONENTS = {
  overview: UsageOverviewTab,
  tokens: UsageTokensTab,
  cost: UsageCostTab,
  providers: UsageProvidersTab,
  models: UsageModelsTab,
  projects: UsageProjectsTab,
  logs: UsageLogsTab,
} as const

export function UsageDashboardView() {
  const usage = useUsageDashboard()
  const navigate = useNavigate()
  const [diagnosticsOpen, setDiagnosticsOpen] = useState(false)
  const [visitedTabs, setVisitedTabs] = useState<Set<string>>(() => new Set(['overview']))
  const runtimeCopy = getRuntimeUnavailableCopy('usage')

  const tabToggleOptions = usage.tabKeys.map((tab) => ({
    value: tab,
    label: usage.t(`usage.dashboard.tabs.${tab}`),
  }))

  const handleTabChange = useCallback((tab: string) => {
    usage.setActiveTab(tab)
    setVisitedTabs((previous) => new Set(previous).add(tab))
  }, [usage])

  const handleHome = useCallback(() => {
    void navigate('/')
  }, [navigate])

  const handleSecondary = useCallback(() => setDiagnosticsOpen(true), [])

  const hasDashboardData = usage.summary != null || usage.trends.length > 0

  const content = Array.from(visitedTabs).map((tab) => {
    const Tab = TAB_COMPONENTS[tab as keyof typeof TAB_COMPONENTS] ?? UsageOverviewTab
    return (
      <div key={tab} hidden={tab !== usage.activeTab}>
        <Tab />
      </div>
    )
  })

  return (
    <UsageDashboardProvider value={usage}>
      <PageShell
        className="usage-page"
        header={(
          <PageHeader
            title={usage.t('usage.title')}
            description={usage.t('usage.subtitle')}
            status={usage.costSummaryCard ? (
              <StatTile
                label={usage.costSummaryCard.label}
                value={usage.costSummaryCard.value}
                hint={usage.costSummaryCard.detail}
              />
            ) : null}
          />
        )}
      >
        <div className="usage-shell">
          <UsageDashboardToolbar
            selectedPlatform={usage.selectedPlatform}
            selectedRange={usage.selectedRange}
            importButtonLabel={usage.importButtonLabel}
            importing={usage.importing}
            runtimeUnavailable={usage.runtimeUnavailable}
            metaItems={!usage.runtimeUnavailable && usage.dashboardReady ? usage.dashboardMetaItems : []}
            onPlatformChange={usage.updateSelectedPlatform}
            onRangeChange={usage.updateSelectedRange}
            onImport={usage.doImport}
          />
          {usage.dashboardReady ? (
            <UsageStaleBanner
              presentation={usage.opsCockpit}
              onPrimaryAction={usage.handleOpsPrimaryAction}
              onSecondaryAction={handleSecondary}
            />
          ) : null}
          {usage.dashboardReady && usage.costSummaryCard ? (
            <section className="usage-hero-row">
              <UsageCostConclusionCard card={usage.costSummaryCard}>
                {usage.summary ? (
                  <UsageTokenBreakdownStrip
                    summary={usage.summary}
                    cacheCreationTokens={usage.cacheCreationTokens}
                  />
                ) : null}
              </UsageCostConclusionCard>
              {usage.otherSummaryCards.length > 0 ? (
                <div className="usage-metric-grid">
                  {usage.otherSummaryCards.map((card) => (
                    <UsageMetricCard key={card.id} card={card} />
                  ))}
                </div>
              ) : null}
            </section>
          ) : null}
          <div className="usage-workspace-switcher">
            <PillToggleGroup
              options={tabToggleOptions}
              value={usage.activeTab}
              onValueChange={handleTabChange}
            />
            <p className="usage-workspace-switcher__summary">
              {usage.selectedPlatformLabel} · {usage.selectedWindowLabel}
            </p>
          </div>
          <UsageDashboardStates
            usage={usage}
            hasDashboardData={hasDashboardData}
            runtimeCopy={runtimeCopy}
            onHome={handleHome}
            content={content}
          />
        </div>
        <LlmusageInstallDialog
          isOpen={usage.showInstallDialog}
          onOpenChange={usage.setShowInstallDialog}
          onRetryImport={usage.doImportAfterInstall}
        />
        <UsageDiagnosticsDrawer
          open={diagnosticsOpen}
          presentation={usage.opsCockpit}
          onOpenChange={setDiagnosticsOpen}
          onRefresh={usage.doImport}
        />
      </PageShell>
    </UsageDashboardProvider>
  )
}
