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

    expect(themeBootstrap.FLAVOR_MODES).toEqual(['neutral', 'clay'])
    expect(themeBootstrap.ACCENT_MODES).toEqual(['clay'])
    expect(themeBootstrap.DEFAULT_FLAVOR).toBe('neutral')
    expect(themeBootstrap.DEFAULT_ACCENT).toBe('clay')
    expect(themeBootstrap).not.toHaveProperty('CATPPUCCIN_FLAVORS')
    expect(themeBootstrap).not.toHaveProperty('isCatppuccinFlavor')
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
    localStorage.setItem('ccr-flavor', 'clay')
    localStorage.setItem('ccr-accent', 'clay')
    installMatchMediaController(true)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    expect(themeBootstrap.readStoredFlavor()).toBe('clay')
    expect(themeBootstrap.readStoredAccent()).toBe('clay')

    themeBootstrap.applyInitialTheme()
    await flushAsyncImport()

    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
    expect(document.documentElement.getAttribute('data-flavor')).toBe('clay')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('clay')
    expect(document.documentElement.getAttribute('data-accent')).toBe('clay')

    themeBootstrap.persistFlavor('neutral')
    themeBootstrap.applyFlavorToDocument('neutral')
    expect(localStorage.getItem('ccr-flavor')).toBe('neutral')
    expect(document.documentElement.getAttribute('data-flavor')).toBe('neutral')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('neutral')

    themeBootstrap.persistAccent('clay')
    themeBootstrap.applyAccentToDocument('clay')
    expect(localStorage.getItem('ccr-accent')).toBe('clay')
    expect(document.documentElement.getAttribute('data-accent')).toBe('clay')

    themeBootstrap.__resetThemeBootstrapForTests()
  })

  it('migrates legacy flavor values from storage into the narrowed flavor domain', async () => {
    installMatchMediaController(false)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    for (const legacy of ['paper', 'graphite', 'catppuccin', 'latte', 'frappe', 'macchiato', 'mocha'] as const) {
      localStorage.setItem('ccr-flavor', legacy)
      expect(themeBootstrap.readStoredFlavor()).toBe('neutral')
    }

    for (const flavor of ['neutral', 'clay'] as const) {
      localStorage.setItem('ccr-flavor', flavor)
      expect(themeBootstrap.readStoredFlavor()).toBe(flavor)
    }

    themeBootstrap.__resetThemeBootstrapForTests()
  })

  it('migrates legacy accent values from storage into the narrowed accent domain', async () => {
    installMatchMediaController(false)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    for (const legacy of ['mauve', 'sage', 'sky', 'slate', 'sand', 'amber', 'rose'] as const) {
      localStorage.setItem('ccr-accent', legacy)
      expect(themeBootstrap.readStoredAccent()).toBe('clay')
    }

    localStorage.setItem('ccr-accent', 'clay')
    expect(themeBootstrap.readStoredAccent()).toBe('clay')

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

  it('keeps resolved flavor equal to the stored flavor across theme changes', async () => {
    localStorage.setItem('ccr-theme', 'system')
    localStorage.setItem('ccr-flavor', 'latte')
    const controller = installMatchMediaController(true)

    const themeBootstrap = await import('@/utils/themeBootstrap')

    themeBootstrap.applyInitialTheme()
    await flushAsyncImport()

    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
    expect(document.documentElement.getAttribute('data-flavor')).toBe('neutral')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('neutral')
    expect(themeBootstrap.resolveFlavorMode('dark', 'neutral')).toBe('neutral')
    expect(themeBootstrap.resolveFlavorMode('light', 'neutral')).toBe('neutral')
    expect(themeBootstrap.resolveFlavorMode('dark', 'clay')).toBe('clay')
    expect(themeBootstrap.resolveFlavorMode('light', 'clay')).toBe('clay')

    controller.setMatches(false)
    await flushAsyncImport()

    expect(document.documentElement.getAttribute('data-theme')).toBe('light')
    expect(document.documentElement.getAttribute('data-flavor')).toBe('neutral')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('neutral')

    themeBootstrap.__resetThemeBootstrapForTests()
  })

  it('does not ship retired Catppuccin flavor selectors', async () => {
    const source = await readFile('src/styles/tokens.css', 'utf8')

    expect(source).not.toMatch(/\[data-resolved-flavor=["']latte["']\]/)
    expect(source).not.toMatch(/html:root\[data-resolved-flavor=["']mocha["']\]/)
    expect(source).not.toMatch(/data-resolved-flavor=["'](?:frappe|macchiato|latte|mocha)["']/)
    expect(source).not.toMatch(
      /\[data-flavor=["'](?:latte|frappe|macchiato|mocha|paper|graphite|catppuccin)["']\]/
    )
  })

  it('pre-initializes first paint with resolved theme and flavor attributes', async () => {
    localStorage.setItem('ccr-theme', 'system')
    localStorage.setItem('ccr-flavor', 'clay')
    localStorage.setItem('ccr-accent', 'clay')
    installMatchMediaController(true)

    runInThisContext(await readBootstrapScript())

    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
    expect(document.documentElement.getAttribute('data-flavor')).toBe('clay')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('clay')
    expect(document.documentElement.getAttribute('data-accent')).toBe('clay')
    expect(document.documentElement.classList.contains('dark')).toBe(true)
  })

  it('migrates legacy stored values in the first-paint script', async () => {
    localStorage.setItem('ccr-flavor', 'graphite')
    localStorage.setItem('ccr-accent', 'slate')
    installMatchMediaController(true)

    runInThisContext(await readBootstrapScript())

    expect(document.documentElement.getAttribute('data-flavor')).toBe('neutral')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('neutral')
    expect(document.documentElement.getAttribute('data-accent')).toBe('clay')
  })

  it('keeps first-paint IIFE migration maps aligned with themeBootstrap', async () => {
    const script = await readBootstrapScript()
    const source = await readFile('src/utils/themeBootstrap.ts', 'utf8')
    const iifeFlavor = script.match(/var flavorMigration=(\{[^;]+\});/)?.[1]
    const iifeAccent = script.match(/var accentMigration=(\{[^;]+\});/)?.[1]
    const tsFlavor = source.match(
      /const FLAVOR_MIGRATION: Readonly<Partial<Record<string, FlavorMode>>> = \{([^}]+)\}/,
    )?.[1]
    const tsAccent = source.match(
      /const ACCENT_MIGRATION: Readonly<Partial<Record<string, AccentMode>>> = \{([^}]+)\}/,
    )?.[1]

    expect(iifeFlavor).toBeTruthy()
    expect(iifeAccent).toBeTruthy()
    expect(tsFlavor).toBeTruthy()
    expect(tsAccent).toBeTruthy()

    const parseMap = (raw: string): Record<string, string> =>
      Object.fromEntries(
        [...raw.matchAll(/([A-Za-z]+)\s*:\s*['"]([A-Za-z]+)['"]/g)].map((match) => [
          match[1],
          match[2],
        ]),
      )

    expect(parseMap(iifeFlavor ?? '')).toEqual(parseMap(tsFlavor ?? ''))
    expect(parseMap(iifeAccent ?? '')).toEqual(parseMap(tsAccent ?? ''))
    expect(parseMap(iifeAccent ?? '')).toMatchObject({
      sage: 'clay',
      sky: 'clay',
      slate: 'clay',
    })
    expect(parseMap(iifeAccent ?? '')).not.toMatchObject({ slate: 'sky' })

    installMatchMediaController(false)
    const flavorLegacy = [
      'paper',
      'graphite',
      'catppuccin',
      'latte',
      'frappe',
      'macchiato',
      'mocha',
    ] as const
    for (const legacy of flavorLegacy) {
      localStorage.clear()
      localStorage.setItem('ccr-flavor', legacy)
      runInThisContext(script)
      expect(document.documentElement.getAttribute('data-flavor'), legacy).toBe('neutral')
      expect(document.documentElement.getAttribute('data-resolved-flavor'), legacy).toBe('neutral')
    }

    const accentLegacy = ['mauve', 'sage', 'sky', 'slate', 'sand', 'amber', 'rose'] as const
    for (const legacy of accentLegacy) {
      localStorage.clear()
      localStorage.setItem('ccr-accent', legacy)
      runInThisContext(script)
      expect(document.documentElement.getAttribute('data-accent'), legacy).toBe('clay')
    }
  })

  it('migrates legacy catppuccin storage to neutral in the first-paint script', async () => {
    localStorage.setItem('ccr-flavor', 'macchiato')
    installMatchMediaController(false)

    runInThisContext(await readBootstrapScript())

    expect(document.documentElement.getAttribute('data-flavor')).toBe('neutral')
    expect(document.documentElement.getAttribute('data-resolved-flavor')).toBe('neutral')
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
