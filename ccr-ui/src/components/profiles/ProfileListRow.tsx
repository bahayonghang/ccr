import { SIcon } from '@/ui'
import type { ProfileRowDescriptor, ProfileRowProfile } from '@/utils/profileDescriptors'
import { truncateMiddle } from '@/utils/text'
import './profiles-shared.css'

export type { ProfileRowDescriptor, ProfileRowProfile }

export interface ProfileListRowProps<T extends ProfileRowProfile> {
  profile: T
  descriptor: ProfileRowDescriptor<T>
  isCurrent: boolean
  disabled?: boolean
  /** 进行中的操作（Codex 用，驱动图标转圈）；Claude 省略 → 恒 null */
  busyAction?: 'apply' | 'delete' | null
  onApply: (name: string) => void
  onEdit: (name: string) => void
  onDelete: (name: string) => void
}

/** 列表密度行：平台差异通过 descriptor 注入。 */
export function ProfileListRow<T extends ProfileRowProfile>({
  profile,
  descriptor,
  isCurrent,
  disabled = false,
  busyAction = null,
  onApply,
  onEdit,
  onDelete,
}: ProfileListRowProps<T>) {
  const isEnabled = profile.enabled !== false
  const tagList = (profile.tags ?? []).slice(0, 3)
  const baseUrlText = descriptor.baseUrl(profile)
  const modelText = descriptor.model(profile)
  const authModeText = descriptor.authMode(profile)
  const showApply = !isCurrent && isEnabled

  return (
    <div
      className={[
        'cp-row',
        'surface-status',
        isCurrent ? 'cp-row--active' : '',
        isEnabled ? '' : 'cp-row--off',
      ]
        .filter(Boolean)
        .join(' ')}
    >
      <span className={isCurrent ? 'cp-row__dot cp-row__dot--good' : 'cp-row__dot'} />
      <span className="cp-row__name" title={profile.name}>
        {profile.name}
      </span>
      <span className="cp-row__label">{profile.description || '—'}</span>
      <span className="cp-row__url" title={baseUrlText}>
        {truncateMiddle(baseUrlText, 20, 12)}
      </span>
      <span className="cp-row__model" title={modelText}>
        {modelText}
      </span>
      <span className="cp-row__meta">{authModeText}</span>
      <div className="cp-row__tags">
        {tagList.map((tag) => (
          <span key={tag} className="cp-tag">
            #{tag}
          </span>
        ))}
      </div>
      <div className="cp-row__actions">
        {showApply ? (
          <button
            type="button"
            className="cp-icon-btn cp-icon-btn--accent"
            title={descriptor.labels.apply}
            aria-label={descriptor.labels.apply}
            disabled={disabled}
            onClick={() => onApply(profile.name)}
          >
            <SIcon
              name={busyAction === 'apply' ? 'RefreshCw' : 'Play'}
              size="w-3 h-3"
              className={busyAction === 'apply' ? 'cp-spin' : undefined}
            />
          </button>
        ) : null}
        <button
          type="button"
          className="cp-icon-btn"
          title={descriptor.labels.edit}
          aria-label={descriptor.labels.edit}
          disabled={disabled}
          onClick={() => onEdit(profile.name)}
        >
          <SIcon name={descriptor.editIcon} size="w-3 h-3" />
        </button>
        <button
          type="button"
          className="cp-icon-btn cp-icon-btn--danger"
          title={descriptor.labels.delete}
          aria-label={descriptor.labels.delete}
          disabled={disabled}
          onClick={() => onDelete(profile.name)}
        >
          <SIcon
            name={busyAction === 'delete' ? 'RefreshCw' : 'Trash2'}
            size="w-3 h-3"
            className={busyAction === 'delete' ? 'cp-spin' : undefined}
          />
        </button>
      </div>
    </div>
  )
}
