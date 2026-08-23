import { useShellT } from '@/shell/i18n'
import { SIcon } from '@/ui'
import type {
  ProfilesInspectorDescriptor,
  ProfilesInspectorField,
  ProfilesInspectorProfile,
} from '@/utils/profileDescriptors'
import type { ProfileDiffRow } from '@/utils/profileDiff'
import { ProfileDiffRows } from './ProfileDiffRows'

interface ProfilesInspectorPreviewProps<T extends ProfilesInspectorProfile> {
  i18nPrefix: string
  headingId: string
  previewProfile: T | null
  isPreviewingCurrent: boolean
  previewFields: ProfilesInspectorField[]
  previewTags: string[]
  diffRows: ProfileDiffRow[]
  sessionWriteAt: string | null
  editIcon: ProfilesInspectorDescriptor<T>['editIcon']
  onEdit: (name: string) => void
}

function fieldValueClass(variant: ProfilesInspectorField['variant']): string {
  if (variant === 'accent') return 'cp-inspector-field__value cp-inspector-field__value--accent'
  if (variant === 'muted') return 'cp-inspector-field__value cp-inspector-field__value--muted'
  return 'cp-inspector-field__value'
}

export function ProfilesInspectorPreview<T extends ProfilesInspectorProfile>({
  i18nPrefix,
  headingId,
  previewProfile,
  isPreviewingCurrent,
  previewFields,
  previewTags,
  diffRows,
  sessionWriteAt,
  editIcon,
  onEdit,
}: ProfilesInspectorPreviewProps<T>) {
  const t = useShellT()

  return (
    <section className="cp-inspector-card surface-card" aria-labelledby={headingId}>
      <header className="cp-inspector-card__head">
        <SIcon name="Sparkles" size="w-3.5 h-3.5" className="cp-inspector-card__icon" />
        <h3 id={headingId} className="cp-inspector-card__title">
          {t(`${i18nPrefix}.previewTitle`)}
        </h3>
        {isPreviewingCurrent && previewProfile ? (
          <span className="cp-inspector-badge">{t(`${i18nPrefix}.currentBadge`)}</span>
        ) : null}
      </header>

      <span className="sr-only" aria-live="polite">
        {previewProfile?.name ?? ''}
      </span>

      {previewProfile ? (
        <div className="cp-inspector-preview">
          <div className="cp-inspector-preview__name">{previewProfile.name}</div>
          {previewProfile.description ? (
            <p className="cp-inspector-preview__desc">{previewProfile.description}</p>
          ) : null}

          <dl className="cp-inspector-fields">
            {previewFields.map((field) => (
              <div key={field.label} className="cp-inspector-field">
                <dt className="cp-inspector-field__label">{field.label}</dt>
                <dd className={fieldValueClass(field.variant)}>{field.value}</dd>
              </div>
            ))}
          </dl>

          {diffRows.length > 0 ? (
            <div className="cp-inspector-diff">
              <div className="cp-inspector-section__head">{t(`${i18nPrefix}.diffTitle`)}</div>
              <ProfileDiffRows rows={diffRows} />
            </div>
          ) : null}

          {previewTags.length > 0 ? (
            <div className="cp-inspector-tags">
              {previewTags.map((tag) => (
                <span key={tag} className="cp-inspector-tag">
                  #{tag}
                </span>
              ))}
            </div>
          ) : null}

          {sessionWriteAt ? (
            <div className="cp-inspector-session">
              {t(`${i18nPrefix}.sessionWrite`, { time: sessionWriteAt })}
            </div>
          ) : null}

          <button type="button" className="cp-inspector-action" onClick={() => onEdit(previewProfile.name)}>
            <SIcon name={editIcon} size="w-3.5 h-3.5" />
            <span>{t(`${i18nPrefix}.editAction`)}</span>
          </button>
        </div>
      ) : (
        <div className="cp-inspector-empty">
          <SIcon name="Folder" size="w-4 h-4" />
          <div className="cp-inspector-empty__title">{t(`${i18nPrefix}.previewEmpty`)}</div>
          <div className="cp-inspector-empty__hint">{t(`${i18nPrefix}.previewEmptyHint`)}</div>
        </div>
      )}
    </section>
  )
}
