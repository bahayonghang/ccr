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

export function showErrorSafe(ui: any, error: unknown, fallbackMessage: string): void {
  const message = getErrorMessage(error)
  ui?.showError(message || fallbackMessage)
}
