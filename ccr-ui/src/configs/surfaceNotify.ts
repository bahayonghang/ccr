import { useUIStore, type ConfirmDialogOptions } from '@/shell/stores/ui'

/** 共享层可导入 shell；base 组件通过 config.notify 间接发通知，避免 feature → shell。 */
export const surfaceNotify = {
  success: (message: string) => {
    useUIStore.getState().showSuccess(message)
  },
  error: (message: string) => {
    useUIStore.getState().showError(message)
  },
  warning: (message: string) => {
    useUIStore.getState().showWarning(message)
  },
  confirm: (options: ConfirmDialogOptions) => useUIStore.getState().requestConfirm(options),
}

export type SurfaceNotify = typeof surfaceNotify
