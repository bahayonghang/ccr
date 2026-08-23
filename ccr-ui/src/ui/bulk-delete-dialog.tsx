import { BaseModal } from './base-modal'
import { SIcon } from './s-icon'

export interface BulkDeleteItem {
  key: string
  label: string
  badge?: string
}

interface BulkDeleteDialogProps {
  isOpen: boolean
  items: BulkDeleteItem[]
  title?: string
  resourceLabel?: string
  message?: string
  overflowMessage?: string
  cancelLabel?: string
  confirmLabel?: string
  loading?: boolean
  onConfirm?: () => void
  onCancel?: () => void
}

export function BulkDeleteDialog({
  isOpen,
  items,
  title = 'Confirm Delete',
  resourceLabel = '项',
  message,
  overflowMessage,
  cancelLabel,
  confirmLabel,
  loading = false,
  onConfirm,
  onCancel,
}: BulkDeleteDialogProps) {
  const defaultMessage = `确认删除选中的 ${items.length} 个${resourceLabel}？此操作不可撤销。`
  const showList = items.length > 0 && items.length <= 10

  return (
    <BaseModal
      modelValue={isOpen}
      title={title}
      closeOnBackdrop
      closeOnEscape
      showClose
      surface="solid"
      size="sm"
      onUpdateModelValue={(open) => {
        if (!open) onCancel?.()
      }}
      onClose={onCancel}
      footer={
        <>
          <button type="button" className="bulk-delete__btn bulk-delete__btn--cancel" onClick={onCancel}>
            {cancelLabel || 'Cancel'}
          </button>
          <button
            type="button"
            className="bulk-delete__btn bulk-delete__btn--confirm"
            disabled={loading}
            onClick={onConfirm}
          >
            <SIcon name={loading ? 'Loader2' : 'Trash2'} size="w-4 h-4" className={loading ? 'animate-spin' : undefined} />
            <span>{confirmLabel || `Delete ${items.length}`}</span>
          </button>
        </>
      }
    >
      <div className="bulk-delete__body">
        <div className="bulk-delete__icon-wrap">
          <SIcon name="AlertTriangle" className="w-6 h-6 text-amber-500" />
        </div>
        <p className="bulk-delete__message">{message || defaultMessage}</p>
        {showList ? (
          <div className="bulk-delete__list">
            {items.map((item) => (
              <div key={item.key} className="bulk-delete__item">
                <span className="truncate">{item.label}</span>
                {item.badge ? <span className="bulk-delete__badge">{item.badge}</span> : null}
              </div>
            ))}
          </div>
        ) : null}
        {items.length > 10 ? (
          <p className="bulk-delete__overflow">
            {overflowMessage || `... 以及其他 ${items.length - 10} 项`}
          </p>
        ) : null}
      </div>
    </BaseModal>
  )
}
