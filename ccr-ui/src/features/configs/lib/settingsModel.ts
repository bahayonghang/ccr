import { z } from 'zod'
import type { FlavorMode, ResolvedThemeMode, ThemeMode } from '@/utils/themeBootstrap'
import type { SupportedLocale } from '@/i18n'
import { CODE_FONT_PRESETS, UI_FONT_PRESETS } from '@/utils/fontPreferences'
import { readStoredCodeFont, readStoredUiFont } from '@/utils/fontPreferences'
import { readStoredFlavor, readStoredTheme, resolveThemeMode } from '@/utils/themeBootstrap'
import { readStoredLocale } from '@/i18n'
import { readPerfTelemetry, readStoredSidebarWidth } from './preferences'

export type SettingsSectionKey = 'appearance' | 'language' | 'shell' | 'diagnostics'

export const appSettingsSchema = z.object({
  theme: z.enum(['light', 'dark', 'system']),
  flavor: z.enum(['neutral', 'clay']),
  locale: z.enum(['zh-CN', 'en-US']),
  uiFont: z.string(),
  codeFont: z.string(),
  uiSelect: z.string(),
  codeSelect: z.string(),
  confirmBeforeExit: z.boolean(),
  closeToTray: z.boolean(),
  openPanelOnTrayClick: z.boolean(),
  sidebarWidth: z.number().min(200).max(480),
  perfTelemetryEnabled: z.boolean(),
})

export type AppSettingsForm = z.infer<typeof appSettingsSchema>

export const FONT_DEFAULT = '__default__'
export const FONT_CUSTOM = '__custom__'

export function fontSelectValue(font: string, presets: readonly string[], customActive: boolean): string {
  if (customActive) return FONT_CUSTOM
  if (font === '') return FONT_DEFAULT
  return presets.includes(font) ? font : FONT_CUSTOM
}

export function isCustomFont(font: string, presets: readonly string[]): boolean {
  return font !== '' && !presets.includes(font)
}

export function previewFamily(font: string, baseVar: string): string {
  return font ? `"${font}", var(${baseVar})` : `var(${baseVar})`
}

export interface AppSettingsSnapshot {
  theme: ThemeMode
  flavor: FlavorMode
  effectiveTheme: ResolvedThemeMode
  locale: SupportedLocale
  uiFont: string
  codeFont: string
  sidebarWidth: number
  perfTelemetryEnabled: boolean
}

export function readSettingsSnapshot(): AppSettingsSnapshot {
  const theme = readStoredTheme()
  return {
    theme,
    flavor: readStoredFlavor(),
    effectiveTheme: resolveThemeMode(theme),
    locale: readStoredLocale(),
    uiFont: readStoredUiFont(),
    codeFont: readStoredCodeFont(),
    sidebarWidth: readStoredSidebarWidth(),
    perfTelemetryEnabled: readPerfTelemetry(),
  }
}

export function formFromSnapshot(snapshot: AppSettingsSnapshot, runtime: {
  confirmBeforeExit: boolean
  closeToTray: boolean
  openPanelOnTrayClick: boolean
}): AppSettingsForm {
  return {
    theme: snapshot.theme,
    flavor: snapshot.flavor,
    locale: snapshot.locale,
    uiFont: snapshot.uiFont,
    codeFont: snapshot.codeFont,
    uiSelect: fontSelectValue(snapshot.uiFont, UI_FONT_PRESETS, isCustomFont(snapshot.uiFont, UI_FONT_PRESETS)),
    codeSelect: fontSelectValue(
      snapshot.codeFont,
      CODE_FONT_PRESETS,
      isCustomFont(snapshot.codeFont, CODE_FONT_PRESETS),
    ),
    confirmBeforeExit: runtime.confirmBeforeExit,
    closeToTray: runtime.closeToTray,
    openPanelOnTrayClick: runtime.openPanelOnTrayClick,
    sidebarWidth: snapshot.sidebarWidth,
    perfTelemetryEnabled: snapshot.perfTelemetryEnabled,
  }
}

export const SETTINGS_SECTIONS: Array<{ key: SettingsSectionKey; icon: string; titleKey: string; captionKey: string }> = [
  { key: 'appearance', icon: 'Sun', titleKey: 'settings.appearance.title', captionKey: 'settings.appearance.navCaption' },
  { key: 'language', icon: 'Languages', titleKey: 'settings.language.title', captionKey: 'settings.language.navCaption' },
  { key: 'shell', icon: 'PanelLeftOpen', titleKey: 'settings.shell.title', captionKey: 'settings.shell.navCaption' },
  { key: 'diagnostics', icon: 'Activity', titleKey: 'settings.diagnostics.title', captionKey: 'settings.diagnostics.navCaption' },
]
