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

const readBootstrapScript = async (): Promise<string> => {
  const source = await readFile('index.html', 'utf8')
  const script = source.match(/<!-- 主题预初始化[\s\S]*?<script>\s*([\s\S]*?)\s*<\/script>/)?.[1]

  expect(script).toBeTruthy()
  return script ?? ''
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
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('neutral')
    expect(document.documentElement.classList.contains('dark')).toBe(false)
    themeBootstrap.__resetThemeBootstrapForTests()
  })

  it('exposes the narrowed flavor and accent domains with neutral as the default flavor', async () => {
    installMatchMediaController(false)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    expect(themeBootstrap.FLAVOR_MODES).toEqual(['neutral', 'clay', 'catppuccin'])
    expect(themeBootstrap.ACCENT_MODES).toEqual(['clay', 'sage', 'sky', 'mauve'])
    expect(themeBootstrap.DEFAULT_FLAVOR).toBe('neutral')
    expect(themeBootstrap.DEFAULT_ACCENT).toBe('clay')
    expect(themeBootstrap.CATPPUCCIN_FLAVORS).toEqual(['catppuccin'])
    expect(themeBootstrap.isCatppuccinFlavor('catppuccin')).toBe(true)
    expect(themeBootstrap.isCatppuccinFlavor('mocha')).toBe(false)
    themeBootstrap.__resetThemeBootstrapForTests()
  })

  it('defaults flavor to neutral and accent to clay when nothing is persisted and seeds DOM dataset attributes', async () => {
    installMatchMediaController(false)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    expect(themeBootstrap.readStoredFlavor()).toBe('neutral')
    expect(themeBootstrap.readStoredAccent()).toBe('clay')

    themeBootstrap.applyInitialTheme()
    await flushAsyncImport()

    expect(document.documentElement.getAttribute('data-flavor')).toBe('neutral')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('neutral')
    expect(document.documentElement.getAttribute('data-accent')).toBe('clay')
    themeBootstrap.__resetThemeBootstrapForTests()
  })

  it('persists and rehydrates non-default flavor and accent independently of theme mode', async () => {
    localStorage.setItem('ccr-theme', 'system')
    localStorage.setItem('ccr-flavor', 'catppuccin')
    localStorage.setItem('ccr-accent', 'sage')
    installMatchMediaController(true)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    expect(themeBootstrap.readStoredFlavor()).toBe('catppuccin')
    expect(themeBootstrap.readStoredAccent()).toBe('sage')

    themeBootstrap.applyInitialTheme()
    await flushAsyncImport()

    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
    expect(document.documentElement.getAttribute('data-flavor')).toBe('catppuccin')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('mocha')
    expect(document.documentElement.getAttribute('data-accent')).toBe('sage')

    themeBootstrap.persistFlavor('clay')
    themeBootstrap.applyFlavorToDocument('clay')
    expect(localStorage.getItem('ccr-flavor')).toBe('clay')
    expect(document.documentElement.getAttribute('data-flavor')).toBe('clay')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('clay')

    themeBootstrap.persistAccent('sky')
    themeBootstrap.applyAccentToDocument('sky')
    expect(localStorage.getItem('ccr-accent')).toBe('sky')
    expect(document.documentElement.getAttribute('data-accent')).toBe('sky')

    themeBootstrap.__resetThemeBootstrapForTests()
  })

  it('migrates legacy flavor values from storage into the narrowed flavor domain', async () => {
    installMatchMediaController(false)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    for (const legacy of ['paper', 'graphite'] as const) {
      localStorage.setItem('ccr-flavor', legacy)
      expect(themeBootstrap.readStoredFlavor()).toBe('neutral')
    }

    for (const legacy of ['latte', 'frappe', 'macchiato', 'mocha'] as const) {
      localStorage.setItem('ccr-flavor', legacy)
      expect(themeBootstrap.readStoredFlavor()).toBe('catppuccin')
    }

    for (const flavor of ['neutral', 'clay', 'catppuccin'] as const) {
      localStorage.setItem('ccr-flavor', flavor)
      expect(themeBootstrap.readStoredFlavor()).toBe(flavor)
    }

    themeBootstrap.__resetThemeBootstrapForTests()
  })

  it('migrates legacy accent values from storage into the narrowed accent domain', async () => {
    installMatchMediaController(false)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    for (const legacy of ['sand', 'amber', 'rose'] as const) {
      localStorage.setItem('ccr-accent', legacy)
      expect(themeBootstrap.readStoredAccent()).toBe('clay')
    }

    localStorage.setItem('ccr-accent', 'slate')
    expect(themeBootstrap.readStoredAccent()).toBe('sky')

    for (const accent of ['clay', 'sage', 'sky', 'mauve'] as const) {
      localStorage.setItem('ccr-accent', accent)
      expect(themeBootstrap.readStoredAccent()).toBe(accent)
    }

    themeBootstrap.__resetThemeBootstrapForTests()
  })

  it('writes migrated flavor and accent values back to storage', async () => {
    localStorage.setItem('ccr-flavor', 'graphite')
    localStorage.setItem('ccr-accent', 'rose')
    installMatchMediaController(false)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    themeBootstrap.migratePersistedFlavor()
    themeBootstrap.migratePersistedAccent()

    expect(localStorage.getItem('ccr-flavor')).toBe('neutral')
    expect(localStorage.getItem('ccr-accent')).toBe('clay')
    themeBootstrap.__resetThemeBootstrapForTests()
  })

  it('does not seed default values into storage when nothing is persisted', async () => {
    installMatchMediaController(false)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    themeBootstrap.migratePersistedFlavor()
    themeBootstrap.migratePersistedAccent()

    expect(localStorage.getItem('ccr-flavor')).toBeNull()
    expect(localStorage.getItem('ccr-accent')).toBeNull()
    themeBootstrap.__resetThemeBootstrapForTests()
  })

  it('resolves the catppuccin flavor from the final light or dark theme instead of raw storage flavor', async () => {
    localStorage.setItem('ccr-theme', 'system')
    localStorage.setItem('ccr-flavor', 'latte')
    const controller = installMatchMediaController(true)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    themeBootstrap.applyInitialTheme()
    await flushAsyncImport()

    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
    expect(document.documentElement.getAttribute('data-flavor')).toBe('catppuccin')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('mocha')
    expect(themeBootstrap.resolveFlavorMode('dark', 'catppuccin')).toBe('mocha')
    expect(themeBootstrap.resolveFlavorMode('light', 'catppuccin')).toBe('latte')
    expect(themeBootstrap.resolveFlavorMode('dark', 'neutral')).toBe('neutral')
    expect(themeBootstrap.resolveFlavorMode('light', 'neutral')).toBe('neutral')
    expect(themeBootstrap.resolveFlavorMode('dark', 'clay')).toBe('clay')
    expect(themeBootstrap.resolveFlavorMode('light', 'clay')).toBe('clay')

    controller.setMatches(false)
    await flushAsyncImport()

    expect(document.documentElement.getAttribute('data-theme')).toBe('light')
    expect(document.documentElement.getAttribute('data-flavor')).toBe('catppuccin')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('latte')

    themeBootstrap.__resetThemeBootstrapForTests()
  })

  it('targets Catppuccin CSS through resolved flavor so system dark plus stored Latte cannot apply Latte surfaces', async () => {
    const source = await readFile('src/styles/tokens.css', 'utf8')

    expect(source).toMatch(/\[data-resolved-flavor=["']latte["']\]/)
    expect(source).toMatch(/html:root\[data-resolved-flavor=["']mocha["']\]/)
    // catppuccin 只解析为 latte|mocha：frappe/macchiato 调色板块与旧 data-flavor 直选块已删除。
    expect(source).not.toMatch(/data-resolved-flavor=["']frappe["']/)
    expect(source).not.toMatch(/data-resolved-flavor=["']macchiato["']/)
    expect(source).not.toMatch(
      /\[data-flavor=["'](?:latte|frappe|macchiato|mocha|paper|graphite)["']\]/
    )
  })

  it('pre-initializes first paint with resolved theme and flavor attributes', async () => {
    localStorage.setItem('ccr-theme', 'system')
    localStorage.setItem('ccr-flavor', 'catppuccin')
    localStorage.setItem('ccr-accent', 'sage')
    installMatchMediaController(true)

    runInThisContext(await readBootstrapScript())

    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
    expect(document.documentElement.getAttribute('data-flavor')).toBe('catppuccin')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('mocha')
    expect(document.documentElement.getAttribute('data-accent')).toBe('sage')
    expect(document.documentElement.classList.contains('dark')).toBe(true)
  })

  it('migrates legacy stored values in the first-paint script', async () => {
    localStorage.setItem('ccr-flavor', 'graphite')
    localStorage.setItem('ccr-accent', 'slate')
    installMatchMediaController(true)

    runInThisContext(await readBootstrapScript())

    expect(document.documentElement.getAttribute('data-flavor')).toBe('neutral')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('neutral')
    expect(document.documentElement.getAttribute('data-accent')).toBe('sky')
  })

  it('resolves legacy catppuccin storage by the resolved theme in the first-paint script', async () => {
    localStorage.setItem('ccr-flavor', 'macchiato')
    installMatchMediaController(false)

    runInThisContext(await readBootstrapScript())

    expect(document.documentElement.getAttribute('data-flavor')).toBe('catppuccin')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('latte')
  })

  it('falls back to neutral flavor and clay accent in the first-paint script when nothing is persisted', async () => {
    installMatchMediaController(false)

    runInThisContext(await readBootstrapScript())

    expect(document.documentElement.getAttribute('data-flavor')).toBe('neutral')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('neutral')
    expect(document.documentElement.getAttribute('data-accent')).toBe('clay')
  })

  it('rejects unknown flavor and accent values in the first-paint script and falls back to defaults', async () => {
    localStorage.setItem('ccr-flavor', 'neko')
    localStorage.setItem('ccr-accent', 'lavender')
    installMatchMediaController(false)

    runInThisContext(await readBootstrapScript())

    expect(document.documentElement.getAttribute('data-flavor')).toBe('neutral')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('neutral')
    expect(document.documentElement.getAttribute('data-accent')).toBe('clay')
  })

  it('pre-initializes first paint with font overrides prepended to the base stacks', async () => {
    localStorage.setItem('ccr-font-ui', 'Inter')
    localStorage.setItem('ccr-font-code', 'JetBrains Mono')
    installMatchMediaController(false)

    runInThisContext(await readBootstrapScript())

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

    runInThisContext(await readBootstrapScript())

    const sans = document.documentElement.style.getPropertyValue('--font-sans')
    expect(sans).toContain('var(--font-sans-base)')
    expect(sans).not.toContain(';')
    expect(sans).not.toContain('}')
  })

  it('rejects unknown flavor and accent values from storage and falls back to defaults', async () => {
    localStorage.setItem('ccr-flavor', 'neko')
    localStorage.setItem('ccr-accent', 'lavender')
    installMatchMediaController(false)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    expect(themeBootstrap.readStoredFlavor()).toBe('neutral')
    expect(themeBootstrap.readStoredAccent()).toBe('clay')
    themeBootstrap.__resetThemeBootstrapForTests()
  })
})
