import { SIcon } from '@/ui'
import { t } from '../locale'
import { SettingsSwitchRow } from './SettingsSwitchRow'

interface DiagnosticsSectionProps {
  enabled: boolean
  onToggle: () => void
}

export function DiagnosticsSection({ enabled, onToggle }: DiagnosticsSectionProps) {
  return (
    <section>
      <div className="app-settings-card">
        <div className="app-settings-card__header">
          <div>
            <p className="app-settings-card__eyebrow">{t('settings.diagnostics.eyebrow')}</p>
            <h2 className="app-settings-card__title">{t('settings.diagnostics.title')}</h2>
          </div>
          <p className="app-settings-card__description">{t('settings.diagnostics.description')}</p>
        </div>
        <div className="app-settings-stack">
          <SettingsSwitchRow
            title={t('settings.diagnostics.perfTitle')}
            description={t('settings.diagnostics.perfDescription')}
            checked={enabled}
            testId="settings-perf-toggle"
            onToggle={onToggle}
          />
          <div className="app-settings-callout">
            <SIcon name="Info" size="w-4 h-4" className="mt-0.5 text-accent-primary" />
            <p>{t('settings.diagnostics.restartNote')}</p>
          </div>
        </div>
      </div>
    </section>
  )
}
