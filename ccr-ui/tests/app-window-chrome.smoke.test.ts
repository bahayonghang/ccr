import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'

const mountAppWithChrome = async (showCustomTitlebar: boolean) => {
  vi.resetModules()

  vi.doMock('vue-router', () => ({
    useRoute: () => ({
      meta: {
        hideGlobalBackground: false,
      },
    }),
    useRouter: () => ({
      currentRoute: {
        value: {
          fullPath: '/',
        },
      },
      push: vi.fn(),
    }),
  }))

  const stub = (name: string) => ({
    default: defineComponent({
      name,
      setup() {
        return () => h('div', { 'data-stub': name })
      },
    }),
  })

  vi.doMock('@/utils/windowChrome', () => ({
    shouldUseCustomTitlebar: () => showCustomTitlebar,
    getWindowChromeTopInset: () => (showCustomTitlebar ? 36 : 0),
  }))
  vi.doMock('@/components/layout/Titlebar.vue', () => stub('Titlebar'))
  vi.doMock('@/components/common/AnimeBackground.vue', () => stub('AnimeBackground'))
  vi.doMock('@/components/common/ToastContainer.vue', () => stub('ToastContainer'))
  vi.doMock('@/components/common/GlobalConfirmDialog.vue', () => stub('GlobalConfirmDialog'))

  const App = (await import('@/App.vue')).default

  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      return () => h(App)
    },
  }))

  app.component(
    'RouterView',
    defineComponent({
      setup() {
        return () => h('div', { 'data-route-view': 'true' })
      },
    }),
  )

  app.mount(el)
  await nextTick()
  await nextTick()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

afterEach(() => {
  vi.resetModules()
  vi.clearAllMocks()
  document.body.innerHTML = ''
})

describe('App window chrome smoke', () => {
  it('adds top inset when the custom titlebar is active', async () => {
    const { el, unmount } = await mountAppWithChrome(true)

    try {
      const shell = el.querySelector('.h-screen.w-screen') as HTMLElement | null
      expect(shell?.style.paddingTop).toBe('36px')
    } finally {
      unmount()
    }
  })

  it('keeps the top inset empty when native chrome is active', async () => {
    const { el, unmount } = await mountAppWithChrome(false)

    try {
      const shell = el.querySelector('.h-screen.w-screen') as HTMLElement | null
      expect(shell?.style.paddingTop || '').toBe('')
    } finally {
      unmount()
    }
  })
})
