// 二次确认 + 执行流程：Claude/Codex Profiles 页共用，统一走 ConfirmModal，
// 替代原生 confirm()/alert()。dialog 内容由调用方在 open 时传入。
import { reactive, ref } from 'vue'

export type ConfirmActionType = 'danger' | 'info' | 'warning'

export interface ConfirmActionOptions {
  title: string
  message: string
  confirmText: string
  type: ConfirmActionType
  /** 可选补充说明行（如 delete 备份提示）；缺省不渲染，保持旧行为 */
  footnote?: string
  action: () => Promise<void>
}

export interface ConfirmActionDialogState {
  title: string
  message: string
  confirmText: string
  type: ConfirmActionType
  footnote: string
}

export function useConfirmAction() {
  const isOpen = ref(false)
  const busy = ref(false)
  const dialog = reactive<ConfirmActionDialogState>({
    title: '',
    message: '',
    confirmText: '',
    type: 'warning',
    footnote: '',
  })

  let pendingAction: (() => Promise<void>) | null = null

  const openConfirmDialog = (options: ConfirmActionOptions) => {
    dialog.title = options.title
    dialog.message = options.message
    dialog.confirmText = options.confirmText
    dialog.type = options.type
    dialog.footnote = options.footnote ?? ''
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
