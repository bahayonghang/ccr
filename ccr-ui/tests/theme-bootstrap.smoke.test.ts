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
})
