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
