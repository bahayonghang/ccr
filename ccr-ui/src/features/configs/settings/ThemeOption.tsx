import { memo, useCallback } from 'react'
import { SIcon } from '@/ui'
import type { ThemeMode } from '@/utils/themeBootstrap'

interface ThemeOptionProps {
  value: ThemeMode
  active: boolean
  icon: string
  title: string
  caption: string
  onSelect: (value: ThemeMode) => void
}

export const ThemeOption = memo(function ThemeOption({
  value,
  active,
  icon,
  title,
  caption,
  onSelect,
}: ThemeOptionProps) {
  const handleClick = useCallback(() => {
    onSelect(value)
  }, [onSelect, value])
  return (
    <button
      type="button"
      role="radio"
      className={`app-settings-group__item ${active ? 'app-settings-group__item--active' : ''}`}
      data-testid={`settings-theme-${value}`}
      aria-checked={active}
      aria-pressed={active}
      onClick={handleClick}
    >
      <span className="app-settings-group__icon">
        <SIcon name={icon} size="w-4 h-4" />
      </span>
      <span className="app-settings-group__copy">
        <span className="app-settings-group__title">{title}</span>
        <span className="app-settings-group__caption">{caption}</span>
      </span>
    </button>
  )
})
