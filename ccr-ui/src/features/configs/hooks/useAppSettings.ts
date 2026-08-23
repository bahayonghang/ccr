import { useCallback, useEffect, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { getEnvironmentName, getTauriVersion, isTauriEnvironment } from '@/api/runtime/environment'
import { translateWithFallback } from '@/i18n/formatMessage'
import { CODE_FONT_PRESETS, UI_FONT_PRESETS } from '@/utils/fontPreferences'
import type { FlavorMode, ThemeMode } from '@/utils/themeBootstrap'
import { t } from '../locale'
import {
  appSettingsSchema,
  FONT_CUSTOM,
  FONT_DEFAULT,
  fontSelectValue,
  formFromSnapshot,
  isCustomFont,
  previewFamily,
  readSettingsSnapshot,
  SETTINGS_SECTIONS,
  type AppSettingsForm,
  type SettingsSectionKey,
} from '../lib/settingsModel'
import {
  applyCodeFontPreference,
  applyFlavorPreference,
  applyLocalePreference,
  applyPerfTelemetry,
  applySidebarWidth,
  applyThemePreference,
  applyUiFontPreference,
  loadRuntimePreferences,
  patchRuntimePreferences,
  resetLayoutWidth,
} from '../lib/preferences'

export function useAppSettings() {
  const snapshot = readSettingsSnapshot()
  const [effectiveTheme, setEffectiveTheme] = useState(snapshot.effectiveTheme)
  const [runtimeVersion, setRuntimeVersion] = useState<string | null>(null)
  const [activeSection, setActiveSection] = useState<SettingsSectionKey>('appearance')
  const [uiCustom, setUiCustom] = useState(isCustomFont(snapshot.uiFont, UI_FONT_PRESETS))
  const [codeCustom, setCodeCustom] = useState(isCustomFont(snapshot.codeFont, CODE_FONT_PRESETS))
  const form = useForm<AppSettingsForm>({
    resolver: zodResolver(appSettingsSchema),
    defaultValues: formFromSnapshot(snapshot, {
      confirmBeforeExit: true,
      closeToTray: false,
      openPanelOnTrayClick: true,
    }),
  })
  const { register, watch, setValue, getValues } = form
  const theme = watch('theme')
  const flavor = watch('flavor')
  const locale = watch('locale')
  const uiFont = watch('uiFont')
  const codeFont = watch('codeFont')
  const confirmBeforeExit = watch('confirmBeforeExit')
  const closeToTray = watch('closeToTray')
  const openPanelOnTrayClick = watch('openPanelOnTrayClick')
  const sidebarWidth = watch('sidebarWidth')
  const perfTelemetryEnabled = watch('perfTelemetryEnabled')

  useEffect(() => {
    void loadRuntimePreferences().then((prefs) => {
      setValue('confirmBeforeExit', prefs.confirm_before_exit)
      setValue('closeToTray', prefs.close_to_tray)
      setValue('openPanelOnTrayClick', prefs.open_panel_on_tray_click)
    })
    if (isTauriEnvironment()) {
      void getTauriVersion().then(setRuntimeVersion)
    }
  }, [setValue])

  const setTheme = useCallback(
    (next: ThemeMode) => {
      setValue('theme', next)
      applyThemePreference(next)
      setEffectiveTheme(next === 'system' ? readSettingsSnapshot().effectiveTheme : next)
    },
    [setValue],
  )
  const setFlavor = useCallback(
    (next: FlavorMode) => {
      setValue('flavor', next)
      applyFlavorPreference(next)
    },
    [setValue],
  )
  const setLocaleValue = useCallback(
    async (next: string) => {
      const normalized = await applyLocalePreference(next)
      setValue('locale', normalized)
    },
    [setValue],
  )
  const onUiSelect = useCallback(
    (value: string) => {
      if (value === FONT_CUSTOM) {
        setUiCustom(true)
        setValue('uiSelect', FONT_CUSTOM)
        return
      }
      setUiCustom(false)
      const font = value === FONT_DEFAULT ? '' : value
      setValue('uiSelect', value)
      setValue('uiFont', font)
      applyUiFontPreference(font)
    },
    [setValue],
  )
  const onCodeSelect = useCallback(
    (value: string) => {
      if (value === FONT_CUSTOM) {
        setCodeCustom(true)
        setValue('codeSelect', FONT_CUSTOM)
        return
      }
      setCodeCustom(false)
      const font = value === FONT_DEFAULT ? '' : value
      setValue('codeSelect', value)
      setValue('codeFont', font)
      applyCodeFontPreference(font)
    },
    [setValue],
  )
  const setUiFont = useCallback(
    (value: string) => {
      setValue('uiFont', value)
      applyUiFontPreference(value)
    },
    [setValue],
  )
  const setCodeFont = useCallback(
    (value: string) => {
      setValue('codeFont', value)
      applyCodeFontPreference(value)
    },
    [setValue],
  )

  const toggleConfirm = useCallback(async () => {
    const next = !getValues('confirmBeforeExit')
    setValue('confirmBeforeExit', next)
    await patchRuntimePreferences({ confirm_before_exit: next })
  }, [getValues, setValue])
  const toggleTray = useCallback(async () => {
    const next = !getValues('closeToTray')
    setValue('closeToTray', next)
    await patchRuntimePreferences({ close_to_tray: next })
  }, [getValues, setValue])
  const togglePanel = useCallback(async () => {
    const next = !getValues('openPanelOnTrayClick')
    setValue('openPanelOnTrayClick', next)
    await patchRuntimePreferences({ open_panel_on_tray_click: next })
  }, [getValues, setValue])
  const togglePerf = useCallback(() => {
    const next = !getValues('perfTelemetryEnabled')
    setValue('perfTelemetryEnabled', next)
    applyPerfTelemetry(next)
  }, [getValues, setValue])
  const onSidebar = useCallback(
    (event: { target: EventTarget | null }) => {
      const next = applySidebarWidth(Number((event.target as HTMLInputElement).value))
      setValue('sidebarWidth', next)
    },
    [setValue],
  )
  const resetLayout = useCallback(() => {
    setValue('sidebarWidth', resetLayoutWidth())
  }, [setValue])

  const runtimeLabel =
    getEnvironmentName() === 'tauri' ? t('settings.summary.runtimeDesktop') : t('settings.summary.runtimeWeb')
  const localeLabel = locale === 'en-US' ? t('language.english') : t('language.chinese')
  const themeSummary =
    theme === 'system'
      ? translateWithFallback(t, 'settings.appearance.systemSummary', `${t('theme.system')} · {resolved}`, {
          resolved: t(`theme.${effectiveTheme}`),
        })
      : t(`theme.${theme}`)

  const sections = useMemo(
    () => SETTINGS_SECTIONS.map((section) => ({ ...section, title: t(section.titleKey), caption: t(section.captionKey) })),
    [],
  )

  return {
    register,
    theme,
    flavor,
    locale,
    uiFont,
    codeFont,
    uiCustom,
    codeCustom,
    uiSelect: fontSelectValue(uiFont, UI_FONT_PRESETS, uiCustom),
    codeSelect: fontSelectValue(codeFont, CODE_FONT_PRESETS, codeCustom),
    confirmBeforeExit,
    closeToTray,
    openPanelOnTrayClick,
    sidebarWidth,
    perfTelemetryEnabled,
    effectiveTheme,
    runtimeVersion,
    runtimeLabel,
    localeLabel,
    themeSummary,
    activeSection,
    setActiveSection,
    sections,
    setTheme,
    setFlavor,
    setLocaleValue,
    onUiSelect,
    onCodeSelect,
    setUiFont,
    setCodeFont,
    toggleConfirm,
    toggleTray,
    togglePanel,
    togglePerf,
    onSidebar,
    resetLayout,
    uiPreview: previewFamily(uiFont, '--font-sans-base'),
    codePreview: previewFamily(codeFont, '--font-mono-base'),
  }
}
