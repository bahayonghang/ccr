export type ThemeMode = 'light' | 'dark' | 'system'
export type ResolvedThemeMode = 'light' | 'dark'
export type FlavorMode = 'neutral' | 'clay'
export type ResolvedFlavor = FlavorMode
export type AccentMode = 'clay'

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
] as const
export const ACCENT_MODES: readonly AccentMode[] = [
  'clay',
] as const

export const DEFAULT_FLAVOR: FlavorMode = 'neutral'
export const DEFAULT_ACCENT: AccentMode = 'clay'

// 旧值域 → 新值域迁移表（flavor → neutral|clay、accent → clay）。
// 读取侧映射；store 初始化时把迁移结果写回 localStorage，index.html 首帧 IIFE 内联同一份逻辑。
const FLAVOR_MIGRATION: Readonly<Partial<Record<string, FlavorMode>>> = {
  paper: 'neutral',
  graphite: 'neutral',
  catppuccin: 'neutral',
  latte: 'neutral',
  frappe: 'neutral',
  macchiato: 'neutral',
  mocha: 'neutral',
}
const ACCENT_MIGRATION: Readonly<Partial<Record<string, AccentMode>>> = {
  mauve: 'clay',
  sage: 'clay',
  sky: 'clay',
  slate: 'clay',
  sand: 'clay',
  amber: 'clay',
  rose: 'clay',
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

export const resolveFlavorMode = (
  _resolvedTheme: ResolvedThemeMode,
  flavor: FlavorMode,
): ResolvedFlavor => {
  return flavor
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

// ---------------------------------------------------------------------------
// 自定义 accent（08-22-design-system design.md §10）：接受颜色值而非枚举成员，
// 运行时注入覆盖 [data-accent] 的第 1 层变量族。持久化与设置界面接线归
// 08-22-shell-port（其 R6），本层只提供变量结构与写入/清除原语。
// ---------------------------------------------------------------------------

/** 自定义 accent 输入：明暗两套主题的主色，`#rrggbb` 十六进制。 */
export interface CustomAccentDefinition {
  light: string
  dark?: string
}

/** data-accent 在自定义态下的值（运行时覆盖态，不入 AccentMode 持久化值域）。 */
export const CUSTOM_ACCENT_MODE = 'custom'

const CUSTOM_ACCENT_STYLE_ID = 'ccr-custom-accent'

/** accent 覆盖必须整族写入的第 1 层变量（与 [data-accent='clay'] 块的集合一致）。 */
export const CUSTOM_ACCENT_VARIABLE_FAMILY = [
  '--color-accent-primary',
  '--color-accent-primary-hover',
  '--color-accent-primary-active',
  '--color-accent-primary-rgb',
  '--color-accent-primary-glow',
  '--color-accent-primary-contrast',
  '--color-accent-primary-contrast-rgb',
  '--color-border-accent',
] as const

const HEX_COLOR_PATTERN = /^#([0-9a-f]{6})$/i

interface Color {
  r: number
  g: number
  b: number
}

const ACCENT_WHITE: Color = { r: 255, g: 248, b: 242 }
const ACCENT_INK: Color = { r: 29, g: 18, b: 7 }
const ACCENT_BLACK: Color = { r: 17, g: 18, b: 22 }

const parseHexColor = (input: string): Color | null => {
  const match = HEX_COLOR_PATTERN.exec(input.trim())
  if (!match) return null

  const hex = match[1]
  return {
    r: Number.parseInt(hex.slice(0, 2), 16),
    g: Number.parseInt(hex.slice(2, 4), 16),
    b: Number.parseInt(hex.slice(4, 6), 16),
  }
}

const mixColor = (base: Color, target: Color, ratio: number): Color => ({
  r: Math.round(base.r + (target.r - base.r) * ratio),
  g: Math.round(base.g + (target.g - base.g) * ratio),
  b: Math.round(base.b + (target.b - base.b) * ratio),
})

const toHex = ({ r, g, b }: Color): string =>
  `#${[r, g, b].map((channel) => channel.toString(16).padStart(2, '0')).join('')}`

const toTriplet = ({ r, g, b }: Color): string => `${r} ${g} ${b}`

// WCAG 相对亮度；≥0.3 视为亮主色（clay 亮色 0.23、暗色 0.36 之间的分界），
// 对比文字换深墨色，否则用暖白。推导值为运行时近似，非契约锚点。
const relativeLuminance = ({ r, g, b }: Color): number => {
  const channels = [r, g, b].map((channel) => {
    const scaled = channel / 255
    return scaled <= 0.03928 ? scaled / 12.92 : ((scaled + 0.055) / 1.055) ** 2.4
  })
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2]
}

const buildCustomAccentRule = (selector: string, primary: Color, isDarkTheme: boolean): string => {
  const hover = mixColor(primary, ACCENT_WHITE, 0.15)
  const active = mixColor(primary, ACCENT_BLACK, 0.12)
  const contrast = relativeLuminance(primary) >= 0.3 ? ACCENT_INK : ACCENT_WHITE
  const alpha = { glow: isDarkTheme ? '16%' : '10%', border: isDarkTheme ? '24%' : '18%' }

  return `${selector} {
  --color-accent-primary: ${toHex(primary)};
  --color-accent-primary-hover: ${toHex(hover)};
  --color-accent-primary-active: ${toHex(active)};
  --color-accent-primary-rgb: ${toTriplet(primary)};
  --color-accent-primary-glow: rgb(${toTriplet(primary)} / ${alpha.glow});
  --color-accent-primary-contrast: ${toHex(contrast)};
  --color-accent-primary-contrast-rgb: ${toTriplet(contrast)};
  --color-border-accent: rgb(${toTriplet(primary)} / ${alpha.border});
}`
}

/**
 * 注入自定义 accent：整族覆盖第 1 层 accent 变量并置 `data-accent='custom'`。
 * 输入非法（非 `#rrggbb`）时不改 DOM 并返回 false。
 */
export const applyCustomAccent = (definition: CustomAccentDefinition): boolean => {
  if (typeof document === 'undefined') return false

  const light = parseHexColor(definition.light)
  const dark = definition.dark === undefined ? light : parseHexColor(definition.dark)
  if (!light || !dark) return false

  const existing = document.getElementById(CUSTOM_ACCENT_STYLE_ID)
  const style = existing ?? document.createElement('style')
  style.id = CUSTOM_ACCENT_STYLE_ID
  style.textContent = [
    buildCustomAccentRule(`[data-accent='${CUSTOM_ACCENT_MODE}']`, light, false),
    buildCustomAccentRule(
      `[data-theme='dark'][data-accent='${CUSTOM_ACCENT_MODE}']`,
      dark,
      true,
    ),
  ].join('\n')
  if (!existing) {
    document.head.appendChild(style)
  }

  document.documentElement.setAttribute('data-accent', CUSTOM_ACCENT_MODE)
  return true
}

/** 清除自定义 accent，恢复到指定枚举 accent（默认 clay）。 */
export const clearCustomAccent = (fallback: AccentMode = DEFAULT_ACCENT): AccentMode => {
  if (typeof document !== 'undefined') {
    document.getElementById(CUSTOM_ACCENT_STYLE_ID)?.remove()
  }
  return applyAccentToDocument(fallback)
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
