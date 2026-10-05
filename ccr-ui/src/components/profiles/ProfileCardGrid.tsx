import type { MouseEvent } from 'react'
import type { ProfileDisplayRecord } from '@/configs/profileDisplayRecord'
import type { ProfilePresentationView } from '@/configs/profilePresentation'
import { resolveRowState } from '@/utils/resolveProfileRowState'
import { useShellT } from '@/shell/i18n'
import { Badge, Button, FieldLabel } from '@/ui'
import { ProfileFieldValue } from './ProfileFieldValue'
import { ProfileOverflowMenu } from './ProfileOverflowMenu'
import './profiles-shared.css'

export interface ProfileCardGridProps {
  records: readonly ProfileDisplayRecord[]
  presentation: ProfilePresentationView
  inspectorOpen: boolean
  onSelect: (name: string) => void
  onEdit: (name: string) => void
  onApply: (name: string) => void
  onToggle?: (name: string, enabled: boolean) => void
  onDelete?: (name: string) => void
}

interface ProfileCardProps {
  record: ProfileDisplayRecord
  presentation: ProfilePresentationView
  onSelect: (name: string) => void
  onEdit: (name: string) => void
  onApply: (name: string) => void
  onToggle?: (name: string, enabled: boolean) => void
  onDelete?: (name: string) => void
}

function stopAnd(run: () => void) {
  return (event: MouseEvent) => {
    event.stopPropagation()
    run()
  }
}

function ProfileCard({
  record,
  presentation,
  onSelect,
  onEdit,
  onApply,
  onToggle,
  onDelete,
}: ProfileCardProps) {
  const t = useShellT()
  const state = resolveRowState(record, presentation)
  const placeholder = t('profilesSurface.placeholder')
  const cardClass = state.emphasized ? 'cp-card cp-card--running' : 'cp-card'
  const statusTone = state.badge.tone === 'accent' ? 'accent' : 'neutral'
  const applyVariant = state.applyTone === 'accent-soft' ? 'accent-soft' : 'ghost'
  const onCardClick = () => onSelect(record.name)
  const onEditClick = stopAnd(() => onEdit(record.name))
  const onApplyClick = stopAnd(() => onApply(record.name))
  const onToggleRecord = onToggle
    ? (enabled: boolean) => onToggle(record.name, enabled)
    : undefined
  const onDeleteRecord = onDelete ? () => onDelete(record.name) : undefined

  return (
    <article className={cardClass} data-name={record.name} onClick={onCardClick}>
      <div className="cp-card__top">
        <span
          className={
            state.dotTone === 'active' ? 'cp-card__dot cp-card__dot--active' : 'cp-card__dot'
          }
        />
        <span className="cp-card__name">{record.name}</span>
        <span className="cp-card__desc">{record.description || placeholder}</span>
        <Badge mode="static" tone={statusTone} data-testid="profile-row-status-badge">
          {t(state.badge.textKey)}
        </Badge>
      </div>
      <div className="cp-card__badges">
        {record.badges.map((badge) => (
          <Badge key={badge.labelKey} mode="static" tone={badge.tone} data-testid="profile-record-badge">
            {t(badge.labelKey)}
          </Badge>
        ))}
      </div>
      <dl className="cp-card__fields">
        {presentation.fieldSlots.map((slot, index) => (
          <div key={slot.labelKey} className="cp-card__field">
            <FieldLabel as="dt">{t(slot.labelKey)}</FieldLabel>
            <dd>
              <ProfileFieldValue
                kind={slot.kind}
                value={record.slots[index]}
                placeholder={placeholder}
              />
            </dd>
          </div>
        ))}
      </dl>
      <div className="cp-card__foot">
        <div className="cp-card__tags">
          {record.tags.map((tag) => (
            <Badge key={tag} mode="static" tone="neutral">
              #{tag}
            </Badge>
          ))}
        </div>
        <ProfileOverflowMenu
          enabled={record.enabled}
          onEdit={() => onEdit(record.name)}
          onToggle={onToggleRecord}
          onDelete={onDeleteRecord}
        />
        <Button variant="quiet" size="sm" onClick={onEditClick}>
          {t('profilesSurface.edit')}
        </Button>
        <Button variant={applyVariant} size="sm" onClick={onApplyClick}>
          {t(state.applyLabelKey)}
        </Button>
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
  onToggle,
  onDelete,
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
          onToggle={onToggle}
          onDelete={onDelete}
        />
      ))}
    </div>
  )
}
