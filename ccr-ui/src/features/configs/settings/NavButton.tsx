import { memo, useCallback } from 'react'
import { SIcon } from '@/ui'
import type { SettingsSectionKey } from '../lib/settingsModel'

interface NavButtonProps {
  sectionKey: SettingsSectionKey
  title: string
  caption: string
  icon: string
  active: boolean
  onSelect: (key: SettingsSectionKey) => void
}

export const NavButton = memo(function NavButton({
  sectionKey,
  title,
  caption,
  icon,
  active,
  onSelect,
}: NavButtonProps) {
  const handleClick = useCallback(() => {
    onSelect(sectionKey)
  }, [onSelect, sectionKey])
  return (
    <button
      type="button"
      className={`app-settings-nav__button ${active ? 'app-settings-nav__button--active' : ''}`}
      data-testid={`settings-section-${sectionKey}`}
      onClick={handleClick}
    >
      <span className="app-settings-nav__icon">
        <SIcon name={icon} size="w-4 h-4" />
      </span>
      <span>
        <span className="app-settings-nav__title">{title}</span>
        <span className="app-settings-nav__caption">{caption}</span>
      </span>
    </button>
  )
})
