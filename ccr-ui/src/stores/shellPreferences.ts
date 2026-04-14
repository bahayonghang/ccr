import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { getSkipExitConfirm, setSkipExitConfirm } from '@/api/runtime/environment'
import {
  normalizeLocale,
  readStoredLocale,
  setLocale,
  type SupportedLocale,
} from '@/i18n'
import { isPerfTelemetryEnabled, setPerfTelemetryEnabled } from '@/utils/perfTelemetry'
import {
  applyThemeToDocument,
  persistTheme,
  readStoredTheme,
  resolveThemeMode,
  type ResolvedThemeMode,
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
  const locale = ref<SupportedLocale>(readStoredLocale())
  const sidebarWidth = ref<number>(readStoredSidebarWidth())
  const confirmBeforeExit = ref(true)
  const perfTelemetryEnabled = ref(isPerfTelemetryEnabled())
  const runtimeHydrated = ref(false)

  const localeLabel = computed(() => (locale.value === 'en-US' ? 'English' : '中文'))

  const initializeTheme = (): void => {
    theme.value = readStoredTheme()
    effectiveTheme.value = applyThemeToDocument(theme.value)
  }

  const setThemePreference = (nextTheme: ThemeMode): void => {
    theme.value = nextTheme
    persistTheme(nextTheme)
    effectiveTheme.value = applyThemeToDocument(nextTheme)
  }

  const toggleThemePreference = (): void => {
    const nextTheme = effectiveTheme.value === 'dark' ? 'light' : 'dark'
    setThemePreference(nextTheme)
  }

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

  const hydrateRuntimePreferences = async (): Promise<void> => {
    if (runtimeHydrated.value) return

    try {
      confirmBeforeExit.value = !(await getSkipExitConfirm())
    } catch {
      confirmBeforeExit.value = true
    } finally {
      runtimeHydrated.value = true
    }
  }

  const setConfirmBeforeExitPreference = async (enabled: boolean): Promise<void> => {
    confirmBeforeExit.value = enabled
    await setSkipExitConfirm(!enabled)
  }

  const setPerfTelemetryPreference = (enabled: boolean): void => {
    perfTelemetryEnabled.value = enabled
    setPerfTelemetryEnabled(enabled)
  }

  return {
    theme,
    effectiveTheme,
    locale,
    localeLabel,
    sidebarWidth,
    confirmBeforeExit,
    perfTelemetryEnabled,
    runtimeHydrated,
    initializeTheme,
    setTheme: setThemePreference,
    toggleTheme: toggleThemePreference,
    setLocalePreference,
    updateSidebarWidth,
    commitSidebarWidth,
    resetLayout,
    hydrateRuntimePreferences,
    setConfirmBeforeExit: setConfirmBeforeExitPreference,
    setPerfTelemetryPreference,
  }
})
