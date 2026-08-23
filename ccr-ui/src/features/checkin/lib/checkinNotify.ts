import { surfaceNotify } from '@/configs/surfaceNotify'

/** feature 层通知入口。info 走 success，避免 feature → shell。 */
export const checkinNotify = {
  success: surfaceNotify.success,
  error: surfaceNotify.error,
  warning: surfaceNotify.warning,
  confirm: surfaceNotify.confirm,
  info: (message: string) => {
    surfaceNotify.success(message)
  },
}
