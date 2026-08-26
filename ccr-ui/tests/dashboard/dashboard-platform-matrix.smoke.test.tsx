import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { describe, expect, it } from 'vitest'
import { DashboardPlatformMatrix } from '@/features/usage/dashboard/DashboardPlatformMatrix'
import {
  buildDashboardPresentation,
  type DashboardPlatformSource,
  type DashboardPresentationInput,
} from '@/views/dashboard/dashboardPresentation'
import type { CliVersionEntry, SystemInfo } from '@/types'
import type { HomeOverviewSeriesItem, HomeUsageOverviewResponse } from '@/types/usage'
import { makeArchiveDiagnostics, makeSourceHealth } from '../helpers/usageFixtures'

const platforms: DashboardPlatformSource[] = [
  {
    title: 'Claude Code',
    desc: 'Claude runtime',
    path: '/claude-code',
    icon: 'Code2',
    iconClass: 'text-platform-claude',
    platformKey: 'claude-code',
    usageKey: 'claude',
    role: 'Core CLI',
    mode: 'cli',
    isRuntimeCli: true,
  },
  {
    title: 'Codex',
    desc: 'Codex runtime',
    path: '/codex',
    icon: 'Settings',
    iconClass: 'text-platform-codex',
    platformKey: 'codex',
    usageKey: 'codex',
    role: 'Core CLI',
    mode: 'cli',
    isRuntimeCli: true,
  },
  {
    title: 'Antigravity',
    desc: 'Antigravity runtime',
    path: '/antigravity',
    icon: 'Sparkles',
    iconClass: 'text-platform-gemini',
    platformKey: 'antigravity',
    usageKey: 'gemini',
    role: 'Core CLI',
    mode: 'cli',
    isRuntimeCli: true,
  },
  {
    title: 'OpenCode',
    desc: 'Managed runtime',
    path: '/opencode',
    icon: 'TerminalSquare',
    iconClass: 'text-accent-info',
    platformKey: 'opencode',
    usageKey: 'opencode',
    role: 'Managed',
    mode: 'managed',
    isRuntimeCli: false,
  },
]

const systemInfo: SystemInfo = {
  hostname: 'workstation',
  os: 'windows',
  os_name: 'Windows',
  os_version: '11',
  kernel_version: '10.0',
  arch: 'x86_64',
  cpu_brand: 'Test CPU',
  cpu_cores: 12,
  cpu_count: 12,
  cpu_usage: 11.4,
  total_memory_gb: 64,
  used_memory_gb: 17.5,
  memory_usage_percent: 27.3,
  total_memory_mb: 65536,
  total_swap_gb: 0,
  used_swap_gb: 0,
  uptime_seconds: 1200,
  ccr_version: '7.0.0',
}

const cliEntry = (platform: string, overrides: Partial<CliVersionEntry> = {}): CliVersionEntry => ({
  ...overrides,
  platform: overrides.platform ?? platform,
  installed: overrides.installed ?? true,
  version: overrides.version === undefined ? '1.0.0' : overrides.version,
  status: overrides.status ?? 'ok',
  elapsed_ms: overrides.elapsed_ms ?? 0,
})

const zeroStats = { sessions: 0, requests: 0, tokens: 0 }
const zeroSeriesItem = (date: string): HomeOverviewSeriesItem => ({
  date,
  claude: zeroStats,
  codex: zeroStats,
  antigravity: zeroStats,
  opencode: zeroStats,
})

const overview = (
  overrides: Partial<HomeUsageOverviewResponse> = {},
): HomeUsageOverviewResponse => ({
  summary: {
    total_sessions: 0,
    total_requests: 0,
    total_tokens: 0,
    active_days: 2,
    platforms: 4,
  },
  by_platform: {
    claude: zeroStats,
    codex: zeroStats,
    gemini: zeroStats,
    opencode: zeroStats,
  },
  series: [zeroSeriesItem('2026-04-01'), zeroSeriesItem('2026-04-02')],
  archive: makeArchiveDiagnostics(),
  bootstrap: {
    usage_import_attempted: false,
    usage_imported_records: 0,
    session_reindex_attempted: false,
    indexed_sessions: 0,
    usage_job_id: null,
    session_job_id: null,
    needs_usage_import: false,
    needs_session_index: false,
    is_warm: true,
  },
  snapshot: {
    generated_at: '2026-04-29T09:00:00Z',
    platform_scope: 'all',
    start_date: null,
    end_date: null,
    cache_ttl_seconds: 30,
    freshness: {
      state: 'fresh',
      latest_completed_at: '2026-04-29T08:00:00Z',
      age_seconds: 3600,
      stale_after_seconds: 86_400,
    },
    readiness: {
      state: 'ready',
      next_action: null,
      detail: 'ready',
      has_live_sources: true,
      has_missing_sources: false,
      has_deleted_sources: false,
      active_usage_import: false,
      active_session_index: false,
      recent_completed_at: null,
    },
    source_health: [],
    drilldown: {
      dimensions: [],
      supports_logs: true,
      supports_projects: true,
      supports_sessions: true,
    },
  },
  empty_reason: null,
  last_updated: '2026-04-29T09:00:00Z',
  ...overrides,
})

const baseInput = (
  overviewOverrides: Partial<HomeUsageOverviewResponse> = {},
): DashboardPresentationInput => ({
  backendStatus: 'ok',
  isNativeRuntime: true,
  systemInfo,
  cliVersions: new Map(
    [cliEntry('claude-code'), cliEntry('codex'), cliEntry('antigravity')].map((entry) => [
      entry.platform,
      entry,
    ]),
  ),
  cliVersionsLoaded: true,
  platforms,
  overview: overview(overviewOverrides),
  usageLoading: false,
  usageError: null,
  logs: [],
})

const renderMatrix = (input: DashboardPresentationInput) => {
  const presentation = buildDashboardPresentation(input)
  return {
    presentation,
    ...render(
      <MemoryRouter>
        <DashboardPlatformMatrix
          rows={presentation.platformRows}
          installedCliCount={presentation.installedCliCount}
          runtimeCliCount={presentation.runtimeCliCount}
        />
      </MemoryRouter>,
    ),
  }
}

describe('dashboard platform matrix', () => {
  it('shows a placeholder when source_health marks the platform missing', () => {
    const { presentation } = renderMatrix(
      baseInput({
        archive: makeArchiveDiagnostics({
          source_health: [
            makeSourceHealth({ source: 'claude', state: 'live' }),
            makeSourceHealth({ source: 'codex', state: 'missing', live_sources: 0, missing_sources: 1 }),
            makeSourceHealth({ source: 'antigravity', state: 'live' }),
            makeSourceHealth({ source: 'opencode', state: 'live' }),
          ],
        }),
      }),
    )

    expect(presentation.platformRows.find((row) => row.usageKey === 'codex')?.trackingHealth).toBe(
      'missing',
    )
    expect(presentation.platformRows.find((row) => row.usageKey === 'codex')?.sparkline).toBeUndefined()
    expect(screen.getByTestId('dashboard-platform-placeholder-codex')).toBeTruthy()
    expect(screen.getByTestId('dashboard-platform-codex').getAttribute('data-tracking')).toBe(
      'missing',
    )
    expect(screen.queryByTestId('dashboard-platform-spark-codex')).toBeNull()
  })

  it('treats gemini usageKey missing when source_health uses the gemini source id', () => {
    renderMatrix(
      baseInput({
        archive: makeArchiveDiagnostics({
          source_health: [
            makeSourceHealth({ source: 'claude', state: 'live' }),
            makeSourceHealth({ source: 'codex', state: 'live' }),
            makeSourceHealth({ source: 'gemini', state: 'missing', live_sources: 0, missing_sources: 1 }),
            makeSourceHealth({ source: 'opencode', state: 'live' }),
          ],
        }),
      }),
    )

    expect(screen.getByTestId('dashboard-platform-placeholder-antigravity')).toBeTruthy()
    expect(screen.getByTestId('dashboard-platform-antigravity').getAttribute('data-tracking')).toBe(
      'missing',
    )
  })

  it('does not treat an all-zero series as a missing placeholder', () => {
    renderMatrix(baseInput())

    expect(screen.queryByTestId('dashboard-platform-placeholder-codex')).toBeNull()
    expect(screen.getByTestId('dashboard-platform-spark-codex').children).toHaveLength(2)
    expect(screen.getByTestId('dashboard-platform-codex').getAttribute('data-tracking')).toBe(
      'unknown',
    )
  })

  it('does not render request zeroes when overview is absent', () => {
    const { presentation } = renderMatrix({
      ...baseInput(),
      backendStatus: 'unsupported',
      isNativeRuntime: false,
      overview: null,
    })

    expect(screen.queryByTestId('dashboard-platform-placeholder-codex')).toBeNull()
    expect(screen.queryByTestId('dashboard-platform-spark-codex')).toBeNull()
    expect(
      presentation.platformRows.every((row) =>
        row.metrics.every(
          (metric) => metric.valueKey === 'dashboard.platforms.untracked' && metric.value === undefined,
        ),
      ),
    ).toBe(true)
  })
})
