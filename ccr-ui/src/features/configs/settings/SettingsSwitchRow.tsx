import { memo, useCallback } from 'react'
import { t } from '../locale'

interface SettingsSwitchRowProps {
  title: string
  description: string
  checked: boolean
  testId: string
  onToggle: () => void
}

export const SettingsSwitchRow = memo(function SettingsSwitchRow({
  title,
  description,
  checked,
  testId,
  onToggle,
}: SettingsSwitchRowProps) {
  const handleClick = useCallback(() => {
    onToggle()
  }, [onToggle])
  return (
    <div className="app-settings-row">
      <div className="app-settings-row__copy">
        <h3 className="app-settings-row__title">{title}</h3>
        <p className="app-settings-row__description">{description}</p>
      </div>
      <button
        type="button"
        role="switch"
        className={`app-settings-switch ${checked ? 'app-settings-switch--active' : ''}`}
        aria-checked={checked}
        data-testid={testId}
        onClick={handleClick}
      >
        <span className="app-settings-switch__track" />
        <span className="app-settings-switch__thumb" />
        <span className="app-settings-switch__label">{checked ? t('settings.enabled') : t('settings.disabled')}</span>
      </button>
    </div>
  )
})
