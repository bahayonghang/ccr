/**
 * Error handling utilities
 */

export function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message
  }
  if (typeof error === 'string') {
    return error
  }
  if (error && typeof error === 'object' && 'message' in error) {
    return String(error.message)
  }
  return '发生未知错误'
}

/**
 * 从未知错误中提取可读消息；提取不到时返回 null，便于调用方回退到本地化默认文案
 * （区别于 getErrorMessage 总是返回兜底字符串）。
 */
export function extractErrorMessage(error: unknown): string | null {
  if (typeof error === 'string') {
    return error
  }
  if (error && typeof error === 'object') {
    const candidate = error as { message?: unknown; error?: unknown; cause?: unknown }
    for (const value of [candidate.message, candidate.error, candidate.cause]) {
      if (typeof value === 'string' && value.trim()) {
        return value
      }
    }
  }
  return null
}

interface ErrorUi {
  showError: (message: string) => void
}

function isErrorUi(value: unknown): value is ErrorUi {
  return typeof value === 'object' &&
    value !== null &&
    'showError' in value &&
    typeof value.showError === 'function'
}

export function showErrorSafe(ui: unknown, error: unknown, fallbackMessage: string): void {
  const message = getErrorMessage(error)
  if (isErrorUi(ui)) {
    ui.showError(message || fallbackMessage)
  }
}
