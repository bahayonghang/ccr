export type ThemeMode = 'light' | 'dark' | 'system'
export type ResolvedThemeMode = 'light' | 'dark'
export type FlavorMode = 'neutral' | 'clay' | 'catppuccin'
export type ResolvedFlavor = 'neutral' | 'clay' | 'latte' | 'mocha'
export type AccentMode = 'clay' | 'sage' | 'sky' | 'mauve'

const THEME_STORAGE_KEY = 'ccr-theme'
const FLAVOR_STORAGE_KEY = 'ccr-flavor'
const ACCENT_STORAGE_KEY = 'ccr-accent'
const THEME_MEDIA_QUERY = '(prefers-color-scheme: dark)'
export const THEME_RESOLUTION_CHANGE_EVENT = 'ccr-theme-resolution-change'

export interface ThemeResolutionChangeDetail {
  theme: ThemeMode
  resolvedTheme: ResolvedThemeMode
  flavor: FlavorMode
  resolvedFlavor: ResolvedFlavor
}

export const FLAVOR_MODES: readonly FlavorMode[] = [
  'neutral',
  'clay',
  'catppuccin',
] as const
export const ACCENT_MODES: readonly AccentMode[] = [
  'clay',
  'sage',
  'sky',
  'mauve',
] as const

export const DEFAULT_FLAVOR: FlavorMode = 'neutral'
export const DEFAULT_ACCENT: AccentMode = 'clay'
export const CATPPUCCIN_FLAVORS: readonly FlavorMode[] = [
  'catppuccin',
] as const

// 旧值域 → 新值域迁移表（flavor 7→3、accent 8→4）。
// 读取侧映射；store 初始化时把迁移结果写回 localStorage，index.html 首帧 IIFE 内联同一份逻辑。
const FLAVOR_MIGRATION: Readonly<Partial<Record<string, FlavorMode>>> = {
  paper: 'neutral',
  graphite: 'neutral',
  latte: 'catppuccin',
  frappe: 'catppuccin',
  macchiato: 'catppuccin',
  mocha: 'catppuccin',
}
const ACCENT_MIGRATION: Readonly<Partial<Record<string, AccentMode>>> = {
  sand: 'clay',
  amber: 'clay',
  rose: 'clay',
  slate: 'sky',
}

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

export const isCatppuccinFlavor = (flavor: string): boolean => {
  return (CATPPUCCIN_FLAVORS as readonly string[]).includes(flavor)
}

export const resolveFlavorMode = (
  resolvedTheme: ResolvedThemeMode,
  flavor: FlavorMode,
): ResolvedFlavor => {
  // catppuccin 是唯一自适应入口：light → latte、dark → mocha，其余直通。
  if (flavor !== 'catppuccin') {
    return flavor
  }

  return resolvedTheme === 'light' ? 'latte' : 'mocha'
}

const notifyThemeResolutionChange = (detail: ThemeResolutionChangeDetail): void => {
  if (typeof window === 'undefined' || typeof window.dispatchEvent !== 'function') return

  window.dispatchEvent(new CustomEvent<ThemeResolutionChangeDetail>(
    THEME_RESOLUTION_CHANGE_EVENT,
    { detail },
  ))
}

const syncResolvedTheme = (theme: ResolvedThemeMode): void => {
  if (typeof document === 'undefined') return

  document.documentElement.classList.toggle('dark', theme === 'dark')
  document.documentElement.setAttribute('data-theme', theme)

  // Avoid pulling the Tauri/window module graph into plain web dev startup.
  // The native bridge exists synchronously when this code runs in the desktop
  // shell, so a simple feature check is enough to preserve native syncing.
  if (!('__TAURI__' in window) && !('__TAURI_INTERNALS__' in window)) {
    return
  }

  void import('@/utils/nativeWindowAppearance')
    .then(({ syncNativeWindowAppearance }) => syncNativeWindowAppearance(theme))
    .catch(() => {
      // 浏览器测试环境或非 Tauri 运行时允许静默降级。
    })
}

const syncResolvedFlavor = (theme: ResolvedThemeMode, flavor: FlavorMode): ResolvedFlavor => {
  const resolvedFlavor = resolveFlavorMode(theme, flavor)

  if (typeof document !== 'undefined') {
    document.documentElement.setAttribute('data-flavor', flavor)
    document.documentElement.setAttribute('data-resolved-flavor', resolvedFlavor)
  }

  return resolvedFlavor
}

const handleSystemThemeChange = (): void => {
  if (readStoredTheme() !== 'system') return

  const resolvedTheme = resolveSystemTheme()
  const flavor = readStoredFlavor()
  syncResolvedTheme(resolvedTheme)
  const resolvedFlavor = syncResolvedFlavor(resolvedTheme, flavor)
  notifyThemeResolutionChange({
    theme: 'system',
    resolvedTheme,
    flavor,
    resolvedFlavor,
  })
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

export const applyThemeToDocument = (
  theme: ThemeMode,
  flavor = readStoredFlavor(),
): ResolvedThemeMode => {
  ensureSystemThemeListener()

  const resolvedTheme = resolveThemeMode(theme)
  syncResolvedTheme(resolvedTheme)
  const resolvedFlavor = syncResolvedFlavor(resolvedTheme, flavor)
  notifyThemeResolutionChange({
    theme,
    resolvedTheme,
    flavor,
    resolvedFlavor,
  })
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

// 迁移表 → 白名单校验 → 非法值回退默认。
export const migrateFlavorValue = (stored: string | null | undefined): FlavorMode => {
  if (stored) {
    const migrated = FLAVOR_MIGRATION[stored]
    if (migrated) {
      return migrated
    }

    if (isFlavorMode(stored)) {
      return stored
    }
  }

  return DEFAULT_FLAVOR
}

export const migrateAccentValue = (stored: string | null | undefined): AccentMode => {
  if (stored) {
    const migrated = ACCENT_MIGRATION[stored]
    if (migrated) {
      return migrated
    }

    if (isAccentMode(stored)) {
      return stored
    }
  }

  return DEFAULT_ACCENT
}

export const readStoredFlavor = (): FlavorMode => {
  if (typeof window === 'undefined') {
    return DEFAULT_FLAVOR
  }

  try {
    return migrateFlavorValue(localStorage.getItem(FLAVOR_STORAGE_KEY))
  } catch {
    return DEFAULT_FLAVOR
  }
}

export const readStoredAccent = (): AccentMode => {
  if (typeof window === 'undefined') {
    return DEFAULT_ACCENT
  }

  try {
    return migrateAccentValue(localStorage.getItem(ACCENT_STORAGE_KEY))
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

// 存储值 ≠ 迁移值时写回，完成一次性迁移；键不存在时不播种默认值。
export const migratePersistedFlavor = (): void => {
  if (typeof window === 'undefined') return

  try {
    const stored = localStorage.getItem(FLAVOR_STORAGE_KEY)
    if (stored === null) return

    const migrated = migrateFlavorValue(stored)
    if (migrated !== stored) {
      persistFlavor(migrated)
    }
  } catch {
    // 忽略存储异常，读取侧仍会按迁移表回退。
  }
}

export const migratePersistedAccent = (): void => {
  if (typeof window === 'undefined') return

  try {
    const stored = localStorage.getItem(ACCENT_STORAGE_KEY)
    if (stored === null) return

    const migrated = migrateAccentValue(stored)
    if (migrated !== stored) {
      persistAccent(migrated)
    }
  } catch {
    // 忽略存储异常，读取侧仍会按迁移表回退。
  }
}

export const applyFlavorToDocument = (
  flavor: FlavorMode,
  theme = readStoredTheme(),
): FlavorMode => {
  ensureSystemThemeListener()

  const resolvedTheme = resolveThemeMode(theme)
  const resolvedFlavor = syncResolvedFlavor(resolvedTheme, flavor)
  notifyThemeResolutionChange({
    theme,
    resolvedTheme,
    flavor,
    resolvedFlavor,
  })
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
  const flavor = readStoredFlavor()
  applyThemeToDocument(theme, flavor)
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
