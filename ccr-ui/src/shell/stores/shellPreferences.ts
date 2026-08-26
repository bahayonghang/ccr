import { create } from 'zustand'
import {
  shellGetPreferences,
  shellSetPreferences,
  type DesktopShellPreferences,
} from '@/api/runtime/environment'
import { normalizeLocale, readStoredLocale, setLocale, type SupportedLocale } from '@/i18n'
import { isPerfTelemetryEnabled, setPerfTelemetryEnabled } from '@/utils/perfTelemetry'
import {
  applyCodeFontToDocument,
  applyFontsToDocument,
  applyUiFontToDocument,
  persistCodeFont,
  persistUiFont,
  readStoredCodeFont,
  readStoredUiFont,
  sanitizeFontFamily,
} from '@/utils/fontPreferences'
import {
  applyAccentToDocument,
  applyFlavorToDocument,
  applyThemeToDocument,
  migrateAccentValue,
  migrateFlavorValue,
  migratePersistedAccent,
  migratePersistedFlavor,
  persistAccent,
  persistFlavor,
  persistTheme,
  readStoredAccent,
  readStoredFlavor,
  readStoredTheme,
  resolveFlavorMode,
  resolveThemeMode,
  THEME_RESOLUTION_CHANGE_EVENT,
  type AccentMode,
  type FlavorMode,
  type ResolvedFlavor,
  type ResolvedThemeMode,
  type ThemeResolutionChangeDetail,
  type ThemeMode,
} from '@/utils/themeBootstrap'

// shellPreferences store（08-22-state-logic-port 批次 4；原 Pinia `stores/shellPreferences.ts` 语义等价迁移）。
//
// 持久化偏差记录：不用 zustand/persist 中间件。原实现的持久化按 key 分散在
// themeBootstrap / fontPreferences 工具内（ccr-theme / ccr-flavor / ccr-accent /
// ccr-font-* / ccr-sidebar-width），且首帧 IIFE 与迁移表和这些 key 逐字节对齐
// （tests/theme-bootstrap 行为锁）。换 persist 中间件的单一 blob 会改变
// key 布局并破坏该契约，故持久化继续经原工具函数逐 key 写入——「存储键不变」
// 以原语义满足。

const SIDEBAR_WIDTH_STORAGE_KEY = 'ccr-sidebar-width'

export const DEFAULT_SIDEBAR_WIDTH = 240
export const MIN_SIDEBAR_WIDTH = 200
export const MAX_SIDEBAR_WIDTH = 480

export const clampSidebarWidth = (value: number): number => {
  if (!Number.isFinite(value)) return DEFAULT_SIDEBAR_WIDTH
  return Math.min(MAX_SIDEBAR_WIDTH, Math.max(MIN_SIDEBAR_WIDTH, Math.round(value)))
}

export const readStoredSidebarWidth = (): number => {
  if (typeof window === 'undefined') {
    return DEFAULT_SIDEBAR_WIDTH
  }

  try {
    const storedWidth = localStorage.getItem(SIDEBAR_WIDTH_STORAGE_KEY)
    return storedWidth ? clampSidebarWidth(Number(storedWidth)) : DEFAULT_SIDEBAR_WIDTH
  } catch {
    return DEFAULT_SIDEBAR_WIDTH
  }
}

const persistSidebarWidth = (value: number): void => {
  if (typeof window === 'undefined') return

  try {
    localStorage.setItem(SIDEBAR_WIDTH_STORAGE_KEY, String(clampSidebarWidth(value)))
  } catch {
    // 忽略存储异常，保持界面可用。
  }
}

interface ShellPreferencesState {
  theme: ThemeMode
  effectiveTheme: ResolvedThemeMode
  flavor: FlavorMode
  resolvedFlavor: ResolvedFlavor
  accent: AccentMode
  uiFont: string
  codeFont: string
  locale: SupportedLocale
  localeLabel: string
  sidebarWidth: number
  confirmBeforeExit: boolean
  closeToTray: boolean
  openPanelOnTrayClick: boolean
  perfTelemetryEnabled: boolean
  runtimeHydrated: boolean
  initializeTheme: () => void
  setTheme: (nextTheme: ThemeMode) => void
  toggleTheme: () => void
  setFlavor: (nextFlavor: FlavorMode) => void
  setAccent: (nextAccent: AccentMode) => void
  setUiFont: (nextFont: string) => void
  setCodeFont: (nextFont: string) => void
  setLocalePreference: (nextLocale: string) => Promise<SupportedLocale>
  updateSidebarWidth: (nextWidth: number, persist?: boolean) => number
  commitSidebarWidth: () => void
  resetLayout: () => void
  hydrateRuntimePreferences: () => Promise<void>
  setConfirmBeforeExit: (enabled: boolean) => Promise<void>
  setCloseToTray: (enabled: boolean) => Promise<void>
  setOpenPanelOnTrayClick: (enabled: boolean) => Promise<void>
  setPerfTelemetryPreference: (enabled: boolean) => void
}

const localeLabelOf = (locale: SupportedLocale): string =>
  locale === 'en-US' ? 'English' : '中文'

export const useShellPreferencesStore = create<ShellPreferencesState>()((set, get) => ({
  theme: readStoredTheme(),
  effectiveTheme: resolveThemeMode(readStoredTheme()),
  flavor: readStoredFlavor(),
  resolvedFlavor: resolveFlavorMode(resolveThemeMode(readStoredTheme()), readStoredFlavor()),
  accent: readStoredAccent(),
  uiFont: readStoredUiFont(),
  codeFont: readStoredCodeFont(),
  locale: readStoredLocale(),
  localeLabel: localeLabelOf(readStoredLocale()),
  sidebarWidth: readStoredSidebarWidth(),
  confirmBeforeExit: true,
  closeToTray: false,
  openPanelOnTrayClick: true,
  perfTelemetryEnabled: isPerfTelemetryEnabled(),
  runtimeHydrated: false,

  initializeTheme: () => {
    const theme = readStoredTheme()
    // 旧值域存储值先写回迁移结果，再按迁移后值域读取。
    migratePersistedFlavor()
    migratePersistedAccent()
    const flavor = readStoredFlavor()
    const accent = readStoredAccent()
    const effectiveTheme = applyThemeToDocument(theme, flavor)
    const resolvedFlavor = resolveFlavorMode(effectiveTheme, flavor)
    applyAccentToDocument(accent)
    const uiFont = readStoredUiFont()
    const codeFont = readStoredCodeFont()
    applyFontsToDocument(uiFont, codeFont)
    set({
      theme,
      flavor,
      accent,
      effectiveTheme,
      resolvedFlavor,
      uiFont,
      codeFont,
    })
  },

  setTheme: (nextTheme) => {
    const { flavor } = get()
    persistTheme(nextTheme)
    const effectiveTheme = applyThemeToDocument(nextTheme, flavor)
    set({
      theme: nextTheme,
      effectiveTheme,
      resolvedFlavor: resolveFlavorMode(effectiveTheme, flavor),
    })
  },

  toggleTheme: () => {
    const nextTheme = get().effectiveTheme === 'dark' ? 'light' : 'dark'
    get().setTheme(nextTheme)
  },

  setFlavor: (nextFlavor) => {
    // 兼容旧选项 UI 传入的旧值域：写入前统一迁移到新值域。
    const normalizedFlavor = migrateFlavorValue(nextFlavor)
    persistFlavor(normalizedFlavor)
    applyFlavorToDocument(normalizedFlavor, get().theme)
    set({
      flavor: normalizedFlavor,
      resolvedFlavor: resolveFlavorMode(get().effectiveTheme, normalizedFlavor),
    })
  },

  setAccent: (nextAccent) => {
    const normalizedAccent = migrateAccentValue(nextAccent)
    persistAccent(normalizedAccent)
    applyAccentToDocument(normalizedAccent)
    set({ accent: normalizedAccent })
  },

  // 字体偏好：空串表示回到内置栈。净化后统一走 persist + apply。
  setUiFont: (nextFont) => {
    const sanitized = sanitizeFontFamily(nextFont)
    persistUiFont(sanitized)
    applyUiFontToDocument(sanitized)
    set({ uiFont: sanitized })
  },

  setCodeFont: (nextFont) => {
    const sanitized = sanitizeFontFamily(nextFont)
    persistCodeFont(sanitized)
    applyCodeFontToDocument(sanitized)
    set({ codeFont: sanitized })
  },

  setLocalePreference: async (nextLocale) => {
    const normalized = normalizeLocale(nextLocale)
    await setLocale(normalized)
    set({ locale: normalized, localeLabel: localeLabelOf(normalized) })
    return normalized
  },

  updateSidebarWidth: (nextWidth, persist = true) => {
    const clampedWidth = clampSidebarWidth(nextWidth)
    if (persist) {
      persistSidebarWidth(clampedWidth)
    }
    set({ sidebarWidth: clampedWidth })
    return clampedWidth
  },

  commitSidebarWidth: () => {
    persistSidebarWidth(get().sidebarWidth)
  },

  resetLayout: () => {
    get().updateSidebarWidth(DEFAULT_SIDEBAR_WIDTH)
  },

  hydrateRuntimePreferences: async () => {
    if (get().runtimeHydrated) return

    try {
      const preferences = await shellGetPreferences()
      set({
        confirmBeforeExit: preferences.confirm_before_exit,
        closeToTray: preferences.close_to_tray,
        openPanelOnTrayClick: preferences.open_panel_on_tray_click,
        runtimeHydrated: true,
      })
    } catch {
      set({ confirmBeforeExit: true, closeToTray: false, openPanelOnTrayClick: true, runtimeHydrated: true })
    }
  },

  setConfirmBeforeExit: async (enabled) => {
    set({ confirmBeforeExit: enabled })
    await syncRuntimePreferences({ confirm_before_exit: enabled })
  },

  setCloseToTray: async (enabled) => {
    set({ closeToTray: enabled })
    await syncRuntimePreferences({ close_to_tray: enabled })
  },

  setOpenPanelOnTrayClick: async (enabled) => {
    set({ openPanelOnTrayClick: enabled })
    await syncRuntimePreferences({ open_panel_on_tray_click: enabled })
  },

  setPerfTelemetryPreference: (enabled) => {
    setPerfTelemetryEnabled(enabled)
    set({ perfTelemetryEnabled: enabled })
  },
}))

// runtime 偏好向后端的异步写（flush）。动作体内引用（模块作用域闭包），
// 不入公开 state 面——与原 store 的内部函数等价。

/** runtime 偏好向后端的异步写（flush），语义与原 store 相同。 */
export const syncRuntimePreferences = async (
  patch?: Partial<DesktopShellPreferences>,
): Promise<DesktopShellPreferences | null> => {
  const state = useShellPreferencesStore.getState()
  try {
    const current = await shellGetPreferences()
    const next = {
      ...current,
      confirm_before_exit: state.confirmBeforeExit,
      close_to_tray: state.closeToTray,
      open_panel_on_tray_click: state.openPanelOnTrayClick,
      ...patch,
    }
    const saved = await shellSetPreferences(next)
    useShellPreferencesStore.setState({
      confirmBeforeExit: saved.confirm_before_exit,
      closeToTray: saved.close_to_tray,
      openPanelOnTrayClick: saved.open_panel_on_tray_click,
    })
    return saved
  } catch {
    return null
  }
}

// 主题解析变化事件（system 模式下 OS 偏好变化）→ 同步 effectiveTheme/resolvedFlavor。
// 模块级注册一次，与原 Pinia setup 的注册时机等价。
if (typeof window !== 'undefined') {
  window.addEventListener(THEME_RESOLUTION_CHANGE_EVENT, ((event: Event) => {
    const detail = (event as CustomEvent<ThemeResolutionChangeDetail>).detail
    if (!detail) return
    const state = useShellPreferencesStore.getState()
    const patch: Partial<ShellPreferencesState> = {}
    if (state.theme === 'system' || detail.theme === state.theme) {
      patch.effectiveTheme = detail.resolvedTheme
    }
    if (detail.flavor === state.flavor) {
      patch.resolvedFlavor = detail.resolvedFlavor
    }
    if (Object.keys(patch).length > 0) {
      useShellPreferencesStore.setState(patch)
    }
  }) as EventListener)
}

// 原 Pinia store 在创建时执行 initializeTheme()；模块加载即初始化，等价。
useShellPreferencesStore.getState().initializeTheme()
