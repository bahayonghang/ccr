import type { ProfileDisplayRecord } from '@/configs/profileDisplayRecord'
import type { ProfilePresentationView } from '@/configs/profilePresentation'
import { resolveRowState } from '@/utils/resolveProfileRowState'
import { useShellT } from '@/shell/i18n'
import './profiles-shared.css'

export interface ProfileTableProps {
  records: readonly ProfileDisplayRecord[]
  presentation: ProfilePresentationView
  onSelect: (name: string) => void
  onEdit: (name: string) => void
  onApply: (name: string) => void
}

interface ProfileTableRowProps {
  record: ProfileDisplayRecord
  presentation: ProfilePresentationView
  onSelect: (name: string) => void
  onEdit: (name: string) => void
  onApply: (name: string) => void
}

function ProfileTableRow({
  record,
  presentation,
  onSelect,
  onEdit,
  onApply,
}: ProfileTableRowProps) {
  const t = useShellT()
  const state = resolveRowState(record, presentation)
  const rowClass = state.emphasized ? 'cp-table__row cp-table__row--running' : 'cp-table__row'
  const applyClass =
    state.applyTone === 'accent-soft' ? 'cp-btn cp-btn--accent-soft' : 'cp-btn cp-btn--ghost'
  const onRowClick = () => onSelect(record.name)
  const onEditClick = () => onEdit(record.name)
  const onApplyClick = () => onApply(record.name)
  const col3 = record.slots[1] || t('profilesSurface.placeholder')
  const col4 = record.slots[2] || t('profilesSurface.placeholder')

  return (
    <div className={rowClass} data-name={record.name} onClick={onRowClick}>
      <div className="cp-table__name">
        <span
          className={
            state.dotTone === 'active' ? 'cp-card__dot cp-card__dot--active' : 'cp-card__dot'
          }
        />
        <span>
          <span className="cp-card__name">{record.name}</span>
          <span className="cp-card__desc">{record.description || t('profilesSurface.placeholder')}</span>
        </span>
      </div>
      <div className="cp-table__mono">{record.slots[0] || t('profilesSurface.placeholder')}</div>
      <div className="cp-table__mono">{col3}</div>
      <div>{presentation.fieldSlots[2]?.chip ? <span className="cp-chip">{col4}</span> : col4}</div>
      <div className="cp-card__tags">
        {record.tags.map((tag) => (
          <span key={tag} className="cp-chip">
            #{tag}
          </span>
        ))}
      </div>
      <div className="cp-table__actions">
        <button type="button" className="cp-btn cp-btn--ghost" onClick={onEditClick}>
          {t('profilesSurface.edit')}
        </button>
        <button type="button" className={applyClass} onClick={onApplyClick}>
          {t(state.applyLabelKey)}
        </button>
      </div>
    </div>
  )
}

/** 六列固定网格表格；外层横向滚动，最小宽度走 CSS `--breakpoint-lg`。 */
export function ProfileTable({ records, presentation, onSelect, onEdit, onApply }: ProfileTableProps) {
  const t = useShellT()
  return (
    <div className="cp-table-scroll" data-testid="profiles-table-scroll">
      <div className="cp-table" data-testid="profiles-table">
        <div className="cp-table__head">
          <span>{t('profilesSurface.table.name')}</span>
          <span>{t(presentation.fieldSlots[0].labelKey)}</span>
          <span>{t(presentation.fieldSlots[1].labelKey)}</span>
          <span>{t(presentation.fieldSlots[2].labelKey)}</span>
          <span>{t('profilesSurface.table.tags')}</span>
          <span>{t('profilesSurface.table.actions')}</span>
        </div>
        {records.map((record) => (
          <ProfileTableRow
            key={record.name}
            record={record}
            presentation={presentation}
            onSelect={onSelect}
            onEdit={onEdit}
            onApply={onApply}
          />
        ))}
      </div>
    </div>
  )
}
