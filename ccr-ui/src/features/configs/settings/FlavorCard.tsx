import { memo, useCallback } from 'react'
import type { FlavorMode } from '@/utils/themeBootstrap'
import { t } from '../locale'
import { PREVIEW_GLYPH_SAMPLE } from '../lib/flavorPreview'

interface FlavorCardProps {
  value: FlavorMode
  active: boolean
  title: string
  caption: string
  previewStyle: Record<string, string>
  onSelect: (value: FlavorMode) => void
}

export const FlavorCard = memo(function FlavorCard({
  value,
  active,
  title,
  caption,
  previewStyle,
  onSelect,
}: FlavorCardProps) {
  const handleClick = useCallback(() => {
    onSelect(value)
  }, [onSelect, value])
  return (
    <button
      type="button"
      className={`app-settings-flavor-card ${active ? 'app-settings-flavor-card--active' : ''}`}
      data-testid={`settings-flavor-${value}`}
      aria-pressed={active}
      onClick={handleClick}
    >
      <span className="app-settings-flavor-card__preview" data-preview-flavor={value} style={previewStyle}>
        <span className="fp-surface">
          <span className="fp-text">{PREVIEW_GLYPH_SAMPLE}</span>
          <span className="fp-muted">{PREVIEW_GLYPH_SAMPLE}</span>
          <i className="fp-accent" />
        </span>
      </span>
      <span className="app-settings-flavor-card__copy">
        <span className="app-settings-option__title">{title}</span>
        <span className="app-settings-option__caption">{caption}</span>
      </span>
      {active ? (
        <span className="app-settings-flavor-card__footer">
          <span className="app-settings-flavor-card__dot" aria-hidden="true" />
          <span className="app-settings-option__status app-settings-option__status--active">{t('settings.active')}</span>
        </span>
      ) : null}
    </button>
  )
})
