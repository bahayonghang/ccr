import type { MouseEvent } from 'react'
import type { ProfileDisplayRecord } from '@/configs/profileDisplayRecord'
import type { ProfilePresentationView } from '@/configs/profilePresentation'
import { resolveRowState } from '@/utils/resolveProfileRowState'
import { useShellT } from '@/shell/i18n'
import { Badge, Button } from '@/ui'
import { ProfileFieldValue } from './ProfileFieldValue'
import { ProfileOverflowMenu } from './ProfileOverflowMenu'
import './profiles-shared.css'

export interface ProfileTableProps {
  records: readonly ProfileDisplayRecord[]
  presentation: ProfilePresentationView
  onSelect: (name: string) => void
  onEdit: (name: string) => void
  onApply: (name: string) => void
  onToggle?: (name: string, enabled: boolean) => void
  onDelete?: (name: string) => void
}

interface ProfileTableRowProps {
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

function ProfileTableRow({
  record,
  presentation,
  onSelect,
  onEdit,
  onApply,
  onToggle,
  onDelete,
}: ProfileTableRowProps) {
  const t = useShellT()
  const state = resolveRowState(record, presentation)
  const placeholder = t('profilesSurface.placeholder')
  const rowClass = state.emphasized ? 'cp-table__row cp-table__row--running' : 'cp-table__row'
  const applyVariant = state.applyTone === 'accent-soft' ? 'accent-soft' : 'ghost'
  const onRowClick = () => onSelect(record.name)
  const onEditClick = stopAnd(() => onEdit(record.name))
  const onApplyClick = stopAnd(() => onApply(record.name))
  const authSlot = presentation.fieldSlots[2]
  const onToggleRecord = onToggle
    ? (enabled: boolean) => onToggle(record.name, enabled)
    : undefined
  const onDeleteRecord = onDelete ? () => onDelete(record.name) : undefined

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
          <span className="cp-card__desc">{record.description || placeholder}</span>
        </span>
      </div>
      <div className="cp-table__mono">
        <ProfileFieldValue
          kind={presentation.fieldSlots[0]?.kind ?? 'url'}
          value={record.slots[0]}
          placeholder={placeholder}
        />
      </div>
      <div className="cp-table__mono">
        <ProfileFieldValue
          kind={presentation.fieldSlots[1]?.kind ?? 'text'}
          value={record.slots[1]}
          placeholder={placeholder}
        />
      </div>
      <div>
        <ProfileFieldValue
          kind={authSlot?.kind === 'chip' ? 'chip' : 'text'}
          value={record.slots[2]}
          placeholder={placeholder}
        />
      </div>
      <div className="cp-card__tags">
        {record.tags.map((tag) => (
          <Badge key={tag} mode="static" tone="neutral">
            #{tag}
          </Badge>
        ))}
      </div>
      <div className="cp-table__actions">
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
    </div>
  )
}

/** 六列固定网格表格；外层横向滚动，最小宽度走 CSS `--breakpoint-lg`。 */
export function ProfileTable({
  records,
  presentation,
  onSelect,
  onEdit,
  onApply,
  onToggle,
  onDelete,
}: ProfileTableProps) {
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
            onToggle={onToggle}
            onDelete={onDelete}
          />
        ))}
      </div>
    </div>
  )
}
