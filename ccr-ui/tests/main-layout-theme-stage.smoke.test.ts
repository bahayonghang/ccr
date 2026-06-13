import { createApp, defineComponent, h, reactive, nextTick } from 'vue'
import { createPinia } from 'pinia'
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest'

vi.mock('vue', async () => {
  const actual = await vi.importActual<typeof import('vue')>('vue')

  return {
    ...actual,
    defineAsyncComponent: () =>
      actual.defineComponent({
        name: 'AsyncComponentStub',
        setup() {
          return () => actual.h('div', { 'data-async-stub': 'true' })
        },
      }),
    Suspense: actual.defineComponent({
      name: 'SuspenseStub',
      setup(_props, { slots }) {
        return () => actual.h('div', { 'data-suspense-stub': 'true' }, slots.default?.())
      },
    }),
  }
})

const routeState = reactive({
  name: 'claude-code',
  fullPath: '/claude-code',
  meta: {
    group: 'claude-code' as string | undefined,
    hideGlobalBackground: true,
    hideSidebar: false,
  },
})

vi.mock('vue-router', async (importOriginal) => {
  const actual = await importOriginal<typeof import('vue-router')>()

  return {
    ...actual,
    useRoute: () => routeState,
    useRouter: () => ({
      back: vi.fn(),
    }),
  }
})

vi.mock('vue-i18n', async (importOriginal) => {
  const actual = await importOriginal<typeof import('vue-i18n')>()

  return {
    ...actual,
    useI18n: () => ({
      t: (key: string) => key,
    }),
  }
})

vi.mock('@/composables/usePageTransition', () => ({
  usePageTransition: () => ({
    transitionName: 'route-fade',
  }),
}))

vi.mock('@/composables/useMainLayoutShell', () => ({
  useMainLayoutShell: () => ({
    closeNavigationLabel: 'Close navigation',
    closeSidebar: vi.fn(),
    handleResizeKeydown: vi.fn(),
    isMobileSidebar: false,
    isResizing: false,
    isSidebarOpen: false,
    isTauri: false,
    showMobileBackdrop: false,
    sidebarShellStyle: { width: '240px' },
    sidebarToggleLabel: 'Toggle navigation',
    startResize: vi.fn(),
    toggleSidebar: vi.fn(),
  }),
}))

vi.mock('@/components/ui/SIcon.vue', () => ({
  default: defineComponent({
    props: {
      name: { type: String, required: true },
      size: { type: String, default: '' },
      class: { type: String, default: '' },
    },
    setup(props) {
      return () => h('span', { 'data-icon': props.name, class: [props.size, props.class] })
    },
  }),
}))

const asyncStub = (name: string) => ({
  default: defineComponent({
    name,
    setup() {
      return () => h('div', { 'data-stub': name })
    },
  }),
})

vi.mock('@/components/BackendStatusBanner.vue', () => asyncStub('BackendStatusBanner'))
vi.mock('@/components/EnvironmentSwitcher.vue', () => asyncStub('EnvironmentSwitcher'))

import MainLayout from '@/components/MainLayout.vue'

beforeAll(() => {
  vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) => {
    callback(0)
    return 1
  })

  vi.stubGlobal('cancelAnimationFrame', vi.fn())
})

afterAll(() => {
  vi.unstubAllGlobals()
})

const mountLayout = async () => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(
    defineComponent({
      setup() {
        return () => h(MainLayout)
      },
    })
  )

  app.use(createPinia())
  app.config.globalProperties.$t = (key: string) => key

  app.component(
    'RouterLink',
    defineComponent({
      props: {
        to: { type: [String, Object], required: true },
        custom: { type: Boolean, default: false },
      },
      setup(props, { slots }) {
        return () => {
          if (props.custom) {
            return slots.default?.({
              href: typeof props.to === 'string' ? props.to : '/',
              navigate: vi.fn(),
            })
          }

          return h('a', {}, slots.default?.())
        }
      },
    })
  )

  app.component(
    'RouterView',
    defineComponent({
      setup(_props, { slots }) {
        const DummyRouteComponent = defineComponent({
          name: 'DummyRouteComponent',
          setup() {
            return () => h('section', { 'data-route-body': 'true' }, 'route body')
          },
        })

        return () => slots.default?.({ Component: DummyRouteComponent })
      },
    })
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
  document.body.innerHTML = ''
})

describe('MainLayout theme stage smoke', () => {
  it('applies theme-stage classes when the route hides the global background', async () => {
    routeState.name = 'claude-code'
    routeState.fullPath = '/claude-code'
    routeState.meta.group = 'claude-code'
    routeState.meta.hideGlobalBackground = true
    routeState.meta.hideSidebar = false

    const { el, unmount } = await mountLayout()

    try {
      const shell = el.firstElementChild
      const main = el.querySelector('main')
      const scrollArea = el.querySelector('.content-scroll-area')

      expect(shell?.classList.contains('layout-shell--theme-stage')).toBe(true)
      expect(main?.classList.contains('content-main--theme-stage')).toBe(true)
      expect(scrollArea?.classList.contains('content-scroll-area--theme-stage')).toBe(true)
    } finally {
      unmount()
    }
  })

  it('skips theme-stage classes when the route keeps the global background', async () => {
    routeState.name = 'dashboard'
    routeState.fullPath = '/'
    routeState.meta.group = undefined
    routeState.meta.hideGlobalBackground = false
    routeState.meta.hideSidebar = false

    const { el, unmount } = await mountLayout()

    try {
      const shell = el.firstElementChild
      const main = el.querySelector('main')
      const scrollArea = el.querySelector('.content-scroll-area')

      expect(shell?.classList.contains('layout-shell--theme-stage')).toBe(false)
      expect(main?.classList.contains('content-main--theme-stage')).toBe(false)
      expect(scrollArea?.classList.contains('content-scroll-area--theme-stage')).toBe(false)
    } finally {
      unmount()
    }
  })

  it('shows a floating scroll-to-top control after the main workspace scrolls down', async () => {
    routeState.name = 'claude-code'
    routeState.fullPath = '/claude-code'
    routeState.meta.group = 'claude-code'
    routeState.meta.hideGlobalBackground = true
    routeState.meta.hideSidebar = false

    const { el, unmount } = await mountLayout()

    try {
      const scrollArea = el.querySelector('.content-scroll-area') as HTMLElement | null
      expect(scrollArea).not.toBeNull()
      expect(el.querySelector('[data-testid="main-scroll-to-top"]')).toBeNull()

      const scrollToMock = vi.fn(({ top }: { top: number }) => {
        if (!scrollArea) return
        scrollArea.scrollTop = top
        scrollArea.dispatchEvent(new Event('scroll'))
      })

      Object.defineProperty(scrollArea as HTMLElement, 'scrollTo', {
        value: scrollToMock,
        configurable: true,
      })

      if (scrollArea) {
        scrollArea.scrollTop = 640
        scrollArea.dispatchEvent(new Event('scroll'))
      }

      await nextTick()
      await nextTick()

      const button = el.querySelector(
        '[data-testid="main-scroll-to-top"]'
      ) as HTMLButtonElement | null
      expect(button).not.toBeNull()

      button?.click()

      expect(scrollToMock).toHaveBeenCalledWith({
        top: 0,
        behavior: 'smooth',
      })
    } finally {
      unmount()
    }
  })

  it('renders the settings dock as the bottom sidebar entry without legacy theme or language toggles', async () => {
    routeState.name = 'dashboard'
    routeState.fullPath = '/'
    routeState.meta.group = undefined
    routeState.meta.hideGlobalBackground = false
    routeState.meta.hideSidebar = false

    const { el, unmount } = await mountLayout()

    try {
      const settingsDockLink = el.querySelector('[data-testid="settings-dock-link"]')

      expect(settingsDockLink).not.toBeNull()
      expect(settingsDockLink?.textContent).toContain('nav.settings')
      expect(el.querySelector('[data-stub="LanguageSwitcher"]')).toBeNull()
      expect(el.querySelector('[data-stub="ThemeToggle"]')).toBeNull()
    } finally {
      unmount()
    }
  })

  it('marks the settings dock as active on the settings route', async () => {
    routeState.name = 'settings'
    routeState.fullPath = '/settings'
    routeState.meta.group = 'settings'
    routeState.meta.hideGlobalBackground = false
    routeState.meta.hideSidebar = false

    const { el, unmount } = await mountLayout()

    try {
      const settingsDockLink = el.querySelector('[data-testid="settings-dock-link"]')

      expect(settingsDockLink?.classList.contains('settings-dock--active')).toBe(true)
      expect(settingsDockLink?.getAttribute('aria-current')).toBe('page')
    } finally {
      unmount()
    }
  })
})
