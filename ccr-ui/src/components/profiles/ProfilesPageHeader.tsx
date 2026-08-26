import { Button, SIcon } from '@/ui'
import type { ProfilePresentationView } from '@/configs/profilePresentation'
import { useShellT } from '@/shell/i18n'
import './profiles-shared.css'

export interface ProfilesPageHeaderProps {
  presentation: ProfilePresentationView
  environmentLabel: string
  environmentOk: boolean
  loading: boolean
  onAdd: () => void
  onReload: () => void
  onExport?: () => void
  onEditSource?: () => void
}

const glyphStyleOf = (platformKey: string) => ({
  backgroundColor: `var(--color-platform-${platformKey}-surface)`,
  borderColor: `var(--color-platform-${platformKey}-border)`,
  color: `var(--color-platform-${platformKey}-text)`,
})

/** 面包屑 + 页头：Off 不进入溢出菜单。 */
export function ProfilesPageHeader({
  presentation,
  environmentLabel,
  environmentOk,
  loading,
  onAdd,
  onReload,
  onExport,
  onEditSource,
}: ProfilesPageHeaderProps) {
  const t = useShellT()
  return (
    <header className="cp-page-header" data-testid="profiles-page-header">
      <div className="cp-breadcrumb">
        <span className="cp-breadcrumb__platform">{t(presentation.nameKey)}</span>
        <span className="cp-breadcrumb__sep" aria-hidden="true">
          /
        </span>
        <span>{t('profilesSurface.breadcrumbProfiles')}</span>
        <span
          className={
            environmentOk ? 'cp-env-badge cp-env-badge--ok' : 'cp-env-badge'
          }
        >
          <span className="cp-env-badge__dot" />
          {environmentLabel}
        </span>
        <span className="cp-file-badge">{presentation.configFile}</span>
      </div>

      <div className="cp-page-header__row">
        <div className="cp-page-header__identity">
          <span className="cp-glyph" style={glyphStyleOf(presentation.key)}>
            {presentation.glyph}
          </span>
          <div>
            <h1 className="cp-page-header__title">{t(presentation.nameKey)}</h1>
            <p className="cp-page-header__path">{t(presentation.configPathKey)}</p>
          </div>
        </div>
        <div className="cp-page-header__actions">
          <Button variant="ghost" size="md" disabled={loading} onClick={onReload}>
            {t('profilesSurface.reload')}
          </Button>
          {onExport ? (
            <Button variant="ghost" size="md" disabled={loading} onClick={onExport}>
              {t('profilesSurface.export')}
            </Button>
          ) : null}
          {onEditSource ? (
            <Button
              variant="ghost"
              size="md"
              data-testid="profiles-edit-source"
              disabled={loading}
              onClick={onEditSource}
            >
              {t('profilesSurface.editSource')}
            </Button>
          ) : null}
          <Button variant="primary" size="md" disabled={loading} onClick={onAdd}>
            <SIcon name="Plus" size="w-3.5 h-3.5" />
            {t('profilesSurface.newProfile')}
          </Button>
        </div>
      </div>
    </header>
  )
}
