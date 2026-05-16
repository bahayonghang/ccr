export type ThemeMode = 'light' | 'dark' | 'system'
export type ResolvedThemeMode = 'light' | 'dark'
export type FlavorMode =
  | 'clay'
  | 'paper'
  | 'graphite'
  | 'latte'
  | 'frappe'
  | 'macchiato'
  | 'mocha'
export type AccentMode =
  | 'clay'
  | 'sand'
  | 'sage'
  | 'sky'
  | 'mauve'
  | 'amber'
  | 'rose'
  | 'slate'

const THEME_STORAGE_KEY = 'ccr-theme'
const FLAVOR_STORAGE_KEY = 'ccr-flavor'
const ACCENT_STORAGE_KEY = 'ccr-accent'
const THEME_MEDIA_QUERY = '(prefers-color-scheme: dark)'

export const FLAVOR_MODES: readonly FlavorMode[] = [
  'clay',
  'paper',
  'graphite',
  'latte',
  'frappe',
  'macchiato',
  'mocha',
] as const
export const ACCENT_MODES: readonly AccentMode[] = [
  'clay',
  'sand',
  'sage',
  'sky',
  'mauve',
  'amber',
  'rose',
  'slate',
] as const

export const DEFAULT_FLAVOR: FlavorMode = 'clay'
export const DEFAULT_ACCENT: AccentMode = 'clay'

let systemThemeMediaQuery: MediaQueryList | null = null
let systemThemeListenerRegistered = false

const resolveSystemTheme = (): ResolvedThemeMode => {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return 'light'
  }

  return window.matchMedia(THEME_MEDIA_QUERY).matches ? 'dark' : 'light'
}

export const resolveThemeMode = (theme: ThemeMode): ResolvedThemeMode => {
  return theme === 'system' ? resolveSystemTheme() : theme
}

const syncResolvedTheme = (theme: ResolvedThemeMode): void => {
  if (typeof document === 'undefined') return

  document.documentElement.classList.toggle('dark', theme === 'dark')
  document.documentElement.setAttribute('data-theme', theme)

  void import('@/utils/nativeWindowAppearance')
    .then(({ syncNativeWindowAppearance }) => syncNativeWindowAppearance(theme))
    .catch(() => {
      // 浏览器测试环境或非 Tauri 运行时允许静默降级。
    })
}

const handleSystemThemeChange = (): void => {
  if (readStoredTheme() !== 'system') return
  syncResolvedTheme(resolveSystemTheme())
}

const ensureSystemThemeListener = (): void => {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return
  }

  if (!systemThemeMediaQuery) {
    systemThemeMediaQuery = window.matchMedia(THEME_MEDIA_QUERY)
  }

  if (systemThemeListenerRegistered) return

  systemThemeMediaQuery.addEventListener('change', handleSystemThemeChange)
  systemThemeListenerRegistered = true
}

export const applyThemeToDocument = (theme: ThemeMode): ResolvedThemeMode => {
  ensureSystemThemeListener()

  const resolvedTheme = resolveThemeMode(theme)
  syncResolvedTheme(resolvedTheme)
  return resolvedTheme
}

export const readStoredTheme = (): ThemeMode => {
  if (typeof window === 'undefined') {
    return 'light'
  }

  try {
    const storedTheme = localStorage.getItem(THEME_STORAGE_KEY)
    return storedTheme === 'dark' || storedTheme === 'system' ? storedTheme : 'light'
  } catch {
    return 'light'
  }
}

export const persistTheme = (theme: ThemeMode): void => {
  if (typeof window === 'undefined') return

  try {
    localStorage.setItem(THEME_STORAGE_KEY, theme)
  } catch {
    // 忽略存储异常，保留当前 UI 状态即可。
  }
}

const isFlavorMode = (value: unknown): value is FlavorMode => {
  return typeof value === 'string' && (FLAVOR_MODES as readonly string[]).includes(value)
}

const isAccentMode = (value: unknown): value is AccentMode => {
  return typeof value === 'string' && (ACCENT_MODES as readonly string[]).includes(value)
}

export const readStoredFlavor = (): FlavorMode => {
  if (typeof window === 'undefined') {
    return DEFAULT_FLAVOR
  }

  try {
    const stored = localStorage.getItem(FLAVOR_STORAGE_KEY)
    return isFlavorMode(stored) ? stored : DEFAULT_FLAVOR
  } catch {
    return DEFAULT_FLAVOR
  }
}

export const readStoredAccent = (): AccentMode => {
  if (typeof window === 'undefined') {
    return DEFAULT_ACCENT
  }

  try {
    const stored = localStorage.getItem(ACCENT_STORAGE_KEY)
    return isAccentMode(stored) ? stored : DEFAULT_ACCENT
  } catch {
    return DEFAULT_ACCENT
  }
}

export const persistFlavor = (flavor: FlavorMode): void => {
  if (typeof window === 'undefined') return

  try {
    localStorage.setItem(FLAVOR_STORAGE_KEY, flavor)
  } catch {
    // 忽略存储异常，保留当前 UI 状态即可。
  }
}

export const persistAccent = (accent: AccentMode): void => {
  if (typeof window === 'undefined') return

  try {
    localStorage.setItem(ACCENT_STORAGE_KEY, accent)
  } catch {
    // 忽略存储异常，保留当前 UI 状态即可。
  }
}

export const applyFlavorToDocument = (flavor: FlavorMode): FlavorMode => {
  if (typeof document !== 'undefined') {
    document.documentElement.setAttribute('data-flavor', flavor)
  }
  return flavor
}

export const applyAccentToDocument = (accent: AccentMode): AccentMode => {
  if (typeof document !== 'undefined') {
    document.documentElement.setAttribute('data-accent', accent)
  }
  return accent
}

export const applyInitialTheme = (): ThemeMode => {
  const theme = readStoredTheme()
  applyThemeToDocument(theme)
  applyFlavorToDocument(readStoredFlavor())
  applyAccentToDocument(readStoredAccent())
  return theme
}

export const __resetThemeBootstrapForTests = (): void => {
  if (systemThemeMediaQuery && systemThemeListenerRegistered) {
    systemThemeMediaQuery.removeEventListener('change', handleSystemThemeChange)
  }

  systemThemeMediaQuery = null
  systemThemeListenerRegistered = false
}
