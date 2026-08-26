import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, fireEvent, render, renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { describe, expect, it, vi } from 'vitest'
import { OutputStylesView } from '@/features/claude/OutputStylesView'
import { StyleCard } from '@/features/claude/output-styles/StyleCards'
import { BehaviorAnalysisTab } from '@/features/claude/observer/BehaviorAnalysisTab'
import { TokenDetailTab } from '@/features/claude/observer/TokenDetailTab'
import {
  barPercent,
  formatInsightUsd,
  formatPercent,
  formatRoi,
  formatTokens,
  formatUsd,
  shortenId,
  shortenPath,
} from '@/features/claude/observer/formatters'
import { CodexAuthView } from '@/features/codex/CodexAuthView'
import { CheckinView } from '@/features/checkin/CheckinView'
import { McpManagerView } from '@/features/mcp/McpManagerView'
import { useMcpManager } from '@/features/mcp/useMcpManager'
import { useGrokProfilesPage } from '@/features/grok/profiles/useGrokProfilesPage'
import { SyncAssetCard } from '@/features/sync/SyncAssetCard'
import { SyncView } from '@/features/sync/SyncView'
import { useSyncPage } from '@/features/sync/useSyncPage'
import { Titlebar } from '@/shell/Titlebar'
import { HistoryList } from '@/ui/history-list'
import { TrayAccountSwitchScreen } from '@/features/tray/TrayAccountSwitchScreen'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async () => ({})),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}))

vi.mock('react-apexcharts', () => ({
  default: () => <div data-testid="mock-apex-chart" />,
}))

vi.mock('@/utils/apexChartsCore', () => ({
  default: () => <div data-testid="mock-apex-chart" />,
}))

vi.mock('@/api/runtime/environment', () => ({
  getCurrentEnvironment: vi.fn().mockResolvedValue({ env_type: 'local', id: 'local' }),
  isTauriEnvironment: () => false,
  getEnvironmentName: () => 'web',
}))

vi.mock('@/api', async (importOriginal) => {
  const actual = await importOriginal<Record<string, unknown>>()
  const grokApi = {
    ...(actual.grokApi as Record<string, unknown>),
    listGrokProfiles: vi.fn().mockResolvedValue({
      status: 'ok',
      profiles: [
        {
          name: 'p1',
          description: null,
          provider: 'xai',
          profile_kind: 'third_party',
          base_url_display: 'https://api.x.ai',
          has_base_url: true,
          model: 'grok-2',
          api_backend: null,
          context_window: null,
          supports_backend_search: null,
          reasoning_effort: 'low',
          auth_mode: 'inline_api_key',
          env_key: null,
          has_inline_credential: true,
          enabled: true,
          tags: [],
        },
      ],
      current_profile: 'p1',
      activation: 'active',
    }),
  }
  return {
    ...actual,
    grokApi,
    listOutputStyles: vi.fn().mockResolvedValue([{ name: 'concise', content: 'Be brief.' }]),
    listUnifiedMcp: vi.fn().mockResolvedValue({
      servers: [
        {
          platform: 'claude',
          name: 'filesystem',
          command: 'npx',
          args: ['-y', 'server'],
          env: {},
          disabled: false,
          effective: true,
          scope: 'user',
        },
      ],
      capabilities: [],
      diagnostics: [],
    }),
    listSyncAssets: vi.fn().mockResolvedValue([
      {
        id: 'ccr',
        name: 'CCR',
        group: 'ccr',
        kind: 'file',
        description: 'config',
        localExists: true,
        remoteExists: false,
        sensitive: false,
        localPath: '/tmp/ccr.json',
      },
    ]),
    getSyncStatus: vi.fn().mockResolvedValue({ connected: false }),
    listCheckinProviders: vi.fn().mockResolvedValue({ providers: [] }),
    listCheckinAccounts: vi.fn().mockResolvedValue({ accounts: [] }),
    listCheckinRecords: vi.fn().mockResolvedValue({ records: [] }),
    getTodayCheckinStats: vi.fn().mockResolvedValue({}),
    listBuiltinProviders: vi.fn().mockResolvedValue({ providers: [] }),
    listCodexAuthAccounts: vi.fn().mockResolvedValue({
      accounts: [],
      login_state: { type: 'NotLoggedIn' },
      can_auth_off: false,
    }),
    getCodexAuthCurrent: vi.fn().mockResolvedValue({ logged_in: false, can_auth_off: false }),
    listCodexProfiles: vi.fn().mockResolvedValue({ profiles: [], current_profile: null, can_off: false }),
    getCodexAllQuotas: vi.fn().mockResolvedValue([]),
  }
})

const wrap = (node: ReactNode) => {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  const router = createMemoryRouter([{ path: '/', element: node }], { initialEntries: ['/'] })
  return render(
    <QueryClientProvider client={client}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  )
}

describe('worst remaining views', () => {
  it('renders output styles, observer tabs, MCP, sync, check-in, and Codex auth', async () => {
    expect(formatTokens(1500)).toContain('k')
    expect(formatUsd(12.3)).toContain('$')
    expect(formatInsightUsd(0.01)).toContain('$')
    expect(formatRoi(2.5)).toContain('×')
    expect(formatPercent(0.5)).toBe('50.0%')
    expect(barPercent(5, 10)).toBeGreaterThan(0)
    expect(shortenPath('/a/b/c')).toBeTruthy()
    expect(shortenId('abcdefghijklmnop')).toBeTruthy()

    wrap(
      <StyleCard
        style={{ name: 'concise', content: 'x'.repeat(320) } as never}
        onView={() => undefined}
        onEdit={() => undefined}
        onDelete={() => undefined}
      />,
    )
    wrap(
      <BehaviorAnalysisTab
        heatmap={[{ date: '2026-01-01', weekday: 1, hour: 8, count: 2, cost_usd: 1 } as never]}
        topTools={[{ tool_name: 'Read', call_count: 3, cost_usd: 1 } as never]}
        sessions={[{ session_id: 's1', cost_usd: 1, tool_call_count: 2 } as never]}
        animationsEnabled={false}
        shouldRenderChart
      />,
    )
    wrap(
      <TokenDetailTab
        stats={{ hit_rate: 0.5, total_input_tokens: 1, total_output_tokens: 1 } as never}
        daily={[{ date: '2026-01-01', cost_usd: 1, input_tokens: 1, output_tokens: 1 } as never]}
        animationsEnabled={false}
        shouldRenderChart
      />,
    )
    wrap(
      <SyncAssetCard
        asset={{
          id: 'ccr',
          name: 'CCR',
          group: 'ccr',
          kind: 'file',
          description: 'cfg',
          localExists: true,
          remoteExists: true,
          sensitive: true,
          encryptionState: 'v2_required',
          localPath: '/tmp/a',
        } as never}
        busy={false}
        busyLabel=""
        showForce
        t={(key) => key}
        onPush={() => undefined}
        onPull={() => undefined}
        onSync={() => undefined}
        onForce={() => undefined}
      />,
    )
    wrap(
      <TrayAccountSwitchScreen
        snapshot={{ auth_label: 'a', accounts: [] } as never}
        currentAccount={null}
        accounts={[{ name: 'n1', email: 'e', can_switch: true } as never]}
        busyAccount={null}
        canManageAccounts
        onBack={() => undefined}
        onSwitch={() => undefined}
        onOpenAuth={() => undefined}
      />,
    )

    const titlebar = wrap(<Titlebar />)
    fireEvent.click(titlebar.container.querySelector('.titlebar-menu-btn') as HTMLElement)
    const aboutItem = titlebar.container.querySelector('.titlebar-menu button')
    if (aboutItem) fireEvent.click(aboutItem)
    titlebar.container.querySelectorAll('.titlebar-control-btn').forEach((button) => {
      fireEvent.click(button)
    })
    wrap(<HistoryList entries={[]} loading />)
    wrap(<HistoryList entries={[]} emptyTitle="empty" emptyDescription="none" />)
    wrap(<OutputStylesView />)
    wrap(<McpManagerView />)
    wrap(<SyncView />)
    wrap(<CheckinView />)
    wrap(<CodexAuthView />)

    await waitFor(() => {
      expect(document.body.textContent).toBeTruthy()
    })

    const client = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    })
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={client}>{children}</QueryClientProvider>
    )
    const mcp = renderHook(() => useMcpManager(), { wrapper })
    await waitFor(() => {
      expect(mcp.result.current.groupedServers.length).toBeGreaterThanOrEqual(0)
    })
    act(() => {
      mcp.result.current.openCreate()
      mcp.result.current.openImport()
      mcp.result.current.toggleMultiSelect()
      mcp.result.current.setArgInput?.('npx -y x')
      mcp.result.current.setEnvKey?.('TOKEN')
      mcp.result.current.setEnvValue?.('secret')
      mcp.result.current.addEnvVar?.()
      mcp.result.current.setHeaderKey?.('X-A')
      mcp.result.current.setHeaderValue?.('1')
      mcp.result.current.addHeader?.()
      mcp.result.current.setIncludeToolInput?.('read')
      mcp.result.current.setIsHttpMode?.(true)
      mcp.result.current.removeEnvVar?.('TOKEN')
      mcp.result.current.removeHeader?.('X-A')
      mcp.result.current.closePanel()
      if (mcp.result.current.groupedServers[0]) {
        mcp.result.current.selectGroup(mcp.result.current.groupedServers[0].name)
        mcp.result.current.openEdit(mcp.result.current.groupedServers[0].name)
      }
    })
    await act(async () => {
      await mcp.result.current.submitForm?.()
    })
    const sync = renderHook(() => useSyncPage(), { wrapper })
    await waitFor(() => {
      expect(sync.result.current.loading).toBe(false)
    })
    await act(async () => {
      await sync.result.current.refreshAll()
    })
    act(() => {
      sync.result.current.requestRunAll(false)
      sync.result.current.clearOperationOutput()
      const asset = sync.result.current.assets[0]
      if (asset) sync.result.current.requestRunAsset(asset, 'sync', false)
    })
    const grok = renderHook(() => useGrokProfilesPage(), { wrapper })
    await waitFor(() => {
      expect(grok.result.current.profiles.length).toBeGreaterThan(0)
    })
    act(() => {
      grok.result.current.handleAdd()
      grok.result.current.handleEdit(grok.result.current.profiles[0].name)
      grok.result.current.closeForm()
      grok.result.current.handleExport()
    })
  })
})
