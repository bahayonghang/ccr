import type { ProfileDisplayRecord } from '@/configs/profileDisplayRecord'
import type { ProfilePresentationView } from '@/configs/profilePresentation'
import { resolveRowState } from '@/utils/resolveProfileRowState'
import { useShellT } from '@/shell/i18n'
import './profiles-shared.css'

export interface ProfileCardGridProps {
  records: readonly ProfileDisplayRecord[]
  presentation: ProfilePresentationView
  inspectorOpen: boolean
  onSelect: (name: string) => void
  onEdit: (name: string) => void
  onApply: (name: string) => void
}

interface ProfileCardProps {
  record: ProfileDisplayRecord
  presentation: ProfilePresentationView
  onSelect: (name: string) => void
  onEdit: (name: string) => void
  onApply: (name: string) => void
}

function ProfileCard({ record, presentation, onSelect, onEdit, onApply }: ProfileCardProps) {
  const t = useShellT()
  const state = resolveRowState(record, presentation)
  const cardClass = state.emphasized ? 'cp-card cp-card--running' : 'cp-card'
  const badgeClass =
    state.badge.tone === 'accent' ? 'cp-card__badge cp-card__badge--accent' : 'cp-card__badge'
  const applyClass =
    state.applyTone === 'accent-soft' ? 'cp-btn cp-btn--accent-soft' : 'cp-btn cp-btn--ghost'
  const onCardClick = () => onSelect(record.name)
  const onEditClick = () => onEdit(record.name)
  const onApplyClick = () => onApply(record.name)

  return (
    <article className={cardClass} data-name={record.name} onClick={onCardClick}>
      <div className="cp-card__top">
        <span
          className={
            state.dotTone === 'active' ? 'cp-card__dot cp-card__dot--active' : 'cp-card__dot'
          }
        />
        <span className="cp-card__name">{record.name}</span>
        <span className="cp-card__desc">{record.description || t('profilesSurface.placeholder')}</span>
        <span className={badgeClass}>{t(state.badge.textKey)}</span>
      </div>
      <div className="cp-card__badges">
        {record.badges.map((badge) => (
          <span key={badge.labelKey} className="cp-chip">
            {t(badge.labelKey)}
          </span>
        ))}
      </div>
      <dl className="cp-card__fields">
        {presentation.fieldSlots.map((slot, index) => (
          <div key={slot.labelKey} className="cp-card__field">
            <dt>{t(slot.labelKey)}</dt>
            <dd className={slot.chip ? 'cp-chip' : undefined}>
              {record.slots[index] || t('profilesSurface.placeholder')}
            </dd>
          </div>
        ))}
      </dl>
      <div className="cp-card__foot">
        <div className="cp-card__tags">
          {record.tags.map((tag) => (
            <span key={tag} className="cp-chip">
              #{tag}
            </span>
          ))}
        </div>
        <button type="button" className="cp-btn cp-btn--ghost" onClick={onEditClick}>
          {t('profilesSurface.edit')}
        </button>
        <button type="button" className={applyClass} onClick={onApplyClick}>
          {t(state.applyLabelKey)}
        </button>
      </div>
    </article>
  )
}

/** 卡片网格：Inspector 展开时两列，否则三列。 */
export function ProfileCardGrid({
  records,
  presentation,
  inspectorOpen,
  onSelect,
  onEdit,
  onApply,
}: ProfileCardGridProps) {
  const gridClass = inspectorOpen ? 'cp-card-grid cp-card-grid--inspector' : 'cp-card-grid'
  return (
    <div className={gridClass} data-testid="profiles-card-grid">
      {records.map((record) => (
        <ProfileCard
          key={record.name}
          record={record}
          presentation={presentation}
          onSelect={onSelect}
          onEdit={onEdit}
          onApply={onApply}
        />
      ))}
    </div>
  )
}
