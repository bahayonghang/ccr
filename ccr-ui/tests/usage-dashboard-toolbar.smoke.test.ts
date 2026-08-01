import { createApp, nextTick } from 'vue'
import { describe, expect, it, vi } from 'vitest'
import UsageDashboardToolbar from '@/components/usage/UsageDashboardToolbar.vue'
import { createI18nStub } from './helpers/i18n-stub'

describe('UsageDashboardToolbar smoke', () => {
  it('renders and emits all seven canonical llmusage source filters', async () => {
    const el = document.createElement('div')
    document.body.appendChild(el)
    const onPlatformUpdate = vi.fn()
    const app = createApp(UsageDashboardToolbar, {
      selectedPlatform: '',
      selectedRange: 'last_30d',
      importButtonLabel: 'Import',
      importing: false,
      runtimeUnavailable: false,
      metaItems: [],
      'onUpdate:selectedPlatform': onPlatformUpdate,
    })
    app.use(createI18nStub())
    app.mount(el)
    await nextTick()

    try {
      const select = el.querySelector('.usage-dashboard-toolbar__select') as HTMLSelectElement
      const options = Array.from(select.options)

      expect(options.map((option) => option.value)).toEqual([
        '',
        'claude',
        'codex',
        'opencode',
        'antigravity',
        'kimi_code',
        'pi',
        'grok',
      ])
      expect(options.map((option) => option.textContent?.trim())).toEqual([
        'All Platforms',
        'Claude',
        'Codex',
        'OpenCode',
        'Antigravity CLI',
        'Kimi Code',
        'Pi / Oh My Pi',
        'Grok Build',
      ])

      select.value = 'antigravity'
      select.dispatchEvent(new Event('change'))
      expect(onPlatformUpdate).toHaveBeenCalledWith('antigravity')
    } finally {
      app.unmount()
      el.remove()
    }
  })
})
