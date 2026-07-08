import { createApp, nextTick } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import UsageLogsTab from '@/components/usage/UsageLogsTab.vue'
import { withUsageDashboardContext } from './helpers/usageDashboardContextStub'

const mountLogsTab = async (repairRecommended = false) => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const repairSpy = vi.fn(async () => undefined)

  const app = createApp(
    withUsageDashboardContext(UsageLogsTab, {
      diagnosticsEmptyDetail: 'No detail',
      diagnosticsEmptyMessage: 'No logs',
      diagnosticsSummary: {
        totalRecords: '42',
        latestRecordAt: '2026-04-01 10:00:00',
        healthLabel: repairRecommended ? 'Repair needed' : 'Healthy',
        healthDetail: repairRecommended ? 'Codex history is incomplete' : 'Looks good',
        repairRecommended,
        canRepairCodex: repairRecommended,
      },
      logModelFilter: '',
      logsLoading: false,
      logsRecords: [
        {
          id: 'row-1',
          platform: 'codex',
          project_path: '/tmp/project',
          record_json: '{}',
          recorded_at: '2026-04-01T02:25:23.755Z',
          source_id: 'source-1',
          model: null,
          input_tokens: 120,
          output_tokens: 24,
          cache_read_tokens: 0,
          cache_creation_tokens: 0,
          cost_with_cache_usd: 0,
          cost_without_cache_usd: 0,
          pricing_status: 'missing',
          pricing_source: null,
        },
      ],
      logsPage: 1,
      logsTotalPages: 1,
      canPrevLogs: false,
      canNextLogs: false,
      hasLogsTotal: true,
      showLogsPager: false,
      repairCodexButtonLabel: 'Repair Codex history',
      formatCost: (value: number) => `$${value.toFixed(2)}`,
      formatTokens: (value: number) => `${value}`,
      loadLogs: vi.fn(async () => undefined),
      repairCodexLogs: repairSpy,
      updateLogModelFilter: vi.fn(),
    })
  )

  app.config.globalProperties.$t = (key: string) => key
  app.mount(el)
  await nextTick()
  await nextTick()

  return {
    el,
    repairSpy,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

afterEach(() => {
  document.body.innerHTML = ''
})

describe('usage logs tab smoke', () => {
  it('renders raw usage rows instead of an empty ledger', async () => {
    const { el, unmount } = await mountLogsTab()

    try {
      expect(el.textContent).toContain('codex')
      expect(el.textContent).toContain('120')
      expect(el.textContent).toContain('usage.dashboard.diagnostics.unknownModel')
      expect(el.querySelectorAll('.diagnostics-tab__row--item')).toHaveLength(1)
    } finally {
      unmount()
    }
  })

  it('shows the codex repair callout when repair is recommended', async () => {
    const { el, repairSpy, unmount } = await mountLogsTab(true)

    try {
      const button = el.querySelector('.diagnostics-tab__repair-button') as HTMLButtonElement | null
      expect(button?.textContent).toContain('Repair Codex history')
      button?.click()
      await nextTick()
      expect(repairSpy).toHaveBeenCalledTimes(1)
    } finally {
      unmount()
    }
  })
})
