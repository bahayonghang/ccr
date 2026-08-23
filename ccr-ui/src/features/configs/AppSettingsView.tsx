import { useCallback, useRef } from 'react'
import { SIcon } from '@/ui'
import { t } from './locale'
import { useAppSettings } from './hooks/useAppSettings'
import type { SettingsSectionKey } from './lib/settingsModel'
import { AppearanceSection } from './settings/AppearanceSection'
import { DiagnosticsSection } from './settings/DiagnosticsSection'
import { LanguageSection } from './settings/LanguageSection'
import { NavButton } from './settings/NavButton'
import { ShellSection } from './settings/ShellSection'
import './styles/app-settings.css'

export function AppSettingsView() {
  const settings = useAppSettings()
  const sectionRefs = useRef<Record<SettingsSectionKey, HTMLElement | null>>({
    appearance: null,
    language: null,
    shell: null,
    diagnostics: null,
  })

  const setActiveSection = settings.setActiveSection
  const scrollTo = useCallback((key: SettingsSectionKey) => {
    setActiveSection(key)
    sectionRefs.current[key]?.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }, [setActiveSection])

  const setAppearanceRef = useCallback((node: HTMLElement | null) => {
    sectionRefs.current.appearance = node
  }, [])
  const setLanguageRef = useCallback((node: HTMLElement | null) => {
    sectionRefs.current.language = node
  }, [])
  const setShellRef = useCallback((node: HTMLElement | null) => {
    sectionRefs.current.shell = node
  }, [])
  const setDiagnosticsRef = useCallback((node: HTMLElement | null) => {
    sectionRefs.current.diagnostics = node
  }, [])

  return (
    <div className="app-settings-view">
      <div className="app-settings-shell">
        <header className="app-settings-hero">
          <div className="app-settings-hero__intro">
            <div className="app-settings-hero__icon">
              <SIcon name="SlidersHorizontal" size="w-6 h-6" />
            </div>
            <div className="space-y-2">
              <p className="app-settings-hero__eyebrow">{t('settings.eyebrow')}</p>
              <div>
                <h1 className="app-settings-hero__title">{t('settings.title')}</h1>
                <p className="app-settings-hero__description">{t('settings.description')}</p>
              </div>
            </div>
          </div>
          <div className="app-settings-summary">
            <span className="app-settings-summary__pill">{settings.runtimeLabel}</span>
            {settings.runtimeVersion ? (
              <span className="app-settings-summary__pill app-settings-summary__pill--mono">v{settings.runtimeVersion}</span>
            ) : null}
            <span className="app-settings-summary__pill">{settings.themeSummary}</span>
            <span className="app-settings-summary__pill">{settings.localeLabel}</span>
            <span className="app-settings-summary__pill app-settings-summary__pill--mono">{settings.sidebarWidth}px</span>
          </div>
        </header>
        <div className="app-settings-layout">
          <aside className="app-settings-nav">
            <div className="app-settings-nav__inner">
              {settings.sections.map((section) => (
                <NavButton
                  key={section.key}
                  sectionKey={section.key}
                  title={section.title}
                  caption={section.caption}
                  icon={section.icon}
                  active={settings.activeSection === section.key}
                  onSelect={scrollTo}
                />
              ))}
            </div>
          </aside>
          <div className="app-settings-content">
            <div ref={setAppearanceRef}>
              <AppearanceSection
                theme={settings.theme}
                flavor={settings.flavor}
                effectiveTheme={settings.effectiveTheme}
                uiCustom={settings.uiCustom}
                codeCustom={settings.codeCustom}
                uiSelect={settings.uiSelect}
                codeSelect={settings.codeSelect}
                uiPreview={settings.uiPreview}
                codePreview={settings.codePreview}
                register={settings.register}
                onTheme={settings.setTheme}
                onFlavor={settings.setFlavor}
                onUiSelect={settings.onUiSelect}
                onCodeSelect={settings.onCodeSelect}
                onUiFont={settings.setUiFont}
                onCodeFont={settings.setCodeFont}
              />
            </div>
            <div ref={setLanguageRef}>
              <LanguageSection locale={settings.locale} onSelect={settings.setLocaleValue} />
            </div>
            <div ref={setShellRef}>
              <ShellSection
                confirmBeforeExit={settings.confirmBeforeExit}
                closeToTray={settings.closeToTray}
                openPanelOnTrayClick={settings.openPanelOnTrayClick}
                sidebarWidth={settings.sidebarWidth}
                register={settings.register}
                onToggleConfirm={settings.toggleConfirm}
                onToggleTray={settings.toggleTray}
                onTogglePanel={settings.togglePanel}
                onSidebar={settings.onSidebar}
                onResetLayout={settings.resetLayout}
              />
            </div>
            <div ref={setDiagnosticsRef}>
              <DiagnosticsSection enabled={settings.perfTelemetryEnabled} onToggle={settings.togglePerf} />
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
