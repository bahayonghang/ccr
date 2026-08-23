import { useUIStore } from '@/shell/stores/ui'
import { ConfirmModal } from '@/ui/confirm-modal'

export function GlobalConfirmDialog() {
  const confirmDialog = useUIStore((state) => state.confirmDialog)
  const resolveConfirmDialog = useUIStore((state) => state.resolveConfirmDialog)

  if (!confirmDialog) return null

  return (
    <ConfirmModal
      isOpen
      title={confirmDialog.title}
      message={confirmDialog.message}
      confirmText={confirmDialog.confirmText}
      cancelText={confirmDialog.cancelText}
      type={confirmDialog.type}
      surface={confirmDialog.surface}
      onConfirm={() => resolveConfirmDialog(true)}
      onCancel={() => resolveConfirmDialog(false)}
      onOpenChange={(open) => {
        if (!open) resolveConfirmDialog(false)
      }}
    />
  )
}
