import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import {
  shellGetPreferences,
  shellSetPreferences,
  type DesktopShellPreferences,
} from '@/api/runtime/environment'
import {
  normalizeLocale,
  readStoredLocale,
  setLocale,
  type SupportedLocale,
} from '@/i18n'
import { isPerfTelemetryEnabled, setPerfTelemetryEnabled } from '@/utils/perfTelemetry'
import {
  applyAccentToDocument,
  applyFlavorToDocument,
  applyThemeToDocument,
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
  type ResolvedThemeMode,
  type ThemeResolutionChangeDetail,
  type ThemeMode,
} from '@/utils/themeBootstrap'

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

export const useShellPreferencesStore = defineStore('shellPreferences', () => {
  const theme = ref<ThemeMode>(readStoredTheme())
  const effectiveTheme = ref<ResolvedThemeMode>(resolveThemeMode(theme.value))
  const flavor = ref<FlavorMode>(readStoredFlavor())
  const resolvedFlavor = ref<FlavorMode>(resolveFlavorMode(effectiveTheme.value, flavor.value))
  const accent = ref<AccentMode>(readStoredAccent())
  const locale = ref<SupportedLocale>(readStoredLocale())
  const sidebarWidth = ref<number>(readStoredSidebarWidth())
  const confirmBeforeExit = ref(true)
  const closeToTray = ref(false)
  const openPanelOnTrayClick = ref(true)
  const perfTelemetryEnabled = ref(isPerfTelemetryEnabled())
  const runtimeHydrated = ref(false)

  const localeLabel = computed(() => (locale.value === 'en-US' ? 'English' : '中文'))

  const initializeTheme = (): void => {
    theme.value = readStoredTheme()
    flavor.value = readStoredFlavor()
    accent.value = readStoredAccent()
    effectiveTheme.value = applyThemeToDocument(theme.value, flavor.value)
    resolvedFlavor.value = resolveFlavorMode(effectiveTheme.value, flavor.value)
    applyAccentToDocument(accent.value)
  }

  const setThemePreference = (nextTheme: ThemeMode): void => {
    theme.value = nextTheme
    persistTheme(nextTheme)
    effectiveTheme.value = applyThemeToDocument(nextTheme, flavor.value)
    resolvedFlavor.value = resolveFlavorMode(effectiveTheme.value, flavor.value)
  }

  const toggleThemePreference = (): void => {
    const nextTheme = effectiveTheme.value === 'dark' ? 'light' : 'dark'
    setThemePreference(nextTheme)
  }

  const setFlavorPreference = (nextFlavor: FlavorMode): void => {
    flavor.value = nextFlavor
    persistFlavor(nextFlavor)
    applyFlavorToDocument(nextFlavor, theme.value)
    resolvedFlavor.value = resolveFlavorMode(effectiveTheme.value, nextFlavor)
  }

  const setAccentPreference = (nextAccent: AccentMode): void => {
    accent.value = nextAccent
    persistAccent(nextAccent)
    applyAccentToDocument(nextAccent)
  }

  const syncThemeResolution = (detail: ThemeResolutionChangeDetail): void => {
    if (theme.value === 'system' || detail.theme === theme.value) {
      effectiveTheme.value = detail.resolvedTheme
    }

    if (detail.flavor === flavor.value) {
      resolvedFlavor.value = detail.resolvedFlavor
    }
  }

  if (typeof window !== 'undefined') {
    window.addEventListener(THEME_RESOLUTION_CHANGE_EVENT, ((event: Event) => {
      const detail = (event as CustomEvent<ThemeResolutionChangeDetail>).detail
      if (detail) {
        syncThemeResolution(detail)
      }
    }) as EventListener)
  }

  initializeTheme()

  const setLocalePreference = async (nextLocale: string): Promise<SupportedLocale> => {
    const normalized = normalizeLocale(nextLocale)
    await setLocale(normalized)
    locale.value = normalized
    return normalized
  }

  const updateSidebarWidth = (nextWidth: number, persist = true): number => {
    const clampedWidth = clampSidebarWidth(nextWidth)
    sidebarWidth.value = clampedWidth

    if (persist) {
      persistSidebarWidth(clampedWidth)
    }

    return clampedWidth
  }

  const commitSidebarWidth = (): void => {
    persistSidebarWidth(sidebarWidth.value)
  }

  const resetLayout = (): void => {
    updateSidebarWidth(DEFAULT_SIDEBAR_WIDTH)
  }

  const syncRuntimePreferences = async (
    patch?: Partial<DesktopShellPreferences>
  ): Promise<DesktopShellPreferences | null> => {
    try {
      const current = await shellGetPreferences()
      const next = {
        ...current,
        confirm_before_exit: confirmBeforeExit.value,
        close_to_tray: closeToTray.value,
        open_panel_on_tray_click: openPanelOnTrayClick.value,
        ...patch,
      }
      const saved = await shellSetPreferences(next)
      confirmBeforeExit.value = saved.confirm_before_exit
      closeToTray.value = saved.close_to_tray
      openPanelOnTrayClick.value = saved.open_panel_on_tray_click
      return saved
    } catch {
      return null
    }
  }

  const hydrateRuntimePreferences = async (): Promise<void> => {
    if (runtimeHydrated.value) return

    try {
      const preferences = await shellGetPreferences()
      confirmBeforeExit.value = preferences.confirm_before_exit
      closeToTray.value = preferences.close_to_tray
      openPanelOnTrayClick.value = preferences.open_panel_on_tray_click
    } catch {
      confirmBeforeExit.value = true
      closeToTray.value = false
      openPanelOnTrayClick.value = true
    } finally {
      runtimeHydrated.value = true
    }
  }

  const setConfirmBeforeExitPreference = async (enabled: boolean): Promise<void> => {
    confirmBeforeExit.value = enabled
    await syncRuntimePreferences({ confirm_before_exit: enabled })
  }

  const setCloseToTrayPreference = async (enabled: boolean): Promise<void> => {
    closeToTray.value = enabled
    await syncRuntimePreferences({ close_to_tray: enabled })
  }

  const setOpenPanelOnTrayClickPreference = async (enabled: boolean): Promise<void> => {
    openPanelOnTrayClick.value = enabled
    await syncRuntimePreferences({ open_panel_on_tray_click: enabled })
  }

  const setPerfTelemetryPreference = (enabled: boolean): void => {
    perfTelemetryEnabled.value = enabled
    setPerfTelemetryEnabled(enabled)
  }

  return {
    theme,
    effectiveTheme,
    flavor,
    resolvedFlavor,
    accent,
    locale,
    localeLabel,
    sidebarWidth,
    confirmBeforeExit,
    closeToTray,
    openPanelOnTrayClick,
    perfTelemetryEnabled,
    runtimeHydrated,
    initializeTheme,
    setTheme: setThemePreference,
    toggleTheme: toggleThemePreference,
    setFlavor: setFlavorPreference,
    setAccent: setAccentPreference,
    setLocalePreference,
    updateSidebarWidth,
    commitSidebarWidth,
    resetLayout,
    hydrateRuntimePreferences,
    setConfirmBeforeExit: setConfirmBeforeExitPreference,
    setCloseToTray: setCloseToTrayPreference,
    setOpenPanelOnTrayClick: setOpenPanelOnTrayClickPreference,
    setPerfTelemetryPreference,
  }
})
