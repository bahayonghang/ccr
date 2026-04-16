import { createPinia } from 'pinia'
import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const runtimeMocks = vi.hoisted(() => ({
  getEnvironmentName: vi.fn(),
  getTauriVersion: vi.fn(),
  isTauriEnvironment: vi.fn(),
  shellGetPreferences: vi.fn(),
  shellSetPreferences: vi.fn(),
  syncNativeWindowAppearance: vi.fn(),
}))

vi.mock('@/api/runtime/environment', () => ({
  getEnvironmentName: (...args: unknown[]) => runtimeMocks.getEnvironmentName(...args),
  getTauriVersion: (...args: unknown[]) => runtimeMocks.getTauriVersion(...args),
  isTauriEnvironment: (...args: unknown[]) => runtimeMocks.isTauriEnvironment(...args),
  shellGetPreferences: (...args: unknown[]) => runtimeMocks.shellGetPreferences(...args),
  shellSetPreferences: (...args: unknown[]) => runtimeMocks.shellSetPreferences(...args),
}))

vi.mock('@/utils/nativeWindowAppearance', () => ({
  syncNativeWindowAppearance: (...args: unknown[]) => runtimeMocks.syncNativeWindowAppearance(...args),
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

const flush = async () => {
  await Promise.resolve()
  await nextTick()
  await nextTick()
}

const listeners: Array<(event: MediaQueryListEvent) => void> = []

const installMatchMedia = (matches: boolean) => {
  vi.stubGlobal('matchMedia', vi.fn().mockImplementation(() => ({
    matches,
    media: '(prefers-color-scheme: dark)',
    addEventListener: (_event: string, listener: (event: MediaQueryListEvent) => void) => {
      listeners.push(listener)
    },
    removeEventListener: vi.fn(),
  })))
}

const mountView = async () => {
  const [{ default: AppSettingsView }, i18nModule] = await Promise.all([
    import('@/views/AppSettingsView.vue'),
    import('@/i18n'),
  ])

  await i18nModule.ensureLocaleLoaded('en-US')
  await i18nModule.ensureLocaleLoaded('zh-CN')

  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      return () => h(AppSettingsView)
    },
  }))

  app.use(createPinia())
  app.use(i18nModule.default)
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
  document.body.innerHTML = ''
  localStorage.clear()
  listeners.length = 0
  vi.resetModules()
  installMatchMedia(true)

  runtimeMocks.getEnvironmentName.mockReset()
  runtimeMocks.getTauriVersion.mockReset()
  runtimeMocks.isTauriEnvironment.mockReset()
  runtimeMocks.shellGetPreferences.mockReset()
  runtimeMocks.shellSetPreferences.mockReset()
  runtimeMocks.syncNativeWindowAppearance.mockReset()

  runtimeMocks.getEnvironmentName.mockReturnValue('tauri')
  runtimeMocks.getTauriVersion.mockResolvedValue('2.10.1')
  runtimeMocks.isTauriEnvironment.mockReturnValue(true)
  runtimeMocks.shellGetPreferences.mockResolvedValue({
    confirm_before_exit: false,
    close_to_tray: false,
    open_panel_on_tray_click: true,
  })
  runtimeMocks.shellSetPreferences.mockImplementation(async (preferences) => preferences)
})

afterEach(() => {
  document.body.innerHTML = ''
  vi.unstubAllGlobals()
})

describe('AppSettingsView smoke', () => {
  it('renders persisted shell preferences and applies updates immediately', async () => {
    localStorage.setItem('ccr-theme', 'system')
    localStorage.setItem('ccr-ui-locale', 'en-US')
    localStorage.setItem('ccr-sidebar-width', '312')
    localStorage.setItem('ccr-ui:perf', '1')

    const { el, unmount } = await mountView()

    try {
      expect(el.textContent).toContain('Settings')
      expect(el.textContent).toContain('Desktop runtime')

      const systemThemeButton = el.querySelector('[data-testid="settings-theme-system"]')
      const englishButton = el.querySelector('[data-testid="settings-language-en-US"]')
      const sidebarSlider = el.querySelector<HTMLInputElement>('[data-testid="settings-sidebar-width-slider"]')
      const exitToggle = el.querySelector('[data-testid="settings-confirm-exit-toggle"]')
      const perfToggle = el.querySelector('[data-testid="settings-perf-toggle"]')

      expect(systemThemeButton?.getAttribute('aria-pressed')).toBe('true')
      expect(englishButton?.getAttribute('aria-pressed')).toBe('true')
      expect(sidebarSlider?.value).toBe('312')
      expect(exitToggle?.getAttribute('aria-checked')).toBe('false')
      expect(perfToggle?.getAttribute('aria-checked')).toBe('true')

      const darkThemeButton = el.querySelector<HTMLElement>('[data-testid="settings-theme-dark"]')
      darkThemeButton?.click()
      await flush()

      expect(localStorage.getItem('ccr-theme')).toBe('dark')
      expect(document.documentElement.getAttribute('data-theme')).toBe('dark')

      const chineseButton = el.querySelector<HTMLElement>('[data-testid="settings-language-zh-CN"]')
      chineseButton?.click()
      await flush()

      expect(localStorage.getItem('ccr-ui-locale')).toBe('zh-CN')
      expect(chineseButton?.getAttribute('aria-pressed')).toBe('true')

      if (sidebarSlider) {
        sidebarSlider.value = '400'
        sidebarSlider.dispatchEvent(new Event('input', { bubbles: true }))
      }
      await flush()

      expect(localStorage.getItem('ccr-sidebar-width')).toBe('400')

      const resetButton = el.querySelector<HTMLElement>('[data-testid="settings-reset-layout"]')
      resetButton?.click()
      await flush()

      expect(localStorage.getItem('ccr-sidebar-width')).toBe('240')
      expect(sidebarSlider?.value).toBe('240')

      exitToggle?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await flush()

      expect(runtimeMocks.shellSetPreferences).toHaveBeenCalledWith({
        confirm_before_exit: true,
        close_to_tray: false,
        open_panel_on_tray_click: true,
      })
      expect(exitToggle?.getAttribute('aria-checked')).toBe('true')

      perfToggle?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await flush()

      expect(localStorage.getItem('ccr-ui:perf')).toBeNull()
      expect(perfToggle?.getAttribute('aria-checked')).toBe('false')
    } finally {
      unmount()
    }
  })
})
