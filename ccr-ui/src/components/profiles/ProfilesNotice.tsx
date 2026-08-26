import './profiles-shared.css'

export interface ProfilesNoticeProps {
  tone: 'warning' | 'danger'
  message: string
  actionLabel?: string
  onAction?: () => void
}

/** 列表页提示条：recovery 等控制器 notice。不含平台名。 */
export function ProfilesNotice({ tone, message, actionLabel, onAction }: ProfilesNoticeProps) {
  const toneClass = tone === 'danger' ? 'cp-notice cp-notice--danger' : 'cp-notice'
  return (
    <div className={toneClass} data-testid="profiles-notice" data-tone={tone}>
      <p className="cp-notice__message">{message}</p>
      {actionLabel && onAction ? (
        <button type="button" className="cp-btn cp-btn--ghost" onClick={onAction}>
          {actionLabel}
        </button>
      ) : null}
    </div>
  )
}
