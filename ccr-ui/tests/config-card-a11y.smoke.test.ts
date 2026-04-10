import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import enUS from '@/i18n/locales/en-US'
import type { ConfigItem } from '@/types'

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

const i18n = createI18n({
  legacy: false,
  locale: 'en-US',
  fallbackLocale: 'en-US',
  missingWarn: false,
  fallbackWarn: false,
  messages: { 'en-US': enUS },
})

const config: ConfigItem = {
  name: 'anthropic-work',
  description: 'Primary work config',
  base_url: 'https://example.test',
  auth_token: 'masked',
  model: 'claude-sonnet',
  is_current: false,
  is_default: false,
  provider: 'anthropic',
  provider_type: 'official',
  usage_count: 7,
  enabled: true,
}

const mountCard = async () => {
  const { default: ConfigCard } = await import('@/components/ConfigCard.vue')
  const edit = vi.fn()
  const switchConfig = vi.fn()
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      return () => h(ConfigCard, {
        config,
        onEdit: edit,
        onSwitch: switchConfig,
      })
    },
  }))
  app.use(i18n)
  app.mount(el)
  await nextTick()

  return {
    el,
    edit,
    switchConfig,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

afterEach(() => {
  document.body.innerHTML = ''
  vi.clearAllMocks()
})

describe('ConfigCard accessibility semantics', () => {
  it('uses separate semantic controls for edit and switch actions', async () => {
    const { el, edit, switchConfig, unmount } = await mountCard()

    try {
      const editButton = el.querySelector('button[aria-label="Edit: anthropic-work"]') as HTMLButtonElement | null
      const switchButton = el.querySelector('button[aria-label="Switch: anthropic-work"]') as HTMLButtonElement | null

      expect(editButton).toBeTruthy()
      expect(switchButton).toBeTruthy()

      editButton?.focus()
      expect(document.activeElement).toBe(editButton)

      editButton?.click()
      expect(edit).toHaveBeenCalledWith('anthropic-work')
      expect(switchConfig).not.toHaveBeenCalled()

      switchButton?.click()
      expect(switchConfig).toHaveBeenCalledWith('anthropic-work')
      expect(edit).toHaveBeenCalledTimes(1)
    } finally {
      unmount()
    }
  })
})
