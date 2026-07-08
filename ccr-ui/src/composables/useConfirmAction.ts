// 二次确认 + 执行流程：Claude/Codex Profiles 页共用，统一走 ConfirmModal，
// 替代原生 confirm()/alert()。dialog 内容由调用方在 open 时传入。
import { reactive, ref } from 'vue'

export type ConfirmActionType = 'danger' | 'info' | 'warning'

export interface ConfirmActionOptions {
  title: string
  message: string
  confirmText: string
  type: ConfirmActionType
  action: () => Promise<void>
}

export interface ConfirmActionDialogState {
  title: string
  message: string
  confirmText: string
  type: ConfirmActionType
}

export function useConfirmAction() {
  const isOpen = ref(false)
  const busy = ref(false)
  const dialog = reactive<ConfirmActionDialogState>({
    title: '',
    message: '',
    confirmText: '',
    type: 'warning',
  })

  let pendingAction: (() => Promise<void>) | null = null

  const openConfirmDialog = (options: ConfirmActionOptions) => {
    dialog.title = options.title
    dialog.message = options.message
    dialog.confirmText = options.confirmText
    dialog.type = options.type
    pendingAction = options.action
    isOpen.value = true
  }

  const executeConfirmedAction = async () => {
    if (!pendingAction) return
    const action = pendingAction
    pendingAction = null
    busy.value = true
    try {
      await action()
    } finally {
      busy.value = false
    }
  }

  return { isOpen, dialog, busy, openConfirmDialog, executeConfirmedAction }
}
