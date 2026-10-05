import { useCallback } from 'react'
import type { UseFormRegister } from 'react-hook-form'
import { SIcon } from '@/ui'
import { CODE_FONT_PRESETS, UI_FONT_PRESETS } from '@/utils/fontPreferences'
import type { FlavorMode, ResolvedThemeMode, ThemeMode } from '@/utils/themeBootstrap'
import { t } from '../locale'
import { flavorPreviewStyle } from '../lib/flavorPreview'
import type { AppSettingsForm } from '../lib/settingsModel'
import { FlavorCard } from './FlavorCard'
import { ThemeOption } from './ThemeOption'

interface AppearanceSectionProps {
  theme: ThemeMode
  flavor: FlavorMode
  effectiveTheme: ResolvedThemeMode
  uiCustom: boolean
  codeCustom: boolean
  uiSelect: string
  codeSelect: string
  uiPreview: string
  codePreview: string
  register: UseFormRegister<AppSettingsForm>
  onTheme: (value: ThemeMode) => void
  onFlavor: (value: FlavorMode) => void
  onUiSelect: (value: string) => void
  onCodeSelect: (value: string) => void
  onUiFont: (value: string) => void
  onCodeFont: (value: string) => void
}

export function AppearanceSection({
  theme,
  flavor,
  effectiveTheme,
  uiCustom,
  codeCustom,
  uiSelect,
  codeSelect,
  uiPreview,
  codePreview,
  register,
  onTheme,
  onFlavor,
  onUiSelect,
  onCodeSelect,
  onUiFont,
  onCodeFont,
}: AppearanceSectionProps) {
  const handleUiSelect = useCallback(
    (event: { target: EventTarget | null }) => {
      onUiSelect((event.target as HTMLSelectElement).value)
    },
    [onUiSelect],
  )
  const handleCodeSelect = useCallback(
    (event: { target: EventTarget | null }) => {
      onCodeSelect((event.target as HTMLSelectElement).value)
    },
    [onCodeSelect],
  )
  const handleUiFont = useCallback(
    (event: { target: EventTarget | null }) => {
      onUiFont((event.target as HTMLInputElement).value)
    },
    [onUiFont],
  )
  const handleCodeFont = useCallback(
    (event: { target: EventTarget | null }) => {
      onCodeFont((event.target as HTMLInputElement).value)
    },
    [onCodeFont],
  )

  return (
    <section>
      <div className="app-settings-card">
        <div className="app-settings-card__header">
          <div>
            <p className="app-settings-card__eyebrow">{t('settings.appearance.theme.eyebrow')}</p>
            <h2 className="app-settings-card__title">{t('settings.appearance.theme.title')}</h2>
          </div>
          <p className="app-settings-card__description">{t('settings.appearance.theme.description')}</p>
        </div>
        <div className="app-settings-group" role="radiogroup" aria-label={t('settings.appearance.theme.title')}>
          <ThemeOption value="light" active={theme === 'light'} icon="Sun" title={t('theme.light')} caption={t('settings.appearance.lightDescription')} onSelect={onTheme} />
          <ThemeOption value="dark" active={theme === 'dark'} icon="Moon" title={t('theme.dark')} caption={t('settings.appearance.darkDescription')} onSelect={onTheme} />
          <ThemeOption value="system" active={theme === 'system'} icon="Monitor" title={t('theme.system')} caption={t('settings.appearance.systemDescription')} onSelect={onTheme} />
        </div>
        {theme === 'system' ? (
          <p className="app-settings-group__resolved">
            {t('settings.appearance.theme.resolvedHint', { resolved: t(`theme.${effectiveTheme}`) })}
          </p>
        ) : null}
        <div className="app-settings-card__split">
          <p className="app-settings-card__eyebrow">{t('settings.appearance.flavor.eyebrow')}</p>
          <h3 className="app-settings-card__subtitle">{t('settings.appearance.flavor.title')}</h3>
          <p className="app-settings-card__description">{t('settings.appearance.flavor.description')}</p>
        </div>
        <div className="app-settings-flavor-grid">
          <FlavorCard
            value="neutral"
            active={flavor === 'neutral'}
            title={t('settings.appearance.flavor.neutral')}
            caption={t('settings.appearance.flavor.neutralDescription')}
            previewStyle={flavorPreviewStyle('neutral', effectiveTheme)}
            onSelect={onFlavor}
          />
          <FlavorCard
            value="clay"
            active={flavor === 'clay'}
            title={t('settings.appearance.flavor.clay')}
            caption={t('settings.appearance.flavor.clayDescription')}
            previewStyle={flavorPreviewStyle('clay', effectiveTheme)}
            onSelect={onFlavor}
          />
        </div>
      </div>
      <div className="app-settings-card">
        <div className="app-settings-card__header">
          <div>
            <p className="app-settings-card__eyebrow">{t('settings.appearance.typography.eyebrow')}</p>
            <h2 className="app-settings-card__title">{t('settings.appearance.typography.title')}</h2>
          </div>
          <p className="app-settings-card__description">{t('settings.appearance.typography.description')}</p>
        </div>
        <div className="app-settings-stack">
          <div className="app-settings-row app-settings-row--font">
            <div className="app-settings-row__copy">
              <h3 className="app-settings-row__title">{t('settings.appearance.typography.uiLabel')}</h3>
              <p className="app-settings-row__description">{t('settings.appearance.typography.uiDescription')}</p>
            </div>
            <div className="app-settings-font-control">
              <select className="app-settings-font-select" aria-label={t('settings.appearance.typography.uiLabel')} data-testid="settings-font-ui" value={uiSelect} onChange={handleUiSelect}>
                <option value="__default__">{t('settings.appearance.typography.systemDefault')}</option>
                {UI_FONT_PRESETS.map((preset) => (
                  <option key={preset} value={preset}>
                    {preset}
                  </option>
                ))}
                <option value="__custom__">{t('settings.appearance.typography.custom')}</option>
              </select>
              {uiCustom ? (
                <input
                  type="text"
                  className="app-settings-font-input"
                  placeholder={t('settings.appearance.typography.customPlaceholder')}
                  data-testid="settings-font-ui-input"
                  {...register('uiFont')}
                  onInput={handleUiFont}
                />
              ) : null}
            </div>
          </div>
          <div className="app-settings-row app-settings-row--font">
            <div className="app-settings-row__copy">
              <h3 className="app-settings-row__title">{t('settings.appearance.typography.codeLabel')}</h3>
              <p className="app-settings-row__description">{t('settings.appearance.typography.codeDescription')}</p>
            </div>
            <div className="app-settings-font-control">
              <select className="app-settings-font-select" aria-label={t('settings.appearance.typography.codeLabel')} data-testid="settings-font-code" value={codeSelect} onChange={handleCodeSelect}>
                <option value="__default__">{t('settings.appearance.typography.systemDefault')}</option>
                {CODE_FONT_PRESETS.map((preset) => (
                  <option key={preset} value={preset}>
                    {preset}
                  </option>
                ))}
                <option value="__custom__">{t('settings.appearance.typography.custom')}</option>
              </select>
              {codeCustom ? (
                <input
                  type="text"
                  className="app-settings-font-input"
                  placeholder={t('settings.appearance.typography.customPlaceholder')}
                  data-testid="settings-font-code-input"
                  {...register('codeFont')}
                  onInput={handleCodeFont}
                />
              ) : null}
            </div>
          </div>
          <div className="app-settings-type-preview" aria-hidden="true">
            <p className="app-settings-type-preview__metrics app-settings-font-preview--mono" style={{ fontFamily: codePreview }}>
              {t('settings.appearance.typography.previewSampleCode')}
            </p>
            <p className="app-settings-type-preview__copy" style={{ fontFamily: uiPreview }}>
              {t('settings.appearance.typography.previewSampleUi')}
            </p>
          </div>
          <div className="app-settings-callout">
            <SIcon name="Info" size="w-4 h-4" className="mt-0.5 text-accent-primary" />
            <p>{t('settings.appearance.typography.resetHint')}</p>
          </div>
        </div>
      </div>
    </section>
  )
}
