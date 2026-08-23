// 二次确认 + 执行流程：Claude/Codex Profiles 页共用，统一走 ConfirmModal，
// 替代原生 confirm()/alert()。dialog 内容由调用方在 open 时传入。
//
// 08-22-state-logic-port 批次 5c：Vue → React（组件本地瞬态）。
// isOpen/busy 为 ref → useState；reactive dialog → 不可变对象 state（整体替换）；
// pendingAction 非响应式闭包变量 → useRef。导出名不变。

import { useCallback, useRef, useState } from 'react'

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

const EMPTY_DIALOG: ConfirmActionDialogState = {
  title: '',
  message: '',
  confirmText: '',
  type: 'warning',
  footnote: '',
}

/** Confirm-then-execute flow shared by the profile views. */
export function useConfirmAction() {
  const [isOpen, setIsOpen] = useState(false)
  const [busy, setBusy] = useState(false)
  const [dialog, setDialog] = useState<ConfirmActionDialogState>(EMPTY_DIALOG)
  const pendingActionRef = useRef<(() => Promise<void>) | null>(null)

  const openConfirmDialog = useCallback((options: ConfirmActionOptions) => {
    setDialog({
      title: options.title,
      message: options.message,
      confirmText: options.confirmText,
      type: options.type,
      footnote: options.footnote ?? '',
    })
    pendingActionRef.current = options.action
    setIsOpen(true)
  }, [])

  const executeConfirmedAction = useCallback(async () => {
    const action = pendingActionRef.current
    if (!action) return
    pendingActionRef.current = null
    setBusy(true)
    try {
      await action()
    } finally {
      setBusy(false)
    }
  }, [])

  return { isOpen, dialog, busy, openConfirmDialog, executeConfirmedAction }
}
