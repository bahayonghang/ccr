import { lazy, Suspense, useEffect } from 'react'
import { ErrorBoundary } from '@/shell/ErrorBoundary'
import { type ActiveConfirmDialog, useUIStore } from '@/shell/stores/ui'

const LazyConfirmModal = lazy(() =>
  import('@/ui/confirm-modal').then((mod) => ({ default: mod.ConfirmModal })),
)

interface ConfirmDialogFailureFallbackProps {
  confirmDialog: ActiveConfirmDialog
  resolveConfirmDialog: (confirmed: boolean) => void
}

function ConfirmDialogFailureFallback({
  confirmDialog,
  resolveConfirmDialog,
}: ConfirmDialogFailureFallbackProps) {
  useEffect(() => {
    if (useUIStore.getState().confirmDialog === confirmDialog) {
      resolveConfirmDialog(false)
    }
  }, [confirmDialog, resolveConfirmDialog])

  return null
}

const confirmDialogBoundaryKeys = new WeakMap<ActiveConfirmDialog, number>()
let nextConfirmDialogBoundaryKey = 1

function getConfirmDialogBoundaryKey(confirmDialog: ActiveConfirmDialog): number {
  const existingKey = confirmDialogBoundaryKeys.get(confirmDialog)
  if (existingKey) return existingKey

  const key = nextConfirmDialogBoundaryKey++
  confirmDialogBoundaryKeys.set(confirmDialog, key)
  return key
}

export function GlobalConfirmDialog() {
  const confirmDialog = useUIStore((state) => state.confirmDialog)
  const resolveConfirmDialog = useUIStore((state) => state.resolveConfirmDialog)

  if (!confirmDialog) return null

  return (
    <ErrorBoundary
      key={getConfirmDialogBoundaryKey(confirmDialog)}
      fallback={(
        <ConfirmDialogFailureFallback
          confirmDialog={confirmDialog}
          resolveConfirmDialog={resolveConfirmDialog}
        />
      )}
    >
      <Suspense fallback={null}>
        <LazyConfirmModal
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
      </Suspense>
    </ErrorBoundary>
  )
}
