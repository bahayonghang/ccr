import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { render, within } from '@testing-library/react'
import type { ReactElement } from 'react'
import { describe, expect, it, vi } from 'vitest'
import { UsageDashboardProvider } from '@/features/usage/UsageDashboardContext'
import { UsageCostTab } from '@/features/usage/components/UsageCostTab'
import { UsageOverviewTab } from '@/features/usage/components/UsageOverviewTab'
import { UsageModelsTab, UsageProjectsTab, UsageProvidersTab } from '@/features/usage/components/UsageTableTabs'
import { UsageTokensTab } from '@/features/usage/components/UsageTokensTab'
import type { UsageDashboardController } from '@/features/usage/useUsageDashboard'
import { formatCost, formatTokens } from '@/views/usage/usageSummaryCards'
import {
  makeModelStat,
  makeProjectStat,
  makeProviderBreakdown,
} from '../helpers/usageFixtures'

vi.mock('react-apexcharts', () => ({
  default: () => <div data-testid="mock-apex-chart" />,
}))

vi.mock('@/utils/apexChartsCore', () => ({
  default: () => <div data-testid="mock-apex-chart" />,
}))

const stylesDir = join(dirname(fileURLToPath(import.meta.url)), '../../src/features/usage/styles')

const headerTexts = (view: ReturnType<typeof render>) =>
  view.getAllByRole('columnheader').map((node) => node.textContent ?? '')

const visibleTextNodes = (node: ParentNode): string[] => {
  const values: string[] = []
  const walker = document.createTreeWalker(node, NodeFilter.SHOW_TEXT)
  for (let current = walker.nextNode(); current; current = walker.nextNode()) {
    const value = current.textContent?.trim() ?? ''
    if (value) values.push(value)
  }
  return values
}

const baseUsage = {
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
  trends: [],
  summary: {
    total_cost_usd: 1,
    total_requests: 2,
    total_input_tokens: 3,
    total_output_tokens: 4,
    total_cache_read_tokens: 5,
  },
  trendSubtitle: 'sub',
  trendGranularityLabel: 'Daily',
  shouldRenderTrendChart: false,
  hasRenderableTrendData: false,
  trendOptions: {},
  trendSeries: [],
  formatCost,
  formatTokens,
  selectedPlatform: 'claude',
  updateSelectedPlatform: () => undefined,
  modelStats: [],
  projectStats: [],
  providerStats: [],
  pieOptions: {},
  pieSeries: [1],
  pieColors: ['#fff'],
  modelTokenPieOptions: {},
  modelTokenPieSeries: [1],
  modelDistribution: [],
  modelTokenDistribution: [],
  shouldRenderDistributionChart: false,
  distributionSubtitle: '',
  overviewHighlights: [],
  topModelRankings: [],
  topProjectRankings: [],
  logsRecords: [],
  logModelFilter: '',
  updateLogModelFilter: () => undefined,
} as unknown as UsageDashboardController

const wrap = (node: ReactElement, overrides: Partial<UsageDashboardController> = {}) =>
  render(
    <UsageDashboardProvider value={{ ...baseUsage, ...overrides } as UsageDashboardController}>
      {node}
    </UsageDashboardProvider>,
  )

describe('usage table layout', () => {
  it('shortens a long project path and keeps the full path in title or secondary', () => {
    const projectPath = 'D:/workspace/team/bahayonghang/ccr-desktop'
    const view = wrap(<UsageProjectsTab />, {
      projectStats: [
        makeProjectStat({
          project_path: projectPath,
          request_count: 4,
          total_tokens: 2000,
          total_cost: 12.5,
        }),
      ],
    })

    const headers = headerTexts(view)
    expect(headers).toHaveLength(5)
    expect(headers.join(' ')).toMatch(/project|Project|项目|usage\.dashboard\.table\.project/)

    const nameCell = within(view.getAllByRole('row')[1]!).getAllByRole('cell')[0]!
    expect(nameCell.textContent).toContain('.../bahayonghang/ccr-desktop')
    expect(nameCell.textContent).toContain(projectPath)
    expect(nameCell.querySelector('[title]')?.getAttribute('title')).toBe(projectPath)
  })

  it('uses a fallback label for a null provider instead of a blank primary', () => {
    const view = wrap(<UsageProvidersTab />, {
      providerStats: [
        makeProviderBreakdown({
          provider: null,
          request_count: 3,
          total_tokens: 100,
          cost_with_cache_usd: 4.99,
        }),
      ],
    })

    const headers = headerTexts(view)
    expect(headers).toHaveLength(5)
    expect(headers.join(' ')).toMatch(/provider|Provider|usage\.dashboard\.table\.provider/)

    const nameCell = within(view.getAllByRole('row')[1]!).getAllByRole('cell')[0]!
    expect(nameCell.textContent?.trim()).toBe('unknown')
  })

  it('splits model name and cost into separate cells and shows share percents', () => {
    const view = wrap(<UsageModelsTab />, {
      modelStats: [
        makeModelStat({
          model: 'gpt-5.6-sol',
          request_count: 10,
          total_tokens: 1000,
          total_cost: 13955.57,
          cost_with_cache: 13955.57,
          pricing_status: 'priced',
        }),
        makeModelStat({
          model: 'claude-opus',
          request_count: 8,
          total_tokens: 800,
          total_cost: 13955.57,
          cost_with_cache: 13955.57,
          pricing_status: 'priced',
        }),
      ],
    })

    const headers = headerTexts(view)
    expect(headers).toHaveLength(6)
    expect(headers.join(' ')).toMatch(/model|Model|模型|usage\.dashboard\.table\.model/)
    expect(view.container.querySelector('.usage-ledger')).toBeTruthy()

    const firstRow = view.getAllByRole('row')[1]!
    const cells = within(firstRow).getAllByRole('cell')
    expect(cells).toHaveLength(6)
    expect(cells[0]?.textContent).toContain('gpt-5.6-sol')
    expect(cells[0]?.textContent).not.toMatch(/\$/)
    expect(cells[3]?.textContent).toMatch(/\$/)
    expect(cells[4]?.textContent).toMatch(/50(\.0)?%/)
    expect(
      visibleTextNodes(firstRow).some((value) => /gpt-5\.6-sol/.test(value) && /\$/.test(value)),
    ).toBe(false)
  })

  it('renders noData for empty stats without a headered ledger', () => {
    const view = wrap(<UsageModelsTab />, { modelStats: [] })
    expect(view.container.textContent).toMatch(/noData|No data|暂无数据/)
    expect(view.queryByRole('table')).toBeNull()
    expect(view.queryAllByRole('row')).toHaveLength(0)
  })

  it('renders the Tokens daily ledger when a trend exists', () => {
    const view = wrap(<UsageTokensTab />, {
      trends: [
        {
          date: '2026-01-01',
          request_count: 1,
          total_tokens: 150,
          input_tokens: 70,
          output_tokens: 50,
          reasoning_output_tokens: 0,
          cache_read_tokens: 25,
          cache_creation_tokens: 5,
          cost_usd: 1,
        },
      ],
    })

    const headers = headerTexts(view)
    expect(headers).toHaveLength(5)
    const joined = headers.join(' ')
    expect(joined).toMatch(/date|Date|日期|usage\.dashboard\.tokens\.date/)
    expect(joined).toMatch(/input|Input|输入|usage\.dashboard\.table\.input/)
    expect(joined).toMatch(/output|Output|输出|usage\.dashboard\.table\.output/)
    expect(joined).toMatch(/cacheRead|Cache Read|缓存读取|usage\.dashboard\.table\.cacheRead/)
    expect(joined).toMatch(/total|Total|总量|usage\.dashboard\.tokens\.modes\.total/)
  })

  it('does not render a Tokens ledger when there are no trends', () => {
    const view = wrap(<UsageTokensTab />, { trends: [] })
    expect(view.container.textContent).toMatch(/noData|No data|暂无数据/)
    expect(view.queryByRole('table')).toBeNull()
  })

  it('keeps overview rank items with index plus name and value', () => {
    const view = wrap(<UsageOverviewTab />, {
      topModelRankings: [
        {
          id: 'gpt-5.6-sol',
          label: 'gpt-5.6-sol',
          title: 'gpt-5.6-sol',
          detail: '12 requests · 1.0K',
          value: '$13.00',
          share: 0.62,
        },
      ],
      topProjectRankings: [
        {
          id: 'D:/Documents/Code/Github/ccr',
          label: '.../Github/ccr',
          title: 'D:/Documents/Code/Github/ccr',
          detail: '1.0K · 3 requests',
          value: '$4.00',
          share: 0.4,
        },
      ],
    })

    const item = view.container.querySelector('.overview-tab__rank-item')
    expect(item?.querySelector('.overview-tab__rank-index')?.textContent).toBe('1')
    expect(item?.querySelector('.overview-tab__rank-label')?.textContent).toContain('gpt-5.6-sol')
    expect(item?.querySelector('.overview-tab__rank-value')?.textContent).toContain('$13.00')
    expect(item?.children.length).toBe(2)

    const projectItem = view.container.querySelectorAll('.overview-tab__rank-item')[1]
    expect(projectItem?.textContent).toContain('Github/ccr')
    expect(projectItem?.querySelector('.overview-tab__rank-label')?.getAttribute('title'))
      .toBe('D:/Documents/Code/Github/ccr')
  })

  it('keeps cost ranking items with index plus name and cost', () => {
    const view = wrap(<UsageCostTab />)
    const item = view.container.querySelector('.cost-tab__ranking-item')
    expect(item?.querySelector('.cost-tab__rank')?.textContent).toBe('1')
    expect(item?.textContent).toContain('Claude')
    expect(item?.textContent).toContain('$1.00')
    expect(item?.children.length).toBe(2)
  })

  it('removes the fake 106rem table min-width from models and providers css', () => {
    const modelsCss = readFileSync(join(stylesDir, 'usage-models-tab.css'), 'utf8')
    const providersCss = readFileSync(join(stylesDir, 'usage-providers-tab.css'), 'utf8')
    const ledgerCss = readFileSync(join(stylesDir, 'usage-ledger.css'), 'utf8')
    const tokensCss = readFileSync(join(stylesDir, 'usage-tokens-tab.css'), 'utf8')
    expect(modelsCss).not.toContain('106rem')
    expect(providersCss).not.toContain('88rem')
    expect(ledgerCss).not.toContain('106rem')
    expect(tokensCss).not.toContain('106rem')
    expect(tokensCss).not.toContain('min-width: 62rem')
    expect(ledgerCss).toContain('--usage-ledger-cols')
  })
})
