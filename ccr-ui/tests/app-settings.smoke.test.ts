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
let mediaController: { setMatches: (matches: boolean) => void }

const installMatchMedia = (initialMatches: boolean) => {
  let matches = initialMatches
  vi.stubGlobal('matchMedia', vi.fn().mockImplementation(() => ({
    get matches() {
      return matches
    },
    media: '(prefers-color-scheme: dark)',
    addEventListener: (_event: string, listener: (event: MediaQueryListEvent) => void) => {
      listeners.push(listener)
    },
    removeEventListener: vi.fn(),
  })))

  return {
    setMatches(nextMatches: boolean) {
      matches = nextMatches
      const event = { matches: nextMatches } as MediaQueryListEvent
      listeners.forEach((listener) => listener(event))
    },
  }
}

const mountView = async ({ hydrateLocales = true } = {}) => {
  const [{ default: AppSettingsView }, i18nModule] = await Promise.all([
    import('@/views/AppSettingsView.vue'),
    import('@/i18n'),
  ])

  if (hydrateLocales) {
    await i18nModule.ensureLocaleLoaded('en-US')
    await i18nModule.ensureLocaleLoaded('zh-CN')
  }

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
  document.documentElement.className = ''
  document.documentElement.removeAttribute('data-theme')
  document.documentElement.removeAttribute('data-flavor')
  document.documentElement.removeAttribute('data-resolved-flavor')
  document.documentElement.removeAttribute('data-accent')
  localStorage.clear()
  listeners.length = 0
  vi.resetModules()
  mediaController = installMatchMedia(true)

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
  it('renders the settings route from boot messages before full locale hydration', async () => {
    localStorage.setItem('ccr-ui-locale', 'en-US')

    const { el, unmount } = await mountView({ hydrateLocales: false })

    try {
      expect(el.textContent).toContain('Settings')
      expect(el.textContent).toContain('Appearance')
      expect(el.textContent).toContain('Surface tone')
      expect(el.textContent).toContain('Workbench')
      expect(el.textContent).toContain('Diagnostics')
      expect(el.textContent).not.toContain('settings.')
    } finally {
      unmount()
    }
  })

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
      const flavorOptions = ['neutral', 'clay', 'catppuccin']
      const accentOptions = ['clay', 'sage', 'sky', 'mauve']

      expect(systemThemeButton?.getAttribute('aria-pressed')).toBe('true')
      expect(systemThemeButton?.getAttribute('aria-checked')).toBe('true')
      expect(systemThemeButton?.getAttribute('role')).toBe('radio')
      expect(englishButton?.getAttribute('aria-pressed')).toBe('true')
      expect(sidebarSlider?.value).toBe('312')
      expect(exitToggle?.getAttribute('aria-checked')).toBe('false')
      expect(perfToggle?.getAttribute('aria-checked')).toBe('true')

      // 主题分段控件：radiogroup 语义 + system 选中时的解析结果指示。
      const themeGroup = systemThemeButton?.closest('[role="radiogroup"]')
      expect(themeGroup).toBeTruthy()
      expect(el.textContent).toContain('Resolved now: Dark mode')

      // 新值域：3 flavor + 4 accent，且每项渲染真实 token 预览（作用域覆写令牌变量）。
      expect(document.documentElement.getAttribute('data-flavor')).toBe('neutral')
      expect(el.querySelector('[data-testid="settings-flavor-neutral"]')?.getAttribute('aria-pressed')).toBe('true')
      for (const option of flavorOptions) {
        expect(el.querySelector(`[data-testid="settings-flavor-${option}"]`)).toBeTruthy()
        const preview = el.querySelector(`[data-preview-flavor="${option}"]`)
        expect(preview).toBeTruthy()
        expect(preview?.getAttribute('style')).toContain('--fp-bg-base')
      }
      for (const option of accentOptions) {
        expect(el.querySelector(`[data-testid="settings-accent-${option}"]`)).toBeTruthy()
        const preview = el.querySelector(`[data-preview-accent="${option}"]`)
        expect(preview).toBeTruthy()
        expect(preview?.getAttribute('style')).toContain('--fp-accent-bg')
      }
      expect(el.querySelector('[data-testid="settings-flavor-paper"]')).toBeNull()
      expect(el.querySelector('[data-testid="settings-flavor-mocha"]')).toBeNull()
      expect(el.querySelector('[data-testid="settings-accent-sand"]')).toBeNull()
      expect(el.querySelector('[data-testid="settings-accent-slate"]')).toBeNull()

      const darkThemeButton = el.querySelector<HTMLElement>('[data-testid="settings-theme-dark"]')
      darkThemeButton?.click()
      await flush()

      expect(localStorage.getItem('ccr-theme')).toBe('dark')
      expect(document.documentElement.getAttribute('data-theme')).toBe('dark')

      const catppuccinFlavorButton = el.querySelector<HTMLElement>('[data-testid="settings-flavor-catppuccin"]')
      catppuccinFlavorButton?.click()
      await flush()

      // catppuccin 直接落库；暗色解析为 mocha，且新值域激活态按 data-flavor 命中。
      expect(localStorage.getItem('ccr-flavor')).toBe('catppuccin')
      expect(document.documentElement.getAttribute('data-flavor')).toBe('catppuccin')
      expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('mocha')
      expect(catppuccinFlavorButton?.getAttribute('aria-pressed')).toBe('true')
      expect(el.textContent).toContain('Mocha')

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

  it('keeps system theme summary and resolved Catppuccin flavor in sync with OS preference changes', async () => {
    localStorage.setItem('ccr-theme', 'system')
    localStorage.setItem('ccr-flavor', 'latte')
    localStorage.setItem('ccr-ui-locale', 'en-US')

    const { el, unmount } = await mountView()

    try {
      // 存储值 latte 在初始化时迁移为 catppuccin 并写回；暗色解析为 mocha。
      expect(el.textContent).toContain('Follow system · Dark mode')
      expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
      expect(document.documentElement.getAttribute('data-flavor')).toBe('catppuccin')
      expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('mocha')
      expect(localStorage.getItem('ccr-flavor')).toBe('catppuccin')

      mediaController.setMatches(false)
      await flush()

      expect(el.textContent).toContain('Follow system · Light mode')
      expect(document.documentElement.getAttribute('data-theme')).toBe('light')
      expect(document.documentElement.getAttribute('data-flavor')).toBe('catppuccin')
      expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('latte')

      const catppuccinFlavorButton = el.querySelector<HTMLElement>('[data-testid="settings-flavor-catppuccin"]')
      catppuccinFlavorButton?.click()
      await flush()

      // catppuccin 为真实选项值：亮色下解析为 latte。
      expect(document.documentElement.getAttribute('data-flavor')).toBe('catppuccin')
      expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('latte')
      expect(catppuccinFlavorButton?.getAttribute('aria-pressed')).toBe('true')

      mediaController.setMatches(true)
      await flush()

      expect(el.textContent).toContain('Follow system · Dark mode')
      expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
      expect(document.documentElement.getAttribute('data-flavor')).toBe('catppuccin')
      expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('mocha')
    } finally {
      unmount()
    }
  })

  it('keeps boot settings messages aligned with the full locale settings key set', async () => {
    const [{ bootLocaleMessages }, enUS, zhCN] = await Promise.all([
      import('@/i18n/bootMessages'),
      import('@/i18n/locales/en-US'),
      import('@/i18n/locales/zh-CN'),
    ])

    const collectKeys = (node: Record<string, unknown>, prefix: string): string[] =>
      Object.entries(node).flatMap(([key, value]) => {
        const path = `${prefix}.${key}`
        return value !== null && typeof value === 'object'
          ? collectKeys(value as Record<string, unknown>, path)
          : [path]
      })

    const fullMessages = {
      'en-US': enUS.default,
      'zh-CN': zhCN.default,
    } as const

    for (const locale of ['en-US', 'zh-CN'] as const) {
      const bootKeys = collectKeys(
        bootLocaleMessages[locale].settings as Record<string, unknown>,
        'settings',
      ).sort()
      const fullKeys = collectKeys(
        (fullMessages[locale] as Record<string, unknown>).settings as Record<string, unknown>,
        'settings',
      ).sort()

      // AC3：首屏副本与语言包键集合一致，且旧 flavor/accent 键无残留。
      expect(bootKeys).toEqual(fullKeys)
      expect(bootKeys.join('\n')).not.toMatch(/flavor\.(paper|graphite|latte|frappe|macchiato|mocha)/)
      expect(bootKeys.join('\n')).not.toMatch(/accent\.(sand|amber|rose|slate)/)
    }
  })
})
