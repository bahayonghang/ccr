import { createApp, nextTick } from 'vue'
import { describe, expect, it, vi } from 'vitest'
import UsageOpsCockpit from '@/components/usage/UsageOpsCockpit.vue'
import { buildUsageOpsCockpit } from '@/views/usage/usageOpsCockpit'
import type { UsageArchiveDiagnostics, UsageSnapshotProjection } from '@/types/usage'
import { createI18nStub } from './helpers/i18n-stub'

const translate = (
  _key: string,
  values: Record<string, number | string> | undefined,
  fallback: string,
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

describe('usage ops cockpit presentation', () => {
  it('turns backend snapshot readiness into next-action cards', () => {
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
    expect(presentation.healthItems.map((item) => item.id)).toEqual([
      'readiness',
      'freshness',
      'source-health',
      'snapshot-cache',
      'scope',
      'drilldown',
    ])
    expect(presentation.healthItems.find((item) => item.id === 'source-health')?.value)
      .toBe('L 1 · M 1 · D 0')
    expect(presentation.sourceItems[0]).toMatchObject({
      id: 'codex',
      label: 'Codex',
      tone: 'warning',
    })
  })

  it('renders source health and emits cockpit actions', async () => {
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
    const app = createApp(UsageOpsCockpit, {
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
      expect(text).toContain('Codex')
      expect(text).toContain('Some sources are degraded')

      const buttons = el.querySelectorAll('button')
      ;(buttons[0] as HTMLButtonElement).click()
      ;(buttons[1] as HTMLButtonElement).click()

      expect(onPrimary).toHaveBeenCalledWith('import')
      expect(onSecondary).toHaveBeenCalled()
    } finally {
      app.unmount()
      el.remove()
    }
  })
})
