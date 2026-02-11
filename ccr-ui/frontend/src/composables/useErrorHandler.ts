import { useUIStore } from '@/stores/ui'
import { getErrorMessage } from '@/types/api'

/**
 * 统一错误处理 composable
 *
 * 桥接 getErrorMessage() 类型安全提取 + useUIStore toast 通知系统，
 * 提供一致的错误处理模式，替代分散的 alert() / console.error()。
 *
 * @example
 * ```vue
 * const { handleError, handleSuccess, withErrorHandling } = useErrorHandler()
 *
 * // 手动处理
 * try {
 *   await saveConfig(data)
 *   handleSuccess('配置保存成功')
 * } catch (e) {
 *   handleError(e, '保存配置')
 * }
 *
 * // 自动包装
 * const save = withErrorHandling(
 *   () => saveConfig(data),
 *   { context: '保存配置', successMessage: '配置保存成功' }
 * )
 * await save()
 * ```
 */
export function useErrorHandler() {
  const ui = useUIStore()

  /**
   * 处理错误：提取消息 → toast 通知 → console 记录
   * @param error - 任意类型的错误对象
   * @param context - 操作上下文描述（如"保存配置"）
   */
  function handleError(error: unknown, context?: string) {
    const message = getErrorMessage(error)
    const displayMessage = context ? `${context}失败: ${message}` : message

    ui.showError(displayMessage)
    console.error(`[ErrorHandler]${context ? ` ${context}:` : ''}`, error)
  }

  /**
   * 显示成功通知
   */
  function handleSuccess(message: string) {
    ui.showSuccess(message)
  }

  /**
   * 显示警告通知
   */
  function handleWarning(message: string) {
    ui.showWarning(message)
  }

  /**
   * 显示信息通知
   */
  function handleInfo(message: string) {
    ui.showInfo(message)
  }

  /**
   * 包装异步函数，自动处理错误和可选的成功通知
   * @param fn - 要执行的异步函数
   * @param options - 配置选项
   * @returns 包装后的函数，错误时返回 undefined
   */
  function withErrorHandling<T>(
    fn: () => Promise<T>,
    options: {
      context?: string
      successMessage?: string
      rethrow?: boolean
    } = {}
  ): () => Promise<T | undefined> {
    return async () => {
      try {
        const result = await fn()
        if (options.successMessage) {
          handleSuccess(options.successMessage)
        }
        return result
      } catch (error) {
        handleError(error, options.context)
        if (options.rethrow) {
          throw error
        }
        return undefined
      }
    }
  }

  return {
    handleError,
    handleSuccess,
    handleWarning,
    handleInfo,
    withErrorHandling,
  }
}
