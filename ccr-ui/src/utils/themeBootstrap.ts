export type ThemeMode = 'light' | 'dark'

const THEME_STORAGE_KEY = 'ccr-theme'

export const applyThemeToDocument = (theme: ThemeMode): void => {
  if (typeof document === 'undefined') return

  document.documentElement.classList.toggle('dark', theme === 'dark')
  document.documentElement.setAttribute('data-theme', theme)
}

export const readStoredTheme = (): ThemeMode => {
  if (typeof window === 'undefined') {
    return 'light'
  }

  try {
    return localStorage.getItem(THEME_STORAGE_KEY) === 'dark' ? 'dark' : 'light'
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
