import { createApp, nextTick } from 'vue'
import { describe, expect, it } from 'vitest'
import UsageProvidersTab from '@/components/usage/UsageProvidersTab.vue'
import type { ProviderBreakdown, UsageFeatureCapability } from '@/types/usage'
import { createI18nStub } from './helpers/i18n-stub'
import { withUsageDashboardContext } from './helpers/usageDashboardContextStub'

const providerStats: ProviderBreakdown[] = [
  {
    provider: 'anyrouter',
    request_count: 8,
    input_tokens: 1200,
    cache_read_tokens: 300,
    cache_creation_tokens: 100,
    output_tokens: 600,
    reasoning_output_tokens: 200,
    total_tokens: 2400,
    cost_with_cache_usd: 1.25,
    cost_without_cache_usd: 1.6,
  },
  {
    provider: null,
    request_count: 2,
    input_tokens: 400,
    cache_read_tokens: 0,
    cache_creation_tokens: 0,
    output_tokens: 200,
    reasoning_output_tokens: 0,
    total_tokens: 600,
    cost_with_cache_usd: 0.3,
    cost_without_cache_usd: 0.4,
  },
]

const supportedCapability: UsageFeatureCapability = {
  supported: true,
  reason: null,
  detail: null,
}

const mountProvidersTab = async (
  stats: ProviderBreakdown[] = providerStats,
  providerCapability: UsageFeatureCapability | null = supportedCapability
) => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(
    withUsageDashboardContext(UsageProvidersTab, {
      providerStats: stats,
      providerCapability,
      selectedWindowLabel: 'Last 7 days',
      formatCost: (value: number) => `$${value.toFixed(2)}`,
      formatTokens: (value: number) => value.toLocaleString(),
    })
  )

  app.use(createI18nStub())
  app.mount(el)
  await nextTick()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

describe('UsageProvidersTab smoke', () => {
  it('renders provider rows, unattributed usage, and official-equivalent cost labels', async () => {
    const mounted = await mountProvidersTab()

    try {
      const text = mounted.el.textContent ?? ''
      const rows = mounted.el.querySelectorAll('tbody tr')

      expect(text).toContain('Usage by relay provider')
      expect(text).toContain('Provider ledger')
      expect(text).toContain('Last 7 days')
      expect(text).toContain('anyrouter')
      expect(text).toContain('Unattributed')
      expect(text).toContain('before the first activation window or after a clear event')
      expect(text).toContain('≈ official-equivalent price')
      expect(text).toContain('$1.55')
      expect(rows).toHaveLength(2)
      expect(rows[0]?.textContent).toContain('anyrouter')
      expect(rows[0]?.textContent).toContain('$1.25')
      expect(rows[1]?.textContent).toContain('Unattributed')
      expect(rows[1]?.textContent).toContain('$0.30')
    } finally {
      mounted.unmount()
    }
  })

  it('renders a localized unsupported state when provider breakdown is unavailable', async () => {
    const mounted = await mountProvidersTab([], {
      supported: false,
      reason: 'schema_unsupported',
      detail: 'provider_label column is missing.',
    })

    try {
      const text = mounted.el.textContent ?? ''

      expect(text).toContain('Provider attribution is unavailable')
      expect(text).toContain('provider_label column is missing.')
      expect(text).toContain('Upgrade llmusage, run sync with provider attribution')
      expect(text).not.toContain('Provider ledger')
      expect(mounted.el.querySelector('tbody tr')).toBeNull()
    } finally {
      mounted.unmount()
    }
  })
})
