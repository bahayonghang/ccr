import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

interface MatchMediaController {
  setMatches: (matches: boolean) => void
}

const installMatchMediaController = (initialMatches: boolean): MatchMediaController => {
  let matches = initialMatches
  const listeners = new Set<(event: MediaQueryListEvent) => void>()

  vi.stubGlobal('matchMedia', vi.fn().mockImplementation(() => ({
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
  })))

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
  document.documentElement.removeAttribute('data-accent')
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
    expect(document.documentElement.getAttribute('data-accent')).toBe('sage')

    themeBootstrap.persistFlavor('latte')
    themeBootstrap.applyFlavorToDocument('latte')
    expect(localStorage.getItem('ccr-flavor')).toBe('latte')
    expect(document.documentElement.getAttribute('data-flavor')).toBe('latte')

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
