import { defineStore } from 'pinia'
import { ref } from 'vue'

interface Toast {
  id: number
  message: string
  type: 'success' | 'error' | 'warning' | 'info'
  duration: number
}

export const useUIStore = defineStore('ui', () => {
  // State
  const toasts = ref<Toast[]>([])
  const globalLoading = ref(false)
  const loadingMessage = ref('')
  const nextToastId = ref(1)

  // Actions
  function removeToast(id: number) {
    const index = toasts.value.findIndex(t => t.id === id)
    if (index !== -1) {
      toasts.value.splice(index, 1)
    }
  }

  function showToast(
    message: string,
    type: 'success' | 'error' | 'warning' | 'info' = 'info',
    duration = 3000
  ) {
    const id = nextToastId.value++
    const toast: Toast = { id, message, type, duration }
    
    toasts.value.push(toast)

    if (duration > 0) {
      setTimeout(() => {
        removeToast(id)
      }, duration)
    }

    return id
  }

  function showSuccess(message: string, duration = 3000) {
    return showToast(message, 'success', duration)
  }

  function showError(message: string, duration = 5000) {
    return showToast(message, 'error', duration)
  }

  function showWarning(message: string, duration = 4000) {
    return showToast(message, 'warning', duration)
  }

  function showInfo(message: string, duration = 3000) {
    return showToast(message, 'info', duration)
  }

  function startLoading(message = '加载中...') {
    globalLoading.value = true
    loadingMessage.value = message
  }

  function stopLoading() {
    globalLoading.value = false
    loadingMessage.value = ''
  }

  function clearToasts() {
    toasts.value = []
  }

  return {
    toasts,
    globalLoading,
    loadingMessage,
    showToast,
    removeToast,
    showSuccess,
    showError,
    showWarning,
    showInfo,
    startLoading,
    stopLoading,
    clearToasts
  }
})
