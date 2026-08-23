import { memo, useCallback, type ReactNode } from 'react'
import { SIcon } from '@/ui'

interface ChoiceButtonProps {
  value: string
  active: boolean
  testId: string
  icon?: string
  title: string
  caption: string
  status?: string
  leading?: ReactNode
  onSelect: (value: string) => void
}

export const ChoiceButton = memo(function ChoiceButton({
  value,
  active,
  testId,
  icon,
  title,
  caption,
  status,
  leading,
  onSelect,
}: ChoiceButtonProps) {
  const handleClick = useCallback(() => {
    onSelect(value)
  }, [onSelect, value])
  return (
    <button
      type="button"
      className={`app-settings-option ${active ? 'app-settings-option--active' : ''}`}
      data-testid={testId}
      aria-pressed={active}
      onClick={handleClick}
    >
      <div className="app-settings-option__meta">
        <span className={`app-settings-option__icon ${leading ? 'app-settings-option__icon--plain' : ''}`}>
          {leading ?? (icon ? <SIcon name={icon} size="w-4 h-4" /> : null)}
        </span>
        <span className="app-settings-option__copy">
          <span className="app-settings-option__title">{title}</span>
          <span className="app-settings-option__caption">{caption}</span>
        </span>
      </div>
      {status ? <span className="app-settings-option__status">{status}</span> : null}
    </button>
  )
})
