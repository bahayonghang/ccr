import { defineStore } from 'pinia'

interface Toast {
  id: number
  message: string
  type: 'success' | 'error' | 'warning' | 'info'
  duration: number
}

interface UIState {
  toasts: Toast[]
  globalLoading: boolean
  loadingMessage: string
  nextToastId: number
}

export const useUIStore = defineStore('ui', {
  state: (): UIState => ({
    toasts: [],
    globalLoading: false,
    loadingMessage: '',
    nextToastId: 1
  }),

  actions: {
    /**
     * 显示 toast 提示
     * @param message 消息内容
     * @param type 消息类型
     * @param duration 显示时长（毫秒），默认 3000
     */
    showToast(
      message: string,
      type: 'success' | 'error' | 'warning' | 'info' = 'info',
      duration = 3000
    ) {
      const id = this.nextToastId++
      const toast: Toast = { id, message, type, duration }
      
      this.toasts.push(toast)

      // 自动移除
      if (duration > 0) {
        setTimeout(() => {
          this.removeToast(id)
        }, duration)
      }

      return id
    },

    /**
     * 移除 toast
     * @param id toast ID
     */
    removeToast(id: number) {
      const index = this.toasts.findIndex(t => t.id === id)
      if (index !== -1) {
        this.toasts.splice(index, 1)
      }
    },

    /**
     * 显示成功提示
     */
    showSuccess(message: string, duration = 3000) {
      return this.showToast(message, 'success', duration)
    },

    /**
     * 显示错误提示
     */
    showError(message: string, duration = 5000) {
      return this.showToast(message, 'error', duration)
    },

    /**
     * 显示警告提示
     */
    showWarning(message: string, duration = 4000) {
      return this.showToast(message, 'warning', duration)
    },

    /**
     * 显示信息提示
     */
    showInfo(message: string, duration = 3000) {
      return this.showToast(message, 'info', duration)
    },

    /**
     * 显示全局加载状态
     * @param message 加载提示文字
     */
    startLoading(message = '加载中...') {
      this.globalLoading = true
      this.loadingMessage = message
    },

    /**
     * 隐藏全局加载状态
     */
    stopLoading() {
      this.globalLoading = false
      this.loadingMessage = ''
    },

    /**
     * 清除所有 toast
     */
    clearToasts() {
      this.toasts = []
    }
  }
})
