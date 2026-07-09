import { createApp, defineComponent, h, nextTick } from 'vue'
import { describe, expect, it, vi } from 'vitest'
import { buildUsageOpsCockpit } from '@/views/usage/usageOpsCockpit'
import type { UsageArchiveDiagnostics, UsageSnapshotProjection } from '@/types/usage'
import { createI18nStub } from './helpers/i18n-stub'

vi.mock('@/components/common/BaseModal.vue', () => ({
  default: defineComponent({
    props: {
      modelValue: { type: Boolean, required: true },
      title: { type: String, default: '' },
    },
    setup(props, { slots }) {
      return () =>
        props.modelValue
          ? h('div', { 'data-title': props.title }, [h('h2', props.title), slots.default?.()])
          : null
    },
  }),
}))

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

import UsageDiagnosticsDrawer from '@/components/usage/UsageDiagnosticsDrawer.vue'

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

const mount = async (presentation: ReturnType<typeof buildUsageOpsCockpit>) => {
  const el = document.createElement('div')
  document.body.appendChild(el)
  const onRefresh = vi.fn()
  const app = createApp(UsageDiagnosticsDrawer, {
    modelValue: true,
    presentation,
    onRefresh,
  })
  app.use(createI18nStub())
  app.mount(el)
  await nextTick()

  return {
    el,
    onRefresh,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

describe('usage diagnostics drawer', () => {
  it('surfaces source health, hints, and alert detail behind the diagnostics entry point', async () => {
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

    const { el, onRefresh, unmount } = await mount(presentation)

    try {
      const text = el.textContent ?? ''
      expect(text).toContain('Usage diagnostics')
      expect(text).toContain('Codex')
      // 抽屉内 L/M/D 缩写同样人话化，不再出现裸字母速记。
      expect(text).toContain('Live 1 · Missing 1 · Deleted 0')
      expect(text).toContain(
        'Some sessions for this source are missing or older than the freshness window.'
      )
      expect(text).toContain('Some sources are degraded')

      const refreshButton = el.querySelector(
        '.usage-diag-source__refresh'
      ) as HTMLButtonElement | null
      expect(refreshButton).not.toBeNull()
      refreshButton?.click()
      expect(onRefresh).toHaveBeenCalled()
    } finally {
      unmount()
    }
  })

  it('hides the operational alerts section entirely when there is nothing to report', async () => {
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

    expect(presentation.alerts).toHaveLength(0)

    const { el, unmount } = await mount(presentation)

    try {
      const text = el.textContent ?? ''
      expect(text).not.toContain('Operational alerts')
      expect(text).not.toContain('No active import or warning')
    } finally {
      unmount()
    }
  })
})
