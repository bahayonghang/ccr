import type { ReactNode } from 'react'
import { BaseModal } from './base-modal'
import { SIcon } from './s-icon'

export type ConfirmModalType = 'danger' | 'info' | 'warning'
export type ConfirmModalSurface = 'glass' | 'solid'

interface ConfirmModalProps {
  isOpen: boolean
  title: string
  message: string
  confirmText?: string
  cancelText?: string
  type?: ConfirmModalType
  surface?: ConfirmModalSurface
  footnote?: string
  details?: ReactNode
  icon?: ReactNode
  onConfirm?: () => void
  onCancel?: () => void
  onOpenChange?: (open: boolean) => void
}

const ICON_NAME: Record<ConfirmModalType, string> = {
  danger: 'AlertTriangle',
  warning: 'AlertCircle',
  info: 'Info',
}

const ICON_WRAP: Record<ConfirmModalType, string> = {
  danger: 'border-accent-danger/20 bg-accent-danger/10',
  warning: 'border-accent-warning/20 bg-accent-warning/10',
  info: 'border-accent-info/20 bg-accent-info/10',
}

const ICON_COLOR: Record<ConfirmModalType, string> = {
  danger: 'text-accent-danger',
  warning: 'text-accent-warning',
  info: 'text-accent-info',
}

const CONFIRM_BUTTON: Record<ConfirmModalType, string> = {
  danger:
    'bg-accent-danger text-[color:var(--color-danger-contrast)] hover:bg-accent-danger/90 focus:ring-accent-danger/30',
  warning:
    'bg-accent-warning text-[color:var(--color-warning-contrast)] hover:bg-accent-warning/90 focus:ring-accent-warning/30',
  info: 'bg-accent-primary text-[color:var(--color-accent-primary-contrast)] hover:bg-accent-primary/90 focus:ring-accent-primary/30',
}

export function ConfirmModal({
  isOpen,
  title,
  message,
  confirmText,
  cancelText,
  type = 'info',
  surface = 'solid',
  footnote,
  details,
  icon,
  onConfirm,
  onCancel,
  onOpenChange,
}: ConfirmModalProps) {
  const handleCancel = () => {
    onCancel?.()
    onOpenChange?.(false)
  }
  const handleConfirm = () => {
    onConfirm?.()
    onOpenChange?.(false)
  }

  return (
    <BaseModal
      modelValue={isOpen}
      description={message}
      closeOnBackdrop={false}
      closeOnEscape
      showClose={false}
      surface={surface}
      contentClass={`confirm-modal confirm-modal--${type}`}
      size="sm"
      onUpdateModelValue={(open) => {
        if (!open) handleCancel()
      }}
      onClose={handleCancel}
      header={({ titleId }) => (
        <h2 id={titleId} className="confirm-modal__title w-full text-center text-lg font-semibold">
          {title}
        </h2>
      )}
      footer={
        <div className="confirm-modal__footer flex w-full gap-3">
          <button
            type="button"
            className="confirm-modal__button confirm-modal__button--cancel flex-1 rounded-xl px-4 py-2.5 text-sm font-medium transition-colors duration-150 focus:outline-none focus:ring-2 focus:ring-accent-primary/30"
            onClick={handleCancel}
          >
            {cancelText || '取消'}
          </button>
          <button
            type="button"
            className={`confirm-modal__button flex-1 rounded-xl px-4 py-2.5 text-sm font-medium shadow-sm focus:outline-none focus:ring-2 transition-colors duration-150 ${CONFIRM_BUTTON[type]}`}
            onClick={handleConfirm}
          >
            {confirmText || '确认'}
          </button>
        </div>
      }
    >
      <div className="confirm-modal__body flex flex-col items-center text-center pb-1">
        <div
          className={`confirm-modal__icon-wrap flex h-14 w-14 items-center justify-center rounded-full border shadow-sm ${ICON_WRAP[type]}`}
        >
          {icon ?? <SIcon name={ICON_NAME[type]} className={`h-7 w-7 ${ICON_COLOR[type]}`} />}
        </div>
        <p className="confirm-modal__message mt-4 text-sm leading-relaxed">{message}</p>
        {details ? <div className="confirm-modal__details mt-3 w-full">{details}</div> : null}
        {footnote ? (
          <p className="confirm-modal__footnote mt-3 text-xs leading-relaxed">{footnote}</p>
        ) : null}
      </div>
    </BaseModal>
  )
}
