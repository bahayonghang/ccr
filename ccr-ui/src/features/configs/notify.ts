import { surfaceNotify } from '@/configs/surfaceNotify'

/** feature 层通知入口。避免 feature → shell。 */
export const configsNotify = {
  success: surfaceNotify.success,
  error: surfaceNotify.error,
  warning: surfaceNotify.warning,
  confirm: surfaceNotify.confirm,
}
