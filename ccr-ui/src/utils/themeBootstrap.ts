export type ThemeMode = 'light' | 'dark' | 'system'
export type ResolvedThemeMode = 'light' | 'dark'

const THEME_STORAGE_KEY = 'ccr-theme'
const THEME_MEDIA_QUERY = '(prefers-color-scheme: dark)'

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

export const applyInitialTheme = (): ThemeMode => {
  const theme = readStoredTheme()
  applyThemeToDocument(theme)
  return theme
}

export const __resetThemeBootstrapForTests = (): void => {
  if (systemThemeMediaQuery && systemThemeListenerRegistered) {
    systemThemeMediaQuery.removeEventListener('change', handleSystemThemeChange)
  }

  systemThemeMediaQuery = null
  systemThemeListenerRegistered = false
}
