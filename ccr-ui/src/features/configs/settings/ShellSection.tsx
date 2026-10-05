import type { UseFormRegister } from 'react-hook-form'
import { SIcon } from '@/ui'
import { t } from '../locale'
import type { AppSettingsForm } from '../lib/settingsModel'
import { SettingsSwitchRow } from './SettingsSwitchRow'

interface ShellSectionProps {
  confirmBeforeExit: boolean
  closeToTray: boolean
  openPanelOnTrayClick: boolean
  sidebarWidth: number
  register: UseFormRegister<AppSettingsForm>
  onToggleConfirm: () => void
  onToggleTray: () => void
  onTogglePanel: () => void
  onSidebar: (event: { target: EventTarget | null }) => void
  onResetLayout: () => void
}

export function ShellSection({
  confirmBeforeExit,
  closeToTray,
  openPanelOnTrayClick,
  sidebarWidth,
  register,
  onToggleConfirm,
  onToggleTray,
  onTogglePanel,
  onSidebar,
  onResetLayout,
}: ShellSectionProps) {
  return (
    <section>
      <div className="app-settings-card">
        <div className="app-settings-card__header">
          <div>
            <p className="app-settings-card__eyebrow">{t('settings.shell.eyebrow')}</p>
            <h2 className="app-settings-card__title">{t('settings.shell.title')}</h2>
          </div>
          <p className="app-settings-card__description">{t('settings.shell.description')}</p>
        </div>
        <div className="app-settings-stack">
          <SettingsSwitchRow
            title={t('settings.shell.exitConfirmTitle')}
            description={t('settings.shell.exitConfirmDescription')}
            checked={confirmBeforeExit}
            testId="settings-confirm-exit-toggle"
            onToggle={onToggleConfirm}
          />
          <SettingsSwitchRow
            title={t('settings.shell.closeToTrayTitle')}
            description={t('settings.shell.closeToTrayDescription')}
            checked={closeToTray}
            testId="settings-close-to-tray-toggle"
            onToggle={onToggleTray}
          />
          <SettingsSwitchRow
            title={t('settings.shell.openPanelOnTrayClickTitle')}
            description={t('settings.shell.openPanelOnTrayClickDescription')}
            checked={openPanelOnTrayClick}
            testId="settings-open-panel-on-tray-click-toggle"
            onToggle={onTogglePanel}
          />
          <div className="app-settings-row app-settings-row--slider">
            <div className="app-settings-row__copy">
              <h3 className="app-settings-row__title">{t('settings.shell.sidebarWidthTitle')}</h3>
              <p className="app-settings-row__description">{t('settings.shell.sidebarWidthDescription')}</p>
            </div>
            <div className="app-settings-slider">
              <input
                type="range"
                min={200}
                max={480}
                step={8}
                className="app-settings-slider__control"
                data-testid="settings-sidebar-width-slider"
                defaultValue={sidebarWidth}
                {...register('sidebarWidth', { valueAsNumber: true })}
                onInput={onSidebar}
              />
              <div className="app-settings-slider__meta">
                <span>200</span>
                <strong>{sidebarWidth}px</strong>
                <span>480</span>
              </div>
            </div>
          </div>
          <div className="app-settings-row">
            <div className="app-settings-row__copy">
              <h3 className="app-settings-row__title">{t('settings.shell.resetLayoutTitle')}</h3>
              <p className="app-settings-row__description">{t('settings.shell.resetLayoutDescription')}</p>
            </div>
            <button
              type="button"
              className="inline-flex items-center gap-2 rounded-lg border border-border-default px-3 py-2 text-sm"
              data-testid="settings-reset-layout"
              onClick={onResetLayout}
            >
              <SIcon name="RotateCw" size="w-4 h-4" />
              {t('settings.shell.resetLayoutAction')}
            </button>
          </div>
        </div>
      </div>
    </section>
  )
}
