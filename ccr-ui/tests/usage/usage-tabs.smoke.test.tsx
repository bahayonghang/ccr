import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, renderHook } from '@testing-library/react'
import type { ReactElement, ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import { useCodexTrayPanel } from '@/composables/useCodexTrayPanel'
import { McpPresetsPanel } from '@/features/mcp/McpPresetsPanel'
import { McpSyncPanel } from '@/features/mcp/McpSyncPanel'
import { EnvironmentSwitcher } from '@/shell/EnvironmentSwitcher'
import { HistoryList } from '@/ui/history-list'
import { MarketplacePagination } from '@/ui/marketplace-pagination'
import { TrayOverview } from '@/features/tray/TrayOverview'
import { UsageDashboardProvider } from '@/features/usage/UsageDashboardContext'
import { UsageCostConclusionCard } from '@/features/usage/components/UsageCostConclusionCard'
import { UsageCostTab } from '@/features/usage/components/UsageCostTab'
import { UsageLogsTab } from '@/features/usage/components/UsageLogsTab'
import { UsageMetricCard } from '@/features/usage/components/UsageMetricCard'
import { UsageModelDistributionCard } from '@/features/usage/components/UsageModelDistributionCard'
import { UsageOverviewTab } from '@/features/usage/components/UsageOverviewTab'
import { UsageSourceSummaryCard } from '@/features/usage/components/UsageSourceSummaryCard'
import { UsageModelsTab, UsageProjectsTab, UsageProvidersTab } from '@/features/usage/components/UsageTableTabs'
import { UsageTokenBreakdownStrip } from '@/features/usage/components/UsageTokenBreakdownStrip'
import { UsageTokensTab } from '@/features/usage/components/UsageTokensTab'
import type { UsageDashboardController } from '@/features/usage/useUsageDashboard'

vi.mock('react-apexcharts', () => ({
  default: () => <div data-testid="mock-apex-chart" />,
}))

vi.mock('@/utils/apexChartsCore', () => ({
  default: () => <div data-testid="mock-apex-chart" />,
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async () => ({})),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}))

const summary = {
  total_cost_usd: 1,
  total_requests: 2,
  total_input_tokens: 3,
  total_output_tokens: 4,
  total_cache_read_tokens: 5,
} as never

const card = {
  id: 'cost',
  icon: 'DollarSign',
  tone: 'accent',
  label: 'Cost',
  value: '$1.00',
  detail: 'detail',
  sparkline: [{ value: 1 }, { value: 2 }],
} as never

const usage = {
  sourceStats: [
    {
      source: 'claude',
      share_tokens: 0.4,
      share_cost: 0.4,
      total_cost: 1,
      total_tokens: 10,
      event_count: 2,
      active_days: 1,
    },
  ],
  trends: [{ date: '2026-01-01', cost_usd: 1, cache_creation_tokens: 0 }],
  summary,
  trendSubtitle: 'sub',
  trendGranularityLabel: 'Daily',
  shouldRenderTrendChart: false,
  hasRenderableTrendData: false,
  trendOptions: {},
  trendSeries: [],
  formatCost: (value: number) => `$${value}`,
  formatTokens: (value: number) => String(value),
  selectedPlatform: 'claude',
  updateSelectedPlatform: () => undefined,
  modelStats: [{ model: 'gpt', total_tokens: 1, total_cost: 1, cost_with_cache: 1 }],
  projectStats: [{ project_path: '/p', total_tokens: 1, total_cost: 1 }],
  providerStats: [{ provider: 'openai', total_tokens: 1, total_cost: 1 }],
  pieOptions: {},
  pieSeries: [1],
  pieColors: ['#fff'],
  modelTokenPieOptions: {},
  modelTokenPieSeries: [1],
  modelDistribution: [{ model: 'gpt', share: 1, tokens: 1, cost: 1 }],
  modelTokenDistribution: [{ model: 'gpt', share: 1, tokens: 1, cost: 1 }],
  shouldRenderDistributionChart: false,
  distributionSubtitle: '',
  overviewHighlights: [],
  topModelRankings: [],
  topProjectRankings: [],
  logsRecords: [],
  logModelFilter: '',
  updateLogModelFilter: () => undefined,
} as unknown as UsageDashboardController

const wrap = (node: ReactElement) =>
  render(<UsageDashboardProvider value={usage}>{node}</UsageDashboardProvider>)

describe('usage dashboard tabs', () => {
  it('renders tab panels and summary cards from a stub controller', () => {
    wrap(<UsageOverviewTab />)
    wrap(<UsageTokensTab />)
    wrap(<UsageCostTab />)
    wrap(<UsageModelsTab />)
    wrap(<UsageProjectsTab />)
    wrap(<UsageProvidersTab />)
    wrap(<UsageLogsTab />)
    wrap(
      <UsageModelDistributionCard
        title="models"
        subtitle="sub"
        modelDistribution={[{ id: 'gpt', label: 'gpt' }]}
        pieColors={['#fff']}
        pieOptions={{}}
        pieSeries={[1]}
        shouldRenderChart={false}
      />,
    )
    wrap(
      <UsageSourceSummaryCard
        sourceStats={usage.sourceStats}
        selectedPlatform="claude"
        formatCost={usage.formatCost}
        formatTokens={usage.formatTokens}
        onSelectSource={() => undefined}
      />,
    )
    wrap(<UsageMetricCard card={card} />)
    wrap(<UsageCostConclusionCard card={card}>child</UsageCostConclusionCard>)
    wrap(<UsageTokenBreakdownStrip summary={summary} cacheCreationTokens={1} />)
    expect(document.body.textContent).toBeTruthy()
  })

  it('renders tray overview and mcp side panels', () => {
    const snapshot = {
      fetched_at: 'now',
      runtime_mode: 'codex',
      runtime_description: 'desc',
      profile_label: 'profile',
      auth_label: 'auth',
      login_state: { type: 'NotLoggedIn' },
      can_manage_accounts: true,
      accounts: [],
    }
    render(
      <TrayOverview
        snapshot={snapshot as never}
        currentAccount={null}
        canManageAccounts
        onOpenMain={() => undefined}
        onOpenSwitch={() => undefined}
        onOpenUsage={() => undefined}
        onOpenAuth={() => undefined}
        onQuit={() => undefined}
      />,
    )
    render(<McpPresetsPanel onInstalled={() => undefined} />)
    render(<McpSyncPanel onSynced={() => undefined} />)
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={client}>{children}</QueryClientProvider>
    )
    const { result } = renderHook(() => useCodexTrayPanel(), { wrapper })
    expect(result.current.screen).toBe('overview')
    render(<EnvironmentSwitcher />)
    render(
      <HistoryList
        entries={[
          {
            id: '1',
            timestamp: new Date().toISOString(),
            operation: 'switch',
            details: 'ok',
          } as never,
        ]}
      />,
    )
    render(<MarketplacePagination currentPage={3} totalItems={80} pageSize={10} onPageChange={() => undefined} />)
  })
})
