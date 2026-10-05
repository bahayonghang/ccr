import { describe, expect, it } from 'vitest'
import {
  buildDashboardPresentation,
  type DashboardPlatformSource,
} from '@/views/dashboard/dashboardPresentation'
import type { CliVersionEntry, SystemInfo } from '@/types'
import type { MonitoringEntry } from '@/composables/useMonitoringFeed'
import type { HomeUsageOverviewResponse } from '@/types/usage'
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

const createCliMap = (
  entries: CliVersionEntry[] = [cliEntry('claude-code'), cliEntry('codex'), cliEntry('antigravity')]
) => new Map(entries.map((entry) => [entry.platform, entry]))

const overview = (
  overrides: Partial<HomeUsageOverviewResponse> = {}
): HomeUsageOverviewResponse => ({
  summary: {
    total_sessions: 18,
    total_requests: 1240,
    total_tokens: 980000,
    active_days: 30,
    platforms: 4,
  },
  by_platform: {
    claude: { sessions: 8, requests: 520, tokens: 420000 },
    codex: { sessions: 7, requests: 640, tokens: 510000 },
    gemini: { sessions: 3, requests: 80, tokens: 50000 },
    opencode: { sessions: 0, requests: 40, tokens: 15000 },
  },
  series: [],
  archive: makeArchiveDiagnostics({
    archive_root: 'C:/Users/test/.ccr/analytics/usage.db',
    live_sources: 3,
    missing_sources: 0,
    deleted_sources: 0,
    archived_sessions: 18,
    recent_completed_at: null,
    history_completed_at: null,
  }),
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
  // 生成类型里 empty_reason 为 string | null（必填），undefined 不再合法；null 同为 falsy，不影响 readiness 断言。
  empty_reason: null,
  last_updated: '2026-04-29T09:00:00Z',
  ...overrides,
})

const createLog = (level: MonitoringEntry['level']): MonitoringEntry => ({
  id: `event-${level}`,
  timestamp: '2026-04-29T09:15:00Z',
  level,
  channel: 'usage',
  eventType: 'usage.import',
  source: 'test',
  message: `${level} event`,
})

const baseInput = () => ({
  backendStatus: 'ok' as const,
  isNativeRuntime: true,
  systemInfo,
  cliVersions: createCliMap(),
  cliVersionsLoaded: true,
  platforms,
  overview: overview(),
  usageLoading: false,
  usageError: null,
  logs: [] as MonitoringEntry[],
})

describe('dashboard presentation', () => {
  it('reports ready when backend, CLI, usage, and signals are healthy', () => {
    const presentation = buildDashboardPresentation(baseInput())

    expect(presentation.readiness.status).toBe('ready')
    expect(presentation.installedCliCount).toBe(3)
    expect(presentation.runtimeCliCount).toBe(3)
    expect(presentation.actions[0]?.id).toBe('command-runner')
    expect(
      presentation.platformRows.every((row) => row.state === 'ready' || row.state === 'managed')
    ).toBe(true)
  })

  it('prioritizes web-preview capability boundaries outside native runtime', () => {
    const presentation = buildDashboardPresentation({
      ...baseInput(),
      backendStatus: 'unsupported',
      isNativeRuntime: false,
      systemInfo: null,
      cliVersions: new Map(),
      overview: null,
    })

    expect(presentation.readiness.status).toBe('web-preview')
    expect(presentation.actions[0]?.id).toBe('web-preview-boundary')
  })

  it('surfaces missing CLI as an attention blocker and routes to that platform', () => {
    const presentation = buildDashboardPresentation({
      ...baseInput(),
      cliVersions: createCliMap([
        cliEntry('claude-code'),
        cliEntry('codex', { installed: false, status: 'not_installed', version: undefined }),
        cliEntry('antigravity'),
      ]),
    })

    expect(presentation.readiness.status).toBe('attention')
    expect(presentation.actions[0]).toMatchObject({
      id: 'install-runtime-cli',
      path: '/codex',
      detail: 'Codex',
    })
  })

  it('routes usage warmup and empty states to the usage dashboard without fake readiness', () => {
    const presentation = buildDashboardPresentation({
      ...baseInput(),
      overview: overview({
        empty_reason: 'no_usage_and_sessions',
        bootstrap: {
          ...overview().bootstrap,
          needs_usage_import: true,
          is_warm: false,
        },
      }),
    })

    expect(presentation.readiness.status).toBe('attention')
    expect(presentation.readiness.reasons.map((reason) => reason.key)).toContain(
      'dashboard.readiness.reasons.usageEmpty'
    )
    expect(presentation.actions.some((action) => action.id === 'open-usage')).toBe(true)
  })

  it('prioritizes recent errors before default actions', () => {
    const presentation = buildDashboardPresentation({
      ...baseInput(),
      logs: [createLog('info'), createLog('warn'), createLog('error')],
    })

    expect(presentation.readiness.status).toBe('attention')
    expect(presentation.signalCounts).toMatchObject({ errors: 1, warnings: 1, total: 3 })
    expect(presentation.actions[0]?.id).toBe('open-monitoring')
  })

  it('ignores frontend and runtime diagnostic channels in signal counts', () => {
    const presentation = buildDashboardPresentation({
      ...baseInput(),
      logs: [
        { ...createLog('error'), id: 'fe', channel: 'frontend' },
        { ...createLog('error'), id: 'rt', channel: 'runtime' },
        createLog('error'),
      ],
    })

    expect(presentation.signalCounts).toMatchObject({ errors: 1, warnings: 0, total: 1 })
    expect(presentation.actions[0]?.id).toBe('open-monitoring')
  })

  it('derives sparkline requests in series date order and maps gemini to antigravity', () => {
    const presentation = buildDashboardPresentation({
      ...baseInput(),
      overview: overview({
        series: [
          {
            date: '2026-04-01',
            claude: { sessions: 0, requests: 1, tokens: 0 },
            codex: { sessions: 0, requests: 4, tokens: 0 },
            antigravity: { sessions: 0, requests: 9, tokens: 0 },
            opencode: { sessions: 0, requests: 2, tokens: 0 },
          },
          {
            date: '2026-04-02',
            claude: { sessions: 0, requests: 3, tokens: 0 },
            codex: { sessions: 0, requests: 5, tokens: 0 },
            antigravity: { sessions: 0, requests: 8, tokens: 0 },
            opencode: { sessions: 0, requests: 0, tokens: 0 },
          },
        ],
      }),
    })

    expect(presentation.platformRows.find((row) => row.usageKey === 'claude')?.sparkline).toEqual([1, 3])
    expect(presentation.platformRows.find((row) => row.usageKey === 'codex')?.sparkline).toEqual([4, 5])
    expect(presentation.platformRows.find((row) => row.usageKey === 'gemini')?.sparkline).toEqual([9, 8])
    expect(presentation.platformRows.find((row) => row.usageKey === 'opencode')?.sparkline).toEqual([2, 0])
  })

  it('leaves sparkline undefined when overview is null or series is empty', () => {
    const withoutOverview = buildDashboardPresentation({
      ...baseInput(),
      overview: null,
    })
    expect(withoutOverview.platformRows.every((row) => row.sparkline === undefined)).toBe(true)
    expect(withoutOverview.platformRows.every((row) => row.trackingHealth === undefined)).toBe(true)

    const emptySeries = buildDashboardPresentation({
      ...baseInput(),
      overview: overview({ series: [] }),
    })
    expect(emptySeries.platformRows.every((row) => row.sparkline === undefined)).toBe(true)
  })

  it('maps source_health onto trackingHealth and accepts gemini source aliases', () => {
    const viaSourceId = buildDashboardPresentation({
      ...baseInput(),
      overview: overview({
        archive: makeArchiveDiagnostics({
          source_health: [makeSourceHealth({ source: 'antigravity', state: 'missing' })],
        }),
      }),
    })
    expect(viaSourceId.platformRows.find((row) => row.usageKey === 'gemini')?.trackingHealth).toBe(
      'missing',
    )
    expect(viaSourceId.platformRows.find((row) => row.usageKey === 'gemini')?.sparkline).toBeUndefined()
    expect(
      viaSourceId.platformRows.find((row) => row.usageKey === 'gemini')?.metrics[0]?.valueKey,
    ).toBe('dashboard.platforms.untracked')

    const viaUsageKey = buildDashboardPresentation({
      ...baseInput(),
      overview: overview({
        archive: makeArchiveDiagnostics({
          source_health: [makeSourceHealth({ source: 'gemini', state: 'degraded' })],
        }),
      }),
    })
    expect(viaUsageKey.platformRows.find((row) => row.usageKey === 'gemini')?.trackingHealth).toBe(
      'degraded',
    )
  })

  it('does not treat an all-zero series as missing when source_health is empty', () => {
    const zeroStats = { sessions: 0, requests: 0, tokens: 0 }
    const presentation = buildDashboardPresentation({
      ...baseInput(),
      overview: overview({
        by_platform: {
          claude: zeroStats,
          codex: zeroStats,
          gemini: zeroStats,
          opencode: zeroStats,
        },
        series: [
          {
            date: '2026-04-01',
            claude: zeroStats,
            codex: zeroStats,
            antigravity: zeroStats,
            opencode: zeroStats,
          },
        ],
      }),
    })

    expect(presentation.platformRows.every((row) => row.trackingHealth === undefined)).toBe(true)
    expect(presentation.platformRows.find((row) => row.usageKey === 'claude')?.sparkline).toEqual([0])
    expect(presentation.platformRows.find((row) => row.usageKey === 'claude')?.metrics[0]?.value).toBe(
      '0',
    )
  })

  it('marks sessions as unindexed or indexing from the overview bootstrap flags', () => {
    const indexed = buildDashboardPresentation(baseInput())
    expect(indexed.sessionIndexState).toBeNull()

    const unindexed = buildDashboardPresentation({
      ...baseInput(),
      overview: overview({
        summary: { total_sessions: 0, total_requests: 1240, total_tokens: 980000, active_days: 30, platforms: 4 },
        bootstrap: { ...overview().bootstrap, needs_session_index: true, is_warm: false },
      }),
    })
    expect(unindexed.sessionIndexState).toBe('unindexed')

    const indexing = buildDashboardPresentation({
      ...baseInput(),
      overview: overview({
        summary: { total_sessions: 0, total_requests: 1240, total_tokens: 980000, active_days: 30, platforms: 4 },
        bootstrap: { ...overview().bootstrap, needs_session_index: true, is_warm: false },
        snapshot: {
          ...overview().snapshot,
          readiness: { ...overview().snapshot.readiness, active_session_index: true },
        },
      }),
    })
    expect(indexing.sessionIndexState).toBe('indexing')
  })
})
