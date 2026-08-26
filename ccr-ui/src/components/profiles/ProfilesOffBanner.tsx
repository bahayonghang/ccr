import { surfaceNotify } from '@/configs/surfaceNotify'
import { useShellT } from '@/shell/i18n'
import './profiles-shared.css'

export interface ProfilesOffBannerProps {
  canOff: boolean
  currentName: string | null
  onOff: () => Promise<void>
}

/** Header 与 StatStrip 之间的 Off 横幅；canOff 为假时不渲染。 */
export function ProfilesOffBanner({ canOff, currentName, onOff }: ProfilesOffBannerProps) {
  const t = useShellT()
  if (!canOff) return null

  const onClick = () => {
    void surfaceNotify
      .confirm({
        title: t('profilesSurface.offTitle'),
        message: t('profilesSurface.offMessage', { name: currentName ?? '' }),
        type: 'warning',
      })
      .then((ok) => {
        if (ok) return onOff()
        return undefined
      })
  }

  return (
    <div className="cp-off-banner" data-testid="profiles-off-banner">
      <span>{t('profilesSurface.offMessage', { name: currentName ?? '' })}</span>
      <button type="button" className="cp-btn cp-btn--ghost" onClick={onClick}>
        {t('profilesSurface.offAction')}
      </button>
    </div>
  )
}
