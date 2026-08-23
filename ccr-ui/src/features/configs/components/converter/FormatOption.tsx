import { memo, useCallback } from 'react'
import type { CliType } from '@/types'
import { t } from '../../locale'

interface FormatOptionProps {
  value: CliType
  label: string
  description: string
  active: boolean
  disabled?: boolean
  onSelect: (value: CliType) => void
}

export const FormatOption = memo(function FormatOption({
  value,
  label,
  description,
  active,
  disabled = false,
  onSelect,
}: FormatOptionProps) {
  const handleClick = useCallback(() => {
    onSelect(value)
  }, [onSelect, value])
  const className = [
    'converter-option-card',
    active ? 'converter-option-card--active' : '',
    disabled ? 'converter-option-card--disabled' : '',
  ]
    .filter(Boolean)
    .join(' ')
  return (
    <button type="button" className={className} disabled={disabled} onClick={handleClick}>
      <div className="converter-option-card__header">
        <span className="converter-option-card__title">{label}</span>
        {active ? <span className="converter-option-badge">{t('converter.selected')}</span> : null}
      </div>
      <p className="converter-option-card__description">{description}</p>
    </button>
  )
})
