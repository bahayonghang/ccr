import {
  shellGetPreferences,
  shellSetPreferences,
  type DesktopShellPreferences,
} from '@/api/runtime/environment'
import { normalizeLocale, setLocale, type SupportedLocale } from '@/i18n'
import {
  applyCodeFontToDocument,
  applyUiFontToDocument,
  persistCodeFont,
  persistUiFont,
  sanitizeFontFamily,
} from '@/utils/fontPreferences'
import { isPerfTelemetryEnabled, setPerfTelemetryEnabled } from '@/utils/perfTelemetry'
import {
  applyFlavorToDocument,
  applyThemeToDocument,
  persistFlavor,
  persistTheme,
  type FlavorMode,
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
  if (typeof window === 'undefined') return DEFAULT_SIDEBAR_WIDTH
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
    // 忽略存储异常
  }
}

type ShellPreferencesStore = typeof import('@/shell/stores/shellPreferences').useShellPreferencesStore

const withShellStore = async (
  run: (store: ShellPreferencesStore) => void,
): Promise<void> => {
  try {
    const mod = await import('@/shell/stores/shellPreferences')
    run(mod.useShellPreferencesStore)
  } catch {
    // 动态加载失败时仍完成 utils/api 写入
  }
}

export function applyThemePreference(theme: ThemeMode): void {
  persistTheme(theme)
  applyThemeToDocument(theme)
  void withShellStore((store) => store.getState().setTheme(theme))
}

export function applyFlavorPreference(flavor: FlavorMode): void {
  persistFlavor(flavor)
  applyFlavorToDocument(flavor)
  void withShellStore((store) => store.getState().setFlavor(flavor))
}

export function applyUiFontPreference(font: string): void {
  const sanitized = sanitizeFontFamily(font)
  persistUiFont(sanitized)
  applyUiFontToDocument(sanitized)
  void withShellStore((store) => store.getState().setUiFont(sanitized))
}

export function applyCodeFontPreference(font: string): void {
  const sanitized = sanitizeFontFamily(font)
  persistCodeFont(sanitized)
  applyCodeFontToDocument(sanitized)
  void withShellStore((store) => store.getState().setCodeFont(sanitized))
}

export async function applyLocalePreference(locale: string): Promise<SupportedLocale> {
  const normalized = normalizeLocale(locale)
  await setLocale(normalized)
  await withShellStore((store) => {
    void store.getState().setLocalePreference(normalized)
  })
  return normalized
}

export function applySidebarWidth(nextWidth: number): number {
  const clamped = clampSidebarWidth(nextWidth)
  persistSidebarWidth(clamped)
  void withShellStore((store) => store.getState().updateSidebarWidth(clamped))
  return clamped
}

export function resetLayoutWidth(): number {
  return applySidebarWidth(DEFAULT_SIDEBAR_WIDTH)
}

export function applyPerfTelemetry(enabled: boolean): void {
  setPerfTelemetryEnabled(enabled)
  void withShellStore((store) => store.getState().setPerfTelemetryPreference(enabled))
}

export function readPerfTelemetry(): boolean {
  return isPerfTelemetryEnabled()
}

export async function loadRuntimePreferences(): Promise<
  Pick<DesktopShellPreferences, 'confirm_before_exit' | 'close_to_tray' | 'open_panel_on_tray_click'>
> {
  try {
    const preferences = await shellGetPreferences()
    void withShellStore((store) => {
      void store.getState().hydrateRuntimePreferences()
    })
    return preferences
  } catch {
    return {
      confirm_before_exit: true,
      close_to_tray: false,
      open_panel_on_tray_click: true,
    }
  }
}

export async function patchRuntimePreferences(
  patch: Partial<DesktopShellPreferences>,
): Promise<DesktopShellPreferences | null> {
  try {
    const current = await shellGetPreferences()
    const saved = await shellSetPreferences({ ...current, ...patch })
    void withShellStore((store) => {
      store.setState({
        confirmBeforeExit: saved.confirm_before_exit,
        closeToTray: saved.close_to_tray,
        openPanelOnTrayClick: saved.open_panel_on_tray_click,
      })
    })
    return saved
  } catch {
    return null
  }
}
