import { createApp, defineComponent, h, nextTick } from 'vue'
import { describe, expect, it, vi } from 'vitest'
import { buildUsageOpsCockpit } from '@/views/usage/usageOpsCockpit'
import type { UsageArchiveDiagnostics, UsageSnapshotProjection } from '@/types/usage'
import { createI18nStub } from './helpers/i18n-stub'

vi.mock('@/components/ui/SIcon.vue', () => ({
  default: defineComponent({
    props: {
      name: { type: String, required: true },
      size: { type: String, default: '' },
    },
    setup(props) {
      return () => h('span', { 'data-icon': props.name, class: props.size })
    },
  }),
}))

import UsageStaleBanner from '@/components/usage/UsageStaleBanner.vue'

const translate = (
  _key: string,
  values: Record<string, number | string> | undefined,
  fallback: string
) => {
  if (!values) return fallback
  return fallback.replace(/\{([a-zA-Z_][a-zA-Z0-9_]*)\}/g, (_match, key) => String(values[key]))
}

const archive: UsageArchiveDiagnostics = {
  archive_root: 'D:/archive',
  live_sources: 1,
  missing_sources: 1,
  deleted_sources: 0,
  archived_sessions: 12,
  recent_completed_at: '2026-05-25T09:00:00Z',
  history_completed_at: '2026-05-25T10:00:00Z',
  freshness: {
    state: 'stale',
    latest_completed_at: '2026-05-24T00:00:00Z',
    age_seconds: 90_000,
    stale_after_seconds: 86_400,
  },
  readiness: {
    state: 'stale',
    next_action: 'refresh_usage',
    detail: 'stale',
    has_live_sources: true,
    has_missing_sources: true,
    has_deleted_sources: false,
    active_usage_import: false,
    active_session_index: false,
    recent_completed_at: '2026-05-25T09:00:00Z',
  },
  source_health: [
    {
      source: 'codex',
      state: 'degraded',
      live_sources: 1,
      missing_sources: 1,
      deleted_sources: 0,
      recent_completed_at: '2026-05-25T09:00:00Z',
      history_completed_at: '2026-05-25T10:00:00Z',
      freshness: {
        state: 'stale',
        latest_completed_at: '2026-05-24T00:00:00Z',
        age_seconds: 90_000,
        stale_after_seconds: 86_400,
      },
    },
  ],
}

const snapshot: UsageSnapshotProjection = {
  generated_at: '2026-05-25T10:05:00Z',
  platform_scope: 'all',
  start_date: '2026-05-01',
  end_date: '2026-05-25',
  cache_ttl_seconds: 30,
  freshness: archive.freshness!,
  readiness: archive.readiness!,
  source_health: archive.source_health!,
  drilldown: {
    dimensions: ['source', 'project_path', 'session_id', 'branch'],
    supports_logs: true,
    supports_projects: true,
    supports_sessions: true,
  },
}

describe('usage stale banner presentation', () => {
  it('turns backend snapshot readiness into a humanized, action-bearing presentation', () => {
    const presentation = buildUsageOpsCockpit({
      archive,
      importDetails: [],
      importing: false,
      importJobBanner: null,
      importJobWarnings: [],
      lastUpdatedAt: null,
      loading: false,
      locale: 'en-US',
      selectedPlatformLabel: 'All Platforms',
      selectedWindowLabel: 'Last 30 Days',
      snapshot,
      translate,
      unsupportedSyncMessage: null,
      warningMessage: null,
    })

    expect(presentation.state).toBe('stale')
    expect(presentation.primaryAction).toBe('import')
    expect(presentation.primaryActionLabel).toBe('Refresh usage')
    expect(presentation.freshnessAgeLabel).toBe('25h ago')
    expect(presentation.healthItems.map((item) => item.id)).toEqual([
      'readiness',
      'freshness',
      'source-health',
      'snapshot-cache',
      'scope',
      'drilldown',
    ])
    // L/M/D 缩写人话化：主层与抽屉统一走 formatSourceCounts。
    expect(presentation.healthItems.find((item) => item.id === 'source-health')?.value).toBe(
      'Live 1 · Missing 1 · Deleted 0'
    )
    expect(presentation.sourceItems[0]).toMatchObject({
      id: 'codex',
      label: 'Codex',
      tone: 'warning',
      hint: 'Some sessions for this source are missing or older than the freshness window.',
    })
  })

  it('renders a compact warning row and emits banner actions', async () => {
    const presentation = buildUsageOpsCockpit({
      archive,
      importDetails: ['codex: missing source'],
      importing: false,
      importJobBanner: null,
      importJobWarnings: [],
      lastUpdatedAt: null,
      loading: false,
      locale: 'en-US',
      selectedPlatformLabel: 'All Platforms',
      selectedWindowLabel: 'Last 30 Days',
      snapshot,
      translate,
      unsupportedSyncMessage: null,
      warningMessage: 'Some sources are degraded',
    })
    const onPrimary = vi.fn()
    const onSecondary = vi.fn()
    const el = document.createElement('div')
    document.body.appendChild(el)
    const app = createApp(UsageStaleBanner, {
      presentation,
      onPrimaryAction: onPrimary,
      onSecondaryAction: onSecondary,
    })
    app.use(createI18nStub())
    app.mount(el)
    await nextTick()

    try {
      const text = el.textContent ?? ''
      expect(text).toContain('Usage data is stale')
      expect(text).toContain('25h ago')
      // 详情类文案（来源清单、告警明细）已迁入诊断抽屉，横幅本身不再渲染它们。
      expect(text).not.toContain('Codex')
      expect(text).not.toContain('Some sources are degraded')

      const buttons = el.querySelectorAll('button')
      expect(buttons).toHaveLength(2)
      ;(buttons[0] as HTMLButtonElement).click()
      ;(buttons[1] as HTMLButtonElement).click()

      expect(onPrimary).toHaveBeenCalledWith('import')
      expect(onSecondary).toHaveBeenCalled()
    } finally {
      app.unmount()
      el.remove()
    }
  })

  it('does not render once the readiness state is healthy', async () => {
    const readyArchive: UsageArchiveDiagnostics = {
      ...archive,
      missing_sources: 0,
      freshness: { ...archive.freshness!, state: 'fresh', age_seconds: 30 },
      readiness: { ...archive.readiness!, state: 'ready', next_action: null },
    }
    const presentation = buildUsageOpsCockpit({
      archive: readyArchive,
      importDetails: [],
      importing: false,
      importJobBanner: null,
      importJobWarnings: [],
      lastUpdatedAt: null,
      loading: false,
      locale: 'en-US',
      selectedPlatformLabel: 'All Platforms',
      selectedWindowLabel: 'Last 30 Days',
      snapshot: {
        ...snapshot,
        freshness: readyArchive.freshness!,
        readiness: readyArchive.readiness!,
      },
      translate,
      unsupportedSyncMessage: null,
      warningMessage: null,
    })

    expect(presentation.state).toBe('ready')
    expect(presentation.freshnessAgeLabel).toBeNull()

    const el = document.createElement('div')
    document.body.appendChild(el)
    const app = createApp(UsageStaleBanner, { presentation })
    app.use(createI18nStub())
    app.mount(el)
    await nextTick()

    try {
      expect(el.querySelector('[data-usage-stale-banner]')).toBeNull()
    } finally {
      app.unmount()
      el.remove()
    }
  })
})
