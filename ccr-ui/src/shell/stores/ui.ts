import { create } from 'zustand'

// ui store（08-22-state-logic-port 批次 4；原 Pinia `stores/ui.ts` 语义等价迁移）。
// 跨页面共享的 UI 编排态：toast / 确认对话框 / 全局 loading（state-disposition.md）。
// 消费必须用选择器：`useUIStore((s) => s.toasts)`（react-rerender-discipline.md）。

export interface Toast {
  id: number
  message: string
  type: 'success' | 'error' | 'warning' | 'info'
  duration: number
}

export interface ConfirmDialogOptions {
  title: string
  message: string
  confirmText?: string
  cancelText?: string
  type?: 'danger' | 'info' | 'warning'
  surface?: 'glass' | 'solid'
}

export interface ActiveConfirmDialog
  extends Required<Pick<ConfirmDialogOptions, 'title' | 'message' | 'type' | 'surface'>> {
  confirmText?: string
  cancelText?: string
}

interface UIState {
  toasts: Toast[]
  globalLoading: boolean
  loadingMessage: string
  confirmDialog: ActiveConfirmDialog | null
  showToast: (message: string, type?: Toast['type'], duration?: number) => number
  removeToast: (id: number) => void
  showSuccess: (message: string, duration?: number) => number
  showError: (message: string, duration?: number) => number
  showWarning: (message: string, duration?: number) => number
  showInfo: (message: string, duration?: number) => number
  requestConfirm: (options: ConfirmDialogOptions) => Promise<boolean>
  resolveConfirmDialog: (confirmed: boolean) => void
  startLoading: (message?: string) => void
  stopLoading: () => void
  clearToasts: () => void
}

// 确认对话框的 resolver 不入 store state（函数非可序列化状态），模块级持有，
// 与原 Pinia 实现的闭包持有等价。
let confirmResolver: ((value: boolean) => void) | null = null
let nextToastId = 1

export const useUIStore = create<UIState>()((set, get) => ({
  toasts: [],
  globalLoading: false,
  loadingMessage: '',
  confirmDialog: null,

  removeToast: (id) => {
    set((state) => ({ toasts: state.toasts.filter((toast) => toast.id !== id) }))
  },

  showToast: (message, type = 'info', duration = 3000) => {
    const id = nextToastId++
    set((state) => ({ toasts: [...state.toasts, { id, message, type, duration }] }))

    if (duration > 0) {
      setTimeout(() => get().removeToast(id), duration)
    }

    return id
  },

  showSuccess: (message, duration = 3000) => get().showToast(message, 'success', duration),
  showError: (message, duration = 5000) => get().showToast(message, 'error', duration),
  showWarning: (message, duration = 4000) => get().showToast(message, 'warning', duration),
  showInfo: (message, duration = 3000) => get().showToast(message, 'info', duration),

  requestConfirm: (options) => {
    if (confirmResolver) {
      get().resolveConfirmDialog(false)
    }

    set({
      confirmDialog: {
        title: options.title,
        message: options.message,
        confirmText: options.confirmText,
        cancelText: options.cancelText,
        type: options.type ?? 'info',
        surface: options.surface ?? 'glass',
      },
    })

    return new Promise((resolve) => {
      confirmResolver = resolve
    })
  },

  resolveConfirmDialog: (confirmed) => {
    const resolver = confirmResolver
    confirmResolver = null
    set({ confirmDialog: null })
    resolver?.(confirmed)
  },

  startLoading: (message = '加载中...') => {
    set({ globalLoading: true, loadingMessage: message })
  },

  stopLoading: () => {
    set({ globalLoading: false, loadingMessage: '' })
  },

  clearToasts: () => {
    set({ toasts: [] })
  },
}))
