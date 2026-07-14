import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { readFile } from 'node:fs/promises'
import { runInThisContext } from 'node:vm'

interface MatchMediaController {
  setMatches: (matches: boolean) => void
}

const installMatchMediaController = (initialMatches: boolean): MatchMediaController => {
  let matches = initialMatches
  const listeners = new Set<(event: MediaQueryListEvent) => void>()

  vi.stubGlobal(
    'matchMedia',
    vi.fn().mockImplementation(() => ({
      get matches() {
        return matches
      },
      media: '(prefers-color-scheme: dark)',
      addEventListener: (_event: string, listener: (event: MediaQueryListEvent) => void) => {
        listeners.add(listener)
      },
      removeEventListener: (_event: string, listener: (event: MediaQueryListEvent) => void) => {
        listeners.delete(listener)
      },
    }))
  )

  return {
    setMatches(nextMatches: boolean) {
      matches = nextMatches
      const event = { matches: nextMatches } as MediaQueryListEvent
      listeners.forEach((listener) => listener(event))
    },
  }
}

const flushMicrotasks = async () => {
  await Promise.resolve()
  await Promise.resolve()
}

const flushAsyncImport = async () => {
  await flushMicrotasks()
  await new Promise((resolve) => setTimeout(resolve, 20))
}

beforeEach(() => {
  localStorage.clear()
  document.documentElement.className = ''
  document.documentElement.removeAttribute('data-theme')
  document.documentElement.removeAttribute('data-flavor')
  document.documentElement.removeAttribute('data-resolved-flavor')
  document.documentElement.removeAttribute('data-accent')
  document.documentElement.removeAttribute('style')
  vi.resetModules()
})

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('themeBootstrap smoke', () => {
  it('resolves system theme to the current OS preference and syncs the native window with the resolved theme', async () => {
    localStorage.setItem('ccr-theme', 'system')
    installMatchMediaController(true)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    expect(themeBootstrap.readStoredTheme()).toBe('system')
    expect(themeBootstrap.applyThemeToDocument('system')).toBe('dark')
    await flushAsyncImport()

    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
    expect(document.documentElement.classList.contains('dark')).toBe(true)
    themeBootstrap.__resetThemeBootstrapForTests()
  })

  it('updates the resolved document theme when the OS preference changes while using system mode', async () => {
    localStorage.setItem('ccr-theme', 'system')
    const controller = installMatchMediaController(true)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    themeBootstrap.applyInitialTheme()
    await flushAsyncImport()

    controller.setMatches(false)
    await flushAsyncImport()

    expect(document.documentElement.getAttribute('data-theme')).toBe('light')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('clay')
    expect(document.documentElement.classList.contains('dark')).toBe(false)
    themeBootstrap.__resetThemeBootstrapForTests()
  })

  it('defaults flavor and accent to clay when nothing is persisted and seeds DOM dataset attributes', async () => {
    installMatchMediaController(false)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    expect(themeBootstrap.readStoredFlavor()).toBe('clay')
    expect(themeBootstrap.readStoredAccent()).toBe('clay')

    themeBootstrap.applyInitialTheme()
    await flushAsyncImport()

    expect(document.documentElement.getAttribute('data-flavor')).toBe('clay')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('clay')
    expect(document.documentElement.getAttribute('data-accent')).toBe('clay')
    themeBootstrap.__resetThemeBootstrapForTests()
  })

  it('persists and rehydrates non-default flavor and accent independently of theme mode', async () => {
    localStorage.setItem('ccr-theme', 'system')
    localStorage.setItem('ccr-flavor', 'mocha')
    localStorage.setItem('ccr-accent', 'sage')
    installMatchMediaController(true)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    expect(themeBootstrap.readStoredFlavor()).toBe('mocha')
    expect(themeBootstrap.readStoredAccent()).toBe('sage')

    themeBootstrap.applyInitialTheme()
    await flushAsyncImport()

    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
    expect(document.documentElement.getAttribute('data-flavor')).toBe('mocha')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('mocha')
    expect(document.documentElement.getAttribute('data-accent')).toBe('sage')

    themeBootstrap.persistFlavor('latte')
    themeBootstrap.applyFlavorToDocument('latte')
    expect(localStorage.getItem('ccr-flavor')).toBe('latte')
    expect(document.documentElement.getAttribute('data-flavor')).toBe('latte')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('frappe')

    themeBootstrap.persistAccent('amber')
    themeBootstrap.applyAccentToDocument('amber')
    expect(localStorage.getItem('ccr-accent')).toBe('amber')
    expect(document.documentElement.getAttribute('data-accent')).toBe('amber')

    themeBootstrap.__resetThemeBootstrapForTests()
  })

  it('accepts all Catppuccin flavor values from storage', async () => {
    installMatchMediaController(false)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    for (const flavor of ['latte', 'frappe', 'macchiato', 'mocha'] as const) {
      localStorage.setItem('ccr-flavor', flavor)
      expect(themeBootstrap.readStoredFlavor()).toBe(flavor)
    }

    themeBootstrap.__resetThemeBootstrapForTests()
  })

  it('resolves Catppuccin flavors from the final light or dark theme instead of raw storage flavor', async () => {
    localStorage.setItem('ccr-theme', 'system')
    localStorage.setItem('ccr-flavor', 'latte')
    const controller = installMatchMediaController(true)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    themeBootstrap.applyInitialTheme()
    await flushAsyncImport()

    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
    expect(document.documentElement.getAttribute('data-flavor')).toBe('latte')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('frappe')
    expect(themeBootstrap.resolveFlavorMode('dark', 'latte')).toBe('frappe')
    expect(themeBootstrap.resolveFlavorMode('dark', 'frappe')).toBe('frappe')
    expect(themeBootstrap.resolveFlavorMode('dark', 'macchiato')).toBe('macchiato')
    expect(themeBootstrap.resolveFlavorMode('dark', 'mocha')).toBe('mocha')
    expect(themeBootstrap.resolveFlavorMode('light', 'mocha')).toBe('latte')

    controller.setMatches(false)
    await flushAsyncImport()

    expect(document.documentElement.getAttribute('data-theme')).toBe('light')
    expect(document.documentElement.getAttribute('data-flavor')).toBe('latte')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('latte')

    themeBootstrap.__resetThemeBootstrapForTests()
  })

  it('targets Catppuccin CSS through resolved flavor so system dark plus stored Latte cannot apply Latte surfaces', async () => {
    const source = await readFile('src/styles/tokens.css', 'utf8')

    expect(source).toMatch(/\[data-resolved-flavor=["']latte["']\]/)
    expect(source).toMatch(/\[data-resolved-flavor=["']frappe["']\]/)
    expect(source).not.toMatch(/\[data-flavor=["']latte["']\]/)
    expect(source).not.toMatch(/\[data-flavor=["']frappe["']\]/)
    expect(source).not.toMatch(/\[data-flavor=["']macchiato["']\]/)
    expect(source).not.toMatch(/\[data-flavor=["']mocha["']\]/)
  })

  it('pre-initializes first paint with resolved theme and flavor attributes', async () => {
    localStorage.setItem('ccr-theme', 'system')
    localStorage.setItem('ccr-flavor', 'latte')
    localStorage.setItem('ccr-accent', 'sage')
    installMatchMediaController(true)

    const source = await readFile('index.html', 'utf8')
    const script = source.match(/<!-- 主题预初始化[\s\S]*?<script>\s*([\s\S]*?)\s*<\/script>/)?.[1]

    expect(script).toBeTruthy()
    runInThisContext(script ?? '')

    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
    expect(document.documentElement.getAttribute('data-flavor')).toBe('latte')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('frappe')
    expect(document.documentElement.getAttribute('data-accent')).toBe('sage')
    expect(document.documentElement.classList.contains('dark')).toBe(true)
  })

  it('pre-initializes first paint with font overrides prepended to the base stacks', async () => {
    localStorage.setItem('ccr-font-ui', 'Inter')
    localStorage.setItem('ccr-font-code', 'JetBrains Mono')
    installMatchMediaController(false)

    const source = await readFile('index.html', 'utf8')
    const script = source.match(/<!-- 主题预初始化[\s\S]*?<script>\s*([\s\S]*?)\s*<\/script>/)?.[1]

    expect(script).toBeTruthy()
    runInThisContext(script ?? '')

    const root = document.documentElement
    expect(root.style.getPropertyValue('--font-sans')).toContain('"Inter"')
    expect(root.style.getPropertyValue('--font-sans')).toContain('var(--font-sans-base)')
    expect(root.style.getPropertyValue('--font-brand')).toContain('"Inter"')
    expect(root.style.getPropertyValue('--font-mono')).toContain('"JetBrains Mono"')
    expect(root.style.getPropertyValue('--font-mono')).toContain('var(--font-mono-base)')
  })

  it('sanitizes malicious font values in the first-paint script', async () => {
    localStorage.setItem('ccr-font-ui', 'Evil"; color: red; }')
    installMatchMediaController(false)

    const source = await readFile('index.html', 'utf8')
    const script = source.match(/<!-- 主题预初始化[\s\S]*?<script>\s*([\s\S]*?)\s*<\/script>/)?.[1]
    runInThisContext(script ?? '')

    const sans = document.documentElement.style.getPropertyValue('--font-sans')
    expect(sans).toContain('var(--font-sans-base)')
    expect(sans).not.toContain(';')
    expect(sans).not.toContain('}')
  })

  it('rejects unknown flavor and accent values from storage and falls back to clay', async () => {
    localStorage.setItem('ccr-flavor', 'neko')
    localStorage.setItem('ccr-accent', 'lavender')
    installMatchMediaController(false)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    expect(themeBootstrap.readStoredFlavor()).toBe('clay')
    expect(themeBootstrap.readStoredAccent()).toBe('clay')
    themeBootstrap.__resetThemeBootstrapForTests()
  })
})
