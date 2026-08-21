import { createPinia } from 'pinia'
import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, h, nextTick } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import enUS from '@/i18n/locales/en-US'

const apiMocks = vi.hoisted(() => ({
  grokAuthCurrent: vi.fn(),
  grokAuthOff: vi.fn(),
}))

const uiMocks = vi.hoisted(() => ({
  requestConfirm: vi.fn(),
  showError: vi.fn(),
  showSuccess: vi.fn(),
}))

vi.mock('@/api/domains/grok', () => ({
  grokAuthCurrent: (...args: unknown[]) => apiMocks.grokAuthCurrent(...args),
  grokAuthOff: (...args: unknown[]) => apiMocks.grokAuthOff(...args),
}))

vi.mock('@/stores/ui', () => ({
  useUIStore: () => uiMocks,
}))

vi.mock('@/components/ModuleSubnav.vue', () => ({
  default: defineComponent({
    template: '<div data-testid="module-subnav" />',
  }),
}))

vi.mock('@/components/ui/Button.vue', () => ({
  default: defineComponent({
    emits: ['click'],
    setup(_props, { slots, emit, attrs }) {
      return () =>
        h(
          'button',
          {
            ...attrs,
            type: 'button',
            onClick: (event: MouseEvent) => emit('click', event),
          },
          slots.default?.(),
        )
    },
  }),
}))

const flush = async () => {
  await Promise.resolve()
  await nextTick()
  await Promise.resolve()
  await nextTick()
}

const mountView = async () => {
  const { default: GrokAuthView } = await import('@/views/grok/GrokAuthView.vue')
  const el = document.createElement('div')
  document.body.appendChild(el)

  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: '/', component: defineComponent({ template: '<div />' }) }],
  })
  const i18n = createI18n({
    legacy: false,
    locale: 'en-US',
    fallbackLocale: 'en-US',
    messages: { 'en-US': enUS },
  })
  const app = createApp(GrokAuthView)
  app.use(createPinia())
  app.use(i18n)
  app.use(router)
  await router.push('/')
  await router.isReady()
  app.mount(el)
  await flush()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

beforeEach(() => {
  vi.clearAllMocks()
  uiMocks.requestConfirm.mockResolvedValue(true)
  apiMocks.grokAuthCurrent.mockResolvedValue({
    status: 'ok',
    logged_in: true,
    can_auth_off: true,
  })
  apiMocks.grokAuthOff.mockResolvedValue({
    status: 'ok',
    changed: true,
    path: 'file',
    warnings: [],
  })
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('GrokAuthView smoke', () => {
  it('shows the signed-in session and auth off action when can_auth_off is true', async () => {
    const { el, unmount } = await mountView()

    try {
      expect(el.querySelector('[data-testid="grok-auth-status"]')?.textContent).toContain('Signed in')
      expect(el.querySelector('[data-testid="grok-auth-off"]')).not.toBeNull()
    } finally {
      unmount()
    }
  })

  it('does not invoke grokAuthOff when the danger confirmation is cancelled', async () => {
    uiMocks.requestConfirm.mockResolvedValue(false)
    const { el, unmount } = await mountView()

    try {
      el.querySelector<HTMLButtonElement>('[data-testid="grok-auth-off"] button')?.click()
      await flush()
      expect(uiMocks.requestConfirm).toHaveBeenCalledWith(
        expect.objectContaining({ type: 'danger' }),
      )
      expect(apiMocks.grokAuthOff).not.toHaveBeenCalled()
    } finally {
      unmount()
    }
  })
})
